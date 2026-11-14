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

const MANAGER_PAGES: [(&str, &str, &str); 23] = [
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
    ("icons", "Icons", "WLDE adaptive application tiles, icon artwork, masks and group-aware context menus."),
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
        "Import, organize and back up local wallpapers for offline use.",
    ),
    (
        "wallpaper-widgets",
        "Widgets",
        "Desktop-anchored and floating widgets, layouts, permissions and isolated hosting.",
    ),
    (
        "wallpaper-performance",
        "Resource & Performance",
        "CPU, GPU, RAM and VRAM pressure policy, frame pacing and playback diagnostics.",
    ),
    (
        "wallpaper-groups",
        "Desktop Groups",
        "Virtual desktop groups with native Shell restoration and recovery safeguards.",
    ),
    (
        "wallpaper-rainmeter",
        "Rainmeter",
        "Compatibility status for imported skins, measures, meters and plugins.",
    ),
    (
        "window-manager",
        "Window Manager",
        "Window layouts, rules, workspaces and compatibility.",
    ),
    (
        "game-mode",
        "Game Mode",
        "Per-application Normal, Optimize and Suspend profiles for foreground games.",
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
            "wallpaper-widgets"
            | "wallpaper-performance"
            | "wallpaper-groups"
            | "wallpaper-rainmeter" => format!("  {title}"),
            _ => title.to_string(),
        };
        application = application
            .page(UiPage::new(id, title, description).navigation_title(navigation_title));
    }
    let theme = UiTheme::dark().name("WinLook Graphite");
    let palette = *theme.palette();
    let styles = UiStyleSheet::new()
        .rule(
            UiSelector::new(UiComponent::Window),
            UiStyle::new()
                .background(palette.window)
                .foreground(palette.text_primary),
        )
        .rule(
            UiSelector::new(UiComponent::Navigation),
            UiStyle::new()
                .background(palette.surface_alt)
                .foreground(palette.text_primary),
        )
        .rule(
            UiSelector::new(UiComponent::Surface),
            UiStyle::new()
                .background(palette.surface)
                .foreground(palette.text_primary),
        )
        .rule(
            UiSelector::new(UiComponent::Card),
            UiStyle::new()
                .background(palette.surface_elevated)
                .foreground(palette.text_primary)
                .padding(UiEdges::all(16.0))
                .border_color(palette.border)
                .border_radius(14.0),
        )
        .rule(
            UiSelector::new(UiComponent::Button),
            UiStyle::new()
                .background(palette.accent)
                .foreground(palette.accent_text)
                .border_radius(8.0),
        );
    application = application.theme(theme).styles(styles);
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
        assert_eq!(application.pages().len(), 23);
        assert_eq!(application.initial_page_id(), "home");
        assert_eq!(application.pages()[0].title(), "Home");
        let page = |id: &str| {
            application
                .pages()
                .iter()
                .find(|item| item.id() == id)
                .unwrap()
        };
        assert_eq!(page("wallpaper").title(), "Wallpaper");
        assert_eq!(
            page("wallpaper-store").navigation_label(),
            "  Wallpaper Store"
        );
        assert_eq!(
            page("wallpaper-store-wallpaper-engine").navigation_label(),
            "    Wallpaper Engine"
        );
        assert_eq!(
            page("wallpaper-library").navigation_label(),
            "  Wallpaper Library"
        );
        assert_eq!(page("wallpaper-widgets").navigation_label(), "  Widgets");
        assert_eq!(
            page("wallpaper-performance").navigation_label(),
            "  Resource & Performance"
        );
        assert_eq!(
            page("wallpaper-groups").navigation_label(),
            "  Desktop Groups"
        );
        assert_eq!(
            page("wallpaper-rainmeter").navigation_label(),
            "  Rainmeter"
        );
        assert!(page("icons").description().contains("adaptive application tiles"));
        assert!(application.pages().iter().any(|item| item.id() == "game-mode"));
        assert_eq!(page("settings").title(), "Settings");
        assert_eq!(application.theme_ref().color_scheme(), UiColorScheme::Dark);
        assert_eq!(application.styles_ref().rules().len(), 5);
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
