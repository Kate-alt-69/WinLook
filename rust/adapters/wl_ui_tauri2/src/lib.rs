#![forbid(unsafe_code)]

//! Adapter for an already configured Tauri 2 application.
//!
//! The host retains ownership of its frontend, plugins, commands, permissions,
//! capabilities, and application builder. This adapter attaches a window and
//! exposes the shared page model to the frontend before its scripts execute.

use serde_json::json;
use tauri::{Builder, Runtime, WebviewUrl, WebviewWindowBuilder};
use wl_ui_core::{UiApplication, UiError};

/// Adds a wl_ui window to an existing Tauri builder.
///
/// The consuming application's Tauri configuration must point build.frontendDist
/// at a directory containing index.html. Before the frontend scripts execute,
/// the application model is assigned to window.__WL_UI_BOOTSTRAP__.
pub fn attach<R: Runtime>(
    builder: Builder<R>,
    application: UiApplication,
) -> Result<Builder<R>, UiError> {
    application.validate()?;

    let window = application.window_config();
    let page_model = json!({
        "title": window.title(),
        "width": window.width(),
        "height": window.height(),
        "initialPage": application.initial_page_id(),
        "pages": application.pages().iter().map(|page| {
            json!({
                "id": page.id(),
                "title": page.title(),
                "description": page.description(),
            })
        }).collect::<Vec<_>>(),
    });
    let serialized = serde_json::to_string(&page_model).map_err(|error| {
        UiError::Backend(format!(
            "could not serialize the wl_ui bootstrap model: {error}"
        ))
    })?;
    let initialization_script = format!("window.__WL_UI_BOOTSTRAP__ = {};", serialized);

    let title = window.title().to_string();
    let width = window.width() as f64;
    let height = window.height() as f64;

    Ok(builder.setup(move |app| {
        WebviewWindowBuilder::new(app, "wl-ui", WebviewUrl::App("index.html".into()))
            .title(&title)
            .inner_size(width, height)
            .initialization_script(initialization_script)
            .build()?;
        Ok(())
    }))
}
