//! Initial BGW runtime host loop.
//!
//! This first runtime slice owns Explorer desktop-host discovery/recovery only.
//! Wallpaper rendering, Shell hooks, Groups, Manager IPC, configuration
//! persistence and startup registration are separate implementation stages.

use std::fmt;
#[cfg(windows)]
use std::thread;
#[cfg(windows)]
use std::time::Duration;

#[cfg(windows)]
use wl_wlde_win32::{
    enumerate_desktop_items, is_same_desktop_host, monitor_desktop_host, watch_desktop_folders,
    DesktopFolder, DesktopInventoryWatcher,
};
#[cfg(windows)]
use wl_wlde::DesktopItemKind;

#[cfg(windows)]
const DESKTOP_HOST_RECHECK_INTERVAL: Duration = Duration::from_secs(5);
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

        // Keep the read-only path map alive only for this verified Explorer host.
        // It is discarded when the host changes or exits; Group persistence comes later.
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

        let mut inventory_watcher = match watch_desktop_folders() {
            Ok(watcher) => Some(watcher),
            Err(error) => {
                report_status(&format!(
                    "Desktop change notifications unavailable; using bounded periodic inventory refresh: {error}"
                ));
                None
            }
        };

        loop {
            let watcher_result = inventory_watcher
                .as_ref()
                .map(|watcher| watcher.wait_for_change(DESKTOP_HOST_RECHECK_INTERVAL));
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
                    thread::sleep(DESKTOP_HOST_RECHECK_INTERVAL);
                    true
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
