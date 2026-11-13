#![forbid(unsafe_code)]

//! Windows 11 native WinUI 3 renderer.

use wl_ui_core::{UiApplication, UiError};

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
