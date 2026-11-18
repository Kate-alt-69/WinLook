//! Parsing and representation of canonical Traton namespace paths.

use std::fmt;
use std::str::FromStr;

/// A parsed, version-aware Windows namespace path.
///
/// Canonical shape:
///
/// `windows.<release>.<subsystem>.<object>[.<member>...][(<argument>)]`
///
/// The argument is an opaque, typed token. For item-scoped taskbar members,
/// use a prefix such as `pinned-shortcut:` or `running-window:`. Resolution
/// into a live Windows object is deliberately handled by a later registry and
/// provider layer; parsing alone does not imply that a target is supported.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NamespacePath {
    target: String,
    members: Vec<String>,
    argument: Option<String>,
}

impl NamespacePath {
    /// Parse a canonical Traton namespace path.
    pub fn parse(input: &str) -> Result<Self, NamespacePathError> {
        input.parse()
    }

    /// Return the release selector, for example `10-22h2`.
    pub fn target(&self) -> &str {
        &self.target
    }

    /// Return every dotted member after the Windows release selector.
    ///
    /// The first member is the subsystem and the second is the object.
    /// Remaining members are object sub-properties or callable selectors.
    pub fn members(&self) -> &[String] {
        &self.members
    }

    /// Return the subsystem component, such as `shell`.
    pub fn subsystem(&self) -> &str {
        // The parser guarantees at least a subsystem and an object component.
        &self.members[0]
    }

    /// Return the object component, such as `taskbar`.
    pub fn object(&self) -> &str {
        // The parser guarantees at least a subsystem and an object component.
        &self.members[1]
    }

    /// Return the optional sub-property/callable path after the object.
    pub fn sub_properties(&self) -> &[String] {
        &self.members[2..]
    }

    /// Return the optional opaque argument.
    ///
    /// The namespace registry determines the expected argument type for a
    /// particular member path. The parser does not treat this as a command.
    pub fn argument(&self) -> Option<&str> {
        self.argument.as_deref()
    }

    /// Rebuild the canonical string representation.
    pub fn to_canonical_string(&self) -> String {
        self.to_string()
    }
}

impl FromStr for NamespacePath {
    type Err = NamespacePathError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.is_empty() {
            return Err(NamespacePathError::EmptyInput);
        }

        if input.bytes().any(|byte| byte.is_ascii_whitespace()) {
            return Err(NamespacePathError::WhitespaceNotAllowed);
        }

        let (path, argument) = split_argument(input)?;
        let components: Vec<&str> = path.split('.').collect();

        if components.first().copied() != Some("windows") {
            return Err(NamespacePathError::InvalidRoot);
        }

        if components.len() < 2 || components[1].is_empty() {
            return Err(NamespacePathError::MissingTarget);
        }

        if !is_valid_target(components[1]) {
            return Err(NamespacePathError::InvalidTarget);
        }

        if components.len() < 4 {
            return Err(NamespacePathError::MissingObjectPath);
        }

        if components[2..].iter().any(|component| component.is_empty()) {
            return Err(NamespacePathError::EmptyMember);
        }

        if components[2..]
            .iter()
            .any(|component| !is_valid_member(component))
        {
            return Err(NamespacePathError::InvalidMember);
        }

        Ok(Self {
            target: components[1].to_owned(),
            members: components[2..]
                .iter()
                .map(|component| (*component).to_owned())
                .collect(),
            argument,
        })
    }
}

impl fmt::Display for NamespacePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("windows.")?;
        formatter.write_str(&self.target)?;

        for member in &self.members {
            formatter.write_str(".")?;
            formatter.write_str(member)?;
        }

        if let Some(argument) = &self.argument {
            formatter.write_str("(")?;
            formatter.write_str(argument)?;
            formatter.write_str(")")?;
        }

        Ok(())
    }
}

/// Errors returned when a Traton namespace path is malformed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NamespacePathError {
    EmptyInput,
    InvalidRoot,
    MissingTarget,
    InvalidTarget,
    MissingObjectPath,
    EmptyMember,
    InvalidMember,
    UnexpectedClosingParenthesis,
    UnclosedArgument,
    NestedArgument,
    EmptyArgument,
    InvalidArgument,
    WhitespaceNotAllowed,
}

impl fmt::Display for NamespacePathError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::EmptyInput => "namespace path is empty",
            Self::InvalidRoot => "namespace path must start with windows",
            Self::MissingTarget => "namespace path is missing its Windows release selector",
            Self::InvalidTarget => "Windows release selector contains invalid characters",
            Self::MissingObjectPath => "namespace path must include a subsystem and an object",
            Self::EmptyMember => "namespace path contains an empty dotted member",
            Self::InvalidMember => {
                "namespace members must be lowercase identifiers separated by dots"
            }
            Self::UnexpectedClosingParenthesis => {
                "namespace path contains an unexpected closing parenthesis"
            }
            Self::UnclosedArgument => "namespace path has an unclosed argument",
            Self::NestedArgument => "namespace path arguments cannot contain nested parentheses",
            Self::EmptyArgument => "namespace path argument is empty",
            Self::InvalidArgument => {
                "namespace argument must be a non-empty opaque identifier token"
            }
            Self::WhitespaceNotAllowed => "whitespace is not allowed in a canonical namespace path",
        };

        formatter.write_str(message)
    }
}

impl std::error::Error for NamespacePathError {}

fn split_argument(input: &str) -> Result<(&str, Option<String>), NamespacePathError> {
    match input.find('(') {
        Some(open_index) => {
            if !input.ends_with(')') {
                return Err(NamespacePathError::UnclosedArgument);
            }

            let argument = &input[open_index + 1..input.len() - 1];

            if argument.contains('(') || argument.contains(')') {
                return Err(NamespacePathError::NestedArgument);
            }

            if argument.is_empty() {
                return Err(NamespacePathError::EmptyArgument);
            }

            if !is_valid_argument(argument) {
                return Err(NamespacePathError::InvalidArgument);
            }

            Ok((&input[..open_index], Some(argument.to_owned())))
        }
        None if input.contains(')') => Err(NamespacePathError::UnexpectedClosingParenthesis),
        None => Ok((input, None)),
    }
}

fn is_valid_target(value: &str) -> bool {
    let mut bytes = value.bytes();

    matches!(bytes.next(), Some(byte) if byte.is_ascii_digit())
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn is_valid_member(value: &str) -> bool {
    let mut bytes = value.bytes();

    matches!(bytes.next(), Some(byte) if byte.is_ascii_lowercase())
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn is_valid_argument(value: &str) -> bool {
    let mut bytes = value.bytes();

    matches!(bytes.next(), Some(byte) if byte.is_ascii_alphanumeric())
        && bytes
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

#[cfg(test)]
mod tests {
    use super::{NamespacePath, NamespacePathError};

    #[test]
    fn parses_an_object_path() {
        let path = NamespacePath::parse("windows.10-22h2.shell.taskbar").unwrap();

        assert_eq!(path.target(), "10-22h2");
        assert_eq!(path.subsystem(), "shell");
        assert_eq!(path.object(), "taskbar");
        assert!(path.sub_properties().is_empty());
        assert_eq!(path.argument(), None);
    }

    #[test]
    fn parses_nested_sub_properties_and_a_typed_item_argument() {
        let source = "windows.10-22h2.shell.taskbar.items.visibility(pinned-shortcut:item-42)";
        let path = NamespacePath::parse(source).unwrap();

        assert_eq!(path.subsystem(), "shell");
        assert_eq!(path.object(), "taskbar");
        assert_eq!(
            path.sub_properties()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec!["items", "visibility"]
        );
        assert_eq!(path.argument(), Some("pinned-shortcut:item-42"));
        assert_eq!(path.to_canonical_string(), source);
    }

    #[test]
    fn parses_a_surface_property_without_an_item_argument() {
        let path = NamespacePath::parse("windows.10-22h2.shell.taskbar.surface.anchor").unwrap();

        assert_eq!(
            path.sub_properties()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec!["surface", "anchor"]
        );
        assert_eq!(path.argument(), None);
    }

    #[test]
    fn accepts_a_running_window_argument_without_persisting_a_handle() {
        let path = NamespacePath::parse(
            "windows.10-22h2.shell.taskbar.items.visibility(running-window:window-group-9)",
        )
        .unwrap();

        assert_eq!(path.argument(), Some("running-window:window-group-9"));
    }

    #[test]
    fn rejects_malformed_roots_and_object_paths() {
        assert_eq!(
            NamespacePath::parse("linux.10-22h2.shell.taskbar"),
            Err(NamespacePathError::InvalidRoot)
        );
        assert_eq!(
            NamespacePath::parse("windows.10-22h2.shell"),
            Err(NamespacePathError::MissingObjectPath)
        );
    }

    #[test]
    fn rejects_empty_or_invalid_members() {
        assert_eq!(
            NamespacePath::parse("windows.10-22h2.shell..taskbar"),
            Err(NamespacePathError::EmptyMember)
        );
        assert_eq!(
            NamespacePath::parse("windows.10-22h2.shell.Taskbar"),
            Err(NamespacePathError::InvalidMember)
        );
    }

    #[test]
    fn rejects_invalid_arguments_and_delimiters() {
        assert_eq!(
            NamespacePath::parse("windows.10-22h2.shell.taskbar.items.visibility()"),
            Err(NamespacePathError::EmptyArgument)
        );
        assert_eq!(
            NamespacePath::parse("windows.10-22h2.shell.taskbar.items.visibility(item/42)"),
            Err(NamespacePathError::InvalidArgument)
        );
        assert_eq!(
            NamespacePath::parse("windows.10-22h2.shell.taskbar.items.visibility(item-42"),
            Err(NamespacePathError::UnclosedArgument)
        );
        assert_eq!(
            NamespacePath::parse("windows.10-22h2.shell.taskbar.items.visibility(item-(42))"),
            Err(NamespacePathError::NestedArgument)
        );
    }

    #[test]
    fn rejects_whitespace_in_canonical_paths() {
        assert_eq!(
            NamespacePath::parse("windows.10-22h2.shell.taskbar.items.visibility(item 42)"),
            Err(NamespacePathError::WhitespaceNotAllowed)
        );
    }
}
