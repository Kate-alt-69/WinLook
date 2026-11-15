//! Read-only filesystem inventory for the current Windows desktop folders.
//!
//! This is deliberately not a Shell namespace/IFolderView enumerator. It
//! snapshots one level of the per-user and Public Desktop filesystem folders;
//! virtual namespace items and Explorer visibility/filter state need a later
//! Shell adapter.

use std::ffi::OsStr;
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use wl_wlde::{DesktopItemKind, DesktopItemRef, DesktopModelError};

/// Maximum number of filesystem entries accepted from both Desktop roots.
pub const MAX_DESKTOP_INVENTORY_ITEMS: usize = 4096;
const MAX_KNOWN_FOLDER_PATH_UNITS: usize = 32_768;

/// The Known Folder from which a desktop entry was discovered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DesktopFolder {
    /// The current user's redirected Desktop folder.
    User,
    /// The machine-wide Public Desktop folder.
    Public,
}

/// Describes how confidently an inventory ID survives filesystem changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DesktopItemIdentityStability {
    /// Derived from a volume serial and filesystem file ID.
    ///
    /// Stable across ordinary same-volume renames on filesystems that preserve
    /// file IDs; not guaranteed across deletion or every filesystem.
    FilesystemFileId,
    /// Derived from the current path because a unique file ID was unavailable
    /// or the entry has multiple hard links. Changes when the path changes.
    PathFallback,
}

/// A desktop item plus its ephemeral filesystem location.
///
/// item() is the portable identity stored in a Group. filesystem_path() is
/// only for immediate local Shell operations; do not store it in Group
/// configuration, templates, remote messages or ordinary diagnostics.
#[derive(Clone)]
pub struct DesktopInventoryItem {
    item: DesktopItemRef,
    filesystem_path: PathBuf,
    source_folder: DesktopFolder,
    identity_stability: DesktopItemIdentityStability,
}

impl DesktopInventoryItem {
    /// Validated, path-free item identity and display metadata.
    pub fn item(&self) -> &DesktopItemRef {
        &self.item
    }

    /// Filesystem location for a local operation in the current session.
    ///
    /// This path is not a durable identifier. Re-enumerate after renames,
    /// moves, Desktop redirection changes or Shell refreshes.
    pub fn filesystem_path(&self) -> &Path {
        &self.filesystem_path
    }

    /// Whether the item came from the user's or Public Desktop root.
    pub fn source_folder(&self) -> DesktopFolder {
        self.source_folder
    }

    /// Whether the ID is filesystem-backed or a path fallback.
    ///
    /// Persistent Group storage must reconcile PathFallback entries before
    /// restoring membership because their IDs change when their paths change.
    pub fn identity_stability(&self) -> DesktopItemIdentityStability {
        self.identity_stability
    }
}

/// Errors from the bounded filesystem Desktop snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DesktopInventoryError {
    /// The filesystem inventory is available only on Windows.
    UnsupportedPlatform,
    /// Windows could not resolve one of the Known Folder paths.
    KnownFolderPath {
        folder: DesktopFolder,
        hresult: u32,
    },
    /// A Known Folder path was null, invalid or not terminated within bounds.
    InvalidKnownFolderPath {
        folder: DesktopFolder,
    },
    /// The root could not be enumerated.
    ReadDirectory { folder: DesktopFolder, code: u32 },
    /// An entry could not be inspected.
    ReadDirectoryEntry { folder: DesktopFolder, code: u32 },
    /// The inventory exceeded its fixed item bound.
    TooManyItems,
    /// A Desktop folder change-notification handle could not be created.
    WatchDirectory { folder: DesktopFolder, code: u32 },
    /// Waiting for a Desktop folder change notification failed.
    WaitForChange { code: u32 },
    /// A signaled Desktop folder notification could not be re-armed.
    RearmDirectoryWatch { folder: DesktopFolder, code: u32 },
    /// The entry label or generated identity failed portable validation.
    InvalidItem(DesktopModelError),
}

impl fmt::Display for DesktopInventoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => {
                f.write_str("desktop filesystem inventory is available only on Windows")
            }
            Self::KnownFolderPath { folder, hresult } => {
                write!(
                    f,
                    "could not resolve the {folder:?} Desktop Known Folder (HRESULT {hresult:#x})"
                )
            }
            Self::InvalidKnownFolderPath { folder } => {
                write!(
                    f,
                    "Windows returned an invalid {folder:?} Desktop Known Folder path"
                )
            }
            Self::ReadDirectory { folder, code } => {
                write!(
                    f,
                    "could not enumerate the {folder:?} Desktop folder (error {code:#x})"
                )
            }
            Self::ReadDirectoryEntry { folder, code } => {
                write!(
                    f,
                    "could not inspect an entry in the {folder:?} Desktop folder (error {code:#x})"
                )
            }
            Self::TooManyItems => {
                write!(
                    f,
                    "Desktop inventory exceeds the {}-item bound",
                    MAX_DESKTOP_INVENTORY_ITEMS
                )
            }
            Self::WatchDirectory { folder, code } => {
                write!(
                    f,
                    "could not watch the {folder:?} Desktop folder (error {code:#x})"
                )
            }
            Self::WaitForChange { code } => {
                write!(
                    f,
                    "waiting for Desktop folder changes failed (error {code:#x})"
                )
            }
            Self::RearmDirectoryWatch { folder, code } => {
                write!(
                    f,
                    "could not re-arm the {folder:?} Desktop folder watcher (error {code:#x})"
                )
            }
            Self::InvalidItem(error) => write!(f, "Desktop item failed validation: {error}"),
        }
    }
}

impl std::error::Error for DesktopInventoryError {}

/// Read-only snapshot of the current user's Desktop plus Public Desktop.
///
/// The Known Folder API is used rather than hardcoded profile paths, so folder
/// redirection and per-user location changes are respected. Enumeration is
/// one-level only: it does not crawl folders, follow links, resolve shortcut
/// targets, inspect file contents or mutate the filesystem.
///
/// IDs prefer a volume-scoped Windows file identity, preserving Group keys
/// across ordinary same-volume renames on filesystems that retain file IDs.
/// Entries with unavailable or ambiguous identities use a marked path-hash
/// fallback. IDs are opaque Group keys, not security credentials. This first
/// pass classifies common launcher extensions as Application; a later Shell-link
/// resolver can refine shortcuts whose targets are documents or folders.
#[cfg(windows)]
pub fn enumerate_desktop_items() -> Result<Vec<DesktopInventoryItem>, DesktopInventoryError> {
    desktop_inventory_platform::enumerate()
}

/// Non-Windows builds expose an explicit unsupported-platform result.
#[cfg(not(windows))]
pub fn enumerate_desktop_items() -> Result<Vec<DesktopInventoryItem>, DesktopInventoryError> {
    Err(DesktopInventoryError::UnsupportedPlatform)
}

/// A bounded change watcher for the user's and Public Desktop folders.
///
/// The watcher observes top-level directory entries only. It never enumerates
/// recursively, reads file contents or mutates desktop files.
pub struct DesktopInventoryWatcher {
    #[cfg(windows)]
    inner: desktop_inventory_platform::Watcher,
}

impl DesktopInventoryWatcher {
    /// Waits for a Desktop folder change or the requested timeout.
    ///
    /// Returns true when the caller should refresh the inventory. When a
    /// Known Folder is temporarily absent, timeout also requests a refresh so
    /// the watcher can notice that the folder has appeared.
    pub fn wait_for_change(&self, timeout: Duration) -> Result<bool, DesktopInventoryError> {
        #[cfg(windows)]
        {
            self.inner.wait_for_change(timeout)
        }
        #[cfg(not(windows))]
        {
            let _ = timeout;
            Err(DesktopInventoryError::UnsupportedPlatform)
        }
    }
}

/// Opens change notifications for the current User and Public Desktop folders.
#[cfg(windows)]
pub fn watch_desktop_folders() -> Result<DesktopInventoryWatcher, DesktopInventoryError> {
    Ok(DesktopInventoryWatcher {
        inner: desktop_inventory_platform::watch()?,
    })
}

/// Non-Windows builds expose an explicit unsupported-platform result.
#[cfg(not(windows))]
pub fn watch_desktop_folders() -> Result<DesktopInventoryWatcher, DesktopInventoryError> {
    Err(DesktopInventoryError::UnsupportedPlatform)
}

fn classify_desktop_entry(path: &Path, is_directory: bool) -> DesktopItemKind {
    if is_directory {
        return DesktopItemKind::Folder;
    }

    let extension = path.extension().and_then(OsStr::to_str).unwrap_or_default();
    if ["lnk", "url", "appref-ms", "exe", "com", "bat", "cmd"]
        .iter()
        .any(|candidate| extension.eq_ignore_ascii_case(candidate))
    {
        DesktopItemKind::Application
    } else {
        DesktopItemKind::File
    }
}

#[cfg(any(windows, test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct DesktopFileIdentity {
    volume_serial_number: u32,
    file_index: u64,
    creation_time_100ns: u64,
    number_of_links: u32,
}

#[cfg(any(windows, test))]
fn stable_desktop_id(identity: DesktopFileIdentity) -> String {
    format!(
        "desktop:file:{:08x}:{:016x}:{:016x}",
        identity.volume_serial_number, identity.file_index, identity.creation_time_100ns
    )
}

#[cfg(any(windows, test))]
fn resolve_desktop_item_identity(
    identity: Option<DesktopFileIdentity>,
    identity_occurrences: usize,
    path_fallback_id: String,
) -> (String, DesktopItemIdentityStability) {
    match identity {
        Some(identity) if identity_occurrences == 1 && identity.number_of_links == 1 => (
            stable_desktop_id(identity),
            DesktopItemIdentityStability::FilesystemFileId,
        ),
        _ => (path_fallback_id, DesktopItemIdentityStability::PathFallback),
    }
}

#[cfg(windows)]
mod desktop_inventory_platform {
    use super::*;
    use sha2::{Digest, Sha256};
    use std::collections::{HashMap, HashSet};
    use std::ffi::{c_void, OsString};
    use std::io::ErrorKind;
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use std::ptr;

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Guid {
        data1: u32,
        data2: u16,
        data3: u16,
        data4: [u8; 8],
    }

    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    const FILE_NOTIFY_CHANGE_DIR_NAME: u32 = 0x0000_0002;
    const WAIT_OBJECT_0: u32 = 0x0000_0000;
    const WAIT_TIMEOUT: u32 = 0x0000_0102;
    const WAIT_FAILED: u32 = 0xFFFF_FFFF;
    const ERROR_FILE_NOT_FOUND: u32 = 2;
    const ERROR_PATH_NOT_FOUND: u32 = 3;
    const MAX_PATH_UNITS: usize = 260;
    const EXTENDED_PATH_PREFIX: &[u16] = &[92, 92, 63, 92];
    const EXTENDED_UNC_PATH_PREFIX: &[u16] = &[92, 92, 63, 92, 85, 78, 67, 92];

    const FILE_READ_ATTRIBUTES: u32 = 0x0000_0080;
    const FILE_SHARE_READ: u32 = 0x0000_0001;
    const FILE_SHARE_WRITE: u32 = 0x0000_0002;
    const FILE_SHARE_DELETE: u32 = 0x0000_0004;
    const OPEN_EXISTING: u32 = 3;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    const INVALID_HANDLE_VALUE: isize = -1;

    // KNOWNFOLDERID values from KnownFolders.h / Microsoft Learn.
    const FOLDERID_DESKTOP: Guid = Guid {
        data1: 0xB4BF_CC3A,
        data2: 0xDB2C,
        data3: 0x424C,
        data4: [0xB0, 0x29, 0x7F, 0xE9, 0x9A, 0x87, 0xC6, 0x41],
    };
    const FOLDERID_PUBLIC_DESKTOP: Guid = Guid {
        data1: 0xC4AA_340D,
        data2: 0xF20F,
        data3: 0x4863,
        data4: [0xAF, 0xEF, 0xF8, 0x7E, 0xF2, 0xE6, 0xBA, 0x25],
    };

    struct CoTaskMemString(*mut u16);

    impl Drop for CoTaskMemString {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    CoTaskMemFree(self.0.cast());
                }
            }
        }
    }

    #[link(name = "shell32")]
    extern "system" {
        fn SHGetKnownFolderPath(
            folder_id: *const Guid,
            flags: u32,
            token: *mut c_void,
            path: *mut *mut u16,
        ) -> i32;
    }

    #[link(name = "ole32")]
    extern "system" {
        fn CoTaskMemFree(memory: *mut c_void);
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn CreateFileW(
            path: *const u16,
            desired_access: u32,
            share_mode: u32,
            security_attributes: *mut c_void,
            creation_disposition: u32,
            flags_and_attributes: u32,
            template_file: Handle,
        ) -> Handle;
        fn GetFileInformationByHandle(
            file: Handle,
            information: *mut ByHandleFileInformation,
        ) -> i32;
        fn CloseHandle(object: Handle) -> i32;
        fn FindFirstChangeNotificationW(
            path: *const u16,
            watch_subtree: i32,
            filter: u32,
        ) -> Handle;
        fn FindNextChangeNotification(change_handle: Handle) -> i32;
        fn FindCloseChangeNotification(change_handle: Handle) -> i32;
        fn WaitForMultipleObjects(
            count: u32,
            handles: *const Handle,
            wait_all: i32,
            milliseconds: u32,
        ) -> u32;
        fn GetLastError() -> u32;
    }

    type Handle = *mut c_void;

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct FileTime {
        low_date_time: u32,
        high_date_time: u32,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct ByHandleFileInformation {
        file_attributes: u32,
        creation_time: FileTime,
        last_access_time: FileTime,
        last_write_time: FileTime,
        volume_serial_number: u32,
        file_size_high: u32,
        file_size_low: u32,
        number_of_links: u32,
        file_index_high: u32,
        file_index_low: u32,
    }

    struct OwnedFileHandle(Handle);

    impl Drop for OwnedFileHandle {
        fn drop(&mut self) {
            if !self.0.is_null() && self.0 as isize != INVALID_HANDLE_VALUE {
                unsafe {
                    CloseHandle(self.0);
                }
            }
        }
    }

    struct DesktopInventoryCandidate {
        path: PathBuf,
        label: String,
        kind: DesktopItemKind,
        source_folder: DesktopFolder,
        file_identity: Option<DesktopFileIdentity>,
    }

    struct ChangeNotification {
        folder: DesktopFolder,
        handle: Handle,
    }

    impl Drop for ChangeNotification {
        fn drop(&mut self) {
            if !self.handle.is_null() {
                unsafe {
                    FindCloseChangeNotification(self.handle);
                }
            }
        }
    }

    pub(super) struct Watcher {
        notifications: Vec<ChangeNotification>,
        refresh_on_timeout: bool,
    }

    pub(super) fn watch() -> Result<Watcher, DesktopInventoryError> {
        let roots = [
            (DesktopFolder::User, known_folder_path(DesktopFolder::User)?),
            (
                DesktopFolder::Public,
                known_folder_path(DesktopFolder::Public)?,
            ),
        ];
        let mut notifications = Vec::with_capacity(2);
        let mut refresh_on_timeout = false;
        let filter = FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_DIR_NAME;

        for (folder, root) in roots {
            if !root.is_dir() {
                refresh_on_timeout = true;
                continue;
            }
            let path_units = root.as_os_str().encode_wide().collect::<Vec<_>>();
            if path_units.starts_with(&[92, 92]) {
                // Remote redirects do not reliably deliver change notifications.
                refresh_on_timeout = true;
            }
            let wide = extended_length_path(path_units);
            let handle = unsafe { FindFirstChangeNotificationW(wide.as_ptr(), 0, filter) };
            if handle.is_null() || handle as isize == -1 {
                let code = unsafe { GetLastError() };
                if code == ERROR_FILE_NOT_FOUND || code == ERROR_PATH_NOT_FOUND {
                    refresh_on_timeout = true;
                    continue;
                }
                return Err(DesktopInventoryError::WatchDirectory { folder, code });
            }
            notifications.push(ChangeNotification { folder, handle });
        }

        Ok(Watcher {
            notifications,
            refresh_on_timeout,
        })
    }

    fn extended_length_path(mut wide: Vec<u16>) -> Vec<u16> {
        let already_extended = wide.starts_with(EXTENDED_PATH_PREFIX);
        if wide.len() >= MAX_PATH_UNITS && !already_extended {
            let mut extended = Vec::with_capacity(wide.len() + EXTENDED_UNC_PATH_PREFIX.len());
            if wide.starts_with(&[92, 92]) {
                extended.extend_from_slice(EXTENDED_UNC_PATH_PREFIX);
                extended.extend_from_slice(&wide[2..]);
            } else {
                extended.extend_from_slice(EXTENDED_PATH_PREFIX);
                extended.extend_from_slice(&wide);
            }
            wide = extended;
        }
        wide.push(0);
        wide
    }

    impl Watcher {
        pub(super) fn wait_for_change(
            &self,
            timeout: Duration,
        ) -> Result<bool, DesktopInventoryError> {
            if self.notifications.is_empty() {
                std::thread::sleep(timeout);
                return Ok(true);
            }

            let handles = self
                .notifications
                .iter()
                .map(|notification| notification.handle)
                .collect::<Vec<_>>();
            let timeout_ms = timeout.as_millis().min(u128::from(u32::MAX - 1)) as u32;
            let result = unsafe {
                WaitForMultipleObjects(handles.len() as u32, handles.as_ptr(), 0, timeout_ms)
            };
            if result == WAIT_TIMEOUT {
                return Ok(self.refresh_on_timeout);
            }
            if result == WAIT_FAILED {
                return Err(DesktopInventoryError::WaitForChange {
                    code: unsafe { GetLastError() },
                });
            }
            let index = result.wrapping_sub(WAIT_OBJECT_0) as usize;
            if index >= self.notifications.len() {
                return Err(DesktopInventoryError::WaitForChange { code: result });
            }
            let notification = &self.notifications[index];
            if unsafe { FindNextChangeNotification(notification.handle) } == 0 {
                return Err(DesktopInventoryError::RearmDirectoryWatch {
                    folder: notification.folder,
                    code: unsafe { GetLastError() },
                });
            }
            Ok(true)
        }
    }

    pub(super) fn enumerate() -> Result<Vec<DesktopInventoryItem>, DesktopInventoryError> {
        let roots = [
            (DesktopFolder::User, known_folder_path(DesktopFolder::User)?),
            (
                DesktopFolder::Public,
                known_folder_path(DesktopFolder::Public)?,
            ),
        ];
        let mut candidates = Vec::new();

        for (folder, root) in roots {
            read_one_level(&root, folder, &mut candidates)?;
        }

        // File IDs identify filesystem objects, not individual hard-link names.
        // Count identities before assignment so distinct entries are not merged.
        let mut identity_counts = HashMap::with_capacity(candidates.len());
        for candidate in &candidates {
            if let Some(identity) = candidate.file_identity {
                *identity_counts.entry(identity).or_insert(0usize) += 1;
            }
        }

        let mut identifiers = HashSet::with_capacity(candidates.len());
        let mut inventory = Vec::with_capacity(candidates.len());
        for candidate in candidates {
            let identity_occurrences = candidate
                .file_identity
                .and_then(|identity| identity_counts.get(&identity).copied())
                .unwrap_or(0);
            let fallback_id = path_fallback_desktop_id(&candidate.path);
            let (id, identity_stability) = resolve_desktop_item_identity(
                candidate.file_identity,
                identity_occurrences,
                fallback_id,
            );
            if !identifiers.insert(id.clone()) {
                continue;
            }

            let item = DesktopItemRef::new(id, candidate.label, candidate.kind)
                .map_err(DesktopInventoryError::InvalidItem)?;
            inventory.push(DesktopInventoryItem {
                item,
                filesystem_path: candidate.path,
                source_folder: candidate.source_folder,
                identity_stability,
            });
        }

        inventory.sort_by(|left, right| {
            left.item
                .label()
                .cmp(right.item.label())
                .then_with(|| left.item.id().as_str().cmp(right.item.id().as_str()))
        });
        Ok(inventory)
    }

    fn known_folder_path(folder: DesktopFolder) -> Result<PathBuf, DesktopInventoryError> {
        let folder_id = match folder {
            DesktopFolder::User => &FOLDERID_DESKTOP,
            DesktopFolder::Public => &FOLDERID_PUBLIC_DESKTOP,
        };
        let mut raw_path = ptr::null_mut();
        let hresult = unsafe { SHGetKnownFolderPath(folder_id, 0, ptr::null_mut(), &mut raw_path) };
        let path = CoTaskMemString(raw_path);

        if hresult < 0 {
            return Err(DesktopInventoryError::KnownFolderPath {
                folder,
                hresult: hresult as u32,
            });
        }
        if path.0.is_null() {
            return Err(DesktopInventoryError::InvalidKnownFolderPath { folder });
        }

        let mut length = 0usize;
        while length < MAX_KNOWN_FOLDER_PATH_UNITS && unsafe { *path.0.add(length) } != 0 {
            length += 1;
        }
        if length == 0 || length == MAX_KNOWN_FOLDER_PATH_UNITS {
            return Err(DesktopInventoryError::InvalidKnownFolderPath { folder });
        }

        let wide = unsafe { std::slice::from_raw_parts(path.0, length) };
        Ok(PathBuf::from(OsString::from_wide(wide)))
    }

    fn read_one_level(
        root: &Path,
        folder: DesktopFolder,
        candidates: &mut Vec<DesktopInventoryCandidate>,
    ) -> Result<(), DesktopInventoryError> {
        let entries = match std::fs::read_dir(root) {
            Ok(entries) => entries,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
            Err(error) => {
                return Err(DesktopInventoryError::ReadDirectory {
                    folder,
                    code: io_code(&error),
                });
            }
        };

        for entry in entries {
            let entry = entry.map_err(|error| DesktopInventoryError::ReadDirectoryEntry {
                folder,
                code: io_code(&error),
            })?;
            let file_type =
                entry
                    .file_type()
                    .map_err(|error| DesktopInventoryError::ReadDirectoryEntry {
                        folder,
                        code: io_code(&error),
                    })?;

            // Never recurse into a child; symlinks are included as leaf entries.
            if !(file_type.is_file() || file_type.is_dir() || file_type.is_symlink()) {
                continue;
            }

            let path = entry.path();
            let label = entry.file_name().to_string_lossy().into_owned();
            let kind = classify_desktop_entry(&path, file_type.is_dir());
            if candidates.len() >= MAX_DESKTOP_INVENTORY_ITEMS {
                return Err(DesktopInventoryError::TooManyItems);
            }
            candidates.push(DesktopInventoryCandidate {
                file_identity: read_file_identity(&path),
                path,
                label,
                kind,
                source_folder: folder,
            });
        }
        Ok(())
    }

    fn read_file_identity(path: &Path) -> Option<DesktopFileIdentity> {
        let wide_path = extended_length_path(path.as_os_str().encode_wide().collect());
        let raw_handle = unsafe {
            CreateFileW(
                wide_path.as_ptr(),
                FILE_READ_ATTRIBUTES,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                ptr::null_mut(),
                OPEN_EXISTING,
                FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
                ptr::null_mut(),
            )
        };
        if raw_handle.is_null() || raw_handle as isize == INVALID_HANDLE_VALUE {
            return None;
        }

        let handle = OwnedFileHandle(raw_handle);
        let mut information = ByHandleFileInformation::default();
        if unsafe { GetFileInformationByHandle(handle.0, &mut information) } == 0 {
            return None;
        }

        Some(DesktopFileIdentity {
            volume_serial_number: information.volume_serial_number,
            file_index: (u64::from(information.file_index_high) << 32)
                | u64::from(information.file_index_low),
            creation_time_100ns: (u64::from(information.creation_time.high_date_time) << 32)
                | u64::from(information.creation_time.low_date_time),
            number_of_links: information.number_of_links,
        })
    }

    fn path_fallback_desktop_id(path: &Path) -> String {
        let mut hasher = Sha256::new();
        for unit in path.as_os_str().encode_wide() {
            hasher.update(unit.to_le_bytes());
        }
        let digest = hasher.finalize();
        let mut id = String::with_capacity(13 + digest.len() * 2);
        id.push_str("desktop:path:");
        for byte in digest {
            use std::fmt::Write as _;
            let _ = write!(id, "{byte:02x}");
        }
        id
    }

    fn io_code(error: &std::io::Error) -> u32 {
        error
            .raw_os_error()
            .map(|code| code as u32)
            .unwrap_or(u32::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_classifies_folders_and_common_launchers_without_resolving_shortcuts() {
        assert_eq!(
            classify_desktop_entry(Path::new("Documents"), true),
            DesktopItemKind::Folder
        );
        assert_eq!(
            classify_desktop_entry(Path::new("Browser.LNK"), false),
            DesktopItemKind::Application
        );
        assert_eq!(
            classify_desktop_entry(Path::new("game.exe"), false),
            DesktopItemKind::Application
        );
        assert_eq!(
            classify_desktop_entry(Path::new("site.url"), false),
            DesktopItemKind::Application
        );
        assert_eq!(
            classify_desktop_entry(Path::new("notes.txt"), false),
            DesktopItemKind::File
        );
    }

    #[test]
    fn filesystem_identity_keys_are_volume_scoped_and_path_independent() {
        let identity = DesktopFileIdentity {
            volume_serial_number: 0x1234_abcd,
            file_index: 0x0123_4567_89ab_cdef,
            creation_time_100ns: 0x0fed_cba9_8765_4321,
            number_of_links: 1,
        };
        let id = stable_desktop_id(identity);
        assert_eq!(id, stable_desktop_id(identity));
        assert!(id.starts_with("desktop:file:"));
        assert!(id.len() <= 127);
        assert_ne!(
            id,
            stable_desktop_id(DesktopFileIdentity {
                volume_serial_number: identity.volume_serial_number + 1,
                ..identity
            })
        );
        assert_ne!(
            id,
            stable_desktop_id(DesktopFileIdentity {
                file_index: identity.file_index + 1,
                ..identity
            })
        );
        assert_ne!(
            id,
            stable_desktop_id(DesktopFileIdentity {
                creation_time_100ns: identity.creation_time_100ns + 1,
                ..identity
            })
        );
    }

    #[test]
    fn missing_or_ambiguous_file_identity_uses_a_path_fallback() {
        let (missing, missing_stability) =
            resolve_desktop_item_identity(None, 0, "desktop:path:missing".into());
        assert_eq!(missing, "desktop:path:missing");
        assert_eq!(missing_stability, DesktopItemIdentityStability::PathFallback);

        let identity = DesktopFileIdentity {
            volume_serial_number: 7,
            file_index: 11,
            creation_time_100ns: 13,
            number_of_links: 2,
        };
        let (ambiguous, ambiguous_stability) =
            resolve_desktop_item_identity(Some(identity), 1, "desktop:path:alias".into());
        assert_eq!(ambiguous, "desktop:path:alias");
        assert_eq!(ambiguous_stability, DesktopItemIdentityStability::PathFallback);

        let (duplicate, duplicate_stability) =
            resolve_desktop_item_identity(Some(identity), 2, "desktop:path:duplicate".into());
        assert_eq!(duplicate, "desktop:path:duplicate");
        assert_eq!(duplicate_stability, DesktopItemIdentityStability::PathFallback);
    }

    #[cfg(not(windows))]
    #[test]
    fn non_windows_inventory_and_watcher_are_explicitly_unsupported() {
        assert!(matches!(
            enumerate_desktop_items(),
            Err(DesktopInventoryError::UnsupportedPlatform)
        ));
        assert!(matches!(
            watch_desktop_folders(),
            Err(DesktopInventoryError::UnsupportedPlatform)
        ));
    }
}
