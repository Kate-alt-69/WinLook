//! Capability state and evidence contracts.

use std::collections::BTreeMap;
use std::fmt;

/// Capability evidence is explicit; missing evidence is never treated as support.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CapabilityStatus {
    Available,
    Partial,
    Experimental,
    Unsupported,
    Unknown,
}

impl CapabilityStatus {
    pub fn permits(self, allow_experimental: bool) -> bool {
        match self {
            Self::Available | Self::Partial => true,
            Self::Experimental => allow_experimental,
            Self::Unsupported | Self::Unknown => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapabilityRecord {
    pub status: CapabilityStatus,
    pub provider_id: Option<String>,
    pub reason: Option<String>,
}

impl CapabilityRecord {
    pub fn unknown() -> Self {
        Self {
            status: CapabilityStatus::Unknown,
            provider_id: None,
            reason: Some("no provider has reported capability evidence".to_owned()),
        }
    }
}

/// Per-target capability snapshot. The host decides when to refresh and publish it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CapabilitySet {
    records: BTreeMap<String, CapabilityRecord>,
}

impl CapabilitySet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(
        &mut self,
        id: impl Into<String>,
        record: CapabilityRecord,
    ) -> Result<(), CapabilityError> {
        let id = id.into();
        validate_token(&id, 128).map_err(|_| CapabilityError::InvalidId)?;
        if record.reason.as_ref().is_some_and(|reason| reason.len() > 512) {
            return Err(CapabilityError::ReasonTooLong);
        }
        if record.provider_id.as_ref().is_some_and(|provider| validate_token(provider, 64).is_err()) {
            return Err(CapabilityError::InvalidProviderId);
        }
        self.records.insert(id, record);
        Ok(())
    }

    pub fn status(&self, id: &str) -> CapabilityStatus {
        self.records
            .get(id)
            .map(|record| record.status)
            .unwrap_or(CapabilityStatus::Unknown)
    }

    pub fn get(&self, id: &str) -> CapabilityRecord {
        self.records
            .get(id)
            .cloned()
            .unwrap_or_else(CapabilityRecord::unknown)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &CapabilityRecord)> {
        self.records.iter().map(|(id, record)| (id.as_str(), record))
    }
}

fn validate_token(value: &str, max_len: usize) -> Result<(), ()> {
    if value.is_empty()
        || value.len() > max_len
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {
        return Err(());
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapabilityError {
    InvalidId,
    InvalidProviderId,
    ReasonTooLong,
}

impl fmt::Display for CapabilityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidId => "capability IDs must be bounded ASCII identifiers",
            Self::InvalidProviderId => "provider IDs must be bounded ASCII identifiers",
            Self::ReasonTooLong => "capability diagnostic reason exceeds 512 bytes",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for CapabilityError {}

#[cfg(test)]
mod tests {
    use super::{CapabilityRecord, CapabilitySet, CapabilityStatus};

    #[test]
    fn missing_capabilities_are_unknown_not_available() {
        let capabilities = CapabilitySet::new();
        assert_eq!(capabilities.status("shell.taskbar.visibility"), CapabilityStatus::Unknown);
        assert!(!capabilities.status("shell.taskbar.visibility").permits(false));
    }

    #[test]
    fn experimental_capabilities_require_explicit_opt_in() {
        assert!(!CapabilityStatus::Experimental.permits(false));
        assert!(CapabilityStatus::Experimental.permits(true));
    }

    #[test]
    fn records_are_validated_and_read_back() {
        let mut capabilities = CapabilitySet::new();
        capabilities
            .set(
                "display.monitor.topology",
                CapabilityRecord {
                    status: CapabilityStatus::Available,
                    provider_id: Some("wl_wlde_win32.monitor-topology".to_owned()),
                    reason: None,
                },
            )
            .unwrap();
        assert_eq!(capabilities.status("display.monitor.topology"), CapabilityStatus::Available);
    }
}
