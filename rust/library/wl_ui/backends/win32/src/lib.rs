#![deny(unsafe_op_in_unsafe_fn)]

//! Baseline User32 renderer shared by the Windows 7/8/8.1 and Windows 10 adapters.
//!
//! This crate intentionally avoids WinRT, Windows App SDK, WebView2, Windows Reactor,
//! and API imports newer than the classic Win32 desktop window/control surface.

pub use wl_ui_core::{UiApplication, UiError, UiPage, UiWindowConfig};

#[cfg(windows)]
mod native;

#[cfg(windows)]
pub fn run_with(application: UiApplication) -> Result<(), UiError> {
    application.validate()?;
    native::run(application)
}

#[cfg(not(windows))]
pub fn run_with(_application: UiApplication) -> Result<(), UiError> {
    Err(UiError::Backend(
        "the wl_ui Win32 backend can only run on Windows".to_string(),
    ))
}
