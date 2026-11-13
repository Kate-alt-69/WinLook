#![forbid(unsafe_code)]

//! Windows 11 native WinUI 3 renderer.

use wl_ui_core::{UiApplication, UiError};

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
