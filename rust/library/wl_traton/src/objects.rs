//! Typed, durable object references. Native handles never cross this boundary.

use std::fmt;

/// Windows/WinLook object category encoded in an item argument.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ObjectReferenceKind {
    PinnedShortcut,
    RunningWindow,
    SystemButton,
    NotificationAreaIcon,
    Monitor,
    Window,
    DesktopItem,
    ExplorerHost,
}

impl ObjectReferenceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PinnedShortcut => "pinned-shortcut",
            Self::RunningWindow => "running-window",
            Self::SystemButton => "system-button",
            Self::NotificationAreaIcon => "notification-icon",
            Self::Monitor => "monitor",
            Self::Window => "window",
            Self::DesktopItem => "desktop-item",
            Self::ExplorerHost => "explorer-host",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "pinned-shortcut" => Some(Self::PinnedShortcut),
            "running-window" => Some(Self::RunningWindow),
            "system-button" => Some(Self::SystemButton),
            "notification-icon" => Some(Self::NotificationAreaIcon),
            "monitor" => Some(Self::Monitor),
            "window" => Some(Self::Window),
            "desktop-item" => Some(Self::DesktopItem),
            "explorer-host" => Some(Self::ExplorerHost),
            _ => None,
        }
    }
}

/// Opaque stable identifier for one object in a particular runtime generation.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectReference {
    kind: ObjectReferenceKind,
    stable_id: String,
}

impl ObjectReference {
    pub fn new(
        kind: ObjectReferenceKind,
        stable_id: impl Into<String>,
    ) -> Result<Self, ObjectReferenceError> {
        let stable_id = stable_id.into();
        validate_stable_id(&stable_id)?;
        Ok(Self { kind, stable_id })
    }

    /// Parses an argument such as pinned-shortcut:item-42.
    pub fn parse(value: &str) -> Result<Self, ObjectReferenceError> {
        let (kind, stable_id) = value
            .split_once(':')
            .ok_or(ObjectReferenceError::MissingTypeSeparator)?;
        let kind = ObjectReferenceKind::parse(kind).ok_or(ObjectReferenceError::UnknownKind)?;
        Self::new(kind, stable_id)
    }

    pub fn kind(&self) -> ObjectReferenceKind {
        self.kind
    }

    pub fn stable_id(&self) -> &str {
        &self.stable_id
    }
}

impl fmt::Display for ObjectReference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.kind.as_str(), self.stable_id)
    }
}

fn validate_stable_id(value: &str) -> Result<(), ObjectReferenceError> {
    if value.is_empty() {
        return Err(ObjectReferenceError::EmptyId);
    }
    if value.len() > 128 {
        return Err(ObjectReferenceError::IdTooLong);
    }
    if !value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(ObjectReferenceError::InvalidId);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectReferenceError {
    MissingTypeSeparator,
    UnknownKind,
    EmptyId,
    IdTooLong,
    InvalidId,
}

impl fmt::Display for ObjectReferenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MissingTypeSeparator => "object references must include a kind prefix and colon",
            Self::UnknownKind => "object reference kind is not registered",
            Self::EmptyId => "object reference stable ID is empty",
            Self::IdTooLong => "object reference stable ID exceeds 128 bytes",
            Self::InvalidId => "object reference stable ID contains unsupported characters",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ObjectReferenceError {}

#[cfg(test)]
mod tests {
    use super::{ObjectReference, ObjectReferenceError, ObjectReferenceKind};

    #[test]
    fn parses_item_scoped_references() {
        let reference = ObjectReference::parse("pinned-shortcut:item-42").unwrap();
        assert_eq!(reference.kind(), ObjectReferenceKind::PinnedShortcut);
        assert_eq!(reference.stable_id(), "item-42");
        assert_eq!(reference.to_string(), "pinned-shortcut:item-42");
    }

    #[test]
    fn rejects_handle_like_or_path_like_identifiers() {
        assert_eq!(
            ObjectReference::parse("window:0x1234"),
            Err(ObjectReferenceError::InvalidId)
        );
        assert_eq!(
            ObjectReference::parse("window:C:\\Windows"),
            Err(ObjectReferenceError::InvalidId)
        );
    }
}
