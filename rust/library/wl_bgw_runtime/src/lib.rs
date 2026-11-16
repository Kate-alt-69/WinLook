//! Initial BGW runtime host loop.
//!
//! The current runtime slice owns Explorer host monitoring, bounded Desktop
//! inventory refresh and validated local WLDE Group/layout state loading.
//! Wallpaper rendering, Shell hooks, Manager IPC and startup registration are
//! separate implementation stages.

use std::fmt;
#[cfg(windows)]
use std::thread;
#[cfg(windows)]
use std::time::{Duration, Instant};

#[cfg(any(windows, test))]
#[allow(dead_code)]
mod desktop_icon_pipeline;
#[cfg(windows)]
mod desktop_namespace_worker;
#[cfg(windows)]
mod desktop_sta;
mod desktop_store;
pub use desktop_store::{
    DesktopItemIdentityRecord, DesktopStateSnapshot, DesktopStateStore, DesktopStateStoreError,
};

#[cfg(windows)]
use desktop_icon_pipeline::{DesktopIconContext, DesktopIconPipeline};
#[cfg(windows)]
use desktop_namespace_worker::{DesktopNamespaceWorker, DesktopNamespaceWorkerPoll};
#[cfg(windows)]
use wl_wlde::DesktopItemKind;
#[cfg(windows)]
use wl_wlde_win32::{
    enumerate_desktop_items, enumerate_monitors, is_same_desktop_host, monitor_desktop_host,
    watch_desktop_folders, DesktopFolder, DesktopInventoryItem, DesktopInventoryWatcher,
    DesktopNamespaceItemKind, DesktopViewProbe,
};

#[cfg(windows)]
fn load_desktop_state(
    store: &DesktopStateStore,
    inventory: &[DesktopInventoryItem],
) -> Result<Option<DesktopStateSnapshot>, String> {
    let topology =
        enumerate_monitors().map_err(|error| format!("monitor topology unavailable: {error}"))?;
    let identities = DesktopItemIdentityRecord::from_inventory(inventory);
    store
        .load(&identities, &topology)
        .map_err(|error| error.to_string())
}

#[cfg(windows)]
const DESKTOP_HOST_RECHECK_INTERVAL: Duration = Duration::from_secs(5);
#[cfg(windows)]
const DESKTOP_ICON_POLL_INTERVAL: Duration = Duration::from_millis(250);
#[cfg(windows)]
const DESKTOP_NAMESPACE_REFRESH_INTERVAL: Duration = Duration::from_secs(30);
#[cfg(windows)]
const SHELL_UNAVAILABLE_RETRY_INTERVAL: Duration = Duration::from_secs(2);

/// Errors that prevent BGW from running on the current platform.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BgwRuntimeError {
    UnsupportedPlatform,
}

impl fmt::Display for BgwRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => {
                f.write_str("the initial BGW Explorer host monitor requires Windows")
            }
        }
    }
}

impl std::error::Error for BgwRuntimeError {}

/// Starts BGW's low-overhead Explorer desktop-host monitor.
///
/// The loop sleeps on the Explorer process handle while the Shell is healthy.
/// It revalidates the Shell window owner every five seconds to catch a desktop
/// host change that does not terminate the original process. When no compatible
/// Explorer desktop is available, it retries discovery every two seconds.
#[cfg(windows)]
pub fn run() -> Result<(), BgwRuntimeError> {
    run_windows()
}

/// The current BGW runtime is Windows-only. Non-Windows hosts fail explicitly.
#[cfg(not(windows))]
pub fn run() -> Result<(), BgwRuntimeError> {
    Err(BgwRuntimeError::UnsupportedPlatform)
}

#[cfg(windows)]
fn run_windows() -> Result<(), BgwRuntimeError> {
    let mut waiting_for_shell_logged = false;
    let desktop_state_store = match DesktopStateStore::for_current_user() {
        Ok(store) => Some(store),
        Err(error) => {
            report_status(&format!(
                "WLDE local Group/layout persistence unavailable; native desktop remains unchanged: {error}"
            ));
            None
        }
    };
    let mut desktop_icon_pipeline = match DesktopIconPipeline::new(DesktopIconContext::initial_bgw())
    {
        Ok(pipeline) => Some(pipeline),
        Err(error) => {
            report_status(&format!(
                "WLDE Shell icon worker unavailable; the native desktop remains unchanged: {error}"
            ));
            None
        }
    };
    let mut desktop_namespace_worker = match DesktopNamespaceWorker::new() {
        Ok(worker) => Some(worker),
        Err(error) => {
            report_status(&format!(
                "WLDE Shell namespace worker unavailable; virtual Desktop items will not be indexed: {error}"
            ));
            None
        }
    };
    let mut host_generation = 0u64;

    loop {
        let monitor = match monitor_desktop_host() {
            Ok(monitor) => monitor,
            Err(error) => {
                if !waiting_for_shell_logged {
                    report_status(&format!(
                        "waiting for an Explorer-owned desktop in this user session: {error}"
                    ));
                    waiting_for_shell_logged = true;
                }
                thread::sleep(SHELL_UNAVAILABLE_RETRY_INTERVAL);
                continue;
            }
        };

        waiting_for_shell_logged = false;
        let identity = monitor.identity().clone();
        host_generation = host_generation.wrapping_add(1).max(1);
        let current_host_generation = host_generation;
        if let Some(worker) = desktop_namespace_worker.as_mut() {
            if worker.request_refresh(current_host_generation, identity.clone()) {
                report_status("WLDE queued a read-only Shell Desktop namespace snapshot");
            }
        }
        report_status(&format!(
            "WLDE desktop host attached: explorer.exe pid={} session={} created={} image={}",
            identity.process_id(),
            identity.session_id(),
            identity.creation_time_100ns(),
            identity.image_path().display()
        ));

        // Keep the read-only inventory alive only for this verified Explorer host.
        // The state store retains only opaque IDs; no filesystem path is persisted.
        let mut desktop_inventory = match enumerate_desktop_items() {
            Ok(items) => {
                let user_count = items
                    .iter()
                    .filter(|item| item.source_folder() == DesktopFolder::User)
                    .count();
                let public_count = items
                    .iter()
                    .filter(|item| item.source_folder() == DesktopFolder::Public)
                    .count();
                let application_count = items
                    .iter()
                    .filter(|item| item.item().kind() == DesktopItemKind::Application)
                    .count();
                let folder_count = items
                    .iter()
                    .filter(|item| item.item().kind() == DesktopItemKind::Folder)
                    .count();
                let file_count = items
                    .iter()
                    .filter(|item| item.item().kind() == DesktopItemKind::File)
                    .count();
                report_status(&format!(
                    "WLDE filesystem Desktop snapshot ready: {} entries (user={} public={} apps={} folders={} files={})",
                    items.len(),
                    user_count,
                    public_count,
                    application_count,
                    folder_count,
                    file_count
                ));
                items
            }
            Err(error) => {
                report_status(&format!(
                    "WLDE filesystem Desktop snapshot unavailable; native desktop remains unchanged: {error}"
                ));
                Vec::new()
            }
        };

        let mut _restored_desktop_state = desktop_state_store
            .as_ref()
            .and_then(|store| load_committed_desktop_state(store, &desktop_inventory));
        if let Some(pipeline) = desktop_icon_pipeline.as_mut() {
            pipeline.begin_host_session();
            let queued = pipeline.request_missing(&desktop_inventory);
            if queued != 0 {
                report_status(&format!(
                    "WLDE queued {queued} Shell icons for background rasterization"
                ));
            }
        }

        let mut inventory_watcher = match watch_desktop_folders() {
            Ok(watcher) => Some(watcher),
            Err(error) => {
                report_status(&format!(
                    "Desktop change notifications unavailable; using bounded periodic inventory refresh: {error}"
                ));
                None
            }
        };
        let mut last_fallback_inventory_refresh = Instant::now();
        let mut last_namespace_refresh = Instant::now();

        loop {
            if let Some(pipeline) = desktop_icon_pipeline.as_mut() {
                let stats = pipeline.poll();
                if stats.completed != 0 || stats.failed != 0 || stats.stale_discarded != 0 {
                    report_status(&format!(
                        "WLDE Shell icons: cached={} failed={} stale-discarded={} cache-bytes={}",
                        stats.completed,
                        stats.failed,
                        stats.stale_discarded,
                        pipeline.cached_bytes()
                    ));
                }
                let queued = pipeline.request_missing(&desktop_inventory);
                if queued != 0 {
                    report_status(&format!(
                        "WLDE queued {queued} additional Shell icons for background rasterization"
                    ));
                }
            }

            if let Some(worker) = desktop_namespace_worker.as_mut() {
                while let Some(update) = worker.poll() {
                    match update {
                        DesktopNamespaceWorkerPoll::Ready {
                            generation,
                            snapshot,
                            view_result,
                        } if generation == current_host_generation => {
                            report_desktop_view_probe(view_result);
                            let filesystem_items = snapshot
                                .items()
                                .iter()
                                .filter(|item| item.is_filesystem_backed())
                                .count();
                            let virtual_folders = snapshot
                                .items()
                                .iter()
                                .filter(|item| {
                                    item.kind() == DesktopNamespaceItemKind::VirtualFolder
                                })
                                .count();
                            let virtual_items = snapshot
                                .items()
                                .iter()
                                .filter(|item| item.kind() == DesktopNamespaceItemKind::VirtualItem)
                                .count();
                            report_status(&format!(
                                "WLDE Shell namespace snapshot ready: {} entries (filesystem={} virtual-folders={} virtual-items={} skipped={}); PIDL identities are session-only",
                                snapshot.len(),
                                filesystem_items,
                                virtual_folders,
                                virtual_items,
                                snapshot.skipped_items()
                            ));
                        }
                        DesktopNamespaceWorkerPoll::Ready { .. } => {
                            report_status(
                                "WLDE discarded a stale Shell namespace snapshot after Explorer host replacement"
                            );
                            if worker.request_refresh(current_host_generation, identity.clone()) {
                                last_namespace_refresh = Instant::now();
                            }
                        }
                        DesktopNamespaceWorkerPoll::Failed {
                            generation,
                            message,
                            view_result,
                        } if generation == current_host_generation => {
                            report_desktop_view_probe(view_result);
                            report_status(&format!(
                                "WLDE Shell namespace snapshot failed; native desktop presentation remains unchanged: {message}"
                            ));
                        }
                        DesktopNamespaceWorkerPoll::Failed { .. } => {
                            report_status(
                                "WLDE discarded an error from a previous Explorer host session",
                            );
                            if worker.request_refresh(current_host_generation, identity.clone()) {
                                last_namespace_refresh = Instant::now();
                            }
                        }
                        DesktopNamespaceWorkerPoll::TimedOut { generation }
                            if generation == current_host_generation =>
                        {
                            report_status(
                                "WLDE Shell namespace enumeration exceeded 20 seconds; BGW remains responsive and will not spawn a replacement worker"
                            );
                        }
                        DesktopNamespaceWorkerPoll::TimedOut { .. } => {
                            report_status(
                                "WLDE previous Shell namespace request timed out after Explorer host replacement"
                            );
                        }
                    }
                }

                if last_namespace_refresh.elapsed() >= DESKTOP_NAMESPACE_REFRESH_INTERVAL
                    && worker.request_refresh(current_host_generation, identity.clone())
                {
                    last_namespace_refresh = Instant::now();
                }
            }

            let icon_work_pending = desktop_icon_pipeline
                .as_ref()
                .is_some_and(DesktopIconPipeline::has_pending_work);
            let wait_interval = if icon_work_pending {
                DESKTOP_ICON_POLL_INTERVAL
            } else {
                DESKTOP_HOST_RECHECK_INTERVAL
            };
            let watcher_result = inventory_watcher
                .as_ref()
                .map(|watcher| watcher.wait_for_change(wait_interval));
            let inventory_changed = match watcher_result {
                Some(Ok(changed)) => changed,
                Some(Err(error)) => {
                    report_status(&format!(
                        "Desktop change notification failed; falling back to periodic inventory refresh: {error}"
                    ));
                    inventory_watcher = None;
                    false
                }
                None => {
                    thread::sleep(wait_interval);
                    if last_fallback_inventory_refresh.elapsed() >= DESKTOP_HOST_RECHECK_INTERVAL {
                        last_fallback_inventory_refresh = Instant::now();
                        true
                    } else {
                        false
                    }
                }
            };

            match monitor.wait_for_exit(Duration::ZERO) {
                Ok(true) => {
                    report_status(&format!(
                        "Explorer desktop process {} exited; rediscovering the Shell host",
                        identity.process_id()
                    ));
                    break;
                }
                Ok(false) => match is_same_desktop_host(&identity) {
                    Ok(true) => {}
                    Ok(false) => {
                        report_status(
                            "Explorer desktop host identity changed; rediscovering the Shell host",
                        );
                        break;
                    }
                    Err(error) => {
                        report_status(&format!(
                            "Explorer desktop host is no longer verifiable: {error}"
                        ));
                        break;
                    }
                },
                Err(error) => {
                    report_status(&format!(
                        "Explorer process wait failed; rediscovering the Shell host: {error}"
                    ));
                    break;
                }
            }
            if inventory_changed {
                match enumerate_desktop_items() {
                    Ok(refreshed) => {
                        let user_count = refreshed
                            .iter()
                            .filter(|item| item.source_folder() == DesktopFolder::User)
                            .count();
                        let public_count = refreshed
                            .iter()
                            .filter(|item| item.source_folder() == DesktopFolder::Public)
                            .count();
                        let application_count = refreshed
                            .iter()
                            .filter(|item| item.item().kind() == DesktopItemKind::Application)
                            .count();
                        let folder_count = refreshed
                            .iter()
                            .filter(|item| item.item().kind() == DesktopItemKind::Folder)
                            .count();
                        let file_count = refreshed
                            .iter()
                            .filter(|item| item.item().kind() == DesktopItemKind::File)
                            .count();
                        report_status(&format!(
                            "Desktop inventory refreshed: {} entries (user={} public={} apps={} folders={} files={})",
                            refreshed.len(),
                            user_count,
                            public_count,
                            application_count,
                            folder_count,
                            file_count
                        ));
                        desktop_inventory = refreshed;
                        _restored_desktop_state = desktop_state_store.as_ref().and_then(|store| {
                            load_committed_desktop_state(store, &desktop_inventory)
                        });
                        if let Some(worker) = desktop_namespace_worker.as_mut() {
                            if worker.request_refresh(current_host_generation, identity.clone()) {
                                last_namespace_refresh = Instant::now();
                            }
                        }
                        if let Some(pipeline) = desktop_icon_pipeline.as_mut() {
                            pipeline.invalidate_path_fallbacks();
                            let queued = pipeline.request_missing(&desktop_inventory);
                            if queued != 0 {
                                report_status(&format!(
                                    "WLDE queued {queued} Shell icons after Desktop inventory refresh"
                                ));
                            }
                        }
                    }
                    Err(error) => report_status(&format!(
                        "Desktop inventory refresh failed; keeping last known snapshot: {error}"
                    )),
                }
            }
        }
        // Dropping the inventory releases its ephemeral path map; dropping the
        // monitor closes the process handle before the next host probe.
        drop(desktop_inventory);
    }
}

#[cfg(windows)]
fn report_desktop_view_probe(result: Result<DesktopViewProbe, String>) {
    match result {
        Ok(probe) => report_status(&format!(
            "WLDE Explorer Desktop IFolderView acquired: owner pid={} Desktop root verified via {}",
            probe.owner_process_id(),
            probe.location().as_str()
        )),
        Err(error) => report_status(&format!(
            "WLDE Explorer Desktop IFolderView unavailable; native presentation remains unchanged: {error}"
        )),
    }
}

#[cfg(windows)]
fn load_committed_desktop_state(
    store: &DesktopStateStore,
    inventory: &[DesktopInventoryItem],
) -> Option<DesktopStateSnapshot> {
    match load_desktop_state(store, inventory) {
        Ok(Some(snapshot)) => {
            report_status(&format!(
                "WLDE local state validated: {} Groups and {} desktop placements; Group presentation remains disabled until crash-safe Shell restoration is available",
                snapshot.groups().len(),
                snapshot.layout().len()
            ));
            Some(snapshot)
        }
        Ok(None) => {
            report_status("No committed local WLDE Group/layout state exists yet");
            None
        }
        Err(error) => {
            report_status(&format!(
                "WLDE local state was not applied; the saved file was preserved and native desktop presentation remains unchanged: {error}"
            ));
            None
        }
    }
}

#[cfg(windows)]
fn report_status(message: &str) {
    report_debug_output(&format!("WinLook BGW: {message}"));
}

/// Reports a fatal BGW startup/runtime error through the platform debug channel.
pub fn report_fatal(message: &str) {
    report_debug_output(&format!("WinLook BGW fatal: {message}"));
}

#[cfg(windows)]
fn report_debug_output(message: &str) {
    use std::iter;
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "kernel32")]
    extern "system" {
        fn OutputDebugStringW(output: *const u16);
    }

    let wide = std::ffi::OsStr::new(message)
        .encode_wide()
        .chain(iter::once(0))
        .collect::<Vec<_>>();
    unsafe {
        OutputDebugStringW(wide.as_ptr());
    }
}

#[cfg(not(windows))]
fn report_debug_output(message: &str) {
    eprintln!("{message}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use wl_wlde_win32::DesktopHostError;

    #[cfg(not(windows))]
    #[test]
    fn runtime_is_explicitly_unsupported_off_windows() {
        assert_eq!(run(), Err(BgwRuntimeError::UnsupportedPlatform));
    }

    #[test]
    fn host_discovery_errors_remain_typed() {
        let error = DesktopHostError::UnsupportedPlatform;
        assert_eq!(
            error.to_string(),
            "desktop-host discovery is available only on Windows"
        );
    }
}
