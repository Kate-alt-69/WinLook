use crate::{UiError, UiPage, UiWindowConfig};

/// A platform-independent description of an application's top-level UI shell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiApplication {
    window: UiWindowConfig,
    pages: Vec<UiPage>,
    initial_page: Option<String>,
}

impl UiApplication {
    /// Creates an application shell with a 1280 × 820 logical window.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            window: UiWindowConfig::new(title),
            pages: Vec::new(),
            initial_page: None,
        }
    }

    /// Replaces the window configuration.
    pub fn window(mut self, window: UiWindowConfig) -> Self {
        self.window = window;
        self
    }

    /// Adds a navigation page. Page IDs must be unique within the application.
    pub fn page(mut self, page: UiPage) -> Self {
        self.pages.push(page);
        self
    }

    /// Selects the initial page by its stable ID.
    ///
    /// When omitted, the first page registered with page() is selected.
    pub fn initial_page(mut self, page_id: impl Into<String>) -> Self {
        self.initial_page = Some(page_id.into());
        self
    }

    /// Returns the window configuration.
    pub fn window_config(&self) -> &UiWindowConfig {
        &self.window
    }

    /// Returns pages in their navigation order.
    pub fn pages(&self) -> &[UiPage] {
        &self.pages
    }

    /// Returns the selected page ID, defaulting to the first registered page.
    pub fn initial_page_id(&self) -> &str {
        self.initial_page
            .as_deref()
            .or_else(|| self.pages.first().map(UiPage::id))
            .unwrap_or("")
    }

    /// Checks the application model before a backend creates native resources.
    pub fn validate(&self) -> Result<(), UiError> {
        if self.window.title().trim().is_empty() {
            return Err(UiError::EmptyWindowTitle);
        }
        if self.window.width() < 320 || self.window.height() < 240 {
            return Err(UiError::InvalidWindowSize);
        }
        if self.pages.is_empty() {
            return Err(UiError::NoPages);
        }

        let mut ids = std::collections::HashSet::with_capacity(self.pages.len());
        for page in &self.pages {
            if page.id().trim().is_empty() {
                return Err(UiError::EmptyPageId);
            }
            if page.title().trim().is_empty() {
                return Err(UiError::EmptyPageTitle);
            }
            if !ids.insert(page.id()) {
                return Err(UiError::DuplicatePageId(page.id().to_string()));
            }
        }

        let initial_page = self.initial_page_id();
        if !ids.contains(initial_page) {
            return Err(UiError::UnknownInitialPage(initial_page.to_string()));
        }

        Ok(())
    }
}
