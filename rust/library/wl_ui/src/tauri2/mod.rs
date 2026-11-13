//! Optional adapter for an already configured Tauri 2 application.
//!
//! Import with `use wl_ui::tauri2;`. The Tauri runtime is pulled in only when
//! the `tauri2` Cargo feature is enabled.

pub use crate::{UiApplication, UiError, UiPage, UiWindowConfig};
pub use wl_ui_tauri2::attach;
