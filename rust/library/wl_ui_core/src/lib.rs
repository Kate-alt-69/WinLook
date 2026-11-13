#![forbid(unsafe_code)]

//! Platform-neutral UI application contracts.
//!
//! This crate has no windowing, rendering, WebView, or operating-system dependencies.
//! Native and Tauri adapters consume the same application model.

mod application;
mod error;
mod page;
mod window;

pub use application::UiApplication;
pub use error::UiError;
pub use page::UiPage;
pub use window::UiWindowConfig;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_a_complete_application() {
        let app = UiApplication::new("Example")
            .page(UiPage::new("home", "Home", "Application overview"))
            .page(UiPage::new("settings", "Settings", "Application preferences"))
            .initial_page("settings");

        assert!(app.validate().is_ok());
        assert_eq!(app.initial_page_id(), "settings");
        assert_eq!(app.pages().len(), 2);
    }

    #[test]
    fn rejects_duplicate_page_ids() {
        let app = UiApplication::new("Example")
            .page(UiPage::new("home", "Home", "First"))
            .page(UiPage::new("home", "Home again", "Second"));

        assert!(matches!(
            app.validate(),
            Err(UiError::DuplicatePageId(id)) if id == "home"
        ));
    }

    #[test]
    fn rejects_an_unknown_initial_page() {
        let app = UiApplication::new("Example")
            .page(UiPage::new("home", "Home", "Overview"))
            .initial_page("missing");

        assert!(matches!(
            app.validate(),
            Err(UiError::UnknownInitialPage(id)) if id == "missing"
        ));
    }

    #[test]
    fn rejects_empty_applications_and_invalid_window_sizes() {
        assert_eq!(UiApplication::new("Example").validate(), Err(UiError::NoPages));

        let invalid = UiApplication::new("Example")
            .window(UiWindowConfig::new("Example").size(0, 0))
            .page(UiPage::new("home", "Home", "Overview"));

        assert_eq!(invalid.validate(), Err(UiError::InvalidWindowSize));
    }

    #[test]
    fn defaults_initial_page_to_first_registered_page() {
        let app = UiApplication::new("Example")
            .page(UiPage::new("home", "Home", "Overview"))
            .page(UiPage::new("settings", "Settings", "Preferences"));

        assert_eq!(app.initial_page_id(), "home");
    }
}
