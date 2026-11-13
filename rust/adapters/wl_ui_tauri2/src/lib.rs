#![forbid(unsafe_code)]

//! Adapter for an already configured Tauri 2 application.
//!
//! The host retains ownership of its frontend, plugins, commands, permissions,
//! capabilities, and application builder. This adapter attaches a window,
//! publishes the shared app model, and installs generated wl_ui theme CSS.

use serde_json::json;
use tauri::{Builder, Runtime, WebviewUrl, WebviewWindowBuilder};
use wl_ui_core::{UiApplication, UiError};

/// Adds a wl_ui window to an existing Tauri builder.
///
/// The consuming application's Tauri configuration must point build.frontendDist
/// at a directory containing index.html. The host frontend opts into the styling
/// contract with `data-wl-component`, CSS `class`/`id`, and the emitted CSS
/// variables documented in docs/wl_ui/styling.md.
pub fn attach<R: Runtime>(
    builder: Builder<R>,
    application: UiApplication,
) -> Result<Builder<R>, UiError> {
    application.validate()?;

    let window = application.window_config();
    let theme_css = application.theme_ref().css_variables()?;
    let style_css = application.styles_ref().to_css()?;
    let style_css = format!("{theme_css}\n{style_css}");
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
        "theme": {
            "scheme": application.theme().color_scheme().as_str(),
            "css": style_css,
        },
    });
    let serialized = serde_json::to_string(&page_model).map_err(|error| {
        UiError::Backend(format!(
            "could not serialize the wl_ui bootstrap model: {error}"
        ))
    })?;
    let css_serialized = serde_json::to_string(&style_css).map_err(|error| {
        UiError::Backend(format!(
            "could not serialize the wl_ui stylesheet: {error}"
        ))
    })?;
    let initialization_script = format!(
        "window.__WL_UI_BOOTSTRAP__ = {serialized};\n\
         (() => {{\n\
           const css = {css_serialized};\n\
           document.write('<style id=\"wl-ui-theme\">' + css + '</style>');\n\
         }})();"
    );

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

#[cfg(test)]
mod tests {
    use wl_ui_core::{
        UiApplication, UiColor, UiComponent, UiPage, UiSelector, UiStyle, UiStyleSheet, UiTheme,
    };

    #[test]
    fn shared_tauri_payload_contains_theme_and_selector_rules() {
        let app = UiApplication::new("Example")
            .page(UiPage::new("home", "Home", "Overview"))
            .theme(UiTheme::dark())
            .styles(UiStyleSheet::new().rule(
                UiSelector::new(UiComponent::Button).class("primary"),
                UiStyle::new().background(UiColor::rgb(20, 40, 60)),
            ));
        assert!(app.validate().is_ok());
        let css = format!(
            "{}\n{}",
            app.theme_ref().css_variables().unwrap(),
            app.styles_ref().to_css().unwrap()
        );
        assert!(css.contains("--wl-color-surface"));
        assert!(css.contains("[data-wl-component=\"button\"].primary"));
    }
}
