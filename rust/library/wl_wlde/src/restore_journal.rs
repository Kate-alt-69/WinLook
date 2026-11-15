//! Portable, bounded write-ahead restoration record model.
//!
//! This module validates typed journal records and computes conservative
//! recovery actions. It deliberately performs no file I/O, ACL checks, handle
//! operations or platform mutations. A platform writer must durably append
//! and flush each transition before calling its native adapter.

use std::fmt;

pub const DESKTOP_RESTORE_JOURNAL_VERSION: u16 = 1;
pub const MAX_DESKTOP_RESTORE_RECORDS: usize = 8_192;
pub const MAX_DESKTOP_RESTORE_JOURNAL_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_DESKTOP_RESTORE_RESOURCE_ID_BYTES: usize = 64;
pub const MAX_DESKTOP_RESTORE_VALUE_BYTES: usize = 256;
pub const MAX_DESKTOP_RESTORE_OWNERSHIP_BYTES: usize = 128;

const JOURNAL_MAGIC: [u8; 4] = *b"WLJR";
const FRAME_HEADER_BYTES: usize = 10;
const FRAME_CHECKSUM_BYTES: usize = 4;
const MAX_RECORD_PAYLOAD_BYTES: usize = 768;

/// Operation phase encoded in every append-only restoration record; the
/// storage backend is responsible for durability.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DesktopRestoreState {
    /// Captured original state; the native adapter has not been called.
    Prepared = 1,
    /// The caller has durably recorded write-ahead intent; the adapter may have run.
    Applying = 2,
    /// The desired state was read back and WinLook ownership was verified.
    Committed = 3,
    /// The caller has durably recorded restoration intent; the adapter may be restoring.
    Restoring = 4,
    /// The original state was read back after restoration.
    Restored = 5,
    /// The operation was proven not to need restoration.
    Aborted = 6,
    /// Current state or ownership could not be proven safe.
    Conflict = 7,
}

impl DesktopRestoreState {
    fn from_byte(value: u8) -> Result<Self, DesktopRestoreJournalError> {
        match value {
            1 => Ok(Self::Prepared),
            2 => Ok(Self::Applying),
            3 => Ok(Self::Committed),
            4 => Ok(Self::Restoring),
            5 => Ok(Self::Restored),
            6 => Ok(Self::Aborted),
            7 => Ok(Self::Conflict),
            _ => Err(DesktopRestoreJournalError::InvalidRecord),
        }
    }

    fn is_terminal(self) -> bool {
        matches!(self, Self::Restored | Self::Aborted | Self::Conflict)
    }
}

/// One immutable operation snapshot with a mutable, append-only state history.
///
/// Adapter IDs are numeric allowlist keys, not names or paths. Resource IDs
/// are short adapter-scoped tokens. The value and ownership fields are opaque
/// bounded adapter payloads; each adapter must validate their internal schema.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DesktopRestoreRecord {
    sequence: u64,
    transaction_id: [u8; 16],
    state: DesktopRestoreState,
    adapter_id: u16,
    adapter_version: u16,
    resource_id: String,
    original_value: Vec<u8>,
    applied_value: Vec<u8>,
    ownership_proof: Vec<u8>,
}

impl DesktopRestoreRecord {
    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    pub fn transaction_id(&self) -> &[u8; 16] {
        &self.transaction_id
    }

    pub fn state(&self) -> DesktopRestoreState {
        self.state
    }

    pub fn adapter_id(&self) -> u16 {
        self.adapter_id
    }

    pub fn adapter_version(&self) -> u16 {
        self.adapter_version
    }

    pub fn resource_id(&self) -> &str {
        &self.resource_id
    }

    pub fn original_value(&self) -> &[u8] {
        &self.original_value
    }

    pub fn applied_value(&self) -> &[u8] {
        &self.applied_value
    }

    pub fn ownership_proof(&self) -> &[u8] {
        &self.ownership_proof
    }

    fn with_state(&self, sequence: u64, state: DesktopRestoreState) -> Self {
        Self {
            sequence,
            transaction_id: self.transaction_id,
            state,
            adapter_id: self.adapter_id,
            adapter_version: self.adapter_version,
            resource_id: self.resource_id.clone(),
            original_value: self.original_value.clone(),
            applied_value: self.applied_value.clone(),
            ownership_proof: self.ownership_proof.clone(),
        }
    }
}

/// In-memory validator for an append-only sequence of restoration records.
///
/// The type does not persist records. The BGW storage implementation must
/// append `encode_desktop_restore_record()` bytes and successfully flush them
/// before each corresponding native adapter action.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DesktopRestoreLedger {
    records: Vec<DesktopRestoreRecord>,
    encoded_bytes: usize,
}

impl DesktopRestoreLedger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn records(&self) -> &[DesktopRestoreRecord] {
        &self.records
    }

    pub fn latest_record(&self, transaction_id: &[u8; 16]) -> Option<&DesktopRestoreRecord> {
        self.records
            .iter()
            .rev()
            .find(|record| &record.transaction_id == transaction_id)
    }

    /// Starts a reversible operation with its original and desired values.
    ///
    /// The platform adapter must have captured and validated all values before
    /// calling this method. This method only validates bounded portable data.
    pub fn begin(
        &mut self,
        transaction_id: [u8; 16],
        adapter_id: u16,
        adapter_version: u16,
        resource_id: impl Into<String>,
        original_value: &[u8],
        applied_value: &[u8],
        ownership_proof: &[u8],
    ) -> Result<&DesktopRestoreRecord, DesktopRestoreJournalError> {
        let resource_id = resource_id.into();
        validate_operation(
            &transaction_id,
            adapter_id,
            adapter_version,
            &resource_id,
            original_value,
            applied_value,
            ownership_proof,
        )?;
        if self.records.iter().any(|record| record.transaction_id == transaction_id) {
            return Err(DesktopRestoreJournalError::DuplicateTransaction);
        }
        let sequence = self.next_sequence()?;
        let record = DesktopRestoreRecord {
            sequence,
            transaction_id,
            state: DesktopRestoreState::Prepared,
            adapter_id,
            adapter_version,
            resource_id,
            original_value: original_value.to_vec(),
            applied_value: applied_value.to_vec(),
            ownership_proof: ownership_proof.to_vec(),
        };
        self.push_validated(record)?;
        Ok(self.records.last().expect("record was just appended"))
    }

    /// Appends a legal state transition while preserving the operation snapshot.
    ///
    /// Applying an operation to the OS is outside this type. Evidence is
    /// required for every transition that claims a readback result:
    /// Committed and Restoring require the desired value plus ownership proof;
    /// Restored and Applying->Aborted require the original value. The caller
    /// must persist and flush the returned Applying record before invoking the
    /// native adapter; this in-memory model cannot enforce a storage boundary.
    pub fn transition(
        &mut self,
        transaction_id: &[u8; 16],
        next_state: DesktopRestoreState,
        observed_value: Option<&[u8]>,
        ownership_proven: bool,
    ) -> Result<&DesktopRestoreRecord, DesktopRestoreJournalError> {
        let previous = self
            .latest_record(transaction_id)
            .cloned()
            .ok_or(DesktopRestoreJournalError::UnknownTransaction)?;
        if !transition_allowed(previous.state, next_state) {
            return Err(DesktopRestoreJournalError::InvalidTransition);
        }
        validate_transition_evidence(
            &previous,
            next_state,
            observed_value,
            ownership_proven,
        )?;
        let sequence = self.next_sequence()?;
        let next = previous.with_state(sequence, next_state);
        self.push_validated(next)?;
        Ok(self.records.last().expect("record was just appended"))
    }

    fn next_sequence(&self) -> Result<u64, DesktopRestoreJournalError> {
        let current = self.records.last().map_or(0, |record| record.sequence);
        current
            .checked_add(1)
            .ok_or(DesktopRestoreJournalError::SequenceOverflow)
    }

    fn push_validated(
        &mut self,
        record: DesktopRestoreRecord,
    ) -> Result<(), DesktopRestoreJournalError> {
        if self.records.len() >= MAX_DESKTOP_RESTORE_RECORDS {
            return Err(DesktopRestoreJournalError::TooManyRecords);
        }
        validate_record(&record)?;
        validate_record_position(&self.records, &record)?;
        let frame_bytes = encoded_record_len(&record)?;
        let total_bytes = self
            .encoded_bytes
            .checked_add(frame_bytes)
            .ok_or(DesktopRestoreJournalError::JournalTooLarge)?;
        if total_bytes > MAX_DESKTOP_RESTORE_JOURNAL_BYTES {
            return Err(DesktopRestoreJournalError::JournalTooLarge);
        }
        self.records
            .try_reserve(1)
            .map_err(|_| DesktopRestoreJournalError::AllocationFailed)?;
        self.records.push(record);
        self.encoded_bytes = total_bytes;
        Ok(())
    }
}

/// Result of decoding a journal byte stream.
///
/// A partial final frame is exposed as an incomplete tail, with the offset of
/// the last complete record. A writer may truncate only that incomplete tail
/// after validating the complete prefix; a complete frame with a bad checksum
/// or invalid schema is corruption and must fail closed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedDesktopRestoreLedger {
    ledger: DesktopRestoreLedger,
    valid_bytes: usize,
    incomplete_tail: bool,
}

impl DecodedDesktopRestoreLedger {
    pub fn ledger(&self) -> &DesktopRestoreLedger {
        &self.ledger
    }

    pub fn into_ledger(self) -> DesktopRestoreLedger {
        self.ledger
    }

    pub fn valid_bytes(&self) -> usize {
        self.valid_bytes
    }

    pub fn has_incomplete_tail(&self) -> bool {
        self.incomplete_tail
    }
}

/// Conservative recovery decision based on an adapter readback.
///
/// `ownership_proven` must come from adapter-specific evidence, not merely
/// equality with the desired value. Never restore when this proof is absent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DesktopRestoreRecoveryAction {
    /// Prepared guarantees no native call began; append a terminal Aborted record.
    AbortPrepared,
    /// Current state still equals the captured original value.
    AbortUnapplied,
    /// Current state is WinLook's desired value and ownership is proven.
    RestoreOriginal,
    /// A committed WinLook-owned value remains active and may be kept for now.
    KeepApplied,
    /// The original value is already present after a prior restore attempt.
    MarkRestored,
    /// The desired value remains WinLook-owned; retry the idempotent restore.
    RetryRestore,
    /// External state or adapter ownership cannot be proven safe.
    Conflict,
    /// The transaction is already terminal and requires no automatic action.
    NoAction,
}

/// Plans recovery for the latest record of one operation.
///
/// Prepared and terminal records do not need an observed value. Other states
/// compare the current adapter value with both snapshots and require ownership
/// proof before treating the applied value as WinLook-owned.
pub fn plan_desktop_restore_recovery(
    record: &DesktopRestoreRecord,
    observed_value: Option<&[u8]>,
    ownership_proven: bool,
) -> DesktopRestoreRecoveryAction {
    match record.state {
        DesktopRestoreState::Prepared => DesktopRestoreRecoveryAction::AbortPrepared,
        DesktopRestoreState::Restored | DesktopRestoreState::Aborted => {
            DesktopRestoreRecoveryAction::NoAction
        }
        DesktopRestoreState::Conflict => DesktopRestoreRecoveryAction::Conflict,
        DesktopRestoreState::Applying
        | DesktopRestoreState::Committed
        | DesktopRestoreState::Restoring => {
            let Some(observed_value) = observed_value else {
                return DesktopRestoreRecoveryAction::Conflict;
            };
            match record.state {
                DesktopRestoreState::Applying => {
                    if observed_value == record.original_value.as_slice() {
                        DesktopRestoreRecoveryAction::AbortUnapplied
                    } else if observed_value == record.applied_value.as_slice() && ownership_proven
                    {
                        DesktopRestoreRecoveryAction::RestoreOriginal
                    } else {
                        DesktopRestoreRecoveryAction::Conflict
                    }
                }
                DesktopRestoreState::Committed => {
                    if observed_value == record.applied_value.as_slice() && ownership_proven {
                        DesktopRestoreRecoveryAction::KeepApplied
                    } else {
                        DesktopRestoreRecoveryAction::Conflict
                    }
                }
                DesktopRestoreState::Restoring => {
                    if observed_value == record.original_value.as_slice() {
                        DesktopRestoreRecoveryAction::MarkRestored
                    } else if observed_value == record.applied_value.as_slice() && ownership_proven
                    {
                        DesktopRestoreRecoveryAction::RetryRestore
                    } else {
                        DesktopRestoreRecoveryAction::Conflict
                    }
                }
                _ => DesktopRestoreRecoveryAction::Conflict,
            }
        }
    }
}

fn encoded_record_len(
    record: &DesktopRestoreRecord,
) -> Result<usize, DesktopRestoreJournalError> {
    validate_record(record)?;
    let payload_len = 8usize
        .checked_add(16)
        .and_then(|value| value.checked_add(1 + 2 + 2 + 1))
        .and_then(|value| value.checked_add(record.resource_id.len()))
        .and_then(|value| value.checked_add(2 + record.original_value.len()))
        .and_then(|value| value.checked_add(2 + record.applied_value.len()))
        .and_then(|value| value.checked_add(2 + record.ownership_proof.len()))
        .ok_or(DesktopRestoreJournalError::RecordTooLarge)?;
    if payload_len > MAX_RECORD_PAYLOAD_BYTES {
        return Err(DesktopRestoreJournalError::RecordTooLarge);
    }
    FRAME_HEADER_BYTES
        .checked_add(payload_len)
        .and_then(|value| value.checked_add(FRAME_CHECKSUM_BYTES))
        .ok_or(DesktopRestoreJournalError::JournalTooLarge)
}

/// Encodes one record as a versioned length-delimited frame with CRC32.
pub fn encode_desktop_restore_record(
    record: &DesktopRestoreRecord,
) -> Result<Vec<u8>, DesktopRestoreJournalError> {
    let frame_len = encoded_record_len(record)?;
    let resource = record.resource_id.as_bytes();
    let payload_len = frame_len - FRAME_HEADER_BYTES - FRAME_CHECKSUM_BYTES;
    let mut output = Vec::new();
    output
        .try_reserve_exact(frame_len)
        .map_err(|_| DesktopRestoreJournalError::AllocationFailed)?;
    output.extend_from_slice(&JOURNAL_MAGIC);
    output.extend_from_slice(&DESKTOP_RESTORE_JOURNAL_VERSION.to_le_bytes());
    output.extend_from_slice(&(payload_len as u32).to_le_bytes());
    output.extend_from_slice(&record.sequence.to_le_bytes());
    output.extend_from_slice(&record.transaction_id);
    output.push(record.state as u8);
    output.extend_from_slice(&record.adapter_id.to_le_bytes());
    output.extend_from_slice(&record.adapter_version.to_le_bytes());
    output.push(resource.len() as u8);
    output.extend_from_slice(resource);
    output.extend_from_slice(&(record.original_value.len() as u16).to_le_bytes());
    output.extend_from_slice(&record.original_value);
    output.extend_from_slice(&(record.applied_value.len() as u16).to_le_bytes());
    output.extend_from_slice(&record.applied_value);
    output.extend_from_slice(&(record.ownership_proof.len() as u16).to_le_bytes());
    output.extend_from_slice(&record.ownership_proof);

    let checksum = crc32(&output);
    output.extend_from_slice(&checksum.to_le_bytes());
    Ok(output)
}

/// Encodes every complete record in a validated ledger.
pub fn encode_desktop_restore_ledger(
    ledger: &DesktopRestoreLedger,
) -> Result<Vec<u8>, DesktopRestoreJournalError> {
    let mut output = Vec::new();
    for record in ledger.records() {
        let frame = encode_desktop_restore_record(record)?;
        let new_len = output
            .len()
            .checked_add(frame.len())
            .ok_or(DesktopRestoreJournalError::JournalTooLarge)?;
        if new_len > MAX_DESKTOP_RESTORE_JOURNAL_BYTES {
            return Err(DesktopRestoreJournalError::JournalTooLarge);
        }
        output
            .try_reserve(frame.len())
            .map_err(|_| DesktopRestoreJournalError::AllocationFailed)?;
        output.extend_from_slice(&frame);
    }
    Ok(output)
}

/// Decodes complete frames and identifies only a partial final append as a torn tail.
pub fn decode_desktop_restore_ledger(
    bytes: &[u8],
) -> Result<DecodedDesktopRestoreLedger, DesktopRestoreJournalError> {
    if bytes.len() > MAX_DESKTOP_RESTORE_JOURNAL_BYTES {
        return Err(DesktopRestoreJournalError::JournalTooLarge);
    }

    let mut ledger = DesktopRestoreLedger::new();
    let mut offset = 0usize;
    let mut incomplete_tail = false;

    while offset < bytes.len() {
        let remaining = &bytes[offset..];
        if remaining.len() < JOURNAL_MAGIC.len() {
            if !is_magic_prefix(remaining) {
                return Err(DesktopRestoreJournalError::InvalidFrameMagic);
            }
            incomplete_tail = true;
            break;
        }
        if remaining[..4] != JOURNAL_MAGIC {
            return Err(DesktopRestoreJournalError::InvalidFrameMagic);
        }
        if remaining.len() < 6 {
            incomplete_tail = true;
            break;
        }
        let version = read_u16(&remaining[4..6])?;
        if version != DESKTOP_RESTORE_JOURNAL_VERSION {
            return Err(DesktopRestoreJournalError::UnsupportedVersion);
        }
        if remaining.len() < FRAME_HEADER_BYTES {
            incomplete_tail = true;
            break;
        }
        let payload_len = read_u32(&remaining[6..10])? as usize;
        if payload_len > MAX_RECORD_PAYLOAD_BYTES {
            return Err(DesktopRestoreJournalError::RecordTooLarge);
        }
        let frame_len = FRAME_HEADER_BYTES
            .checked_add(payload_len)
            .and_then(|value| value.checked_add(FRAME_CHECKSUM_BYTES))
            .ok_or(DesktopRestoreJournalError::JournalTooLarge)?;
        if remaining.len() < frame_len {
            incomplete_tail = true;
            break;
        }

        let checksum_offset = FRAME_HEADER_BYTES + payload_len;
        let expected_checksum = read_u32(&remaining[checksum_offset..checksum_offset + 4])?;
        let actual_checksum = crc32(&remaining[..checksum_offset]);
        if expected_checksum != actual_checksum {
            return Err(DesktopRestoreJournalError::ChecksumMismatch);
        }

        let record = decode_record_payload(&remaining[FRAME_HEADER_BYTES..checksum_offset])?;
        ledger.push_validated(record)?;
        offset += frame_len;
    }

    Ok(DecodedDesktopRestoreLedger {
        ledger,
        valid_bytes: offset,
        incomplete_tail,
    })
}

/// Typed codec and state-machine errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DesktopRestoreJournalError {
    InvalidTransactionId,
    InvalidAdapterId,
    InvalidAdapterVersion,
    InvalidResourceId,
    EmptyOwnershipProof,
    ValueTooLarge,
    UnchangedValue,
    TooManyRecords,
    SequenceOverflow,
    InvalidSequence,
    DuplicateTransaction,
    UnknownTransaction,
    InvalidTransition,
    MissingObservation,
    ReadbackMismatch,
    OwnershipNotProven,
    InvalidRecord,
    InvalidFrameMagic,
    UnsupportedVersion,
    RecordTooLarge,
    ChecksumMismatch,
    JournalTooLarge,
    AllocationFailed,
}

impl fmt::Display for DesktopRestoreJournalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "desktop restore journal error: {self:?}")
    }
}

impl std::error::Error for DesktopRestoreJournalError {}

fn validate_operation(
    transaction_id: &[u8; 16],
    adapter_id: u16,
    adapter_version: u16,
    resource_id: &str,
    original_value: &[u8],
    applied_value: &[u8],
    ownership_proof: &[u8],
) -> Result<(), DesktopRestoreJournalError> {
    if transaction_id.iter().all(|byte| *byte == 0) {
        return Err(DesktopRestoreJournalError::InvalidTransactionId);
    }
    if adapter_id == 0 {
        return Err(DesktopRestoreJournalError::InvalidAdapterId);
    }
    if adapter_version == 0 {
        return Err(DesktopRestoreJournalError::InvalidAdapterVersion);
    }
    validate_resource_id(resource_id)?;
    if original_value.len() > MAX_DESKTOP_RESTORE_VALUE_BYTES
        || applied_value.len() > MAX_DESKTOP_RESTORE_VALUE_BYTES
    {
        return Err(DesktopRestoreJournalError::ValueTooLarge);
    }
    if original_value == applied_value {
        return Err(DesktopRestoreJournalError::UnchangedValue);
    }
    if ownership_proof.is_empty() {
        return Err(DesktopRestoreJournalError::EmptyOwnershipProof);
    }
    if ownership_proof.len() > MAX_DESKTOP_RESTORE_OWNERSHIP_BYTES {
        return Err(DesktopRestoreJournalError::ValueTooLarge);
    }
    Ok(())
}

fn validate_resource_id(value: &str) -> Result<(), DesktopRestoreJournalError> {
    if value.is_empty()
        || value.len() > MAX_DESKTOP_RESTORE_RESOURCE_ID_BYTES
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
    {
        return Err(DesktopRestoreJournalError::InvalidResourceId);
    }
    Ok(())
}

fn validate_record(record: &DesktopRestoreRecord) -> Result<(), DesktopRestoreJournalError> {
    if record.sequence == 0 {
        return Err(DesktopRestoreJournalError::InvalidSequence);
    }
    validate_operation(
        &record.transaction_id,
        record.adapter_id,
        record.adapter_version,
        &record.resource_id,
        &record.original_value,
        &record.applied_value,
        &record.ownership_proof,
    )
}

fn validate_record_position(
    records: &[DesktopRestoreRecord],
    record: &DesktopRestoreRecord,
) -> Result<(), DesktopRestoreJournalError> {
    let expected_sequence = records
        .last()
        .map_or(Some(1), |previous| previous.sequence.checked_add(1))
        .ok_or(DesktopRestoreJournalError::SequenceOverflow)?;
    if record.sequence != expected_sequence {
        return Err(DesktopRestoreJournalError::InvalidSequence);
    }

    match records
        .iter()
        .rev()
        .find(|previous| previous.transaction_id == record.transaction_id)
    {
        None => {
            if record.state != DesktopRestoreState::Prepared {
                return Err(DesktopRestoreJournalError::InvalidTransition);
            }
        }
        Some(previous) => {
            if previous.state.is_terminal() {
                return Err(DesktopRestoreJournalError::InvalidTransition);
            }
            if !transition_allowed(previous.state, record.state) {
                return Err(DesktopRestoreJournalError::InvalidTransition);
            }
            if previous.adapter_id != record.adapter_id
                || previous.adapter_version != record.adapter_version
                || previous.resource_id != record.resource_id
                || previous.original_value != record.original_value
                || previous.applied_value != record.applied_value
                || previous.ownership_proof != record.ownership_proof
            {
                return Err(DesktopRestoreJournalError::InvalidRecord);
            }
        }
    }
    Ok(())
}

fn validate_transition_evidence(
    previous: &DesktopRestoreRecord,
    next: DesktopRestoreState,
    observed_value: Option<&[u8]>,
    ownership_proven: bool,
) -> Result<(), DesktopRestoreJournalError> {
    let original_readback_required = matches!(
        (previous.state, next),
        (DesktopRestoreState::Applying, DesktopRestoreState::Aborted)
            | (DesktopRestoreState::Restoring, DesktopRestoreState::Restored)
    );
    let applied_readback_required = matches!(
        (previous.state, next),
        (DesktopRestoreState::Applying, DesktopRestoreState::Committed)
            | (DesktopRestoreState::Applying, DesktopRestoreState::Restoring)
            | (DesktopRestoreState::Committed, DesktopRestoreState::Restoring)
    );

    if original_readback_required {
        let observed =
            observed_value.ok_or(DesktopRestoreJournalError::MissingObservation)?;
        if observed != previous.original_value.as_slice() {
            return Err(DesktopRestoreJournalError::ReadbackMismatch);
        }
    } else if applied_readback_required {
        let observed =
            observed_value.ok_or(DesktopRestoreJournalError::MissingObservation)?;
        if observed != previous.applied_value.as_slice() {
            return Err(DesktopRestoreJournalError::ReadbackMismatch);
        }
        if !ownership_proven {
            return Err(DesktopRestoreJournalError::OwnershipNotProven);
        }
    }
    Ok(())
}

fn transition_allowed(current: DesktopRestoreState, next: DesktopRestoreState) -> bool {
    match current {
        DesktopRestoreState::Prepared => matches!(
            next,
            DesktopRestoreState::Applying
                | DesktopRestoreState::Aborted
                | DesktopRestoreState::Conflict
        ),
        DesktopRestoreState::Applying => matches!(
            next,
            DesktopRestoreState::Committed
                | DesktopRestoreState::Restoring
                | DesktopRestoreState::Aborted
                | DesktopRestoreState::Conflict
        ),
        DesktopRestoreState::Committed => {
            matches!(next, DesktopRestoreState::Restoring | DesktopRestoreState::Conflict)
        }
        DesktopRestoreState::Restoring => {
            matches!(next, DesktopRestoreState::Restored | DesktopRestoreState::Conflict)
        }
        DesktopRestoreState::Restored | DesktopRestoreState::Aborted | DesktopRestoreState::Conflict => false,
    }
}

fn is_magic_prefix(bytes: &[u8]) -> bool {
    bytes.len() < JOURNAL_MAGIC.len() && &JOURNAL_MAGIC[..bytes.len()] == bytes
}

fn decode_record_payload(
    payload: &[u8],
) -> Result<DesktopRestoreRecord, DesktopRestoreJournalError> {
    let mut cursor = PayloadCursor::new(payload);
    let sequence = cursor.u64()?;
    let transaction_id = cursor.array_16()?;
    let state = DesktopRestoreState::from_byte(cursor.u8()?)?;
    let adapter_id = cursor.u16()?;
    let adapter_version = cursor.u16()?;
    let resource_len = usize::from(cursor.u8()?);
    let resource_id = String::from_utf8(cursor.bytes(resource_len)?.to_vec())
        .map_err(|_| DesktopRestoreJournalError::InvalidResourceId)?;
    let original_len = usize::from(cursor.u16()?);
    let original_value = cursor.bytes(original_len)?.to_vec();
    let applied_len = usize::from(cursor.u16()?);
    let applied_value = cursor.bytes(applied_len)?.to_vec();
    let ownership_len = usize::from(cursor.u16()?);
    let ownership_proof = cursor.bytes(ownership_len)?.to_vec();
    if !cursor.is_empty() {
        return Err(DesktopRestoreJournalError::InvalidRecord);
    }
    let record = DesktopRestoreRecord {
        sequence,
        transaction_id,
        state,
        adapter_id,
        adapter_version,
        resource_id,
        original_value,
        applied_value,
        ownership_proof,
    };
    validate_record(&record)?;
    Ok(record)
}

struct PayloadCursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> PayloadCursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn bytes(&mut self, length: usize) -> Result<&'a [u8], DesktopRestoreJournalError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(DesktopRestoreJournalError::InvalidRecord)?;
        let slice = self
            .bytes
            .get(self.offset..end)
            .ok_or(DesktopRestoreJournalError::InvalidRecord)?;
        self.offset = end;
        Ok(slice)
    }

    fn u8(&mut self) -> Result<u8, DesktopRestoreJournalError> {
        Ok(*self.bytes(1)?.first().expect("one byte was read"))
    }

    fn u16(&mut self) -> Result<u16, DesktopRestoreJournalError> {
        read_u16(self.bytes(2)?)
    }

    fn u64(&mut self) -> Result<u64, DesktopRestoreJournalError> {
        let bytes = self.bytes(8)?;
        let mut value = [0u8; 8];
        value.copy_from_slice(bytes);
        Ok(u64::from_le_bytes(value))
    }

    fn array_16(&mut self) -> Result<[u8; 16], DesktopRestoreJournalError> {
        let bytes = self.bytes(16)?;
        let mut value = [0u8; 16];
        value.copy_from_slice(bytes);
        Ok(value)
    }

    fn is_empty(&self) -> bool {
        self.offset == self.bytes.len()
    }
}

fn read_u16(bytes: &[u8]) -> Result<u16, DesktopRestoreJournalError> {
    let value = bytes
        .get(..2)
        .ok_or(DesktopRestoreJournalError::InvalidRecord)?;
    Ok(u16::from_le_bytes([value[0], value[1]]))
}

fn read_u32(bytes: &[u8]) -> Result<u32, DesktopRestoreJournalError> {
    let value = bytes
        .get(..4)
        .ok_or(DesktopRestoreJournalError::InvalidRecord)?;
    Ok(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ledger_with_prepared() -> (DesktopRestoreLedger, [u8; 16]) {
        let mut ledger = DesktopRestoreLedger::new();
        let transaction_id = [0x41; 16];
        ledger
            .begin(
                transaction_id,
                1,
                1,
                "desktop.icon_visibility",
                &[0],
                &[1],
                &[0xA5, 0x5A],
            )
            .unwrap();
        (ledger, transaction_id)
    }

    #[test]
    fn codec_round_trips_a_complete_operation_history() {
        let (mut ledger, transaction_id) = ledger_with_prepared();
        ledger
            .transition(&transaction_id, DesktopRestoreState::Applying, None, false)
            .unwrap();
        ledger
            .transition(
                &transaction_id,
                DesktopRestoreState::Committed,
                Some(&[1]),
                true,
            )
            .unwrap();

        let encoded = encode_desktop_restore_ledger(&ledger).unwrap();
        let decoded = decode_desktop_restore_ledger(&encoded).unwrap();
        assert_eq!(decoded.ledger(), &ledger);
        assert_eq!(decoded.valid_bytes(), encoded.len());
        assert!(!decoded.has_incomplete_tail());
    }

    #[test]
    fn decoder_recognizes_every_truncated_suffix_as_an_incomplete_final_append() {
        let (ledger, _) = ledger_with_prepared();
        let encoded = encode_desktop_restore_ledger(&ledger).unwrap();
        for end in 1..encoded.len() {
            let decoded = decode_desktop_restore_ledger(&encoded[..end]).unwrap();
            assert!(decoded.has_incomplete_tail(), "cut at byte {end}");
            assert_eq!(decoded.valid_bytes(), 0, "cut at byte {end}");
            assert!(decoded.ledger().records().is_empty());
        }
        let complete = decode_desktop_restore_ledger(&encoded).unwrap();
        assert!(!complete.has_incomplete_tail());
        assert_eq!(complete.ledger(), &ledger);
    }

    #[test]
    fn unsupported_version_is_rejected_even_in_a_partial_final_header() {
        let (ledger, _) = ledger_with_prepared();
        let mut encoded = encode_desktop_restore_ledger(&ledger).unwrap();
        encoded[4] = 0xFF;
        assert_eq!(
            decode_desktop_restore_ledger(&encoded[..7]),
            Err(DesktopRestoreJournalError::UnsupportedVersion)
        );
    }

    #[test]
    fn complete_frame_with_bad_checksum_is_corruption() {
        let (ledger, _) = ledger_with_prepared();
        let mut encoded = encode_desktop_restore_ledger(&ledger).unwrap();
        let last = encoded.len() - 1;
        encoded[last] ^= 0x80;
        assert_eq!(
            decode_desktop_restore_ledger(&encoded),
            Err(DesktopRestoreJournalError::ChecksumMismatch)
        );
    }

    #[test]
    fn global_sequences_allow_interleaved_operations_and_reject_gaps() {
        let mut ledger = DesktopRestoreLedger::new();
        let first = [1; 16];
        let second = [2; 16];
        ledger
            .begin(first, 1, 1, "desktop.item.one", &[0], &[1], &[1])
            .unwrap();
        ledger
            .begin(second, 1, 1, "desktop.item.two", &[0], &[1], &[2])
            .unwrap();
        ledger
            .transition(&first, DesktopRestoreState::Applying, None, false)
            .unwrap();
        assert_eq!(ledger.records()[2].sequence(), 3);

        let encoded = encode_desktop_restore_ledger(&ledger).unwrap();
        assert_eq!(
            decode_desktop_restore_ledger(&encoded).unwrap().ledger(),
            &ledger
        );
    }

    #[test]
    fn invalid_transitions_and_duplicate_transactions_are_rejected() {
        let (mut ledger, transaction_id) = ledger_with_prepared();
        assert_eq!(
            ledger.transition(&transaction_id, DesktopRestoreState::Committed, None, false),
            Err(DesktopRestoreJournalError::InvalidTransition)
        );
        assert_eq!(
            ledger.begin(
                transaction_id,
                1,
                1,
                "desktop.icon_visibility",
                &[0],
                &[1],
                &[1]
            ),
            Err(DesktopRestoreJournalError::DuplicateTransaction)
        );
        ledger
            .transition(&transaction_id, DesktopRestoreState::Aborted, None, false)
            .unwrap();
        assert_eq!(
            ledger.transition(&transaction_id, DesktopRestoreState::Applying, None, false),
            Err(DesktopRestoreJournalError::InvalidTransition)
        );
    }

    #[test]
    fn commit_requires_exact_readback_and_adapter_ownership_proof() {
        let (mut ledger, transaction_id) = ledger_with_prepared();
        ledger
            .transition(&transaction_id, DesktopRestoreState::Applying, None, false)
            .unwrap();

        assert_eq!(
            ledger.transition(&transaction_id, DesktopRestoreState::Committed, None, false),
            Err(DesktopRestoreJournalError::MissingObservation)
        );
        assert_eq!(
            ledger.transition(
                &transaction_id,
                DesktopRestoreState::Committed,
                Some(&[0]),
                true
            ),
            Err(DesktopRestoreJournalError::ReadbackMismatch)
        );
        assert_eq!(
            ledger.transition(
                &transaction_id,
                DesktopRestoreState::Committed,
                Some(&[1]),
                false
            ),
            Err(DesktopRestoreJournalError::OwnershipNotProven)
        );
        assert_eq!(
            ledger.latest_record(&transaction_id).unwrap().state(),
            DesktopRestoreState::Applying
        );
        ledger
            .transition(
                &transaction_id,
                DesktopRestoreState::Committed,
                Some(&[1]),
                true
            )
            .unwrap();
    }

    #[test]
    fn aborting_after_apply_requires_readback_of_the_original_value() {
        let (mut ledger, transaction_id) = ledger_with_prepared();
        ledger
            .transition(&transaction_id, DesktopRestoreState::Applying, None, false)
            .unwrap();
        assert_eq!(
            ledger.transition(&transaction_id, DesktopRestoreState::Aborted, None, false),
            Err(DesktopRestoreJournalError::MissingObservation)
        );
        assert_eq!(
            ledger.transition(
                &transaction_id,
                DesktopRestoreState::Aborted,
                Some(&[1]),
                true,
            ),
            Err(DesktopRestoreJournalError::ReadbackMismatch)
        );
        ledger
            .transition(
                &transaction_id,
                DesktopRestoreState::Aborted,
                Some(&[0]),
                false,
            )
            .unwrap();
        assert_eq!(
            ledger.latest_record(&transaction_id).unwrap().state(),
            DesktopRestoreState::Aborted
        );
    }

    #[test]
    fn operation_fields_and_resource_ids_are_bounded_and_path_free() {
        let mut ledger = DesktopRestoreLedger::new();
        assert_eq!(
            ledger.begin([0; 16], 1, 1, "desktop.icon", &[0], &[1], &[1]),
            Err(DesktopRestoreJournalError::InvalidTransactionId)
        );
        assert_eq!(
            ledger.begin([1; 16], 0, 1, "desktop.icon", &[0], &[1], &[1]),
            Err(DesktopRestoreJournalError::InvalidAdapterId)
        );
        assert_eq!(
            ledger.begin([1; 16], 1, 1, "../shell/resource", &[0], &[1], &[1]),
            Err(DesktopRestoreJournalError::InvalidResourceId)
        );
        assert_eq!(
            ledger.begin([1; 16], 1, 1, "desktop.icon", &[0], &[1], &[]),
            Err(DesktopRestoreJournalError::EmptyOwnershipProof)
        );
    }

    #[test]
    fn recovery_requires_exact_state_and_adapter_ownership() {
        let (mut ledger, transaction_id) = ledger_with_prepared();
        let prepared = ledger.latest_record(&transaction_id).unwrap().clone();
        assert_eq!(
            plan_desktop_restore_recovery(&prepared, None, false),
            DesktopRestoreRecoveryAction::AbortPrepared
        );

        ledger
            .transition(&transaction_id, DesktopRestoreState::Applying, None, false)
            .unwrap();
        let applying = ledger.latest_record(&transaction_id).unwrap().clone();
        assert_eq!(
            plan_desktop_restore_recovery(&applying, None, false),
            DesktopRestoreRecoveryAction::Conflict
        );
        assert_eq!(
            plan_desktop_restore_recovery(&applying, Some(&[0]), false),
            DesktopRestoreRecoveryAction::AbortUnapplied
        );
        assert_eq!(
            plan_desktop_restore_recovery(&applying, Some(&[1]), false),
            DesktopRestoreRecoveryAction::Conflict
        );
        assert_eq!(
            plan_desktop_restore_recovery(&applying, Some(&[1]), true),
            DesktopRestoreRecoveryAction::RestoreOriginal
        );

        ledger
            .transition(
                &transaction_id,
                DesktopRestoreState::Committed,
                Some(&[1]),
                true,
            )
            .unwrap();
        let committed = ledger.latest_record(&transaction_id).unwrap().clone();
        assert_eq!(
            plan_desktop_restore_recovery(&committed, Some(&[1]), true),
            DesktopRestoreRecoveryAction::KeepApplied
        );
        assert_eq!(
            plan_desktop_restore_recovery(&committed, Some(&[1]), false),
            DesktopRestoreRecoveryAction::Conflict
        );

        ledger
            .transition(
                &transaction_id,
                DesktopRestoreState::Restoring,
                Some(&[1]),
                true,
            )
            .unwrap();
        let restoring = ledger.latest_record(&transaction_id).unwrap().clone();
        assert_eq!(
            plan_desktop_restore_recovery(&restoring, Some(&[0]), false),
            DesktopRestoreRecoveryAction::MarkRestored
        );
        assert_eq!(
            plan_desktop_restore_recovery(&restoring, Some(&[1]), true),
            DesktopRestoreRecoveryAction::RetryRestore
        );
        assert_eq!(
            plan_desktop_restore_recovery(&restoring, Some(&[2]), true),
            DesktopRestoreRecoveryAction::Conflict
        );
        assert_eq!(
            ledger.transition(&transaction_id, DesktopRestoreState::Restored, None, false),
            Err(DesktopRestoreJournalError::MissingObservation)
        );
        assert_eq!(
            ledger.transition(
                &transaction_id,
                DesktopRestoreState::Restored,
                Some(&[1]),
                false
            ),
            Err(DesktopRestoreJournalError::ReadbackMismatch)
        );
        ledger
            .transition(
                &transaction_id,
                DesktopRestoreState::Restored,
                Some(&[0]),
                false,
            )
            .unwrap();
        assert_eq!(
            ledger.latest_record(&transaction_id).unwrap().state(),
            DesktopRestoreState::Restored
        );
    }

    #[test]
    fn ledger_rejects_records_that_would_exceed_the_total_byte_bound() {
        let (mut ledger, transaction_id) = ledger_with_prepared();
        ledger.encoded_bytes = MAX_DESKTOP_RESTORE_JOURNAL_BYTES;
        assert_eq!(
            ledger.transition(&transaction_id, DesktopRestoreState::Applying, None, false),
            Err(DesktopRestoreJournalError::JournalTooLarge)
        );
        assert_eq!(ledger.records().len(), 1);
    }

    #[test]
    fn conflict_is_terminal_and_encoded_as_an_explicit_state() {
        let (mut ledger, transaction_id) = ledger_with_prepared();
        ledger
            .transition(&transaction_id, DesktopRestoreState::Conflict, None, false)
            .unwrap();
        let conflict = ledger.latest_record(&transaction_id).unwrap();
        assert_eq!(
            plan_desktop_restore_recovery(conflict, None, false),
            DesktopRestoreRecoveryAction::Conflict
        );
        assert_eq!(
            ledger.transition(
                &transaction_id,
                DesktopRestoreState::Restored,
                Some(&[1]),
                false
            ),
            Err(DesktopRestoreJournalError::InvalidTransition)
        );
    }
}
