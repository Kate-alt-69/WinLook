//! Bounded, read-only enumeration of the Windows Shell Desktop namespace.
//!
//! This is distinct from the filesystem-only User/Public Desktop scan. Returned
//! PIDL-derived IDs are session-scoped diagnostics and MUST NOT be persisted
//! as Group/layout identities. The API never changes Shell state.

use std::fmt;
use std::path::PathBuf;

pub const MAX_DESKTOP_NAMESPACE_ITEMS: usize = 4096;
#[cfg(any(windows, test))]
const MAX_PIDL_BYTES: usize = 64 * 1024;
#[cfg(windows)]
const MAX_SHELL_NAME_UNITS: usize = 32_768;
#[cfg(windows)]
const MAX_DESKTOP_NAMESPACE_TEXT_BYTES: usize = 16 * 1024 * 1024;
#[cfg(any(windows, test))]
const MAX_DESKTOP_NAMESPACE_LABEL_BYTES: usize = 255;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DesktopNamespaceItemKind {
    Application,
    File,
    Folder,
    VirtualItem,
    VirtualFolder,
}

/// A single item returned by the Shell's Desktop namespace enumerator.
///
/// The ID is a SHA-256 digest of the current Shell PIDL bytes. It is deliberately
/// session-scoped: Shell PIDL encodings are implementation details and are not
/// stable filesystem or profile identifiers. The optional filesystem path is
/// ephemeral and must never be written into WLDE's persistent state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DesktopNamespaceItem {
    id: String,
    label: String,
    kind: DesktopNamespaceItemKind,
    filesystem_backed: bool,
    filesystem_path: Option<PathBuf>,
}

impl DesktopNamespaceItem {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn kind(&self) -> DesktopNamespaceItemKind {
        self.kind
    }

    pub fn is_filesystem_backed(&self) -> bool {
        self.filesystem_backed
    }

    pub fn filesystem_path(&self) -> Option<&std::path::Path> {
        self.filesystem_path.as_deref()
    }
}

/// One bounded, read-only snapshot of the root Desktop Shell namespace.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DesktopNamespaceSnapshot {
    items: Vec<DesktopNamespaceItem>,
    skipped_items: usize,
}

impl DesktopNamespaceSnapshot {
    pub fn items(&self) -> &[DesktopNamespaceItem] {
        &self.items
    }

    pub fn skipped_items(&self) -> usize {
        self.skipped_items
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DesktopNamespaceError {
    UnsupportedPlatform,
    ComInitialization(i32),
    DesktopFolder(i32),
    Enumeration(i32),
    TooManyItems,
    TooMuchText,
    AllocationFailed,
}

impl fmt::Display for DesktopNamespaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => {
                f.write_str("Shell Desktop namespace enumeration is available only on Windows")
            }
            Self::ComInitialization(code) => {
                write!(f, "COM STA initialization failed with HRESULT {code:#x}")
            }
            Self::DesktopFolder(code) => {
                write!(f, "SHGetDesktopFolder failed with HRESULT {code:#x}")
            }
            Self::Enumeration(code) => {
                write!(f, "Shell Desktop namespace enumeration failed with HRESULT {code:#x}")
            }
            Self::TooManyItems => write!(
                f,
                "Shell Desktop namespace exceeds the {MAX_DESKTOP_NAMESPACE_ITEMS}-item bound"
            ),
            Self::TooMuchText => {
                f.write_str("Shell Desktop namespace text exceeds the bounded snapshot size")
            }
            Self::AllocationFailed => {
                f.write_str("not enough memory for a bounded Shell Desktop namespace snapshot")
            }
        }
    }
}

impl std::error::Error for DesktopNamespaceError {}

/// Enumerates the root Desktop Shell namespace without modifying it.
///
/// The call initializes COM as an STA on the calling thread and balances that
/// initialization before returning. Call it from a dedicated BGW worker: Shell
/// namespace providers may be slow or unresponsive. The enumeration uses a
/// null owner window so providers must not display prompts.
#[cfg(windows)]
pub fn enumerate_desktop_namespace_items(
) -> Result<DesktopNamespaceSnapshot, DesktopNamespaceError> {
    desktop_namespace_platform::enumerate()
}

/// Non-Windows builds expose an explicit unsupported-platform result.
#[cfg(not(windows))]
pub fn enumerate_desktop_namespace_items(
) -> Result<DesktopNamespaceSnapshot, DesktopNamespaceError> {
    Err(DesktopNamespaceError::UnsupportedPlatform)
}

#[cfg(any(windows, test))]
fn classify_namespace_item(
    filesystem_backed: bool,
    is_folder: bool,
    parsing_name_or_label: &str,
) -> DesktopNamespaceItemKind {
    if !filesystem_backed {
        return if is_folder {
            DesktopNamespaceItemKind::VirtualFolder
        } else {
            DesktopNamespaceItemKind::VirtualItem
        };
    }
    if is_folder {
        return DesktopNamespaceItemKind::Folder;
    }

    let file_name = parsing_name_or_label
        .rsplit(|character| character == '\\' || character == '/')
        .next()
        .unwrap_or(parsing_name_or_label);
    let extension = file_name.rsplit_once('.').map(|(_, ext)| ext).unwrap_or("");
    if [
        "lnk",
        "url",
        "appref-ms",
        "exe",
        "com",
        "bat",
        "cmd",
    ]
    .iter()
    .any(|candidate| extension.eq_ignore_ascii_case(candidate))
    {
        DesktopNamespaceItemKind::Application
    } else {
        DesktopNamespaceItemKind::File
    }
}

#[cfg(any(windows, test))]
fn truncate_label(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_owned();
    }
    let mut boundary = max_bytes;
    while !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    value[..boundary].to_owned()
}

/// Validates one single-level child PIDL stored in a byte buffer.
///
/// IShellFolder::EnumObjects returns one SHITEMID followed by a two-byte
/// terminator. A multi-level or unterminated PIDL is rejected.
#[cfg(any(windows, test))]
fn bounded_child_pidl_length(bytes: &[u8]) -> Option<usize> {
    if bytes.len() < 4 {
        return None;
    }
    let child_bytes = usize::from(u16::from_le_bytes([bytes[0], bytes[1]]));
    if child_bytes < 2 {
        return None;
    }
    let total = child_bytes.checked_add(2)?;
    if total > MAX_PIDL_BYTES || bytes.len() < total {
        return None;
    }
    if bytes[child_bytes] != 0 || bytes[child_bytes + 1] != 0 {
        return None;
    }
    Some(total)
}

#[cfg(windows)]
mod desktop_namespace_platform {
    use super::*;
    use sha2::{Digest, Sha256};
    use std::ffi::{c_void, OsString};
    use std::os::windows::ffi::OsStringExt;
    use std::ptr::{self, NonNull};

    type HResult = i32;
    type Handle = *mut c_void;

    const COINIT_APARTMENTTHREADED: u32 = 0x2;
    const SHCONTF_FOLDERS: u32 = 0x20;
    const SHCONTF_NONFOLDERS: u32 = 0x40;
    const SFGAO_FOLDER: u32 = 0x2000_0000;
    const SFGAO_FILESYSTEM: u32 = 0x4000_0000;
    const SHGDN_NORMAL: u32 = 0;
    const SHGDN_FORPARSING: u32 = 0x8000;

    #[repr(C)]
    struct ItemIdList {
        _unused: [u8; 0],
    }

    #[repr(C)]
    struct ShellFolder {
        vtable: *const ShellFolderVTable,
    }

    #[repr(C)]
    struct EnumIdList {
        vtable: *const EnumIdListVTable,
    }

    #[repr(C)]
    struct ShellFolderVTable {
        query_interface: usize,
        add_ref: usize,
        release: unsafe extern "system" fn(*mut ShellFolder) -> u32,
        parse_display_name: usize,
        enum_objects: unsafe extern "system" fn(
            *mut ShellFolder,
            Handle,
            u32,
            *mut *mut EnumIdList,
        ) -> HResult,
        bind_to_object: usize,
        bind_to_storage: usize,
        compare_ids: usize,
        create_view_object: usize,
        get_attributes_of: unsafe extern "system" fn(
            *mut ShellFolder,
            u32,
            *const *const ItemIdList,
            *mut u32,
        ) -> HResult,
        get_ui_object_of: usize,
        get_display_name_of: unsafe extern "system" fn(
            *mut ShellFolder,
            *const ItemIdList,
            u32,
            *mut StrRet,
        ) -> HResult,
        set_name_of: usize,
    }

    #[repr(C)]
    struct EnumIdListVTable {
        query_interface: usize,
        add_ref: usize,
        release: unsafe extern "system" fn(*mut EnumIdList) -> u32,
        next: unsafe extern "system" fn(
            *mut EnumIdList,
            u32,
            *mut *mut ItemIdList,
            *mut u32,
        ) -> HResult,
        skip: usize,
        reset: usize,
        clone: usize,
    }

    #[repr(C)]
    union StrRetValue {
        ole_str: *mut u16,
        offset: u32,
        c_str: [u8; 260],
    }

    #[repr(C)]
    struct StrRet {
        kind: u32,
        value: StrRetValue,
    }

    #[link(name = "shell32")]
    extern "system" {
        fn SHGetDesktopFolder(folder: *mut *mut ShellFolder) -> HResult;
    }

    #[link(name = "ole32")]
    extern "system" {
        fn CoInitializeEx(reserved: *mut c_void, concurrency_model: u32) -> HResult;
        fn CoUninitialize();
        fn CoTaskMemFree(memory: *mut c_void);
    }

    #[link(name = "shlwapi")]
    extern "system" {
        fn StrRetToBufW(
            source: *mut StrRet,
            pidl: *const ItemIdList,
            destination: *mut u16,
            destination_chars: u32,
        ) -> HResult;
    }

    struct ComApartment;

    impl ComApartment {
        fn initialize() -> Result<Self, DesktopNamespaceError> {
            let result = unsafe { CoInitializeEx(ptr::null_mut(), COINIT_APARTMENTTHREADED) };
            if result < 0 {
                Err(DesktopNamespaceError::ComInitialization(result))
            } else {
                // S_OK and S_FALSE both require one matching CoUninitialize.
                Ok(Self)
            }
        }
    }

    impl Drop for ComApartment {
        fn drop(&mut self) {
            unsafe {
                CoUninitialize();
            }
        }
    }

    struct OwnedShellFolder(NonNull<ShellFolder>);

    impl Drop for OwnedShellFolder {
        fn drop(&mut self) {
            unsafe {
                let vtable = self.0.as_ref().vtable;
                ((*vtable).release)(self.0.as_ptr());
            }
        }
    }

    struct OwnedEnumIdList(NonNull<EnumIdList>);

    impl Drop for OwnedEnumIdList {
        fn drop(&mut self) {
            unsafe {
                let vtable = self.0.as_ref().vtable;
                ((*vtable).release)(self.0.as_ptr());
            }
        }
    }

    struct OwnedPidl(NonNull<ItemIdList>);

    impl Drop for OwnedPidl {
        fn drop(&mut self) {
            unsafe {
                CoTaskMemFree(self.0.as_ptr().cast());
            }
        }
    }

    fn shell_pidl_id(bytes: &[u8]) -> String {
        use std::fmt::Write as _;

        let digest = Sha256::digest(bytes);
        let mut id = String::with_capacity(18 + digest.len() * 2);
        id.push_str("desktop:shell-pidl:");
        for byte in digest {
            let _ = write!(id, "{byte:02x}");
        }
        id
    }

    unsafe fn copy_child_pidl(
        pidl: *const ItemIdList,
    ) -> Result<Option<Vec<u8>>, DesktopNamespaceError> {
        if pidl.is_null() {
            return Ok(None);
        }

        let bytes = pidl.cast::<u8>();
        let child_bytes = usize::from(unsafe {
            ptr::read_unaligned(bytes.cast::<u16>())
        });
        if child_bytes < 2 {
            return Ok(None);
        }
        let total = child_bytes
            .checked_add(2)
            .ok_or(DesktopNamespaceError::TooManyItems)?;
        if total > MAX_PIDL_BYTES {
            return Ok(None);
        }
        let mut copy = Vec::new();
        copy.try_reserve_exact(total)
            .map_err(|_| DesktopNamespaceError::AllocationFailed)?;
        for offset in 0..total {
            copy.push(unsafe { ptr::read(bytes.add(offset)) });
        }
        if bounded_child_pidl_length(&copy) != Some(total) {
            return Ok(None);
        }
        Ok(Some(copy))
    }

    fn read_shell_name(
        folder: &OwnedShellFolder,
        pidl: *const ItemIdList,
        flags: u32,
        buffer: &mut [u16],
    ) -> Option<usize> {
        let mut result = StrRet {
            kind: 0,
            value: StrRetValue { offset: 0 },
        };
        let vtable = unsafe { folder.0.as_ref().vtable };
        let get_name_result = unsafe {
            ((*vtable).get_display_name_of)(
                folder.0.as_ptr(),
                pidl,
                flags,
                &mut result,
            )
        };
        if get_name_result < 0 {
            return None;
        }

        // StrRetToBufW handles all STRRET variants and frees STRRET_WSTR
        // storage as documented by the Windows Shell API.
        let result = unsafe {
            StrRetToBufW(
                &mut result,
                pidl,
                buffer.as_mut_ptr(),
                buffer.len().min(u32::MAX as usize) as u32,
            )
        };
        if result < 0 {
            return None;
        }
        buffer.iter().position(|character| *character == 0)
    }

    fn get_attributes(
        folder: &OwnedShellFolder,
        pidl: *const ItemIdList,
    ) -> Option<u32> {
        let child_items = [pidl];
        let mut attributes = SFGAO_FOLDER | SFGAO_FILESYSTEM;
        let vtable = unsafe { folder.0.as_ref().vtable };
        let result = unsafe {
            ((*vtable).get_attributes_of)(
                folder.0.as_ptr(),
                1,
                child_items.as_ptr(),
                &mut attributes,
            )
        };
        (result >= 0).then_some(attributes)
    }

    pub(super) fn enumerate(
    ) -> Result<DesktopNamespaceSnapshot, DesktopNamespaceError> {
        let _apartment = ComApartment::initialize()?;

        let mut raw_folder = ptr::null_mut();
        let result = unsafe { SHGetDesktopFolder(&mut raw_folder) };
        if result < 0 {
            return Err(DesktopNamespaceError::DesktopFolder(result));
        }
        let folder = OwnedShellFolder(
            NonNull::new(raw_folder).ok_or(DesktopNamespaceError::DesktopFolder(result))?,
        );

        // A null owner prevents namespace providers from displaying prompts.
        let mut raw_enumerator = ptr::null_mut();
        let vtable = unsafe { folder.0.as_ref().vtable };
        let result = unsafe {
            ((*vtable).enum_objects)(
                folder.0.as_ptr(),
                ptr::null_mut(),
                SHCONTF_FOLDERS | SHCONTF_NONFOLDERS,
                &mut raw_enumerator,
            )
        };
        if result < 0 {
            return Err(DesktopNamespaceError::Enumeration(result));
        }
        let Some(raw_enumerator) = NonNull::new(raw_enumerator) else {
            return Ok(DesktopNamespaceSnapshot::default());
        };
        let enumerator = OwnedEnumIdList(raw_enumerator);

        let mut items = Vec::new();
        let mut skipped_items = 0usize;
        let mut enumerated_items = 0usize;
        let mut stored_text_bytes = 0usize;
        let mut name_buffer = vec![0u16; MAX_SHELL_NAME_UNITS];

        loop {
            let mut raw_pidl = ptr::null_mut();
            let mut fetched = 0u32;
            let enumerator_vtable = unsafe { enumerator.0.as_ref().vtable };
            let result = unsafe {
                ((*enumerator_vtable).next)(
                    enumerator.0.as_ptr(),
                    1,
                    &mut raw_pidl,
                    &mut fetched,
                )
            };
            if result < 0 {
                return Err(DesktopNamespaceError::Enumeration(result));
            }
            if fetched == 0 {
                break;
            }
            enumerated_items += 1;
            if enumerated_items > MAX_DESKTOP_NAMESPACE_ITEMS {
                return Err(DesktopNamespaceError::TooManyItems);
            }

            let Some(raw_pidl) = NonNull::new(raw_pidl) else {
                skipped_items = skipped_items.saturating_add(1);
                continue;
            };
            let pidl = OwnedPidl(raw_pidl);
            let Some(pidl_bytes) = (unsafe { copy_child_pidl(pidl.0.as_ptr()) })? else {
                skipped_items = skipped_items.saturating_add(1);
                continue;
            };
            let Some(attributes) = get_attributes(&folder, pidl.0.as_ptr()) else {
                skipped_items = skipped_items.saturating_add(1);
                continue;
            };

            let Some(label_units) = read_shell_name(
                &folder,
                pidl.0.as_ptr(),
                SHGDN_NORMAL,
                &mut name_buffer,
            ) else {
                skipped_items = skipped_items.saturating_add(1);
                continue;
            };
            let label_value = String::from_utf16_lossy(&name_buffer[..label_units]);
            let label = truncate_label(&label_value, MAX_DESKTOP_NAMESPACE_LABEL_BYTES);
            if label.trim().is_empty() {
                skipped_items = skipped_items.saturating_add(1);
                continue;
            }

            let filesystem_backed = attributes & SFGAO_FILESYSTEM != 0;
            let is_folder = attributes & SFGAO_FOLDER != 0;
            let filesystem_path = if filesystem_backed {
                read_shell_name(
                    &folder,
                    pidl.0.as_ptr(),
                    SHGDN_FORPARSING,
                    &mut name_buffer,
                )
                .map(|path_units| PathBuf::from(OsString::from_wide(&name_buffer[..path_units])))
                .filter(|path| path.is_absolute())
            } else {
                None
            };

            let classification_name = filesystem_path
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_else(|| label.clone());
            let kind = classify_namespace_item(
                filesystem_backed,
                is_folder,
                &classification_name,
            );

            let path_bytes = filesystem_path
                .as_ref()
                .map(|path| path.as_os_str().encode_wide().count().saturating_mul(2))
                .unwrap_or(0);
            stored_text_bytes = stored_text_bytes
                .checked_add(label.len())
                .and_then(|value| value.checked_add(path_bytes))
                .ok_or(DesktopNamespaceError::TooMuchText)?;
            if stored_text_bytes > MAX_DESKTOP_NAMESPACE_TEXT_BYTES {
                return Err(DesktopNamespaceError::TooMuchText);
            }

            items.push(DesktopNamespaceItem {
                id: shell_pidl_id(&pidl_bytes),
                label,
                kind,
                filesystem_backed,
                filesystem_path,
            });
        }

        Ok(DesktopNamespaceSnapshot {
            items,
            skipped_items,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_one_bounded_child_pidl() {
        let pidl = [4, 0, 0xaa, 0xbb, 0, 0];
        assert_eq!(bounded_child_pidl_length(&pidl), Some(6));
    }

    #[test]
    fn rejects_unterminated_or_multilevel_pidls() {
        assert_eq!(bounded_child_pidl_length(&[4, 0, 0xaa, 0xbb]), None);
        assert_eq!(bounded_child_pidl_length(&[4, 0, 0xaa, 0xbb, 2, 0]), None);
    }

    #[test]
    fn rejects_oversized_child_pidls() {
        let mut pidl = vec![0u8; MAX_PIDL_BYTES + 1];
        pidl[0..2].copy_from_slice(&((MAX_PIDL_BYTES - 1) as u16).to_le_bytes());
        assert_eq!(bounded_child_pidl_length(&pidl), None);
    }

    #[test]
    fn classifies_filesystem_and_virtual_items_separately() {
        assert_eq!(
            classify_namespace_item(true, true, "C:\\Desktop\\Folder"),
            DesktopNamespaceItemKind::Folder
        );
        assert_eq!(
            classify_namespace_item(true, false, "C:\\Desktop\\Browser.LNK"),
            DesktopNamespaceItemKind::Application
        );
        assert_eq!(
            classify_namespace_item(true, false, "C:\\Desktop\\Readme.txt"),
            DesktopNamespaceItemKind::File
        );
        assert_eq!(
            classify_namespace_item(false, true, "Control Panel"),
            DesktopNamespaceItemKind::VirtualFolder
        );
        assert_eq!(
            classify_namespace_item(false, false, "Virtual item"),
            DesktopNamespaceItemKind::VirtualItem
        );
    }

    #[test]
    fn label_truncation_preserves_utf8_boundaries() {
        let value = "a".repeat(254) + "🪟";
        let truncated = truncate_label(&value, MAX_DESKTOP_NAMESPACE_LABEL_BYTES);
        assert_eq!(truncated.len(), 254);
        assert!(truncated.is_char_boundary(truncated.len()));
    }
}
