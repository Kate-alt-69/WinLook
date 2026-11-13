#![forbid(unsafe_code)]

//! Reusable app shell and styling foundation with explicit renderer modules.
//! Keep the shared app model renderer-neutral; select a native backend explicitly.

pub use wl_ui_core::{
    UiAlignContent, UiAlignItems, UiApplication, UiBackground, UiBorderStyle, UiBoxSizing,
    UiColor, UiColorScheme, UiComponent, UiCornerRadii, UiCursor, UiDisplay, UiEasing,
    UiEdges, UiEffects, UiError, UiFlexDirection, UiFlexWrap, UiFontStyle, UiGradientStop,
    UiGridTrack, UiJustifyContent, UiJustifyItems, UiLength, UiLinearGradient,
    UiMediaCondition, UiMetrics, UiObjectFit, UiOverflow, UiPage, UiPalette, UiPosition,
    UiSelector, UiShadow, UiStyle, UiStyleRule, UiStyleSheet, UiTextAlign,
    UiTextDecoration, UiTextOverflow, UiTextTransform, UiTheme, UiTransition,
    UiTransitionProperty, UiTransform, UiTypography, UiVisualState, UiWhiteSpace,
    UiWindowConfig,
};

const MANAGER_PAGES: [(&str, &str, &str); 14] = [
    ("home", "Home", "A quick overview of WinLook and the background runtime."),
    ("appearance", "Appearance", "Themes, colors, visual preferences and appearance profiles."),
    ("audio-media", "Audio & Media", "Audio-session controls and system media playback information."),
    ("file-explorer", "File Explorer", "File Explorer presentation and supported Shell integrations."),
    ("icons", "Icons", "Desktop and system icon presentation."),
    ("wallpaper", "Wallpaper", "Wallpapers, desktop scenes, widgets and playback."),
    ("window-manager", "Window Manager", "Window layouts, rules, workspaces and compatibility."),
    ("displays", "Displays", "Monitor topology, display settings and per-display capabilities."),
    ("notifications", "Notifications", "WinLook notifications, popups and on-screen display."),
    ("templates", "Templates", "Importable WinLook configurations and reusable presets."),
    ("extensions", "Extensions", "Optional extensions, capabilities and permissions."),
    ("compatibility", "Compatibility", "Windows build, API capabilities and verified feature support."),
    ("diagnostics", "Diagnostics", "BGW health, subsystem state and resource-governor diagnostics."),
    ("settings", "Settings", "Manager preferences, startup behaviour and application settings."),
];

/// Creates the standard WinLook Manager navigation/application model.
pub fn manager_app() -> UiApplication {
    let mut application =
        UiApplication::new("WinLook").window(UiWindowConfig::new("WinLook").size(1280, 820));
    for (id, title, description) in MANAGER_PAGES {
        application = application.page(UiPage::new(id, title, description));
    }
    application.initial_page("home")
}

#[cfg(feature = "tauri2")]
pub mod tauri2;
#[cfg(feature = "windows10")]
pub mod windows10;
#[cfg(feature = "windows11")]
pub mod windows11;
#[cfg(any(feature = "windows7", feature = "windows8", feature = "windows81"))]
pub mod windows7;
#[cfg(feature = "windows8")]
pub mod windows8;
#[cfg(feature = "windows81")]
pub mod windows81;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manager_navigation_ids_are_unique_and_ordered() {
        let application = manager_app();
        assert!(application.validate().is_ok());
        assert_eq!(application.pages().len(), 14);
        assert_eq!(application.initial_page_id(), "home");
        assert_eq!(application.pages()[0].title(), "Home");
        assert_eq!(application.pages()[13].id(), "settings");
    }

    #[test]
    fn page_model_has_no_backend_specific_types() {
        let app = UiApplication::new("Example")
            .page(UiPage::new("home", "Home", "Overview"))
            .initial_page("home");
        assert_eq!(app.initial_page_id(), "home");
        assert!(app.validate().is_ok());
    }
}
