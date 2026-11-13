//! Windows 10 native desktop backend.
//!
//! Import with `use wl_ui::windows10;`. This backend uses classic Win32/User32
//! and deliberately does not link Windows App SDK, WinUI 3, Windows Reactor,
//! WebView2, or the Windows 11 dynamic package-dependency APIs.

use crate::manager_app;

pub use crate::{UiApplication, UiError, UiPage, UiWindowConfig};

/// Human-readable backend identity.
pub const BACKEND: &str = "Native Win32 / User32";

/// Minimum OS generation this backend is designed for.
pub const MINIMUM_WINDOWS_VERSION: &str = "Windows 10";

/// Starts the standard WinLook Manager model using native Win32 controls.
pub fn run() -> Result<(), UiError> {
    run_with(manager_app())
}

/// Starts an application model using the Windows 10 native Win32 renderer.
pub fn run_with(application: UiApplication) -> Result<(), UiError> {
    wl_ui_windows10::run_with(application)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_the_native_backend_contract() {
        assert_eq!(BACKEND, "Native Win32 / User32");
        assert_eq!(MINIMUM_WINDOWS_VERSION, "Windows 10");
        assert!(manager_app().validate().is_ok());
    }
}
