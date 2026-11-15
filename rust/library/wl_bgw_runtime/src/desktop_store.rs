//! Versioned, per-user persistence for local WLDE Groups and desktop layout.
//!
//! The store contains only opaque item IDs, identity kinds, Group labels and
//! normalized placements. It never stores file paths, file contents, executable
//! payloads or source icon data. BGW is the single writer.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use wl_wlde::{
    DesktopGroup, DesktopGroups, DesktopItemId, DesktopItemRef, DesktopLayout,
    DesktopLayoutEntry, DesktopLayoutTarget, DesktopModelError, MonitorTopology,
    MAX_DESKTOP_GROUP_MEMBERS, MAX_DESKTOP_GROUPS, MAX_DESKTOP_INVENTORY_ITEMS,
    MAX_DESKTOP_LAYOUT_ENTRIES,
};
use wl_wlde_win32::{DesktopInventoryItem, DesktopItemIdentityStability};

const MAGIC: &[u8; 8] = b"WLDEDS01";
const FORMAT_VERSION: u16 = 1;
const HEADER_LENGTH: usize = 18;
const MAX_STATE_FILE_BYTES: usize = 4 * 1024 * 1024;
const MAX_IDENTIFIER_BYTES: usize = 127;
const MAX_LABEL_BYTES: usize = 255;
const MAX_MONITOR_ID_BYTES: usize = 63;
const TEMP_FILE_ATTEMPTS: usize = 16;

static TEMP_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// The item metadata needed to validate persisted Group/layout references.
///
/// DesktopInventoryItem also contains an ephemeral path for immediate Shell
/// operations. The store deliberately copies only its portable item reference
/// and identity-stability classification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DesktopItemIdentityRecord {
    item: DesktopItemRef,
    stability: DesktopItemIdentityStability,
}

impl DesktopItemIdentityRecord {
    pub fn new(item: DesktopItemRef, stability: DesktopItemIdentityStability) -> Self {
        Self { item, stability }
    }

    pub fn item(&self) -> &DesktopItemRef {
        &self.item
    }

    pub fn stability(&self) -> DesktopItemIdentityStability {
        self.stability
    }

    pub fn from_inventory(items: &[DesktopInventoryItem]) -> Vec<Self> {
        items
            .iter()
            .map(|item| Self::new(item.item().clone(), item.identity_stability()))
            .collect()
    }
}

/// A fully validated snapshot restored from the local state file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DesktopStateSnapshot {
    groups: DesktopGroups,
    layout: DesktopLayout,
}

impl DesktopStateSnapshot {
    pub fn groups(&self) -> &DesktopGroups {
        &self.groups
    }

    pub fn layout(&self) -> &DesktopLayout {
        &self.layout
    }
}

/// Errors from the local WLDE Groups/layout state store.
#[derive(Debug)]
pub enum DesktopStateStoreError {
    UnsupportedPlatform,
    LocalAppDataUnavailable,
    InvalidPath,
    Io(std::io::Error),
    InvalidData,
    ChecksumMismatch,
    UnsupportedVersion(u16),
    StaleReferences {
        filesystem_file_ids: usize,
        path_fallback_ids: usize,
    },
    InvalidModel(DesktopModelError),
}

impl fmt::Display for DesktopStateStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => {
                f.write_str("the per-user WLDE state store is available only on Windows")
            }
            Self::LocalAppDataUnavailable => {
                f.write_str("Windows did not provide a valid per-user Local AppData directory")
            }
            Self::InvalidPath => f.write_str("the WLDE state file path is invalid"),
            Self::Io(error) => write!(f, "WLDE state file operation failed: {error}"),
            Self::InvalidData => {
                f.write_str("the WLDE state file is malformed or exceeds its format bounds")
            }
            Self::ChecksumMismatch => {
                f.write_str("the WLDE state file failed its integrity check")
            }
            Self::UnsupportedVersion(version) => {
                write!(f, "WLDE state format version {version} is not supported")
            }
            Self::StaleReferences {
                filesystem_file_ids,
                path_fallback_ids,
            } => write!(
                f,
                "WLDE state contains stale desktop references (filesystem IDs: {filesystem_file_ids}, path fallbacks: {path_fallback_ids})"
            ),
            Self::InvalidModel(error) => {
                write!(f, "the saved WLDE state failed model validation: {error}")
            }
        }
    }
}

impl std::error::Error for DesktopStateStoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::InvalidModel(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for DesktopStateStoreError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<DesktopModelError> for DesktopStateStoreError {
    fn from(value: DesktopModelError) -> Self {
        Self::InvalidModel(value)
    }
}

/// Single-writer local persistence owned by BGW.
///
/// Use for_current_user() in the Windows session. at_path() exists for
/// controlled embedding and tests; callers remain responsible for choosing a
/// user-private directory.
#[derive(Clone, Debug)]
pub struct DesktopStateStore {
    path: PathBuf,
}

impl DesktopStateStore {
    /// Creates the normal per-user store under LOCALAPPDATA/WinLook/WLDE.
    #[cfg(windows)]
    pub fn for_current_user() -> Result<Self, DesktopStateStoreError> {
        let root = std::env::var_os("LOCALAPPDATA")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .ok_or(DesktopStateStoreError::LocalAppDataUnavailable)?;
        Ok(Self::at_path(
            root.join("WinLook")
                .join("WLDE")
                .join("desktop-state.v1.bin"),
        ))
    }

    #[cfg(not(windows))]
    pub fn for_current_user() -> Result<Self, DesktopStateStoreError> {
        Err(DesktopStateStoreError::UnsupportedPlatform)
    }

    pub fn at_path(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Loads and validates the last committed snapshot.
    ///
    /// Missing files return Ok(None). Corrupt files, stale item identities,
    /// changed monitor IDs or incomplete layouts return an error and are never
    /// overwritten by this method.
    pub fn load(
        &self,
        inventory: &[DesktopItemIdentityRecord],
        topology: &MonitorTopology,
    ) -> Result<Option<DesktopStateSnapshot>, DesktopStateStoreError> {
        let mut file = match File::open(&self.path) {
            Ok(file) => file,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        let metadata = file.metadata()?;
        if metadata.len() > MAX_STATE_FILE_BYTES as u64 {
            return Err(DesktopStateStoreError::InvalidData);
        }
        let mut bytes = Vec::with_capacity(metadata.len() as usize);
        file.take((MAX_STATE_FILE_BYTES + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX_STATE_FILE_BYTES {
            return Err(DesktopStateStoreError::InvalidData);
        }

        let stored = decode_file(&bytes)?;
        let (available_items, identity_map) = build_inventory_index(inventory)?;
        let stale = count_stale_references(&stored, &identity_map);
        if stale.filesystem_file_ids != 0 || stale.path_fallback_ids != 0 {
            return Err(DesktopStateStoreError::StaleReferences {
                filesystem_file_ids: stale.filesystem_file_ids,
                path_fallback_ids: stale.path_fallback_ids,
            });
        }

        let groups = stored
            .groups
            .into_iter()
            .map(|group| {
                let members = group
                    .members
                    .into_iter()
                    .map(|member| DesktopItemId::new(member.id))
                    .collect::<Result<Vec<_>, _>>()?;
                DesktopGroup::new(group.id, group.label, members)
            })
            .collect::<Result<Vec<_>, DesktopModelError>>()?;
        let groups = DesktopGroups::new(groups, &available_items)?;

        let entries = stored
            .layout
            .into_iter()
            .map(|entry| {
                let target = match entry.target {
                    StoredTarget::Item(member) => {
                        DesktopLayoutTarget::Item(DesktopItemId::new(member.id)?)
                    }
                    StoredTarget::Group(group_id) => DesktopLayoutTarget::Group(group_id),
                };
                DesktopLayoutEntry::new(
                    target,
                    entry.monitor_id,
                    entry.x_basis_points,
                    entry.y_basis_points,
                    entry.z_order,
                )
            })
            .collect::<Result<Vec<_>, DesktopModelError>>()?;
        let layout = DesktopLayout::new(entries, &available_items, &groups, topology)?;

        Ok(Some(DesktopStateSnapshot { groups, layout }))
    }

    /// Validates and atomically commits the supplied local Groups/layout.
    ///
    /// The current inventory and monitor snapshot are required so stale item
    /// references or session-only monitor IDs cannot be written as valid state.
    pub fn save(
        &self,
        groups: &DesktopGroups,
        layout: &DesktopLayout,
        inventory: &[DesktopItemIdentityRecord],
        topology: &MonitorTopology,
    ) -> Result<(), DesktopStateStoreError> {
        let (available_items, identity_map) = build_inventory_index(inventory)?;
        let groups = DesktopGroups::new(groups.groups().to_vec(), &available_items)?;
        let layout = DesktopLayout::new(
            layout.entries().to_vec(),
            &available_items,
            &groups,
            topology,
        )?;

        let mut stored_groups = Vec::with_capacity(groups.len());
        for group in groups.groups() {
            let mut members = Vec::with_capacity(group.members().len());
            for member in group.members() {
                members.push(StoredMember {
                    id: member.as_str().to_owned(),
                    stability: *identity_map
                        .get(member.as_str())
                        .ok_or(DesktopStateStoreError::InvalidData)?,
                });
            }
            stored_groups.push(StoredGroup {
                id: group.id().to_owned(),
                label: group.label().to_owned(),
                members,
            });
        }

        let mut stored_layout = Vec::with_capacity(layout.len());
        for entry in layout.entries() {
            let target = match entry.target() {
                DesktopLayoutTarget::Item(item_id) => StoredTarget::Item(StoredMember {
                    id: item_id.as_str().to_owned(),
                    stability: *identity_map
                        .get(item_id.as_str())
                        .ok_or(DesktopStateStoreError::InvalidData)?,
                }),
                DesktopLayoutTarget::Group(group_id) => {
                    StoredTarget::Group(group_id.clone())
                }
            };
            stored_layout.push(StoredLayoutEntry {
                target,
                monitor_id: entry.monitor_id().to_owned(),
                x_basis_points: entry.x_basis_points(),
                y_basis_points: entry.y_basis_points(),
                z_order: entry.z_order(),
            });
        }

        let bytes = encode_file(&StoredDesktopState {
            groups: stored_groups,
            layout: stored_layout,
        })?;
        write_atomically(&self.path, &bytes)?;
        Ok(())
    }
}

fn build_inventory_index(
    inventory: &[DesktopItemIdentityRecord],
) -> Result<
    (
        Vec<DesktopItemRef>,
        HashMap<String, DesktopItemIdentityStability>,
    ),
    DesktopStateStoreError,
> {
    if inventory.len() > MAX_DESKTOP_INVENTORY_ITEMS {
        return Err(DesktopStateStoreError::InvalidData);
    }
    let mut items = Vec::with_capacity(inventory.len());
    let mut identities = HashMap::with_capacity(inventory.len());
    for entry in inventory {
        let id = entry.item.id().as_str().to_owned();
        if identities.insert(id, entry.stability).is_some() {
            return Err(DesktopStateStoreError::InvalidModel(
                DesktopModelError::DuplicateItemIdentifier,
            ));
        }
        items.push(entry.item.clone());
    }
    Ok((items, identities))
}

#[derive(Default)]
struct StaleReferenceCounts {
    filesystem_file_ids: usize,
    path_fallback_ids: usize,
}

fn count_stale_references(
    stored: &StoredDesktopState,
    current: &HashMap<String, DesktopItemIdentityStability>,
) -> StaleReferenceCounts {
    let mut counts = StaleReferenceCounts::default();
    let mut seen = HashSet::new();

    let mut inspect = |member: &StoredMember| {
        if current.get(&member.id) != Some(&member.stability) && seen.insert(member.id.clone()) {
            match member.stability {
                DesktopItemIdentityStability::FilesystemFileId => {
                    counts.filesystem_file_ids += 1;
                }
                DesktopItemIdentityStability::PathFallback => {
                    counts.path_fallback_ids += 1;
                }
            }
        }
    };

    for group in &stored.groups {
        for member in &group.members {
            inspect(member);
        }
    }
    for entry in &stored.layout {
        if let StoredTarget::Item(member) = &entry.target {
            inspect(member);
        }
    }
    counts
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StoredDesktopState {
    groups: Vec<StoredGroup>,
    layout: Vec<StoredLayoutEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StoredGroup {
    id: String,
    label: String,
    members: Vec<StoredMember>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StoredMember {
    id: String,
    stability: DesktopItemIdentityStability,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum StoredTarget {
    Item(StoredMember),
    Group(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StoredLayoutEntry {
    target: StoredTarget,
    monitor_id: String,
    x_basis_points: u16,
    y_basis_points: u16,
    z_order: u32,
}

fn encode_file(stored: &StoredDesktopState) -> Result<Vec<u8>, DesktopStateStoreError> {
    if stored.groups.len() > MAX_DESKTOP_GROUPS
        || stored.layout.len() > MAX_DESKTOP_LAYOUT_ENTRIES
    {
        return Err(DesktopStateStoreError::InvalidData);
    }

    let mut payload = Writer::default();
    payload.u16(stored.groups.len() as u16);
    for group in &stored.groups {
        payload.string(&group.id, MAX_IDENTIFIER_BYTES)?;
        payload.string(&group.label, MAX_LABEL_BYTES)?;
        if group.members.is_empty() || group.members.len() > MAX_DESKTOP_GROUP_MEMBERS {
            return Err(DesktopStateStoreError::InvalidData);
        }
        payload.u16(group.members.len() as u16);
        for member in &group.members {
            payload.string(&member.id, MAX_IDENTIFIER_BYTES)?;
            payload.u8(encode_stability(member.stability));
        }
    }

    payload.u16(stored.layout.len() as u16);
    for entry in &stored.layout {
        match &entry.target {
            StoredTarget::Item(member) => {
                payload.u8(0);
                payload.string(&member.id, MAX_IDENTIFIER_BYTES)?;
                payload.u8(encode_stability(member.stability));
            }
            StoredTarget::Group(group_id) => {
                payload.u8(1);
                payload.string(group_id, MAX_IDENTIFIER_BYTES)?;
            }
        }
        payload.string(&entry.monitor_id, MAX_MONITOR_ID_BYTES)?;
        payload.u16(entry.x_basis_points);
        payload.u16(entry.y_basis_points);
        payload.u32(entry.z_order);
    }

    let payload = payload.bytes;
    if payload.len() + HEADER_LENGTH > MAX_STATE_FILE_BYTES {
        return Err(DesktopStateStoreError::InvalidData);
    }

    let mut file = Writer::default();
    file.bytes.extend_from_slice(MAGIC);
    file.u16(FORMAT_VERSION);
    file.u32(payload.len() as u32);
    file.u32(crc32(&payload));
    file.bytes.extend_from_slice(&payload);
    Ok(file.bytes)
}

fn decode_file(bytes: &[u8]) -> Result<StoredDesktopState, DesktopStateStoreError> {
    if bytes.len() < HEADER_LENGTH || bytes.len() > MAX_STATE_FILE_BYTES {
        return Err(DesktopStateStoreError::InvalidData);
    }
    let mut header = Reader::new(bytes);
    if header.take(MAGIC.len())? != MAGIC {
        return Err(DesktopStateStoreError::InvalidData);
    }
    let version = header.u16()?;
    if version != FORMAT_VERSION {
        return Err(DesktopStateStoreError::UnsupportedVersion(version));
    }
    let payload_length = header.u32()? as usize;
    let expected_checksum = header.u32()?;
    if payload_length != header.remaining() {
        return Err(DesktopStateStoreError::InvalidData);
    }
    let payload = header.take(payload_length)?;
    if crc32(payload) != expected_checksum {
        return Err(DesktopStateStoreError::ChecksumMismatch);
    }

    let mut reader = Reader::new(payload);
    let group_count = reader.u16()? as usize;
    if group_count > MAX_DESKTOP_GROUPS {
        return Err(DesktopStateStoreError::InvalidData);
    }
    let mut groups = Vec::with_capacity(group_count);
    for _ in 0..group_count {
        let id = reader.string(MAX_IDENTIFIER_BYTES)?;
        let label = reader.string(MAX_LABEL_BYTES)?;
        let member_count = reader.u16()? as usize;
        if member_count == 0 || member_count > MAX_DESKTOP_GROUP_MEMBERS {
            return Err(DesktopStateStoreError::InvalidData);
        }
        let mut members = Vec::with_capacity(member_count);
        for _ in 0..member_count {
            members.push(StoredMember {
                id: reader.string(MAX_IDENTIFIER_BYTES)?,
                stability: decode_stability(reader.u8()?)?,
            });
        }
        groups.push(StoredGroup { id, label, members });
    }

    let layout_count = reader.u16()? as usize;
    if layout_count > MAX_DESKTOP_LAYOUT_ENTRIES {
        return Err(DesktopStateStoreError::InvalidData);
    }
    let mut layout = Vec::with_capacity(layout_count);
    for _ in 0..layout_count {
        let target = match reader.u8()? {
            0 => StoredTarget::Item(StoredMember {
                id: reader.string(MAX_IDENTIFIER_BYTES)?,
                stability: decode_stability(reader.u8()?)?,
            }),
            1 => StoredTarget::Group(reader.string(MAX_IDENTIFIER_BYTES)?),
            _ => return Err(DesktopStateStoreError::InvalidData),
        };
        layout.push(StoredLayoutEntry {
            target,
            monitor_id: reader.string(MAX_MONITOR_ID_BYTES)?,
            x_basis_points: reader.u16()?,
            y_basis_points: reader.u16()?,
            z_order: reader.u32()?,
        });
    }
    if reader.remaining() != 0 {
        return Err(DesktopStateStoreError::InvalidData);
    }

    Ok(StoredDesktopState { groups, layout })
}

fn encode_stability(stability: DesktopItemIdentityStability) -> u8 {
    match stability {
        DesktopItemIdentityStability::FilesystemFileId => 0,
        DesktopItemIdentityStability::PathFallback => 1,
    }
}

fn decode_stability(value: u8) -> Result<DesktopItemIdentityStability, DesktopStateStoreError> {
    match value {
        0 => Ok(DesktopItemIdentityStability::FilesystemFileId),
        1 => Ok(DesktopItemIdentityStability::PathFallback),
        _ => Err(DesktopStateStoreError::InvalidData),
    }
}

#[derive(Default)]
struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    fn u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    fn u16(&mut self, value: u16) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn string(&mut self, value: &str, maximum: usize) -> Result<(), DesktopStateStoreError> {
        if value.len() > maximum || value.contains('\0') {
            return Err(DesktopStateStoreError::InvalidData);
        }
        self.u16(value.len() as u16);
        self.bytes.extend_from_slice(value.as_bytes());
        Ok(())
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.offset)
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], DesktopStateStoreError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(DesktopStateStoreError::InvalidData)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(DesktopStateStoreError::InvalidData)?;
        self.offset = end;
        Ok(value)
    }

    fn u8(&mut self) -> Result<u8, DesktopStateStoreError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, DesktopStateStoreError> {
        let bytes = self.take(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn u32(&mut self) -> Result<u32, DesktopStateStoreError> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn string(&mut self, maximum: usize) -> Result<String, DesktopStateStoreError> {
        let length = self.u16()? as usize;
        if length > maximum {
            return Err(DesktopStateStoreError::InvalidData);
        }
        String::from_utf8(self.take(length)?.to_vec())
            .map_err(|_| DesktopStateStoreError::InvalidData)
    }
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

fn write_atomically(path: &Path, bytes: &[u8]) -> Result<(), DesktopStateStoreError> {
    let parent = path
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .ok_or(DesktopStateStoreError::InvalidPath)?;
    let file_name = path.file_name().ok_or(DesktopStateStoreError::InvalidPath)?;
    fs::create_dir_all(parent)?;

    for _ in 0..TEMP_FILE_ATTEMPTS {
        let sequence = TEMP_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temporary_name = format!(
            ".{}.{}.{}.tmp",
            file_name.to_string_lossy(),
            std::process::id(),
            sequence
        );
        let temporary_path = parent.join(temporary_name);
        let mut file = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary_path)
        {
            Ok(file) => file,
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        };

        let result = (|| {
            file.write_all(bytes)?;
            file.sync_all()?;
            drop(file);
            fs::rename(&temporary_path, path)?;
            Ok::<(), std::io::Error>(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary_path);
        }
        result?;
        return Ok(());
    }

    Err(DesktopStateStoreError::Io(std::io::Error::new(
        ErrorKind::AlreadyExists,
        "could not allocate a unique WLDE temporary state file",
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    use wl_wlde::{DesktopItemKind, Monitor, Rect};

    struct TestDirectory(PathBuf);

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn test_directory() -> TestDirectory {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "winlook-wlde-store-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create isolated test directory");
        TestDirectory(path)
    }

    fn fixture() -> (
        Vec<DesktopItemIdentityRecord>,
        DesktopGroups,
        DesktopLayout,
        MonitorTopology,
    ) {
        let application = DesktopItemRef::new(
            "desktop:file:00000001:0000000000000001:0000000000000001",
            "Editor",
            DesktopItemKind::Application,
        )
        .expect("valid application item");
        let notes = DesktopItemRef::new("desktop:path:notes", "Notes.txt", DesktopItemKind::File)
            .expect("valid file item");
        let inventory = vec![
            DesktopItemIdentityRecord::new(
                application.clone(),
                DesktopItemIdentityStability::FilesystemFileId,
            ),
            DesktopItemIdentityRecord::new(
                notes.clone(),
                DesktopItemIdentityStability::PathFallback,
            ),
        ];
        let groups = DesktopGroups::new(
            vec![DesktopGroup::new(
                "group:work",
                "Work",
                vec![DesktopItemId::new(application.id().as_str()).unwrap()],
            )
            .unwrap()],
            &[application.clone(), notes.clone()],
        )
        .unwrap();
        let topology = topology();
        let layout = DesktopLayout::new(
            vec![
                DesktopLayoutEntry::new(
                    DesktopLayoutTarget::Group("group:work".to_owned()),
                    "DISPLAY1",
                    1500,
                    2000,
                    0,
                )
                .unwrap(),
                DesktopLayoutEntry::new(
                    DesktopLayoutTarget::Item(notes.id().clone()),
                    "DISPLAY1",
                    4500,
                    2000,
                    1,
                )
                .unwrap(),
            ],
            &[application, notes],
            &groups,
            &topology,
        )
        .unwrap();
        (inventory, groups, layout, topology)
    }

    fn topology() -> MonitorTopology {
        let rectangle = Rect::new(0.0, 0.0, 1920.0, 1080.0).unwrap();
        MonitorTopology::new(vec![
            Monitor::new("DISPLAY1", rectangle, rectangle, true).unwrap(),
        ])
        .unwrap()
    }

    #[test]
    fn state_round_trips_groups_layout_and_identity_kinds() {
        let directory = test_directory();
        let store = DesktopStateStore::at_path(directory.0.join("desktop-state.v1.bin"));
        let (inventory, groups, layout, topology) = fixture();

        store
            .save(&groups, &layout, &inventory, &topology)
            .expect("commit valid state");
        let loaded = store
            .load(&inventory, &topology)
            .expect("load valid state")
            .expect("state exists");

        assert_eq!(loaded.groups(), &groups);
        assert_eq!(loaded.layout(), &layout);
    }

    #[test]
    fn path_fallback_rename_is_reported_without_overwriting_saved_state() {
        let directory = test_directory();
        let store = DesktopStateStore::at_path(directory.0.join("desktop-state.v1.bin"));
        let (inventory, groups, layout, topology) = fixture();
        store
            .save(&groups, &layout, &inventory, &topology)
            .expect("commit valid state");

        let saved = fs::read(store.path()).expect("read saved bytes");
        let changed_inventory = vec![
            inventory[0].clone(),
            DesktopItemIdentityRecord::new(
                DesktopItemRef::new(
                    "desktop:path:notes-renamed",
                    "Notes.txt",
                    DesktopItemKind::File,
                )
                .unwrap(),
                DesktopItemIdentityStability::PathFallback,
            ),
        ];

        assert!(matches!(
            store.load(&changed_inventory, &topology),
            Err(DesktopStateStoreError::StaleReferences {
                filesystem_file_ids: 0,
                path_fallback_ids: 1
            })
        ));
        assert_eq!(fs::read(store.path()).unwrap(), saved);
    }

    #[test]
    fn checksum_failure_does_not_mutate_or_repair_the_file() {
        let directory = test_directory();
        let store = DesktopStateStore::at_path(directory.0.join("desktop-state.v1.bin"));
        let (inventory, groups, layout, topology) = fixture();
        store
            .save(&groups, &layout, &inventory, &topology)
            .expect("commit valid state");

        let mut bytes = fs::read(store.path()).expect("read saved bytes");
        let last = bytes.last_mut().expect("state file is non-empty");
        *last ^= 0x80;
        fs::write(store.path(), &bytes).expect("corrupt test state");
        let corrupt = fs::read(store.path()).unwrap();

        assert!(matches!(
            store.load(&inventory, &topology),
            Err(DesktopStateStoreError::ChecksumMismatch)
        ));
        assert_eq!(fs::read(store.path()).unwrap(), corrupt);
    }

    #[test]
    fn absent_store_is_not_an_error() {
        let directory = test_directory();
        let store = DesktopStateStore::at_path(directory.0.join("missing.bin"));
        let (inventory, _, _, topology) = fixture();
        assert!(store.load(&inventory, &topology).unwrap().is_none());
    }

    #[test]
    fn malformed_trailing_bytes_are_rejected() {
        let mut file = encode_file(&StoredDesktopState {
            groups: Vec::new(),
            layout: Vec::new(),
        })
        .unwrap();
        file.push(0);
        assert!(matches!(
            decode_file(&file),
            Err(DesktopStateStoreError::InvalidData)
        ));
    }
}
