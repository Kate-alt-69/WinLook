//! Typed object/member registry and request validation.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::capability::{CapabilitySet, CapabilityStatus};
use crate::namespace::NamespacePath;
use crate::objects::{ObjectReference, ObjectReferenceError, ObjectReferenceKind};
use crate::target::WindowsTarget;

/// Semantic operation requested against a registered member.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OperationAction {
    Read,
    Write,
    Invoke,
    Subscribe,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ObjectOwnership {
    Windows,
    WinLook,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgumentPolicy {
    Forbidden,
    Optional,
    Required,
}

/// Values that can cross the typed Traton request boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropertyValue {
    Boolean(bool),
    Integer(i64),
    Text(String),
    Enumeration(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValueSchema {
    Boolean,
    Integer { minimum: i64, maximum: i64 },
    Text { max_bytes: usize },
    Enumeration { values: Vec<String> },
}

impl ValueSchema {
    pub fn validate(&self, value: &PropertyValue) -> Result<(), RegistryError> {
        match (self, value) {
            (Self::Boolean, PropertyValue::Boolean(_)) => Ok(()),
            (Self::Integer { minimum, maximum }, PropertyValue::Integer(value))
                if value >= minimum && value <= maximum =>
            {
                Ok(())
            }
            (Self::Text { max_bytes }, PropertyValue::Text(value)) if value.len() <= *max_bytes => {
                Ok(())
            }
            (Self::Enumeration { values }, PropertyValue::Enumeration(value))
                if values.iter().any(|allowed| allowed == value) =>
            {
                Ok(())
            }
            (Self::Integer { .. }, PropertyValue::Integer(_)) => {
                Err(RegistryError::ValueOutOfRange)
            }
            (Self::Text { .. }, PropertyValue::Text(_)) => Err(RegistryError::ValueOutOfRange),
            (Self::Enumeration { .. }, PropertyValue::Enumeration(_)) => {
                Err(RegistryError::UnknownEnumValue)
            }
            _ => Err(RegistryError::WrongValueType),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectKind {
    Taskbar,
    DisplayMonitor,
    ShellHost,
    Window,
    DesktopItem,
}

/// A registered member is data, not an executable native call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemberDefinition {
    pub path: String,
    pub actions: BTreeSet<OperationAction>,
    pub value_schema: ValueSchema,
    pub argument_policy: ArgumentPolicy,
    pub argument_kinds: BTreeSet<ObjectReferenceKind>,
    pub capability_id: String,
    pub ownership: ObjectOwnership,
    pub minimum_build: Option<u32>,
    pub maximum_build: Option<u32>,
    pub provider_ids: Vec<String>,
    pub experimental: bool,
}

impl MemberDefinition {
    pub fn supports(&self, action: OperationAction) -> bool {
        self.actions.contains(&action)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectDefinition {
    pub subsystem: String,
    pub object: String,
    pub kind: ObjectKind,
    pub members: BTreeMap<String, MemberDefinition>,
}

impl ObjectDefinition {
    pub fn new(
        subsystem: impl Into<String>,
        object: impl Into<String>,
        kind: ObjectKind,
    ) -> Result<Self, RegistryError> {
        let subsystem = subsystem.into();
        let object = object.into();
        validate_segment(&subsystem)?;
        validate_segment(&object)?;
        Ok(Self {
            subsystem,
            object,
            kind,
            members: BTreeMap::new(),
        })
    }

    pub fn register_member(&mut self, member: MemberDefinition) -> Result<(), RegistryError> {
        validate_member_path(&member.path)?;
        validate_identifier(&member.capability_id)
            .map_err(|_| RegistryError::InvalidCapabilityId)?;
        if member.provider_ids.is_empty() {
            return Err(RegistryError::MemberHasNoProviderCandidates);
        }
        for provider_id in &member.provider_ids {
            validate_identifier(provider_id).map_err(|_| RegistryError::InvalidProviderId)?;
        }
        if member
            .minimum_build
            .zip(member.maximum_build)
            .is_some_and(|(min, max)| min > max)
        {
            return Err(RegistryError::InvalidBuildRange);
        }
        if self.members.contains_key(&member.path) {
            return Err(RegistryError::DuplicateMember);
        }
        self.members.insert(member.path.clone(), member);
        Ok(())
    }

    pub fn key(&self) -> String {
        format!("{}.{}", self.subsystem, self.object)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ObjectRegistry {
    objects: BTreeMap<String, ObjectDefinition>,
}

impl ObjectRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_object(&mut self, object: ObjectDefinition) -> Result<(), RegistryError> {
        let key = object.key();
        if self.objects.contains_key(&key) {
            return Err(RegistryError::DuplicateObject);
        }
        self.objects.insert(key, object);
        Ok(())
    }

    pub fn object(&self, subsystem: &str, object: &str) -> Option<&ObjectDefinition> {
        self.objects.get(&format!("{subsystem}.{object}"))
    }

    pub fn object_count(&self) -> usize {
        self.objects.len()
    }

    /// Creates the first typed catalogue. Capabilities remain Unknown until a provider probes.
    pub fn standard() -> Self {
        let mut registry = Self::new();

        let mut taskbar =
            ObjectDefinition::new("shell", "taskbar", ObjectKind::Taskbar).expect("static object");
        taskbar
            .register_member(MemberDefinition {
                path: "surface.anchor".to_owned(),
                actions: [OperationAction::Read, OperationAction::Write]
                    .into_iter()
                    .collect(),
                value_schema: ValueSchema::Enumeration {
                    values: ["top", "bottom", "left", "right", "free"]
                        .into_iter()
                        .map(str::to_owned)
                        .collect(),
                },
                argument_policy: ArgumentPolicy::Forbidden,
                argument_kinds: BTreeSet::new(),
                capability_id: "wlde.taskbar.surface".to_owned(),
                ownership: ObjectOwnership::WinLook,
                minimum_build: Some(10_240),
                maximum_build: None,
                provider_ids: vec!["wlde.taskbar.surface".to_owned()],
                experimental: false,
            })
            .expect("static member");
        taskbar
            .register_member(MemberDefinition {
                path: "items.visibility".to_owned(),
                actions: [OperationAction::Read, OperationAction::Write]
                    .into_iter()
                    .collect(),
                value_schema: ValueSchema::Enumeration {
                    values: ["visible", "hidden"]
                        .into_iter()
                        .map(str::to_owned)
                        .collect(),
                },
                argument_policy: ArgumentPolicy::Required,
                argument_kinds: [
                    ObjectReferenceKind::PinnedShortcut,
                    ObjectReferenceKind::RunningWindow,
                    ObjectReferenceKind::SystemButton,
                    ObjectReferenceKind::NotificationAreaIcon,
                ]
                .into_iter()
                .collect(),
                capability_id: "shell.taskbar.item-visibility".to_owned(),
                ownership: ObjectOwnership::Windows,
                minimum_build: Some(19_045),
                maximum_build: None,
                provider_ids: vec!["shell.taskbar.visibility.verified".to_owned()],
                experimental: true,
            })
            .expect("static member");
        taskbar
            .register_member(MemberDefinition {
                path: crate::taskbar::TASKBAR_WINDOW_COUNT_MEMBER.to_owned(),
                actions: [OperationAction::Read].into_iter().collect(),
                value_schema: ValueSchema::Integer {
                    minimum: 1,
                    maximum: crate::taskbar::MAX_TASKBAR_WINDOWS as i64,
                },
                argument_policy: ArgumentPolicy::Forbidden,
                argument_kinds: BTreeSet::new(),
                capability_id: crate::taskbar::TASKBAR_WINDOW_DISCOVERY_CAPABILITY_ID.to_owned(),
                ownership: ObjectOwnership::Windows,
                minimum_build: Some(19_045),
                maximum_build: Some(19_045),
                provider_ids: vec![crate::taskbar::TASKBAR_WINDOW_DISCOVERY_PROVIDER_ID.to_owned()],
                experimental: false,
            })
            .expect("static member");
        registry.register_object(taskbar).expect("static object");

        let mut display = ObjectDefinition::new("display", "monitor", ObjectKind::DisplayMonitor)
            .expect("static object");
        display
            .register_member(MemberDefinition {
                path: "topology".to_owned(),
                actions: [OperationAction::Read].into_iter().collect(),
                value_schema: ValueSchema::Integer {
                    minimum: 0,
                    maximum: 16,
                },
                argument_policy: ArgumentPolicy::Forbidden,
                argument_kinds: BTreeSet::new(),
                capability_id: "display.monitor.topology".to_owned(),
                ownership: ObjectOwnership::Windows,
                minimum_build: Some(10_240),
                maximum_build: None,
                provider_ids: vec!["wl_wlde_win32.monitor-topology".to_owned()],
                experimental: false,
            })
            .expect("static member");
        registry.register_object(display).expect("static object");

        registry
    }

    /// Resolves a parsed path into a typed operation only when build, argument,
    /// value, permission and capability checks all succeed.
    pub fn validate(
        &self,
        path: &NamespacePath,
        action: OperationAction,
        value: Option<PropertyValue>,
        target: &WindowsTarget,
        capabilities: &CapabilitySet,
        allow_experimental: bool,
    ) -> Result<ValidatedOperation, RegistryError> {
        if !target.matches_selector(path.target()) {
            return Err(RegistryError::TargetMismatch);
        }

        let object = self
            .object(path.subsystem(), path.object())
            .ok_or(RegistryError::UnknownObject)?;
        let member_path = path.sub_properties().join(".");
        if member_path.is_empty() {
            return Err(RegistryError::MissingMember);
        }
        let member = object
            .members
            .get(&member_path)
            .ok_or(RegistryError::UnknownMember)?;

        if !member.supports(action) {
            return Err(RegistryError::ActionNotAllowed);
        }
        if member
            .minimum_build
            .is_some_and(|minimum| target.build_number() < minimum)
            || member
                .maximum_build
                .is_some_and(|maximum| target.build_number() > maximum)
        {
            return Err(RegistryError::BuildNotSupported);
        }
        if member.experimental && !allow_experimental {
            return Err(RegistryError::ExperimentalConsentRequired);
        }

        let argument = path
            .argument()
            .map(ObjectReference::parse)
            .transpose()
            .map_err(RegistryError::InvalidArgument)?;
        match (member.argument_policy, argument.as_ref()) {
            (ArgumentPolicy::Forbidden, Some(_)) | (ArgumentPolicy::Required, None) => {
                return Err(RegistryError::ArgumentPolicyMismatch);
            }
            (_, Some(reference)) if !member.argument_kinds.contains(&reference.kind()) => {
                return Err(RegistryError::ArgumentKindNotAllowed);
            }
            _ => {}
        }

        match action {
            OperationAction::Write => {
                let value_ref = value.as_ref().ok_or(RegistryError::MissingValue)?;
                member.value_schema.validate(value_ref)?;
            }
            _ if value.is_some() => return Err(RegistryError::UnexpectedValue),
            _ => {}
        }

        let capability_status = capabilities.status(&member.capability_id);
        if !capability_status.permits(allow_experimental) {
            return Err(RegistryError::CapabilityUnavailable {
                capability_id: member.capability_id.clone(),
                status: capability_status,
            });
        }

        Ok(ValidatedOperation {
            path: path.clone(),
            action,
            value,
            argument,
            capability_id: member.capability_id.clone(),
            ownership: member.ownership,
            provider_ids: member.provider_ids.clone(),
            target_build: target.build_number(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedOperation {
    pub path: NamespacePath,
    pub action: OperationAction,
    pub value: Option<PropertyValue>,
    pub argument: Option<ObjectReference>,
    pub capability_id: String,
    pub ownership: ObjectOwnership,
    pub provider_ids: Vec<String>,
    pub target_build: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegistryError {
    InvalidObjectSegment,
    InvalidMemberPath,
    InvalidCapabilityId,
    InvalidProviderId,
    InvalidBuildRange,
    DuplicateObject,
    DuplicateMember,
    MemberHasNoProviderCandidates,
    TargetMismatch,
    UnknownObject,
    MissingMember,
    UnknownMember,
    ActionNotAllowed,
    BuildNotSupported,
    ExperimentalConsentRequired,
    ArgumentPolicyMismatch,
    ArgumentKindNotAllowed,
    InvalidArgument(ObjectReferenceError),
    MissingValue,
    UnexpectedValue,
    WrongValueType,
    ValueOutOfRange,
    UnknownEnumValue,
    CapabilityUnavailable {
        capability_id: String,
        status: CapabilityStatus,
    },
}

impl fmt::Display for RegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidObjectSegment => {
                formatter.write_str("object identifiers must be lowercase ASCII segments")
            }
            Self::InvalidMemberPath => {
                formatter.write_str("member paths must contain lowercase ASCII segments")
            }
            Self::InvalidCapabilityId => formatter.write_str("capability identifier is invalid"),
            Self::InvalidProviderId => formatter.write_str("provider identifier is invalid"),
            Self::InvalidBuildRange => formatter.write_str("member build range is inverted"),
            Self::DuplicateObject => formatter.write_str("object definition is already registered"),
            Self::DuplicateMember => formatter.write_str("member definition is already registered"),
            Self::MemberHasNoProviderCandidates => {
                formatter.write_str("member definition has no provider candidates")
            }
            Self::TargetMismatch => formatter
                .write_str("path release selector does not match the detected Windows target"),
            Self::UnknownObject => formatter.write_str("object is not registered"),
            Self::MissingMember => {
                formatter.write_str("path does not select a property or callable member")
            }
            Self::UnknownMember => {
                formatter.write_str("property or callable member is not registered")
            }
            Self::ActionNotAllowed => {
                formatter.write_str("requested action is not allowed for this member")
            }
            Self::BuildNotSupported => {
                formatter.write_str("member is outside its declared Windows build range")
            }
            Self::ExperimentalConsentRequired => {
                formatter.write_str("this member requires explicit experimental consent")
            }
            Self::ArgumentPolicyMismatch => {
                formatter.write_str("member argument does not match its required/forbidden policy")
            }
            Self::ArgumentKindNotAllowed => {
                formatter.write_str("object argument kind is not allowed for this member")
            }
            Self::InvalidArgument(error) => {
                write!(formatter, "invalid typed object argument: {error}")
            }
            Self::MissingValue => formatter.write_str("write operation is missing its typed value"),
            Self::UnexpectedValue => {
                formatter.write_str("this operation does not accept a property value")
            }
            Self::WrongValueType => {
                formatter.write_str("property value does not match the member schema")
            }
            Self::ValueOutOfRange => {
                formatter.write_str("property value is outside the member's declared range")
            }
            Self::UnknownEnumValue => {
                formatter.write_str("enumeration value is not registered for this member")
            }
            Self::CapabilityUnavailable {
                capability_id,
                status,
            } => {
                write!(
                    formatter,
                    "capability {capability_id} is not available ({status:?})"
                )
            }
        }
    }
}

impl std::error::Error for RegistryError {}

fn validate_segment(value: &str) -> Result<(), RegistryError> {
    if value.is_empty()
        || !value.as_bytes()[0].is_ascii_lowercase()
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'_'
        })
    {
        return Err(RegistryError::InvalidObjectSegment);
    }
    Ok(())
}

fn validate_member_path(value: &str) -> Result<(), RegistryError> {
    if value.is_empty() {
        return Err(RegistryError::InvalidMemberPath);
    }
    for segment in value.split('.') {
        validate_segment(segment).map_err(|_| RegistryError::InvalidMemberPath)?;
    }
    Ok(())
}

fn validate_identifier(value: &str) -> Result<(), ()> {
    if value.is_empty()
        || value.len() > 128
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        })
    {
        return Err(());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ObjectRegistry, OperationAction, PropertyValue, RegistryError};
    use crate::capability::{CapabilityRecord, CapabilitySet, CapabilityStatus};
    use crate::namespace::NamespacePath;
    use crate::objects::ObjectReferenceKind;
    use crate::target::{Architecture, WindowsTarget, WindowsTargetInfo};

    fn target(build: u32) -> WindowsTarget {
        WindowsTarget::from_info(WindowsTargetInfo {
            product_name: Some("Windows 10 Pro".to_owned()),
            display_version: Some("22H2".to_owned()),
            edition: None,
            installation_type: None,
            build_number: build,
            update_build_revision: Some(1),
            process_architecture: Architecture::X64,
            system_architecture: Architecture::X64,
            process_id: 1,
            session_id: 1,
        })
        .unwrap()
    }

    #[test]
    fn resolves_an_item_scoped_member_only_after_capability_is_available() {
        let registry = ObjectRegistry::standard();
        let path = NamespacePath::parse(
            "windows.10-22h2.shell.taskbar.items.visibility(pinned-shortcut:item-42)",
        )
        .unwrap();
        let mut capabilities = CapabilitySet::new();
        assert!(matches!(
            registry.validate(
                &path,
                OperationAction::Write,
                Some(PropertyValue::Enumeration("hidden".to_owned())),
                &target(19_045),
                &capabilities,
                true
            ),
            Err(RegistryError::CapabilityUnavailable { .. })
        ));
        capabilities
            .set(
                "shell.taskbar.item-visibility",
                CapabilityRecord {
                    status: CapabilityStatus::Experimental,
                    provider_id: None,
                    reason: Some("no native build adapter has shipped".to_owned()),
                },
            )
            .unwrap();
        let resolved = registry
            .validate(
                &path,
                OperationAction::Write,
                Some(PropertyValue::Enumeration("hidden".to_owned())),
                &target(19_045),
                &capabilities,
                true,
            )
            .unwrap();
        assert_eq!(
            resolved.argument.as_ref().unwrap().kind(),
            ObjectReferenceKind::PinnedShortcut
        );
    }

    #[test]
    fn unknown_or_other_release_selectors_do_not_resolve() {
        let registry = ObjectRegistry::standard();
        let path = NamespacePath::parse("windows.11.shell.taskbar.surface.anchor").unwrap();
        assert_eq!(
            registry.validate(
                &path,
                OperationAction::Read,
                None,
                &target(19_045),
                &CapabilitySet::new(),
                false
            ),
            Err(RegistryError::TargetMismatch)
        );
    }

    #[test]
    fn member_build_ranges_are_enforced() {
        let registry = ObjectRegistry::standard();
        let path = NamespacePath::parse(
            "windows.10-22h2.shell.taskbar.items.visibility(running-window:window-1)",
        )
        .unwrap();
        let mut capabilities = CapabilitySet::new();
        capabilities
            .set(
                "shell.taskbar.item-visibility",
                CapabilityRecord {
                    status: CapabilityStatus::Available,
                    provider_id: None,
                    reason: None,
                },
            )
            .unwrap();
        assert_eq!(
            registry.validate(
                &path,
                OperationAction::Read,
                None,
                &target(19_044),
                &capabilities,
                true
            ),
            Err(RegistryError::BuildNotSupported)
        );
    }
}
