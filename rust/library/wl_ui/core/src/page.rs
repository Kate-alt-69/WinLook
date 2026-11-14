/// A page entry shared by every wl_ui rendering backend.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiPage {
    id: String,
    title: String,
    navigation_title: String,
    description: String,
}

impl UiPage {
    /// Creates a page with a stable ID, a visible title, and a short description.
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let title = title.into();
        Self {
            id: id.into(),
            navigation_title: title.clone(),
            title,
            description: description.into(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    /// Sets the concise label rendered in navigation without changing the page heading.
    pub fn navigation_title(mut self, value: impl Into<String>) -> Self {
        self.navigation_title = value.into();
        self
    }

    pub fn navigation_label(&self) -> &str {
        &self.navigation_title
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}
