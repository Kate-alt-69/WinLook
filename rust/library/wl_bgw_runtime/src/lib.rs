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
mod desktop_store;
pub use desktop_store::{
    DesktopItemIdentityRecord, DesktopStateSnapshot, DesktopStateStore, DesktopStateStoreError,
};

#[cfg(windows)]
use desktop_icon_pipeline::{DesktopIconContext, DesktopIconPipeline};
#[cfg(windows)]
use wl_traton::TranslationProvider;
#[cfg(windows)]
use wl_wlde::DesktopItemKind;
#[cfg(windows)]
use wl_wlde_win32::{
    enumerate_desktop_items, enumerate_monitors, is_same_desktop_host, monitor_desktop_host,
    watch_desktop_folders, DesktopFolder, DesktopInventoryItem, DesktopInventoryWatcher,
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
fn verify_current_desktop_host(
    monitor: &wl_wlde_win32::DesktopHostMonitor,
    identity: &wl_wlde_win32::DesktopHostIdentity,
) -> Result<(), String> {
    if monitor
        .wait_for_exit(Duration::ZERO)
        .map_err(|error| format!("could not verify the Explorer process handle: {error}"))?
    {
        return Err("the Explorer desktop process exited during taskbar discovery".to_owned());
    }

    if !is_same_desktop_host(identity)
        .map_err(|error| format!("the Explorer desktop host could not be revalidated: {error}"))?
    {
        return Err(
            "the Explorer desktop host generation changed during taskbar discovery".to_owned(),
        );
    }

    Ok(())
}

#[cfg(windows)]
fn run_windows() -> Result<(), BgwRuntimeError> {
    let windows_target = match wl_traton::WindowsTarget::detect() {
        Ok(target) => {
            let ubr = target
                .update_build_revision()
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unknown".to_owned());
            let edition = target.edition().unwrap_or("unknown");
            report_status(&format!(
                "Traton target: release={} build={}.{} edition={} process-arch={} native-arch={} session={}",
                target.release_selector(),
                target.build_number(),
                ubr,
                edition,
                target.process_architecture(),
                target.system_architecture(),
                target.session_id()
            ));
            Some(target)
        }
        Err(error) => {
            report_status(&format!(
                "Traton target discovery unavailable; native integrations remain gated: {error}"
            ));
            None
        }
    };

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
        report_status(&format!(
            "WLDE desktop host attached: explorer.exe pid={} session={} created={} image={}",
            identity.process_id(),
            identity.session_id(),
            identity.creation_time_100ns(),
            identity.image_path().display()
        ));

        if let Some(target) = windows_target.as_ref() {
            if target.build_number() == 19_045 {
                let provider = wl_traton::Win10_22H2TaskbarDiscoveryProvider::new(
                    target.clone(),
                    identity.process_id(),
                );
                let discovery_result: Result<i64, String> = (|| {
                    verify_current_desktop_host(&monitor, &identity)?;

                    let mut capabilities = wl_traton::CapabilitySet::new();
                    capabilities
                        .set(
                            wl_traton::TASKBAR_WINDOW_DISCOVERY_CAPABILITY_ID,
                            wl_traton::CapabilityRecord {
                                status: provider.probe(target),
                                provider_id: Some(provider.descriptor().id.clone()),
                                reason: Some(
                                    "read-only Shell window discovery; item-level taskbar data is not included"
                                        .to_owned(),
                                ),
                            },
                        )
                        .map_err(|error| error.to_string())?;
                    let path = wl_traton::NamespacePath::parse(
                        "windows.10-22h2.shell.taskbar.windows.count",
                    )
                    .map_err(|error| error.to_string())?;
                    let registry = wl_traton::ObjectRegistry::standard();
                    let operation = registry
                        .validate(
                            &path,
                            wl_traton::OperationAction::Read,
                            None,
                            target,
                            &capabilities,
                            false,
                        )
                        .map_err(|error| error.to_string())?;
                    let catalog = wl_traton::ProviderCatalog::standard();
                    let candidate_selected = catalog
                        .candidates(&operation, target, false)
                        .iter()
                        .any(|candidate| candidate.id == provider.descriptor().id);
                    if !candidate_selected {
                        return Err("no compatible Traton taskbar provider was selected".to_owned());
                    }
                    let count = match provider
                        .read(&operation)
                        .map_err(|error| error.to_string())?
                    {
                        wl_traton::PropertyValue::Integer(count) => count,
                        _ => {
                            return Err(
                                "Traton taskbar provider returned an unexpected value type"
                                    .to_owned(),
                            );
                        }
                    };

                    verify_current_desktop_host(&monitor, &identity)?;
                    Ok(count)
                })();
                match discovery_result {
                    Ok(count) => report_status(&format!(
                        "Traton read-only taskbar snapshot: {count} Explorer-owned window(s)"
                    )),
                    Err(error) => report_status(&format!(
                        "Traton taskbar discovery unavailable; native taskbar remains unchanged: {error}"
                    )),
                }
            }
        }

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
