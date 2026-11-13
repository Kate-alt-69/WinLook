use std::fmt;

/// Errors returned by the shared UI model, styling system or a selected backend.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiError {
    EmptyWindowTitle,
    InvalidWindowSize,
    NoPages,
    EmptyPageId,
    EmptyPageTitle,
    DuplicatePageId(String),
    UnknownInitialPage(String),
    InvalidStyle(String),
    Backend(String),
}

impl fmt::Display for UiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyWindowTitle => formatter.write_str("the window title cannot be empty"),
            Self::InvalidWindowSize => formatter.write_str("the window must be at least 320 × 240"),
            Self::NoPages => formatter.write_str("an application must register at least one page"),
            Self::EmptyPageId => formatter.write_str("page IDs cannot be empty"),
            Self::EmptyPageTitle => formatter.write_str("page titles cannot be empty"),
            Self::DuplicatePageId(id) => write!(formatter, "duplicate page ID: {id}"),
            Self::UnknownInitialPage(id) => {
                write!(formatter, "initial page ID does not exist: {id}")
            }
            Self::InvalidStyle(message) => write!(formatter, "invalid UI style: {message}"),
            Self::Backend(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for UiError {}
