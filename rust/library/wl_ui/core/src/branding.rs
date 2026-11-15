use crate::UiError;

/// Optional application identity shared by native and CSS-capable renderers.
///
/// The mark is a short monogram (for example, "WL"), not a filesystem image path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiBranding {
    title: String,
    mark: String,
    tagline: String,
}

impl UiBranding {
    /// Creates brand metadata with a visible application title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            mark: String::new(),
            tagline: String::new(),
        }
    }

    /// Sets a short text mark suitable for a native badge or logo tile.
    pub fn mark(mut self, value: impl Into<String>) -> Self {
        self.mark = value.into();
        self
    }

    /// Sets the supporting brand line shown beneath the application title.
    pub fn tagline(mut self, value: impl Into<String>) -> Self {
        self.tagline = value.into();
        self
    }

    pub fn title(&self) -> &str { &self.title }
    pub fn mark_text(&self) -> &str { &self.mark }
    pub fn tagline_text(&self) -> &str { &self.tagline }

    pub(crate) fn validate(&self) -> Result<(), UiError> {
        if self.title.trim().is_empty() || self.title.chars().count() > 128 {
            return Err(UiError::InvalidBranding(
                "brand title must contain 1 to 128 characters".to_string(),
            ));
        }
        if self.mark.chars().count() > 8 || self.mark.chars().any(char::is_control) {
            return Err(UiError::InvalidBranding(
                "brand mark must contain at most 8 non-control characters".to_string(),
            ));
        }
        if self.tagline.chars().count() > 256 || self.tagline.chars().any(char::is_control) {
            return Err(UiError::InvalidBranding(
                "brand tagline must contain at most 256 non-control characters".to_string(),
            ));
        }
        Ok(())
    }
}
