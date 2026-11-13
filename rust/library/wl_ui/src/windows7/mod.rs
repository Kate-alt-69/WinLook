//! Experimental classic Win32 backend for Windows 7.
//!
//! Use Rust 1.77 (the last stable release with built-in Windows 7 support), or
//! a custom modern Rust toolchain containing the Tier 3
//! `x86_64-win7-windows-msvc` / `i686-win7-windows-msvc` target. Enabling this
//! Cargo feature while compiling with a modern standard `*-pc-windows-msvc`
//! target does not lower that target's Windows 10 minimum.

use crate::manager_app;

pub use crate::{UiApplication, UiError, UiPage, UiWindowConfig};

/// Human-readable backend identity.
pub const BACKEND: &str = "Classic Win32 / User32 (legacy Rust target)";

/// Minimum OS generation this backend is designed for.
pub const MINIMUM_WINDOWS_VERSION: &str = "Windows 7";

/// Starts the standard WinLook Manager model using baseline Win32 APIs.
pub fn run() -> Result<(), UiError> {
    run_with(manager_app())
}

/// Starts an application model using the classic Win32 renderer.
pub fn run_with(application: UiApplication) -> Result<(), UiError> {
    wl_ui_windows7::run_with(application)
}
