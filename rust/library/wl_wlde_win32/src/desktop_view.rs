//! Read-only acquisition and ownership validation for Explorer's live Desktop view.
//!
//! The adapter uses the registered SWC_DESKTOP Shell window, resolves its active
//! IShellView/IFolderView and verifies both the view HWND owner and Desktop-root
//! PIDL. It never returns COM pointers, HWNDs or PIDLs to the caller.

use std::fmt;

use crate::DesktopHostIdentity;

/// Which documented/sample-compatible pvarLoc representation found the Desktop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DesktopViewLocation {
    /// Raymond Chen's Shell sample passes CSIDL_DESKTOP (VT_I4) with SWC_DESKTOP.
    CsidlDesktop,
    /// An independent read-only experiment passes VT_EMPTY with SWC_DESKTOP.
    Empty,
}

impl DesktopViewLocation {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CsidlDesktop => "CSIDL_DESKTOP",
            Self::Empty => "VT_EMPTY",
        }
    }
}

/// Successful, path-free evidence that BGW acquired the current Explorer Desktop view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DesktopViewProbe {
    owner_process_id: u32,
    location: DesktopViewLocation,
}

impl DesktopViewProbe {
    pub const fn owner_process_id(&self) -> u32 {
        self.owner_process_id
    }

    pub const fn location(&self) -> DesktopViewLocation {
        self.location
    }
}

/// Errors returned when the live Explorer Desktop view cannot be proven safe to use.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DesktopViewError {
    UnsupportedPlatform,
    ComInitialization(i32),
    ShellWindowsActivation(i32),
    DesktopWindowNotFound {
        location: DesktopViewLocation,
        hresult: i32,
    },
    MissingDispatch(DesktopViewLocation),
    UnexpectedNull(&'static str),
    AcquisitionAttemptsFailed {
        csidl_error: String,
        empty_error: String,
    },
    QueryServiceProvider(i32),
    QueryService(i32),
    QueryActiveShellView(i32),
    QueryFolderView(i32),
    GetViewWindow(i32),
    InvalidViewWindow,
    ViewOwnerUnavailable,
    ViewOwnerMismatch {
        expected_process_id: u32,
        actual_process_id: u32,
    },
    GetViewFolder(i32),
    QueryPersistFolder(i32),
    GetCurrentFolder(i32),
    GetDesktopFolder(i32),
    DesktopRootMismatch,
    HostIdentityChanged,
    HostIdentityUnavailable(String),
}

impl fmt::Display for DesktopViewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => {
                f.write_str("Explorer Desktop view acquisition is available only on Windows")
            }
            Self::ComInitialization(code) => {
                write!(f, "COM STA initialization failed with HRESULT {code:#x}")
            }
            Self::ShellWindowsActivation(code) => {
                write!(f, "CoCreateInstance(CLSID_ShellWindows) failed with HRESULT {code:#x}")
            }
            Self::DesktopWindowNotFound { location, hresult } => write!(
                f,
                "IShellWindows::FindWindowSW did not find SWC_DESKTOP using {} (HRESULT {hresult:#x})",
                location.as_str()
            ),
            Self::MissingDispatch(location) => write!(
                f,
                "FindWindowSW returned no IDispatch for {}",
                location.as_str()
            ),
            Self::UnexpectedNull(operation) => {
                write!(f, "{operation} returned S_OK with a null output pointer")
            }
            Self::AcquisitionAttemptsFailed {
                csidl_error,
                empty_error,
            } => write!(
                f,
                "Desktop view acquisition failed for both pvarLoc forms (CSIDL_DESKTOP: {csidl_error}; VT_EMPTY: {empty_error})"
            ),
            Self::QueryServiceProvider(code) => {
                write!(f, "QueryInterface(IID_IServiceProvider) failed with HRESULT {code:#x}")
            }
            Self::QueryService(code) => {
                write!(f, "QueryService(SID_STopLevelBrowser) failed with HRESULT {code:#x}")
            }
            Self::QueryActiveShellView(code) => {
                write!(f, "QueryActiveShellView failed with HRESULT {code:#x}")
            }
            Self::QueryFolderView(code) => {
                write!(f, "QueryInterface(IID_IFolderView) failed with HRESULT {code:#x}")
            }
            Self::GetViewWindow(code) => {
                write!(f, "IShellView::GetWindow failed with HRESULT {code:#x}")
            }
            Self::InvalidViewWindow => {
                f.write_str("IShellView returned a null HWND")
            }
            Self::ViewOwnerUnavailable => {
                f.write_str("GetWindowThreadProcessId could not identify the view window owner")
            }
            Self::ViewOwnerMismatch {
                expected_process_id,
                actual_process_id,
            } => write!(
                f,
                "Desktop view HWND belongs to PID {actual_process_id}, not the monitored Explorer PID {expected_process_id}"
            ),
            Self::GetViewFolder(code) => {
                write!(f, "IFolderView::GetFolder failed with HRESULT {code:#x}")
            }
            Self::QueryPersistFolder(code) => {
                write!(f, "QueryInterface(IID_IPersistFolder2) failed with HRESULT {code:#x}")
            }
            Self::GetCurrentFolder(code) => {
                write!(f, "IPersistFolder2::GetCurFolder failed with HRESULT {code:#x}")
            }
            Self::GetDesktopFolder(code) => {
                write!(f, "SHGetFolderLocation(CSIDL_DESKTOP) failed with HRESULT {code:#x}")
            }
            Self::DesktopRootMismatch => {
                f.write_str("the active view folder PIDL does not match the virtual Desktop namespace root")
            }
            Self::HostIdentityChanged => {
                f.write_str("the Explorer desktop process changed during Desktop view acquisition")
            }
            Self::HostIdentityUnavailable(message) => {
                write!(f, "the Explorer desktop process could not be revalidated: {message}")
            }
        }
    }
}

impl std::error::Error for DesktopViewError {}

/// Acquires and validates the actual Explorer Desktop IFolderView without mutating it.
///
/// Call from BGW's dedicated message-aware STA worker. All COM interfaces and
/// temporary PIDLs are released on the calling thread. A successful result
/// contains only the owner PID and the pvarLoc representation that succeeded.
#[cfg(windows)]
pub fn probe_desktop_folder_view(
    expected_host: &DesktopHostIdentity,
) -> Result<DesktopViewProbe, DesktopViewError> {
    desktop_view_platform::probe(expected_host)
}

/// Non-Windows builds never guess or emulate an Explorer Desktop view.
#[cfg(not(windows))]
pub fn probe_desktop_folder_view(
    _expected_host: &DesktopHostIdentity,
) -> Result<DesktopViewProbe, DesktopViewError> {
    Err(DesktopViewError::UnsupportedPlatform)
}

#[cfg(any(windows, test))]
mod com_abi {
    use super::*;
    const VT_EMPTY: u16 = 0;
    const VT_I4: u16 = 3;
    const CSIDL_DESKTOP: i32 = 0;
    const SWC_DESKTOP: i32 = 0x8;
    const SWFO_NEEDDISPATCH: i32 = 0x1;
    // IDispatch occupies slots 0-6; Count/Item/_NewEnum/Register/
    // RegisterPending/Revoke/OnNavigate/OnActivated precede FindWindowSW.
    const ISHELL_WINDOWS_FIND_WINDOW_SW_SLOT: usize = 15;
    // IUnknown + IOleWindow + the first ten IShellBrowser methods.
    const ISHELL_BROWSER_QUERY_ACTIVE_SHELL_VIEW_SLOT: usize = 15;
    
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Guid {
        data1: u32,
        data2: u16,
        data3: u16,
        data4: [u8; 8],
    }
    
    const CLSID_SHELL_WINDOWS: Guid = Guid {
        data1: 0x9ba0_5972,
        data2: 0xf6a8,
        data3: 0x11cf,
        data4: [0xa4, 0x42, 0x00, 0xa0, 0xc9, 0x0a, 0x8f, 0x39],
    };
    const IID_ISHELL_WINDOWS: Guid = Guid {
        data1: 0x85cb_6900,
        data2: 0x4d95,
        data3: 0x11cf,
        data4: [0x96, 0x0c, 0x00, 0x80, 0xc7, 0xf4, 0xee, 0x85],
    };
    const IID_ISERVICE_PROVIDER: Guid = Guid {
        data1: 0x6d51_40c1,
        data2: 0x7436,
        data3: 0x11ce,
        data4: [0x80, 0x34, 0x00, 0xaa, 0x00, 0x60, 0x09, 0xfa],
    };
    const SID_STOP_LEVEL_BROWSER: Guid = Guid {
        data1: 0x4c96_be40,
        data2: 0x915c,
        data3: 0x11cf,
        data4: [0x99, 0xd3, 0x00, 0xaa, 0x00, 0x4a, 0xe8, 0x37],
    };
    const IID_ISHELL_BROWSER: Guid = Guid {
        data1: 0x0002_14e2,
        data2: 0x0000,
        data3: 0x0000,
        data4: [0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
    };
    const IID_IFOLDER_VIEW: Guid = Guid {
        data1: 0xcde7_25b0,
        data2: 0xccc9,
        data3: 0x4519,
        data4: [0x91, 0x7e, 0x32, 0x5d, 0x72, 0xfa, 0xb4, 0xce],
    };
    const IID_ISHELL_FOLDER: Guid = Guid {
        data1: 0x0002_14e6,
        data2: 0x0000,
        data3: 0x0000,
        data4: [0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
    };
    const IID_IPERSIST_FOLDER2: Guid = Guid {
        data1: 0x1ac3_d9f0,
        data2: 0x175c,
        data3: 0x11d1,
        data4: [0x95, 0xbe, 0x00, 0x60, 0x97, 0x97, 0xea, 0x4f],
    };
    
    #[repr(C)]
    union VariantValue {
        long_value: i32,
        // VARIANT's BRECORD arm is two pointers: 16 bytes on x64, 8 on x86.
        // This gives VARIANT its ABI-correct size of 24 bytes on x64 and 16 on x86.
        zero: [usize; 2],
    }
    
    #[repr(C)]
    struct Variant {
        vt: u16,
        reserved1: u16,
        reserved2: u16,
        reserved3: u16,
        value: VariantValue,
    }
    
    impl Variant {
        fn empty() -> Self {
            Self {
                vt: VT_EMPTY,
                reserved1: 0,
                reserved2: 0,
                reserved3: 0,
                value: VariantValue { zero: [0; 2] },
            }
        }
    
        fn desktop_csidl() -> Self {
            Self {
                vt: VT_I4,
                reserved1: 0,
                reserved2: 0,
                reserved3: 0,
                value: VariantValue {
                    long_value: CSIDL_DESKTOP,
                },
            }
        }
    }
    
    
}

#[cfg(any(windows, test))]
use com_abi::*;

#[cfg(windows)]
mod desktop_view_platform {
    use super::*;
    use std::ffi::c_void;
    use std::mem::transmute_copy;
    use std::ptr::{self, NonNull};

    use crate::is_same_desktop_host;

    type HResult = i32;
    type Handle = *mut c_void;

    const S_OK: HResult = 0;
    const CLSCTX_ALL: u32 = 0x17;
    const COINIT_APARTMENTTHREADED: u32 = 0x2;
    #[repr(C)]
    struct ComObject {
        vtable: *const *const c_void,
    }

    type QueryInterfaceFn = unsafe extern "system" fn(Handle, *const Guid, *mut Handle) -> HResult;
    type ReleaseFn = unsafe extern "system" fn(Handle) -> u32;

    unsafe fn vtable_method<T: Copy>(object: Handle, index: usize) -> T {
        let vtable = unsafe { (*object.cast::<ComObject>()).vtable };
        let address = unsafe { *vtable.add(index) };
        unsafe { transmute_copy(&address) }
    }

    struct OwnedComPtr(NonNull<c_void>);

    impl OwnedComPtr {
        unsafe fn from_raw(raw: Handle) -> Option<Self> {
            NonNull::new(raw).map(Self)
        }

        fn as_raw(&self) -> Handle {
            self.0.as_ptr()
        }
    }

    impl Drop for OwnedComPtr {
        fn drop(&mut self) {
            unsafe {
                let release = vtable_method::<ReleaseFn>(self.as_raw(), 2);
                release(self.as_raw());
            }
        }
    }

    fn query_interface(
        object: &OwnedComPtr,
        iid: &Guid,
        failure: fn(i32) -> DesktopViewError,
    ) -> Result<OwnedComPtr, DesktopViewError> {
        let query = unsafe { vtable_method::<QueryInterfaceFn>(object.as_raw(), 0) };
        let mut raw = ptr::null_mut();
        let result = unsafe { query(object.as_raw(), iid, &mut raw) };
        if result != S_OK {
            if let Some(unexpected) = unsafe { OwnedComPtr::from_raw(raw) } {
                drop(unexpected);
            }
            return Err(failure(result));
        }
        unsafe { OwnedComPtr::from_raw(raw) }
            .ok_or(DesktopViewError::UnexpectedNull("QueryInterface"))
    }

    struct ComApartment;

    impl ComApartment {
        fn initialize() -> Result<Self, DesktopViewError> {
            let result = unsafe { CoInitializeEx(ptr::null_mut(), COINIT_APARTMENTTHREADED) };
            if result < 0 {
                Err(DesktopViewError::ComInitialization(result))
            } else {
                // S_OK and S_FALSE both require a matching CoUninitialize.
                Ok(Self)
            }
        }
    }

    impl Drop for ComApartment {
        fn drop(&mut self) {
            unsafe { CoUninitialize() };
        }
    }

    #[link(name = "ole32")]
    extern "system" {
        fn CoInitializeEx(reserved: *mut c_void, concurrency_model: u32) -> HResult;
        fn CoUninitialize();
        fn CoCreateInstance(
            class_id: *const Guid,
            outer: Handle,
            context: u32,
            interface_id: *const Guid,
            object: *mut Handle,
        ) -> HResult;
        fn CoTaskMemFree(memory: Handle);
    }

    #[link(name = "shell32")]
    extern "system" {
        fn SHGetFolderLocation(
            window: Handle,
            csidl: i32,
            token: Handle,
            reserved: u32,
            pidl: *mut Handle,
        ) -> HResult;
        fn ILIsEqual(first: Handle, second: Handle) -> i32;
    }

    #[link(name = "user32")]
    extern "system" {
        fn GetWindowThreadProcessId(window: Handle, process_id: *mut u32) -> u32;
    }

    struct OwnedPidl(NonNull<c_void>);

    impl OwnedPidl {
        fn as_raw(&self) -> Handle {
            self.0.as_ptr()
        }
    }

    impl Drop for OwnedPidl {
        fn drop(&mut self) {
            unsafe { CoTaskMemFree(self.as_raw()) };
        }
    }

    fn create_shell_windows() -> Result<OwnedComPtr, DesktopViewError> {
        let mut raw = ptr::null_mut();
        let result = unsafe {
            CoCreateInstance(
                &CLSID_SHELL_WINDOWS,
                ptr::null_mut(),
                CLSCTX_ALL,
                &IID_ISHELL_WINDOWS,
                &mut raw,
            )
        };
        if result != S_OK {
            if let Some(unexpected) = unsafe { OwnedComPtr::from_raw(raw) } {
                drop(unexpected);
            }
            return Err(DesktopViewError::ShellWindowsActivation(result));
        }
        unsafe { OwnedComPtr::from_raw(raw) }.ok_or(DesktopViewError::UnexpectedNull(
            "CoCreateInstance(CLSID_ShellWindows)",
        ))
    }

    fn find_desktop_dispatch(
        shell_windows: &OwnedComPtr,
        location: DesktopViewLocation,
    ) -> Result<OwnedComPtr, DesktopViewError> {
        // IShellWindows inherits IDispatch. The Windows SDK ExDisp.Idl
        // places FindWindowSW at vtable slot 15.
        type FindWindowSwFn = unsafe extern "system" fn(
            Handle,
            *mut Variant,
            *mut Variant,
            i32,
            *mut i32,
            i32,
            *mut Handle,
        ) -> HResult;

        let find = unsafe {
            vtable_method::<FindWindowSwFn>(
                shell_windows.as_raw(),
                ISHELL_WINDOWS_FIND_WINDOW_SW_SLOT,
            )
        };
        let mut location_variant = match location {
            DesktopViewLocation::CsidlDesktop => Variant::desktop_csidl(),
            DesktopViewLocation::Empty => Variant::empty(),
        };
        let mut root_variant = Variant::empty();
        let mut ignored_legacy_long_hwnd = 0i32;
        let mut raw_dispatch = ptr::null_mut();
        let result = unsafe {
            find(
                shell_windows.as_raw(),
                &mut location_variant,
                &mut root_variant,
                SWC_DESKTOP,
                &mut ignored_legacy_long_hwnd,
                SWFO_NEEDDISPATCH,
                &mut raw_dispatch,
            )
        };
        if result != S_OK {
            if let Some(unexpected) = unsafe { OwnedComPtr::from_raw(raw_dispatch) } {
                drop(unexpected);
            }
            return Err(DesktopViewError::DesktopWindowNotFound {
                location,
                hresult: result,
            });
        }
        unsafe { OwnedComPtr::from_raw(raw_dispatch) }
            .ok_or(DesktopViewError::MissingDispatch(location))
    }

    fn acquire_and_validate(
        shell_windows: &OwnedComPtr,
        expected_host: &DesktopHostIdentity,
        location: DesktopViewLocation,
    ) -> Result<DesktopViewProbe, DesktopViewError> {
        let dispatch = find_desktop_dispatch(shell_windows, location)?;
        let service_provider = query_interface(
            &dispatch,
            &IID_ISERVICE_PROVIDER,
            DesktopViewError::QueryServiceProvider,
        )?;

        // IServiceProvider::QueryService is its first method after IUnknown.
        type QueryServiceFn =
            unsafe extern "system" fn(Handle, *const Guid, *const Guid, *mut Handle) -> HResult;
        let query_service =
            unsafe { vtable_method::<QueryServiceFn>(service_provider.as_raw(), 3) };
        let mut raw_browser = ptr::null_mut();
        let result = unsafe {
            query_service(
                service_provider.as_raw(),
                &SID_STOP_LEVEL_BROWSER,
                &IID_ISHELL_BROWSER,
                &mut raw_browser,
            )
        };
        if result != S_OK {
            if let Some(unexpected) = unsafe { OwnedComPtr::from_raw(raw_browser) } {
                drop(unexpected);
            }
            return Err(DesktopViewError::QueryService(result));
        }
        let browser = unsafe { OwnedComPtr::from_raw(raw_browser) }.ok_or(
            DesktopViewError::UnexpectedNull("QueryService(IShellBrowser)"),
        )?;

        // IShellBrowser inherits IOleWindow. QueryActiveShellView is slot 15.
        type QueryActiveShellViewFn = unsafe extern "system" fn(Handle, *mut Handle) -> HResult;
        let query_active = unsafe {
            vtable_method::<QueryActiveShellViewFn>(
                browser.as_raw(),
                ISHELL_BROWSER_QUERY_ACTIVE_SHELL_VIEW_SLOT,
            )
        };
        let mut raw_view = ptr::null_mut();
        let result = unsafe { query_active(browser.as_raw(), &mut raw_view) };
        if result != S_OK {
            if let Some(unexpected) = unsafe { OwnedComPtr::from_raw(raw_view) } {
                drop(unexpected);
            }
            return Err(DesktopViewError::QueryActiveShellView(result));
        }
        let view = unsafe { OwnedComPtr::from_raw(raw_view) }.ok_or(
            DesktopViewError::UnexpectedNull("QueryActiveShellView(IShellView)"),
        )?;

        let folder_view =
            query_interface(&view, &IID_IFOLDER_VIEW, DesktopViewError::QueryFolderView)?;

        // IShellView inherits IOleWindow; GetWindow is vtable slot 3.
        type GetWindowFn = unsafe extern "system" fn(Handle, *mut Handle) -> HResult;
        let get_window = unsafe { vtable_method::<GetWindowFn>(view.as_raw(), 3) };
        let mut view_window = ptr::null_mut();
        let result = unsafe { get_window(view.as_raw(), &mut view_window) };
        if result != S_OK {
            return Err(DesktopViewError::GetViewWindow(result));
        }
        if view_window.is_null() {
            return Err(DesktopViewError::InvalidViewWindow);
        }

        let mut owner_process_id = 0u32;
        let owner_thread_id =
            unsafe { GetWindowThreadProcessId(view_window, &mut owner_process_id) };
        if owner_thread_id == 0 || owner_process_id == 0 {
            return Err(DesktopViewError::ViewOwnerUnavailable);
        }
        if owner_process_id != expected_host.process_id() {
            return Err(DesktopViewError::ViewOwnerMismatch {
                expected_process_id: expected_host.process_id(),
                actual_process_id: owner_process_id,
            });
        }

        // IFolderView::GetFolder is vtable slot 5. Query IPersistFolder2 from
        // that folder and compare its absolute PIDL with CSIDL_DESKTOP.
        type GetFolderFn = unsafe extern "system" fn(Handle, *const Guid, *mut Handle) -> HResult;
        let get_folder = unsafe { vtable_method::<GetFolderFn>(folder_view.as_raw(), 5) };
        let mut raw_folder = ptr::null_mut();
        let result =
            unsafe { get_folder(folder_view.as_raw(), &IID_ISHELL_FOLDER, &mut raw_folder) };
        if result != S_OK {
            if let Some(unexpected) = unsafe { OwnedComPtr::from_raw(raw_folder) } {
                drop(unexpected);
            }
            return Err(DesktopViewError::GetViewFolder(result));
        }
        let folder = unsafe { OwnedComPtr::from_raw(raw_folder) }
            .ok_or(DesktopViewError::UnexpectedNull("IFolderView::GetFolder"))?;
        let persist_folder = query_interface(
            &folder,
            &IID_IPERSIST_FOLDER2,
            DesktopViewError::QueryPersistFolder,
        )?;

        // IPersistFolder2 inherits IPersistFolder and IPersist; GetCurFolder
        // is vtable slot 5 after IUnknown.
        type GetCurFolderFn = unsafe extern "system" fn(Handle, *mut Handle) -> HResult;
        let get_current = unsafe { vtable_method::<GetCurFolderFn>(persist_folder.as_raw(), 5) };
        let mut current_folder = ptr::null_mut();
        let result = unsafe { get_current(persist_folder.as_raw(), &mut current_folder) };
        if result != S_OK {
            if !current_folder.is_null() {
                unsafe { CoTaskMemFree(current_folder) };
            }
            return Err(DesktopViewError::GetCurrentFolder(result));
        }
        let Some(current_folder) = NonNull::new(current_folder) else {
            return Err(DesktopViewError::UnexpectedNull(
                "IPersistFolder2::GetCurFolder",
            ));
        };
        let current_folder = OwnedPidl(current_folder);

        let mut desktop_folder = ptr::null_mut();
        let result = unsafe {
            SHGetFolderLocation(
                ptr::null_mut(),
                CSIDL_DESKTOP,
                ptr::null_mut(),
                0,
                &mut desktop_folder,
            )
        };
        if result != S_OK {
            if !desktop_folder.is_null() {
                unsafe { CoTaskMemFree(desktop_folder) };
            }
            return Err(DesktopViewError::GetDesktopFolder(result));
        }
        let Some(desktop_folder) = NonNull::new(desktop_folder) else {
            return Err(DesktopViewError::UnexpectedNull(
                "SHGetFolderLocation(CSIDL_DESKTOP)",
            ));
        };
        let desktop_folder = OwnedPidl(desktop_folder);

        if unsafe { ILIsEqual(current_folder.as_raw(), desktop_folder.as_raw()) } == 0 {
            return Err(DesktopViewError::DesktopRootMismatch);
        }

        // The view may have been replaced while COM calls were in flight.
        match is_same_desktop_host(expected_host) {
            Ok(true) => {}
            Ok(false) => return Err(DesktopViewError::HostIdentityChanged),
            Err(error) => return Err(DesktopViewError::HostIdentityUnavailable(error.to_string())),
        }

        Ok(DesktopViewProbe {
            owner_process_id,
            location,
        })
    }

    pub(super) fn probe(
        expected_host: &DesktopHostIdentity,
    ) -> Result<DesktopViewProbe, DesktopViewError> {
        let _apartment = ComApartment::initialize()?;
        let shell_windows = create_shell_windows()?;

        let csidl_result = acquire_and_validate(
            &shell_windows,
            expected_host,
            DesktopViewLocation::CsidlDesktop,
        );
        if csidl_result.is_ok() {
            return csidl_result;
        }

        let empty_result =
            acquire_and_validate(&shell_windows, expected_host, DesktopViewLocation::Empty);
        if empty_result.is_ok() {
            return empty_result;
        }

        Err(DesktopViewError::AcquisitionAttemptsFailed {
       ssert_eq!(ISHELL_WINDOWS_FIND_WINDOW_SW_SLOT, 15);
            assert_eq!(ISHELL_BROWSER_QUERY_ACTIVE_SHELL_VIEW_SLOT, 15);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of, size_of};

    fn guid_fields(guid: Guid) -> (u32, u16, u16, [u8; 8]) {
        (guid.data1, guid.data2, guid.data3, guid.data4)
    }

    #[test]
    fn com_layouts_match_windows_abi_expectations() {
        assert_eq!(size_of::<Guid>(), 16);
        assert_eq!(align_of::<Guid>(), 4);
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(size_of::<Variant>(), 24);
            assert_eq!(align_of::<Variant>(), 8);
        }
        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(size_of::<Variant>(), 16);
            assert_eq!(align_of::<Variant>(), 4);
        }
    }

    #[test]
    fn variants_encode_the_two_find_window_sw_locator_forms() {
        let empty = Variant::empty();
        assert_eq!(empty.vt, VT_EMPTY);
        assert_eq!(empty.reserved1, 0);
        assert_eq!(empty.reserved2, 0);
        assert_eq!(empty.reserved3, 0);

        let desktop = Variant::desktop_csidl();
        assert_eq!(desktop.vt, VT_I4);
        assert_eq!(desktop.reserved1, 0);
        assert_eq!(desktop.reserved2, 0);
        assert_eq!(desktop.reserved3, 0);
        assert_eq!(unsafe { desktop.value.long_value }, CSIDL_DESKTOP);
    }

    #[test]
    fn desktop_and_interface_guids_have_expected_values() {
        assert_eq!(
            guid_fields(CLSID_SHELL_WINDOWS),
            (
                0x9ba0_5972,
                0xf6a8,
                0x11cf,
                [0xa4, 0x42, 0x00, 0xa0, 0xc9, 0x0a, 0x8f, 0x39],
            )
        );
        assert_eq!(
            guid_fields(IID_ISHELL_WINDOWS),
            (
                0x85cb_6900,
                0x4d95,
                0x11cf,
                [0x96, 0x0c, 0x00, 0x80, 0xc7, 0xf4, 0xee, 0x85],
            )
        );
        assert_eq!(
            guid_fields(IID_ISERVICE_PROVIDER),
            (
                0x6d51_40c1,
                0x7436,
                0x11ce,
                [0x80, 0x34, 0x00, 0xaa, 0x00, 0x60, 0x09, 0xfa],
            )
        );
        assert_eq!(
            guid_fields(SID_STOP_LEVEL_BROWSER),
            (
                0x4c96_be40,
                0x915c,
                0x11cf,
                [0x99, 0xd3, 0x00, 0xaa, 0x00, 0x4a, 0xe8, 0x37],
            )
        );
        assert_eq!(
            guid_fields(IID_ISHELL_BROWSER),
            (
                0x0002_14e2,
                0x0000,
                0x0000,
                [0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
            )
        );
        assert_eq!(
            guid_fields(IID_IFOLDER_VIEW),
            (
                0xcde7_25b0,
                0xccc9,
                0x4519,
                [0x91, 0x7e, 0x32, 0x5d, 0x72, 0xfa, 0xb4, 0xce],
            )
        );
        assert_eq!(
            guid_fields(IID_ISHELL_FOLDER),
            (
                0x0002_14e6,
                0x0000,
                0x0000,
                [0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
            )
        );
        assert_eq!(
            guid_fields(IID_IPERSIST_FOLDER2),
            (
                0x1ac3_d9f0,
                0x175c,
                0x11d1,
                [0x95, 0xbe, 0x00, 0x60, 0x97, 0x97, 0xea, 0x4f],
            )
        );
        assert_eq!(SWC_DESKTOP, 0x8);
        assert_eq!(SWFO_NEEDDISPATCH, 0x1);
        assert_eq!(ISHELL_WINDOWS_FIND_WINDOW_SW_SLOT, 15);
        assert_eq!(ISHELL_BROWSER_QUERY_ACTIVE_SHELL_VIEW_SLOT, 15);
    }
}
