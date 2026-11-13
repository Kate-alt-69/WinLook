#![forbid(unsafe_code)]

//! Classic Win32 renderer for custom Windows 7-compatible Rust targets.
//!
//! Use Rust 1.77's Win7-compatible standard library, or a custom Rust toolchain
//! built with x86_64-win7-windows-msvc / i686-win7-windows-msvc. The modern
//! standard pc-windows-msvc target still requires Windows 10 or later.

use wl_ui_win32::{UiApplication, UiError};

/// Starts an application model using baseline User32 functionality.
pub fn run_with(application: UiApplication) -> Result<(), UiError> {
    wl_ui_win32::run_with(application)
}
