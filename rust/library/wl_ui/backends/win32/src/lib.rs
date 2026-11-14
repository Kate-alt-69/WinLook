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

/// Opens a native file picker and returns the selected path.
pub fn browse_file(title: &str, filter_description: &str, filter_pattern: &str)
    -> Result<Option<std::path::PathBuf>, UiError>
{
    browse_file_from(title, filter_description, filter_pattern, None)
}

/// Opens a native file picker at an optional starting directory.
pub fn browse_file_from(
    title: &str,
    filter_description: &str,
    filter_pattern: &str,
    initial_directory: Option<&std::path::Path>,
) -> Result<Option<std::path::PathBuf>, UiError> {
    #[cfg(windows)]
    {
        native::browse_file_from(title, filter_description, filter_pattern, initial_directory)
    }
    #[cfg(not(windows))]
    {
        let _ = (title, filter_description, filter_pattern, initial_directory);
        Err(UiError::Backend("the native file picker is only available on Windows".to_string()))
    }
}

/// Opens a validated HTTPS URL using the user's default browser.
pub fn open_url(url: &str) -> Result<(), UiError> {
    if !url.starts_with("https://") || url.contains('\0') {
        return Err(UiError::Backend("only HTTPS links are allowed".to_string()));
    }
    #[cfg(windows)]
    {
        native::open_url(url)
    }
    #[cfg(not(windows))]
    {
        let _ = url;
        Err(UiError::Backend("opening external links is only available on Windows".to_string()))
    }
}
