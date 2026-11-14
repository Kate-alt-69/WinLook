#![forbid(unsafe_code)]

//! Windows 11 native WinUI 3 renderer.

use wl_ui_core::{UiApplication, UiError};
pub use wl_ui_win32::{
    discover_desktop_host, enumerate_monitors, is_same_desktop_host, monitor_desktop_host,
    refresh_topology, should_refresh_topology, DesktopHostError, DesktopHostIdentity,
    DesktopHostMonitor, MonitorError, DBT_DEVNODES_CHANGED, WM_DEVICECHANGE, WM_DISPLAYCHANGE,
};

/// Native Windows Reactor/WinUI 3 component and layout API.
///
/// This re-export is target-gated because the native Windows App SDK backend
/// is only linked on Windows. Application code can build its own components
/// without depending directly on the implementation crate name.
#[cfg(windows)]
pub use windows_reactor::*;

#[cfg(windows)]
mod custom;

#[cfg(windows)]
pub fn run_with(application: UiApplication) -> Result<(), UiError> {
    application.validate()?;
    custom::run(application)
}

#[cfg(not(windows))]
pub fn run_with(_application: UiApplication) -> Result<(), UiError> {
    Err(UiError::Backend(
        "the wl_ui_windows11 backend can only run on Windows".to_string(),
    ))
}
