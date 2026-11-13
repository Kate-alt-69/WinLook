//! Windows 11 native WinUI 3 backend.
//!
//! Import with `use wl_ui::windows11;`. This module is opt-in and is the only
//! Windows backend that depends on Windows Reactor / Windows App SDK.

use crate::manager_app;

pub use crate::{UiApplication, UiError, UiPage, UiWindowConfig};

/// Human-readable backend identity.
pub const BACKEND: &str = "Native WinUI 3";

/// Minimum OS generation this backend is designed for.
pub const MINIMUM_WINDOWS_VERSION: &str = "Windows 11";

/// Starts the standard WinLook Manager model using the WinUI 3 renderer.
pub fn run() -> Result<(), UiError> {
    run_with(manager_app())
}

/// Starts an application model using the Windows 11 WinUI 3 renderer.
pub fn run_with(application: UiApplication) -> Result<(), UiError> {
    wl_ui_windows11::run_with(application)
}
