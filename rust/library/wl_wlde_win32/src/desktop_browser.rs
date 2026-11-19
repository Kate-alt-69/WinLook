//! WinLook-owned Shell desktop view hosting through the documented ExplorerBrowser control.
//!
//! This adapter never attaches to Explorer's live desktop view. The caller supplies
//! a parent HWND owned by the current process and STA thread. ExplorerBrowser creates
//! its own Shell view beneath that HWND; snapshots reacquire IFolderView and return
//! only transient enumeration ordinals and view-client coordinates.

use std::ffi::c_void;
use std::fmt;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::rc::Rc;

/// Maximum number of entries accepted in one owned Shell-view snapshot.
pub const MAX_DESKTOP_BROWSER_ITEMS: usize = 4_096;

/// A validated rectangle relative to the parent HWND's client area.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DesktopBrowserBounds {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl DesktopBrowserBounds {
    pub fn new(left: i32, top: i32, width: i32, height: i32) -> Result<Self, DesktopBrowserError> {
        if width <= 0 || height <= 0 {
            return Err(DesktopBrowserError::InvalidBounds);
        }
        let right = left
            .checked_add(width)
            .ok_or(DesktopBrowserError::InvalidBounds)?;
        let bottom = top
            .checked_add(height)
            .ok_or(DesktopBrowserError::InvalidBounds)?;
        Ok(Self {
            left,
            top,
            right,
            bottom,
        })
    }

    pub fn width(self) -> i32 {
        self.right - self.left
    }

    pub fn height(self) -> i32 {
        self.bottom - self.top
    }
}

/// A same-process, same-thread HWND supplied by a WinLook-owned native host.
pub struct DesktopBrowserWindow {
    handle: NonNull<c_void>,
    process_id: u32,
    thread_id: u32,
    _thread_bound: PhantomData<Rc<()>>,
}

impl DesktopBrowserWindow {
    /// Wraps a host HWND after verifying that it belongs to this process and thread.
    ///
    /// # Safety
    ///
    /// The handle must remain a live HWND until every OwnedDesktopBrowser using it
    /// has been shut down and dropped. The caller must destroy the HWND only after
    /// its ExplorerBrowser control has been destroyed.
    pub unsafe fn from_raw(handle: *mut c_void) -> Result<Self, DesktopBrowserError> {
        #[cfg(windows)]
        {
            unsafe { platform::validate_parent_window(handle) }
        }
        #[cfg(not(windows))]
        {
            let _ = handle;
            Err(DesktopBrowserError::UnsupportedPlatform)
        }
    }
}

/// A point in the ExplorerBrowser view's client coordinate space.
///
/// These values are not screen coordinates, monitor coordinates, or durable layout
/// positions. Convert them only after obtaining the current view/window geometry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DesktopBrowserPoint {
    pub x: i32,
    pub y: i32,
}

/// One transient entry in the current Shell view enumeration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DesktopBrowserItemPosition {
    /// Ordinal in this snapshot only; never persist it as a Shell item identity.
    pub enumeration_index: u32,
    /// Missing when this Shell item does not expose a position in this view.
    pub position: Option<DesktopBrowserPoint>,
    /// HRESULT from IFolderView::GetItemPosition when position is unavailable.
    pub position_error: Option<i32>,
}

/// Bounded observation of a WinLook-owned ExplorerBrowser view.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DesktopBrowserSnapshot {
    view_generation: u64,
    items: Vec<DesktopBrowserItemPosition>,
}

impl DesktopBrowserSnapshot {
    pub fn view_generation(&self) -> u64 {
        self.view_generation
    }

    pub fn items(&self) -> &[DesktopBrowserItemPosition] {
        &self.items
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// Errors returned by the WinLook-owned Shell view adapter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DesktopBrowserError {
    UnsupportedPlatform,
    InvalidParentWindow,
    ParentWindowNotOwned,
    ParentWindowThreadMismatch,
    InvalidBounds,
    ComInitialization(i32),
    BrowserActivation(i32),
    BrowserInitialize(i32),
    DesktopFolderLocation(i32),
    UnexpectedNull(&'static str),
    BrowseToDesktop(i32),
    GetCurrentView(i32),
    QueryInterface(i32),
    ViewWindowUnavailable,
    ViewWindowNotOwned,
    ViewWindowOutsideParent,
    SetRect(i32),
    ItemCount(i32),
    InvalidItemCount,
    TooManyItems(usize),
    Items(i32),
    EnumeratorNext(i32),
    EnumeratorContract,
    UnstableItemCount { before: usize, after: usize },
    ViewChangedDuringSnapshot,
    ViewGenerationOverflow,
    AllocationFailed,
    Destroy(i32),
}

impl fmt::Display for DesktopBrowserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => {
                f.write_str("owned Shell desktop views are available only on Windows")
            }
            Self::InvalidParentWindow => {
                f.write_str("the ExplorerBrowser parent HWND is null or invalid")
            }
            Self::ParentWindowNotOwned => {
                f.write_str("the ExplorerBrowser parent HWND is not owned by this process")
            }
            Self::ParentWindowThreadMismatch => {
                f.write_str("the ExplorerBrowser parent HWND belongs to another thread")
            }
            Self::InvalidBounds => {
                f.write_str("ExplorerBrowser bounds must have positive, non-overflowing dimensions")
            }
            Self::ComInitialization(code) => {
                write!(f, "ExplorerBrowser STA initialization failed with HRESULT {code:#x}")
            }
            Self::BrowserActivation(code) => {
                write!(f, "CoCreateInstance(CLSID_ExplorerBrowser) failed with HRESULT {code:#x}")
            }
            Self::BrowserInitialize(code) => {
                write!(f, "IExplorerBrowser::Initialize failed with HRESULT {code:#x}")
            }
            Self::DesktopFolderLocation(code) => {
                write!(f, "SHGetFolderLocation(CSIDL_DESKTOP) failed with HRESULT {code:#x}")
            }
            Self::UnexpectedNull(operation) => {
                write!(f, "{operation} returned success with a null output pointer")
            }
            Self::BrowseToDesktop(code) => {
                write!(f, "IExplorerBrowser::BrowseToIDList(Desktop) failed with HRESULT {code:#x}")
            }
            Self::GetCurrentView(code) => {
                write!(f, "IExplorerBrowser::GetCurrentView(IID_IFolderView) failed with HRESULT {code:#x}")
            }
            Self::QueryInterface(code) => {
                write!(f, "QueryInterface failed with HRESULT {code:#x}")
            }
            Self::ViewWindowUnavailable => {
                f.write_str("the owned Shell view did not expose a valid HWND")
            }
            Self::ViewWindowNotOwned => {
                f.write_str("the ExplorerBrowser view HWND is not owned by this process")
            }
            Self::ViewWindowOutsideParent => {
                f.write_str("the ExplorerBrowser view HWND is not a child of the supplied WinLook host")
            }
            Self::SetRect(code) => {
                write!(f, "IExplorerBrowser::SetRect failed with HRESULT {code:#x}")
            }
            Self::ItemCount(code) => {
                write!(f, "IFolderView::ItemCount failed with HRESULT {code:#x}")
            }
            Self::InvalidItemCount => {
                f.write_str("IFolderView returned a negative item count")
            }
            Self::TooManyItems(count) => {
                write!(f, "Shell view item count {count} exceeds the fixed limit of {MAX_DESKTOP_BROWSER_ITEMS}")
            }
            Self::Items(code) => {
                write!(f, "IFolderView::Items failed with HRESULT {code:#x}")
            }
            Self::EnumeratorNext(code) => {
                write!(f, "IEnumIDList::Next failed with HRESULT {code:#x}")
            }
            Self::EnumeratorContract => {
                f.write_str("IEnumIDList returned an invalid fetched count or null PIDL")
            }
            Self::UnstableItemCount { before, after } => {
                write!(f, "Shell view item count changed during snapshot ({before} to {after})")
            }
            Self::ViewChangedDuringSnapshot => {
                f.write_str("ExplorerBrowser replaced its view while a snapshot was being enumerated")
            }
            Self::ViewGenerationOverflow => {
                f.write_str("ExplorerBrowser view generation counter overflowed")
            }
            Self::AllocationFailed => {
                f.write_str("could not allocate the bounded Shell view snapshot")
            }
            Self::Destroy(code) => {
                write!(f, "IExplorerBrowser::Destroy failed with HRESULT {code:#x}")
            }
        }
    }
}

impl std::error::Error for DesktopBrowserError {}

/// Hosts the Desktop namespace in a view created by this process.
///
/// The parent HWND must be created by WinLook on the current STA thread. This type
/// is deliberately !Send/!Sync: every COM call and its Drop must run on that thread.
/// It does not hide Explorer's native desktop, change Shell registrations, persist
/// PIDLs, or enable Group presentation.
pub struct OwnedDesktopBrowser {
    #[cfg(windows)]
    inner: platform::OwnedDesktopBrowser,
    #[cfg(not(windows))]
    _unavailable: PhantomData<Rc<()>>,
}

impl OwnedDesktopBrowser {
    pub fn new(
        parent: DesktopBrowserWindow,
        bounds: DesktopBrowserBounds,
    ) -> Result<Self, DesktopBrowserError> {
        #[cfg(windows)]
        {
            Ok(Self {
                inner: platform::OwnedDesktopBrowser::create(parent, bounds)?,
            })
        }
        #[cfg(not(windows))]
        {
            let _ = (parent, bounds);
            Err(DesktopBrowserError::UnsupportedPlatform)
        }
    }

    /// Resizes the Shell view inside the same WinLook-owned parent HWND.
    pub fn resize(&mut self, bounds: DesktopBrowserBounds) -> Result<(), DesktopBrowserError> {
        #[cfg(windows)]
        {
            self.inner.resize(bounds)
        }
        #[cfg(not(windows))]
        {
            let _ = bounds;
            Err(DesktopBrowserError::UnsupportedPlatform)
        }
    }

    /// Reacquires IFolderView and returns a bounded, transient item/position snapshot.
    pub fn snapshot(&mut self) -> Result<DesktopBrowserSnapshot, DesktopBrowserError> {
        #[cfg(windows)]
        {
            self.inner.snapshot()
        }
        #[cfg(not(windows))]
        {
            Err(DesktopBrowserError::UnsupportedPlatform)
        }
    }

    /// Current monotonically increasing view identity generation.
    pub fn view_generation(&self) -> u64 {
        #[cfg(windows)]
        {
            self.inner.view_generation
        }
        #[cfg(not(windows))]
        {
            0
        }
    }

    /// Explicitly destroys ExplorerBrowser's child view resources.
    pub fn shutdown(&mut self) -> Result<(), DesktopBrowserError> {
        #[cfg(windows)]
        {
            self.inner.shutdown()
        }
        #[cfg(not(windows))]
        {
            Err(DesktopBrowserError::UnsupportedPlatform)
        }
    }
}

#[cfg(windows)]
mod platform {
    use super::*;
    use std::mem::transmute_copy;
    use std::ptr;

    type HResult = i32;
    type Handle = *mut c_void;

    const CLSCTX_INPROC_SERVER: u32 = 0x1;
    const COINIT_APARTMENTTHREADED: u32 = 0x2;
    const CSIDL_DESKTOP: i32 = 0;
    const SBSP_ABSOLUTE: u32 = 0x1;
    const FVM_ICON: u32 = 0x1;
    const SVGIO_ALLVIEW: u32 = 0x2;

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Guid {
        data1: u32,
        data2: u16,
        data3: u16,
        data4: [u8; 8],
    }

    const CLSID_EXPLORER_BROWSER: Guid = Guid {
        data1: 0x71f9_6385,
        data2: 0xddd6,
        data3: 0x48d3,
        data4: [0xa0, 0xc1, 0xae, 0x06, 0xe8, 0xb0, 0x55, 0xfb],
    };
    const IID_IEXPLORER_BROWSER: Guid = Guid {
        data1: 0xdfd3_b6b5,
        data2: 0xc10c,
        data3: 0x4be9,
        data4: [0x85, 0xf6, 0xa6, 0x69, 0x69, 0xf4, 0x02, 0xf6],
    };
    const IID_IFOLDER_VIEW: Guid = Guid {
        data1: 0xcde7_25b0,
        data2: 0xccc9,
        data3: 0x4519,
        data4: [0x91, 0x7e, 0x32, 0x5d, 0x72, 0xfa, 0xb4, 0xce],
    };
    const IID_ISHELL_VIEW: Guid = Guid {
        data1: 0x0002_14e3,
        data2: 0x0000,
        data3: 0x0000,
        data4: [0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
    };
    const IID_IUNKNOWN: Guid = Guid {
        data1: 0x0000_0000,
        data2: 0x0000,
        data3: 0x0000,
        data4: [0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
    };
    const IID_IENUM_IDLIST: Guid = Guid {
        data1: 0x0002_14f2,
        data2: 0x0000,
        data3: 0x0000,
        data4: [0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
    };

    #[repr(C)]
    struct ComObject {
        vtable: *const *const c_void,
    }

    #[repr(C)]
    struct WinRect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    #[repr(C)]
    struct FolderSettings {
        view_mode: u32,
        flags: u32,
    }

    #[repr(C)]
    struct Point {
        x: i32,
        y: i32,
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
        unsafe fn from_raw(raw: Handle, operation: &'static str) -> Result<Self, DesktopBrowserError> {
            NonNull::new(raw)
                .map(Self)
                .ok_or(DesktopBrowserError::UnexpectedNull(operation))
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
        operation: &'static str,
    ) -> Result<OwnedComPtr, DesktopBrowserError> {
        let query = unsafe { vtable_method::<QueryInterfaceFn>(object.as_raw(), 0) };
        let mut raw = ptr::null_mut();
        let result = unsafe { query(object.as_raw(), iid, &mut raw) };
        if result < 0 {
            if let Some(unexpected) = NonNull::new(raw) {
                let unexpected = OwnedComPtr(unexpected);
                drop(unexpected);
            }
            return Err(DesktopBrowserError::QueryInterface(result));
        }
        unsafe { OwnedComPtr::from_raw(raw, operation) }
    }

    struct ComApartment;

    impl ComApartment {
        fn initialize() -> Result<Self, DesktopBrowserError> {
            let result = unsafe {
                CoInitializeEx(ptr::null_mut(), COINIT_APARTMENTTHREADED)
            };
            if result < 0 {
                Err(DesktopBrowserError::ComInitialization(result))
            } else {
                Ok(Self)
            }
        }
    }

    impl Drop for ComApartment {
        fn drop(&mut self) {
            unsafe { CoUninitialize() };
        }
    }

    struct OwnedPidl(NonNull<c_void>);

    impl Drop for OwnedPidl {
        fn drop(&mut self) {
            unsafe { CoTaskMemFree(self.0.as_ptr()) };
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
    }

    #[link(name = "user32")]
    extern "system" {
        fn IsWindow(window: Handle) -> i32;
        fn IsChild(parent: Handle, child: Handle) -> i32;
        fn GetWindowThreadProcessId(window: Handle, process_id: *mut u32) -> u32;
        fn GetCurrentProcessId() -> u32;
        fn GetCurrentThreadId() -> u32;
    }

    pub(super) fn validate_parent_window(
        raw: *mut c_void,
    ) -> Result<DesktopBrowserWindow, DesktopBrowserError> {
        if raw.is_null() || unsafe { IsWindow(raw) } == 0 {
            return Err(DesktopBrowserError::InvalidParentWindow);
        }

        let mut process_id = 0;
        let thread_id = unsafe { GetWindowThreadProcessId(raw, &mut process_id) };
        if thread_id == 0 || process_id == 0 {
            return Err(DesktopBrowserError::InvalidParentWindow);
        }
        if process_id != unsafe { GetCurrentProcessId() } {
            return Err(DesktopBrowserError::ParentWindowNotOwned);
        }
        if thread_id != unsafe { GetCurrentThreadId() } {
            return Err(DesktopBrowserError::ParentWindowThreadMismatch);
        }

        let handle = NonNull::new(raw).ok_or(DesktopBrowserError::InvalidParentWindow)?;
        Ok(DesktopBrowserWindow {
            handle,
            process_id,
            thread_id,
            _thread_bound: PhantomData,
        })
    }

    pub(super) struct OwnedDesktopBrowser {
        // Keep the COM object before the apartment so it is released before CoUninitialize.
        browser: OwnedComPtr,
        apartment: ComApartment,
        parent: DesktopBrowserWindow,
        initialized: bool,
        view_identity: usize,
        view_window: usize,
        view_generation: u64,
    }

    impl OwnedDesktopBrowser {
        pub(super) fn create(
            parent: DesktopBrowserWindow,
            bounds: DesktopBrowserBounds,
        ) -> Result<Self, DesktopBrowserError> {
            let apartment = ComApartment::initialize()?;
            validate_parent_again(&parent)?;

            let mut raw_browser = ptr::null_mut();
            let result = unsafe {
                CoCreateInstance(
                    &CLSID_EXPLORER_BROWSER,
                    ptr::null_mut(),
                    CLSCTX_INPROC_SERVER,
                    &IID_IEXPLORER_BROWSER,
                    &mut raw_browser,
                )
            };
            if result < 0 {
                if let Some(unexpected) = NonNull::new(raw_browser) {
                    drop(OwnedComPtr(unexpected));
                }
                return Err(DesktopBrowserError::BrowserActivation(result));
            }
            let browser = unsafe {
                OwnedComPtr::from_raw(raw_browser, "CoCreateInstance(CLSID_ExplorerBrowser)")?
            };

            type InitializeFn = unsafe extern "system" fn(
                Handle,
                Handle,
                *const WinRect,
                *const FolderSettings,
            ) -> HResult;
            let initialize = unsafe { vtable_method::<InitializeFn>(browser.as_raw(), 3) };
            let rect = bounds.as_win_rect();
            let folder_settings = FolderSettings {
                view_mode: FVM_ICON,
                flags: 0,
            };
            let result = unsafe {
                initialize(
                    browser.as_raw(),
                    parent.handle.as_ptr(),
                    &rect,
                    &folder_settings,
                )
            };
            if result < 0 {
                return Err(DesktopBrowserError::BrowserInitialize(result));
            }

            let mut raw_desktop_pidl = ptr::null_mut();
            let result = unsafe {
                SHGetFolderLocation(
                    ptr::null_mut(),
                    CSIDL_DESKTOP,
                    ptr::null_mut(),
                    0,
                    &mut raw_desktop_pidl,
                )
            };
            if result < 0 {
                if !raw_desktop_pidl.is_null() {
                    unsafe { CoTaskMemFree(raw_desktop_pidl) };
                }
                let destroy = unsafe { vtable_method::<DestroyFn>(browser.as_raw(), 4) };
                let _ = unsafe { destroy(browser.as_raw()) };
                return Err(DesktopBrowserError::DesktopFolderLocation(result));
            }
            let Some(desktop_pidl_raw) = NonNull::new(raw_desktop_pidl) else {
                let destroy = unsafe { vtable_method::<DestroyFn>(browser.as_raw(), 4) };
                let _ = unsafe { destroy(browser.as_raw()) };
                return Err(DesktopBrowserError::UnexpectedNull(
                    "SHGetFolderLocation(CSIDL_DESKTOP)",
                ));
            };
            let desktop_pidl = OwnedPidl(desktop_pidl_raw);

            type BrowseToIdListFn = unsafe extern "system" fn(Handle, Handle, u32) -> HResult;
            let browse = unsafe { vtable_method::<BrowseToIdListFn>(browser.as_raw(), 13) };
            let result =
                unsafe { browse(browser.as_raw(), desktop_pidl.0.as_ptr(), SBSP_ABSOLUTE) };
            drop(desktop_pidl);
            if result < 0 {
                let destroy = unsafe { vtable_method::<DestroyFn>(browser.as_raw(), 4) };
                let _ = unsafe { destroy(browser.as_raw()) };
                return Err(DesktopBrowserError::BrowseToDesktop(result));
            }

            let mut owned = Self {
                browser,
                apartment,
                parent,
                initialized: true,
                view_identity: 0,
                view_window: 0,
                view_generation: 1,
            };
            let (_view, identity, window) = owned.acquire_current_view()?;
            owned.view_identity = identity;
            owned.view_window = window;
            Ok(owned)
        }

        pub(super) fn resize(
            &mut self,
            bounds: DesktopBrowserBounds,
        ) -> Result<(), DesktopBrowserError> {
            self.ensure_initialized()?;
            type SetRectFn = unsafe extern "system" fn(Handle, *mut Handle, WinRect) -> HResult;
            let set_rect = unsafe { vtable_method::<SetRectFn>(self.browser.as_raw(), 5) };
            let mut defer_window_pos = ptr::null_mut();
            let result = unsafe {
                set_rect(
                    self.browser.as_raw(),
                    &mut defer_window_pos,
                    bounds.as_win_rect(),
                )
            };
            if result < 0 {
                Err(DesktopBrowserError::SetRect(result))
            } else {
                Ok(())
            }
        }

        pub(super) fn snapshot(&mut self) -> Result<DesktopBrowserSnapshot, DesktopBrowserError> {
            self.ensure_initialized()?;
            let (view, identity, window) = self.acquire_current_view()?;
            self.update_view_identity(identity, window)?;

            let before = item_count(&view)?;
            let count =
                usize::try_from(before).map_err(|_| DesktopBrowserError::InvalidItemCount)?;
            if count > MAX_DESKTOP_BROWSER_ITEMS {
                return Err(DesktopBrowserError::TooManyItems(count));
            }

            let mut items = Vec::new();
            items
                .try_reserve_exact(count)
                .map_err(|_| DesktopBrowserError::AllocationFailed)?;

            let enumerator = create_item_enumerator(&view)?;
            type NextFn = unsafe extern "system" fn(Handle, u32, *mut Handle, *mut u32) -> HResult;
            let next = unsafe { vtable_method::<NextFn>(enumerator.as_raw(), 3) };
            type GetItemPositionFn =
                unsafe extern "system" fn(Handle, Handle, *mut Point) -> HResult;
            let get_position = unsafe { vtable_method::<GetItemPositionFn>(view.as_raw(), 10) };

            for enumeration_index in 0..count {
                let mut raw_pidl = ptr::null_mut();
                let mut fetched = 0u32;
                let result = unsafe { next(enumerator.as_raw(), 1, &mut raw_pidl, &mut fetched) };
                if result < 0 {
                    if let Some(unexpected) = NonNull::new(raw_pidl) {
                        drop(OwnedPidl(unexpected));
                    }
                    return Err(DesktopBrowserError::EnumeratorNext(result));
                }
                if fetched != 1 {
                    if let Some(unexpected) = NonNull::new(raw_pidl) {
                        drop(OwnedPidl(unexpected));
                    }
                    return Err(DesktopBrowserError::UnstableItemCount {
                        before: count,
                        after: enumeration_index,
                    });
                }
                let pidl = unsafe {
                    OwnedPidl(
                        NonNull::new(raw_pidl).ok_or(DesktopBrowserError::EnumeratorContract)?,
                    )
                };

                let mut point = Point { x: 0, y: 0 };
                let position_result =
                    unsafe { get_position(view.as_raw(), pidl.0.as_ptr(), &mut point) };
                let (position, position_error) = if position_result < 0 {
                    (None, Some(position_result))
                } else {
                    (
                        Some(DesktopBrowserPoint {
                            x: point.x,
                            y: point.y,
                        }),
                        None,
                    )
                };
                items.push(DesktopBrowserItemPosition {
                    enumeration_index: enumeration_index as u32,
                    position,
                    position_error,
                });
                drop(pidl);
            }

            // A final extra read catches a same-count/enumerator disagreement.
            let mut extra_pidl = ptr::null_mut();
            let mut extra_fetched = 0u32;
            let result =
                unsafe { next(enumerator.as_raw(), 1, &mut extra_pidl, &mut extra_fetched) };
            if result < 0 {
                if let Some(unexpected) = NonNull::new(extra_pidl) {
                    drop(OwnedPidl(unexpected));
                }
                return Err(DesktopBrowserError::EnumeratorNext(result));
            }
            if extra_fetched != 0 {
                if let Some(unexpected) = NonNull::new(extra_pidl) {
                    drop(OwnedPidl(unexpected));
                }
                return Err(DesktopBrowserError::UnstableItemCount {
                    before: count,
                    after: count.saturating_add(extra_fetched as usize),
                });
            }
            if !extra_pidl.is_null() {
                drop(unsafe { OwnedPidl(NonNull::new_unchecked(extra_pidl)) });
            }

            let after = item_count(&view)?;
            let after_count =
                usize::try_from(after).map_err(|_| DesktopBrowserError::InvalidItemCount)?;
            if after_count != count {
                return Err(DesktopBrowserError::UnstableItemCount {
                    before: count,
                    after: after_count,
                });
            }

            let (_current_view, final_identity, final_window) = self.acquire_current_view()?;
            if final_identity != identity || final_window != window {
                self.update_view_identity(final_identity, final_window)?;
                return Err(DesktopBrowserError::ViewChangedDuringSnapshot);
            }

            Ok(DesktopBrowserSnapshot {
                view_generation: self.view_generation,
                items,
            })
        }

        pub(super) fn shutdown(&mut self) -> Result<(), DesktopBrowserError> {
            if !self.initialized {
                return Ok(());
            }
            type DestroyFn = unsafe extern "system" fn(Handle) -> HResult;
            let destroy = unsafe { vtable_method::<DestroyFn>(self.browser.as_raw(), 4) };
            let result = unsafe { destroy(self.browser.as_raw()) };
            self.initialized = false;
            if result < 0 {
                Err(DesktopBrowserError::Destroy(result))
            } else {
                Ok(())
            }
        }

        fn ensure_initialized(&self) -> Result<(), DesktopBrowserError> {
            if self.initialized {
                Ok(())
            } else {
                Err(DesktopBrowserError::UnexpectedNull(
                    "destroyed IExplorerBrowser",
                ))
            }
        }

        fn update_view_identity(
            &mut self,
            identity: usize,
            window: usize,
        ) -> Result<(), DesktopBrowserError> {
            if identity == self.view_identity && window == self.view_window {
                return Ok(());
            }
            self.view_generation = self
                .view_generation
                .checked_add(1)
                .ok_or(DesktopBrowserError::ViewGenerationOverflow)?;
            self.view_identity = identity;
            self.view_window = window;
            Ok(())
        }

        fn acquire_current_view(&self) -> Result<(OwnedComPtr, usize, usize), DesktopBrowserError> {
            type GetCurrentViewFn =
                unsafe extern "system" fn(Handle, *const Guid, *mut Handle) -> HResult;
            let get_view = unsafe { vtable_method::<GetCurrentViewFn>(self.browser.as_raw(), 17) };
            let mut raw_view = ptr::null_mut();
            let result =
                unsafe { get_view(self.browser.as_raw(), &IID_IFOLDER_VIEW, &mut raw_view) };
            if result < 0 {
                if let Some(unexpected) = NonNull::new(raw_view) {
                    drop(OwnedComPtr(unexpected));
                }
                return Err(DesktopBrowserError::GetCurrentView(result));
            }
            let view =
                unsafe { OwnedComPtr::from_raw(raw_view, "IExplorerBrowser::GetCurrentView") }?;

            let identity = query_interface(&view, &IID_IUNKNOWN, "IFolderView IUnknown")?;
            let identity_address = identity.as_raw() as usize;
            drop(identity);

            let shell_view = query_interface(&view, &IID_ISHELL_VIEW, "IFolderView IShellView")?;
            type GetWindowFn = unsafe extern "system" fn(Handle, *mut Handle) -> HResult;
            let get_window = unsafe { vtable_method::<GetWindowFn>(shell_view.as_raw(), 3) };
            let mut raw_window = ptr::null_mut();
            let result = unsafe { get_window(shell_view.as_raw(), &mut raw_window) };
            if result < 0 || raw_window.is_null() {
                return Err(DesktopBrowserError::ViewWindowUnavailable);
            }
            if unsafe { IsWindow(raw_window) } == 0 {
                return Err(DesktopBrowserError::ViewWindowUnavailable);
            }
            let mut process_id = 0u32;
            let thread_id = unsafe { GetWindowThreadProcessId(raw_window, &mut process_id) };
            if thread_id == 0
                || process_id != self.parent.process_id
                || thread_id != self.parent.thread_id
            {
                return Err(DesktopBrowserError::ViewWindowNotOwned);
            }
            if unsafe { IsChild(self.parent.handle.as_ptr(), raw_window) } == 0 {
                return Err(DesktopBrowserError::ViewWindowOutsideParent);
            }
            Ok((view, identity_address, raw_window as usize))
        }
    }

    impl Drop for OwnedDesktopBrowser {
        fn drop(&mut self) {
            let _ = self.shutdown();
            let _ = &self.apartment;
        }
    }

    fn validate_parent_again(parent: &DesktopBrowserWindow) -> Result<(), DesktopBrowserError> {
        if unsafe { IsWindow(parent.handle.as_ptr()) } == 0 {
            return Err(DesktopBrowserError::InvalidParentWindow);
        }
        let mut process_id = 0u32;
        let thread_id =
            unsafe { GetWindowThreadProcessId(parent.handle.as_ptr(), &mut process_id) };
        if process_id != parent.process_id || process_id != unsafe { GetCurrentProcessId() } {
            return Err(DesktopBrowserError::ParentWindowNotOwned);
        }
        if thread_id != parent.thread_id || thread_id != unsafe { GetCurrentThreadId() } {
            return Err(DesktopBrowserError::ParentWindowThreadMismatch);
        }
        Ok(())
    }

    fn item_count(view: &OwnedComPtr) -> Result<i32, DesktopBrowserError> {
        type ItemCountFn = unsafe extern "system" fn(Handle, u32, *mut i32) -> HResult;
        let item_count = unsafe { vtable_method::<ItemCountFn>(view.as_raw(), 6) };
        let mut count = 0i32;
        let result = unsafe { item_count(view.as_raw(), SVGIO_ALLVIEW, &mut count) };
        if result < 0 {
            return Err(DesktopBrowserError::ItemCount(result));
        }
        if count < 0 {
            return Err(DesktopBrowserError::InvalidItemCount);
        }
        Ok(count)
    }

    fn create_item_enumerator(view: &OwnedComPtr) -> Result<OwnedComPtr, DesktopBrowserError> {
        type ItemsFn = unsafe extern "system" fn(Handle, u32, *const Guid, *mut Handle) -> HResult;
        let items = unsafe { vtable_method::<ItemsFn>(view.as_raw(), 7) };
        let mut raw_enumerator = ptr::null_mut();
        let result = unsafe {
            items(
                view.as_raw(),
                SVGIO_ALLVIEW,
                &IID_IENUM_IDLIST,
                &mut raw_enumerator,
            )
        };
        if result < 0 {
            if let Some(unexpected) = NonNull::new(raw_enumerator) {
                drop(OwnedComPtr(unexpected));
            }
            return Err(DesktopBrowserError::Items(result));
        }
        unsafe { OwnedComPtr::from_raw(raw_enumerator, "IFolderView::Items") }
    }

    impl DesktopBrowserBounds {
        fn as_win_rect(self) -> WinRect {
            WinRect {
                left: self.left,
                top: self.top,
                right: self.right,
                bottom: self.bottom,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browser_bounds_reject_zero_and_overflowing_dimensions() {
        assert_eq!(
            DesktopBrowserBounds::new(0, 0, 0, 100),
            Err(DesktopBrowserError::InvalidBounds)
        );
        assert_eq!(
            DesktopBrowserBounds::new(i32::MAX - 2, 0, 4, 100),
            Err(DesktopBrowserError::InvalidBounds)
        );
        let bounds = DesktopBrowserBounds::new(-1920, 20, 1920, 1080).unwrap();
        assert_eq!(bounds.width(), 1920);
        assert_eq!(bounds.height(), 1080);
    }

    #[cfg(not(windows))]
    #[test]
    fn non_windows_host_creation_is_explicitly_unsupported() {
        assert!(matches!(
            unsafe { DesktopBrowserWindow::from_raw(std::ptr::null_mut()) },
            Err(DesktopBrowserError::UnsupportedPlatform)
        ));
    }
}
