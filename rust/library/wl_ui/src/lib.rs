#![forbid(unsafe_code)]

//! Reusable app shell and styling foundation with explicit renderer modules.
//! Keep the shared app model renderer-neutral; select a native backend explicitly.

pub use wl_ui_core::{
    UiAlignContent, UiAlignItems, UiAnimation, UiAnimationDirection, UiAnimationFillMode,
    UiAnimationIterations, UiAnimationPlayState, UiApplication, UiBackground, UiBorderStyle,
    UiBoxSizing, UiColor, UiColorScheme, UiComponent, UiCornerRadii, UiCursor, UiDisplay, UiEasing,
    UiEdges, UiEffects, UiError, UiFlexDirection, UiFlexWrap, UiFontStyle, UiGradientStop,
    UiGridTrack, UiJustifyContent, UiJustifyItems, UiKeyframe, UiKeyframes, UiLength,
    UiLinearGradient, UiMediaCondition, UiMetrics, UiObjectFit, UiOverflow, UiPage, UiPalette,
    UiPosition, UiSelector, UiShadow, UiStyle, UiStyleRule, UiStyleSheet, UiTextAlign,
    UiTextDecoration, UiTextOverflow, UiTextTransform, UiTheme, UiTransform, UiTransition,
    UiTransitionProperty, UiTypography, UiVisualState, UiWhiteSpace, UiWindowConfig,
};

pub use wl_animate as animate;
pub use wl_animate::{FrameClock, FrameClockError, FrameTick};
pub use wl_ui_core::{UiBackend, UiBackendCapabilities};

/// Runs a validated application through an application-provided renderer.
/// This is the portable entry point for platforms without a built-in adapter.
pub fn run_with_backend<B: UiBackend>(
    application: UiApplication,
    backend: &mut B,
) -> Result<(), UiError> {
    application.validate()?;
    backend.run(application)
}

const MANAGER_PAGES: [(&str, &str, &str); 18] = [
    (
        "home",
        "Home",
        "A quick overview of WinLook and the background runtime.",
    ),
    (
        "appearance",
        "Appearance",
        "Themes, colors, visual preferences and appearance profiles.",
    ),
    (
        "audio-media",
        "Audio & Media",
        "Audio-session controls and system media playback information.",
    ),
    (
        "file-explorer",
        "File Explorer",
        "File Explorer presentation and supported Shell integrations.",
    ),
    ("icons", "Icons", "Desktop and system icon presentation."),
    (
        "wallpaper",
        "Wallpaper",
        "Wallpapers, desktop scenes, widgets and playback.",
    ),
    (
        "wallpaper-store",
        "Wallpaper Store",
        "Discover Wallpaper Engine and free, openly licensed wallpaper sources.",
    ),
    (
        "wallpaper-store-wallpaper-engine",
        "Wallpaper Engine",
        "Browse Steam Workshop or hand a local project or media file to an installed Wallpaper Engine.",
    ),
    (
        "wallpaper-store-free",
        "Free Catalogues",
        "Browse free image catalogues with per-item license and attribution checks.",
    ),
    (
        "wallpaper-library",
        "Wallpaper Library",
        "Import and organize local wallpapers for offline use.",
    ),
    (
        "window-manager",
        "Window Manager",
        "Window layouts, rules, workspaces and compatibility.",
    ),
    (
        "displays",
        "Displays",
        "Monitor topology, display settings and per-display capabilities.",
    ),
    (
        "notifications",
        "Notifications",
        "WinLook notifications, popups and on-screen display.",
    ),
    (
        "templates",
        "Templates",
        "Importable WinLook configurations and reusable presets.",
    ),
    (
        "extensions",
        "Extensions",
        "Optional extensions, capabilities and permissions.",
    ),
    (
        "compatibility",
        "Compatibility",
        "Windows build, API capabilities and verified feature support.",
    ),
    (
        "diagnostics",
        "Diagnostics",
        "BGW health, subsystem state and resource-governor diagnostics.",
    ),
    (
        "settings",
        "Settings",
        "Manager preferences, startup behaviour and application settings.",
    ),
];

/// Creates the standard WinLook Manager navigation/application model.
pub fn manager_app() -> UiApplication {
    let mut application =
        UiApplication::new("WinLook").window(UiWindowConfig::new("WinLook").size(1280, 820));
    for (id, title, description) in MANAGER_PAGES {
        let navigation_title = match id {
            "wallpaper-store" | "wallpaper-library" => format!("  {title}"),
            "wallpaper-store-wallpaper-engine" | "wallpaper-store-free" => format!("    {title}"),
            _ => title.to_string(),
        };
        application = application.page(
            UiPage::new(id, title, description).navigation_title(navigation_title),
        );
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

    struct TestBackend {
        ran: bool,
    }

    impl UiBackend for TestBackend {
        fn name(&self) -> &'static str {
            "test-host"
        }

        fn capabilities(&self) -> UiBackendCapabilities {
            UiBackendCapabilities {
                native_window: true,
                css: true,
                animation: true,
                reduced_motion: true,
                accessibility: true,
            }
        }

        fn run(&mut self, application: UiApplication) -> Result<(), UiError> {
            application.validate()?;
            self.ran = true;
            Ok(())
        }
    }

    #[test]
    fn custom_backend_runs_without_a_windows_feature() {
        let app = UiApplication::new("Portable").page(UiPage::new("home", "Home", "Portable host"));
        let mut backend = TestBackend { ran: false };
        run_with_backend(app, &mut backend).unwrap();
        assert!(backend.ran);
        assert!(backend.capabilities().animation);
        assert_eq!(backend.name(), "test-host");
    }

    #[test]
    fn shared_frame_clock_is_reexported_by_wl_ui() {
        let mut clock = FrameClock::sixty_fps();
        let first = clock.tick(std::time::Duration::ZERO).unwrap();
        assert_eq!(first.delta, std::time::Duration::ZERO);
        assert_eq!(first.frame_index, 0);
    }

    #[test]
    fn manager_navigation_ids_are_unique_and_ordered() {
        let application = manager_app();
        assert!(application.validate().is_ok());
        assert_eq!(application.pages().len(), 18);
        assert_eq!(application.initial_page_id(), "home");
        assert_eq!(application.pages()[0].title(), "Home");
        assert_eq!(application.pages()[5].id(), "wallpaper");
        assert_eq!(application.pages()[6].id(), "wallpaper-store");
        assert_eq!(application.pages()[7].id(), "wallpaper-store-wallpaper-engine");
        assert_eq!(application.pages()[8].id(), "wallpaper-store-free");
        assert_eq!(application.pages()[6].navigation_label(), "  Wallpaper Store");
        assert_eq!(application.pages()[7].navigation_label(), "    Wallpaper Engine");
        assert_eq!(application.pages()[17].id(), "settings");
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
