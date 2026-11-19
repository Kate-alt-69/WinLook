//! Provider metadata, selection and reversible-operation contract.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::capability::CapabilityStatus;
use crate::registry::{OperationAction, ObjectOwnership, PropertyValue, ValidatedOperation};
use crate::target::WindowsTarget;

pub const MAX_RESTORE_PAYLOAD_BYTES: usize = 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProviderKind {
    DocumentedApi,
    VerifiedAdapter,
    WinLookSurface,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderDescriptor {
    pub id: String,
    pub kind: ProviderKind,
    pub ownership: ObjectOwnership,
    pub actions: BTreeSet<OperationAction>,
    pub minimum_build: Option<u32>,
    pub maximum_build: Option<u32>,
    pub requires_consent: bool,
}

impl ProviderDescriptor {
    pub fn supports(&self, action: OperationAction, target: &WindowsTarget) -> bool {
        self.actions.contains(&action)
            && self
                .minimum_build
                .map_or(true, |minimum| target.build_number() >= minimum)
            && self
                .maximum_build
                .map_or(true, |maximum| target.build_number() <= maximum)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProviderCatalog {
    providers: BTreeMap<String, ProviderDescriptor>,
}

impl ProviderCatalog {
    pub fn register(&mut self, descriptor: ProviderDescriptor) -> Result<(), ProviderError> {
        if descriptor.id.is_empty()
            || descriptor.id.len() > 64
            || !descriptor.id.bytes().all(|byte| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b'.' | b'-' | b'_')
            })
        {
            return Err(ProviderError::InvalidProviderId);
        }
        if descriptor
            .minimum_build
            .zip(descriptor.maximum_build)
            .is_some_and(|(minimum, maximum)| minimum > maximum)
        {
            return Err(ProviderError::InvalidBuildRange);
        }
        if self.providers.contains_key(&descriptor.id) {
            return Err(ProviderError::DuplicateProvider);
        }
        self.providers.insert(descriptor.id.clone(), descriptor);
        Ok(())
    }

    /// Candidate order is documented API, verified build adapter, then a
    /// semantically equivalent WinLook-owned surface.
    pub fn candidates(
        &self,
        operation: &ValidatedOperation,
        target: &WindowsTarget,
        allow_experimental_consent: bool,
    ) -> Vec<&ProviderDescriptor> {
        let mut candidates: Vec<_> = operation
            .provider_ids
            .iter()
            .filter_map(|id| self.providers.get(id))
            .filter(|provider| {
                provider.ownership == operation.ownership
                    && provider.supports(operation.action, target)
                    && (!provider.requires_consent || allow_experimental_consent)
            })
            .collect();
        candidates.sort_by_key(|provider| provider.kind);
        candidates
    }

    pub fn get(&self, id: &str) -> Option<&ProviderDescriptor> {
        self.providers.get(id)
    }
}

/// The only durable payload a provider may hand to the existing BGW journal.
/// BGW remains the single writer and owns the durable transaction protocol.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RestoreSnapshot {
    pub provider_id: String,
    pub provider_version: u32,
    pub payload: Vec<u8>,
}

impl RestoreSnapshot {
    pub fn validate(&self) -> Result<(), ProviderError> {
        if self.payload.len() > MAX_RESTORE_PAYLOAD_BYTES {
            return Err(ProviderError::RestorePayloadTooLarge);
        }
        if self.provider_id.is_empty() || self.provider_id.len() > 64 {
            return Err(ProviderError::InvalidProviderId);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppliedState {
    pub value: Option<PropertyValue>,
    pub ownership_token: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProviderError {
    InvalidProviderId,
    InvalidBuildRange,
    DuplicateProvider,
    Unsupported,
    Failed { code: String, message: String },
    OwnershipNotProven,
    VerificationFailed,
    RestorePayloadTooLarge,
}

impl fmt::Display for ProviderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProviderId => formatter.write_str("provider ID is invalid"),
            Self::InvalidBuildRange => formatter.write_str("provider build range is inverted"),
            Self::DuplicateProvider => formatter.write_str("provider ID is already registered"),
            Self::Unsupported => formatter.write_str("provider does not implement this operation"),
            Self::Failed { code, message } => write!(formatter, "provider failed ({code}): {message}"),
            Self::OwnershipNotProven => formatter.write_str("provider cannot prove it still owns the applied state"),
            Self::VerificationFailed => formatter.write_str("provider readback did not verify the requested state"),
            Self::RestorePayloadTooLarge => formatter.write_str("provider restore payload exceeds the bounded limit"),
        }
    }
}

impl std::error::Error for ProviderError {}

/// Providers implement only the operations they genuinely support. The host
/// must persist the snapshot through the restoration service before apply.
pub trait TranslationProvider: Send + Sync {
    fn descriptor(&self) -> &ProviderDescriptor;

    fn probe(&self, _target: &WindowsTarget) -> CapabilityStatus {
        CapabilityStatus::Unknown
    }

    fn read(&self, _operation: &ValidatedOperation) -> Result<PropertyValue, ProviderError> {
        Err(ProviderError::Unsupported)
    }

    fn capture(&self, _operation: &ValidatedOperation) -> Result<RestoreSnapshot, ProviderError> {
        Err(ProviderError::Unsupported)
    }

    fn apply(&self, _operation: &ValidatedOperation) -> Result<AppliedState, ProviderError> {
        Err(ProviderError::Unsupported)
    }

    fn inspect(&self, _operation: &ValidatedOperation) -> Result<AppliedState, ProviderError> {
        Err(ProviderError::Unsupported)
    }

    fn prove_ownership(
        &self,
        _operation: &ValidatedOperation,
        _applied: &AppliedState,
    ) -> Result<bool, ProviderError> {
        Ok(false)
    }

    fn verify(
        &self,
        _operation: &ValidatedOperation,
        _applied: &AppliedState,
    ) -> Result<bool, ProviderError> {
        Ok(false)
    }

    fn restore(
        &self,
        _operation: &ValidatedOperation,
        _snapshot: &RestoreSnapshot,
        _applied: &AppliedState,
    ) -> Result<(), ProviderError> {
        Err(ProviderError::Unsupported)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationStatus {
    Success,
    Partial,
    Unsupported,
    Skipped,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationResult {
    pub requested_path: String,
    pub resolved_target: String,
    pub applied_members: Vec<String>,
    pub provider_id: Option<String>,
    pub status: OperationStatus,
    pub fallback: Option<String>,
    pub diagnostic_code: Option<String>,
    pub message: String,
}

impl OperationResult {
    pub fn unsupported(path: impl Into<String>, target: &WindowsTarget, message: impl Into<String>) -> Self {
        Self {
            requested_path: path.into(),
            resolved_target: target.release_selector().to_owned(),
            applied_members: Vec::new(),
            provider_id: None,
            status: OperationStatus::Unsupported,
            fallback: None,
            diagnostic_code: Some("TRATON_UNSUPPORTED".to_owned()),
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{ProviderCatalog, ProviderDescriptor, ProviderKind};
    use crate::registry::{ObjectOwnership, OperationAction};
    use crate::target::{Architecture, WindowsTarget, WindowsTargetInfo};

    fn target() -> WindowsTarget {
        WindowsTarget::from_info(WindowsTargetInfo {
            product_name: Some("Windows 10 Pro".to_owned()),
            display_version: Some("22H2".to_owned()),
            edition: None,
            installation_type: None,
            build_number: 19_045,
            update_build_revision: None,
            process_architecture: Architecture::X64,
            system_architecture: Architecture::X64,
            process_id: 1,
            session_id: 1,
        })
        .unwrap()
    }

    #[test]
    fn provider_order_prefers_documented_apis_then_verified_adapters() {
        let actions = [OperationAction::Read].into_iter().collect::<BTreeSet<_>>();
        let mut catalog = ProviderCatalog::default();
        for (id, kind) in [
            ("surface", ProviderKind::WinLookSurface),
            ("verified", ProviderKind::VerifiedAdapter),
            ("documented", ProviderKind::DocumentedApi),
        ] {
            catalog
                .register(ProviderDescriptor {
                    id: id.to_owned(),
                    kind,
                    ownership: ObjectOwnership::Windows,
                    actions: actions.clone(),
                    minimum_build: Some(19_045),
                    maximum_build: None,
                    requires_consent: false,
                })
                .unwrap();
        }
        let operation = crate::registry::ValidatedOperation {
            path: crate::namespace::NamespacePath::parse(
                "windows.10-22h2.shell.taskbar.items.visibility(pinned-shortcut:item-1)",
            )
            .unwrap(),
            action: OperationAction::Read,
            value: None,
            argument: None,
            capability_id: "test".to_owned(),
            ownership: ObjectOwnership::Windows,
            provider_ids: vec![
                "surface".to_owned(),
                "verified".to_owned(),
                "documented".to_owned(),
            ],
            target_build: 19_045,
        };
        let ordered = catalog.candidates(&operation, &target(), false);
        assert_eq!(ordered[0].kind, ProviderKind::DocumentedApi);
        assert_eq!(ordered[1].kind, ProviderKind::VerifiedAdapter);
        assert_eq!(ordered[2].kind, ProviderKind::WinLookSurface);
    }
}
