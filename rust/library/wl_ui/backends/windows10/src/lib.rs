#![forbid(unsafe_code)]

//! Lightweight native Win32 backend for current Rust MSVC targets.
//!
//! This package deliberately does not link Windows App SDK, WinUI 3, WebView2,
//! Windows Reactor, or Windows 11-only dynamic package dependency APIs.

pub use wl_ui_win32::{
    enumerate_monitors, refresh_topology, should_refresh_topology, MonitorError,
    DBT_DEVNODES_CHANGED, WM_DEVICECHANGE, WM_DISPLAYCHANGE,
};
use wl_ui_win32::{UiApplication, UiError};

/// Starts an application model using baseline Win32/User32 functionality.
pub fn run_with(application: UiApplication) -> Result<(), UiError> {
    wl_ui_win32::run_with(application)
}
