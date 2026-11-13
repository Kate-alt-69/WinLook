/// A page entry shared by every wl_ui rendering backend.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiPage {
    id: String,
    title: String,
    description: String,
}

impl UiPage {
    /// Creates a page with a stable ID, a visible title, and a short description.
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}
