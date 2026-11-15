//! Modern Win32 monitor enumeration for the portable WLDE scene core.
//!
//! The adapter returns owned snapshots and never retains HMONITOR handles.
//! Hosts own their event loops and decide when to refresh.

#![deny(unsafe_op_in_unsafe_fn)]

use std::fmt;
use std::time::Duration;

use wl_wlde::{MonitorTopology, TopologyError};

mod desktop_inventory;
mod desktop_shell_icon;
pub use desktop_inventory::{
    enumerate_desktop_items, watch_desktop_folders, DesktopFolder, DesktopInventoryError,
    DesktopInventoryItem, DesktopInventoryWatcher, DesktopItemIdentityStability,
    MAX_DESKTOP_INVENTORY_ITEMS,
};
pub use desktop_shell_icon::{
    extract_desktop_shell_icon, rasterize_desktop_shell_icon_rgba, DesktopShellIcon,
    DesktopShellIconError, DesktopShellIconRgba, DesktopShellIconSize,
    MAX_DESKTOP_SHELL_ICON_DIMENSION,
};

/// Win32 message sent after the display resolution or configuration changes.
pub const WM_DISPLAYCHANGE: u32 = 0x007E;
/// Win32 device-change broadcast message.
pub const WM_DEVICECHANGE: u32 = 0x0219;
/// `WM_DEVICECHANGE` event indicating that the device tree changed.
pub const DBT_DEVNODES_CHANGED: usize = 0x0007;

/// Errors returned while building a monitor topology snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MonitorError {
    /// The adapter was called on a target other than Windows.
    UnsupportedPlatform,
    /// A Win32 API failed.
    Win32Call {
        /// Name of the failed API.
        function: &'static str,
        /// Value returned by GetLastError.
        code: u32,
    },
    /// More than the portable 16-monitor bound was discovered.
    TooManyMonitors,
    /// No attached desktop monitors could be enumerated.
    NoMonitors,
    /// Windows returned an empty or invalid UTF-16 display-device name.
    InvalidDeviceName,
    /// Windows returned an invalid monitor/work-area rectangle or a monitor handle.
    InvalidMonitorInfo,
    /// The enumerated records failed the portable topology validator.
    InvalidTopology(TopologyError),
}

impl fmt::Display for MonitorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => {
                f.write_str("Win32 monitor enumeration is only available on Windows")
            }
            Self::Win32Call { function, code } => {
                write!(f, "{function} failed with Win32 error {code:#x}")
            }
            Self::TooManyMonitors => {
                write!(f, "monitor count exceeds the WLDE limit of 16")
            }
            Self::NoMonitors => f.write_str("Windows reported no attached desktop monitors"),
            Self::InvalidDeviceName => {
                f.write_str("Windows returned an invalid monitor device name")
            }
            Self::InvalidMonitorInfo => {
                f.write_str("Windows returned invalid monitor geometry or a monitor handle")
            }
            Self::InvalidTopology(error) => {
                write!(f, "Windows monitor topology failed validation: {error}")
            }
        }
    }
}

impl std::error::Error for MonitorError {}

/// Returns true for host-window messages that should trigger monitor re-enumeration.
///
/// The function intentionally does not install a listener or perform the refresh;
/// a host should call `refresh_topology` from its own window/message loop.
pub const fn should_refresh_topology(message: u32, w_param: usize) -> bool {
    message == WM_DISPLAYCHANGE || (message == WM_DEVICECHANGE && w_param == DBT_DEVNODES_CHANGED)
}

/// Enumerates the current Windows desktop-attached monitor topology.
#[cfg(windows)]
pub fn enumerate_monitors() -> Result<MonitorTopology, MonitorError> {
    platform::enumerate()
}

/// Returns an explicit unsupported-platform error on non-Windows hosts.
#[cfg(not(windows))]
pub fn enumerate_monitors() -> Result<MonitorTopology, MonitorError> {
    Err(MonitorError::UnsupportedPlatform)
}

/// Replaces `current` only after a complete new topology was successfully enumerated.
///
/// On failure the caller retains the last known-good snapshot.
pub fn refresh_topology(current: &mut MonitorTopology) -> Result<(), MonitorError> {
    let replacement = enumerate_monitors()?;
    *current = replacement;
    Ok(())
}

#[cfg(windows)]
mod platform {
    use std::ffi::c_void;
    use std::mem::{size_of, zeroed};
    use std::ptr;

    use wl_wlde::{Monitor, MonitorTopology, Rect, MAX_MONITORS};

    use crate::MonitorError;

    const DISPLAY_DEVICE_ATTACHED_TO_DESKTOP: u32 = 0x00000001;
    const DISPLAY_DEVICE_MIRRORING_DRIVER: u32 = 0x00000008;
    const MONITORINFOF_PRIMARY: u32 = 0x00000001;
    const MAX_DISPLAY_ADAPTER_SCAN: u32 = 64;
    const DEVICE_NAME_CAPACITY: usize = 32;

    type HMonitor = *mut c_void;
    type Hdc = *mut c_void;
    type MonitorEnumCallback = unsafe extern "system" fn(HMonitor, Hdc, *mut WinRect, isize) -> i32;

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct WinRect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    #[repr(C)]
    struct MonitorInfo {
        cb_size: u32,
        rc_monitor: WinRect,
        rc_work: WinRect,
        flags: u32,
    }

    // MONITORINFOEXW begins with the full MONITORINFO layout.
    #[repr(C)]
    struct MonitorInfoExW {
        monitor_info: MonitorInfo,
        device: [u16; DEVICE_NAME_CAPACITY],
    }

    #[repr(C)]
    struct DisplayDeviceW {
        cb: u32,
        device_name: [u16; DEVICE_NAME_CAPACITY],
        device_string: [u16; 128],
        state_flags: u32,
        device_id: [u16; 128],
        device_key: [u16; 128],
    }

    #[link(name = "user32")]
    extern "system" {
        fn EnumDisplayDevicesW(
            device: *const u16,
            device_index: u32,
            display_device: *mut DisplayDeviceW,
            flags: u32,
        ) -> i32;
        fn EnumDisplayMonitors(
            hdc: Hdc,
            clip_rect: *const WinRect,
            callback: Option<MonitorEnumCallback>,
            data: isize,
        ) -> i32;
        fn GetMonitorInfoW(monitor: HMonitor, info: *mut MonitorInfo) -> i32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetLastError() -> u32;
    }

    struct EnumerationContext {
        attached_devices: Vec<String>,
        monitors: Vec<Monitor>,
        error: Option<MonitorError>,
    }

    pub(super) fn enumerate() -> Result<MonitorTopology, MonitorError> {
        let attached_devices = attached_display_devices()?;
        let mut context = EnumerationContext {
            attached_devices,
            monitors: Vec::with_capacity(MAX_MONITORS),
            error: None,
        };

        let callback_data = (&mut context as *mut EnumerationContext) as isize;
        let succeeded = unsafe {
            EnumDisplayMonitors(
                ptr::null_mut(),
                ptr::null(),
                Some(monitor_callback),
                callback_data,
            )
        };

        if let Some(error) = context.error {
            return Err(error);
        }
        if succeeded == 0 {
            return Err(MonitorError::Win32Call {
                function: "EnumDisplayMonitors",
                code: unsafe { GetLastError() },
            });
        }
        if context.monitors.is_empty() {
            return Err(MonitorError::NoMonitors);
        }

        MonitorTopology::new(context.monitors).map_err(MonitorError::InvalidTopology)
    }

    fn attached_display_devices() -> Result<Vec<String>, MonitorError> {
        let mut attached = Vec::with_capacity(MAX_MONITORS);
        let mut index = 0;

        loop {
            if index >= MAX_DISPLAY_ADAPTER_SCAN {
                let mut probe: DisplayDeviceW = unsafe { zeroed() };
                probe.cb = size_of::<DisplayDeviceW>() as u32;
                let more_devices =
                    unsafe { EnumDisplayDevicesW(ptr::null(), index, &mut probe, 0) };
                if more_devices != 0 {
                    return Err(MonitorError::TooManyMonitors);
                }
                break;
            }

            let mut device: DisplayDeviceW = unsafe { zeroed() };
            device.cb = size_of::<DisplayDeviceW>() as u32;
            let found = unsafe { EnumDisplayDevicesW(ptr::null(), index, &mut device, 0) };
            if found == 0 {
                break;
            }

            let is_attached = device.state_flags & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP != 0;
            let is_mirroring = device.state_flags & DISPLAY_DEVICE_MIRRORING_DRIVER != 0;
            if is_attached && !is_mirroring {
                let name = decode_device_name(&device.device_name)?;
                if !attached.iter().any(|existing| existing == &name) {
                    if attached.len() >= MAX_MONITORS {
                        return Err(MonitorError::TooManyMonitors);
                    }
                    attached.push(name);
                }
            }
            index += 1;
        }

        if attached.is_empty() {
            return Err(MonitorError::NoMonitors);
        }
        Ok(attached)
    }

    unsafe extern "system" fn monitor_callback(
        monitor: HMonitor,
        _hdc: Hdc,
        _rect: *mut WinRect,
        data: isize,
    ) -> i32 {
        if data == 0 {
            return 0;
        }
        let context = unsafe { &mut *(data as *mut EnumerationContext) };
        if context.error.is_some() {
            return 0;
        }
        if monitor.is_null() {
            context.error = Some(MonitorError::InvalidMonitorInfo);
            return 0;
        }

        let mut info: MonitorInfoExW = unsafe { zeroed() };
        info.monitor_info.cb_size = size_of::<MonitorInfoExW>() as u32;
        let succeeded = unsafe { GetMonitorInfoW(monitor, &mut info.monitor_info) };
        if succeeded == 0 {
            context.error = Some(MonitorError::Win32Call {
                function: "GetMonitorInfoW",
                code: unsafe { GetLastError() },
            });
            return 0;
        }

        let name = match decode_device_name(&info.device) {
            Ok(name) => name,
            Err(error) => {
                context.error = Some(error);
                return 0;
            }
        };
        if !context
            .attached_devices
            .iter()
            .any(|device| device == &name)
        {
            // Ignore pseudo-monitors and devices not attached to the desktop.
            return 1;
        }
        if context.monitors.len() >= MAX_MONITORS {
            context.error = Some(MonitorError::TooManyMonitors);
            return 0;
        }

        let bounds = match convert_rect(info.monitor_info.rc_monitor) {
            Ok(rect) => rect,
            Err(error) => {
                context.error = Some(error);
                return 0;
            }
        };
        let work_area = match convert_rect(info.monitor_info.rc_work) {
            Ok(rect) => rect,
            Err(error) => {
                context.error = Some(error);
                return 0;
            }
        };
        let primary = info.monitor_info.flags & MONITORINFOF_PRIMARY != 0;

        match Monitor::new(name, bounds, work_area, primary) {
            Ok(record) => {
                context.monitors.push(record);
                1
            }
            Err(error) => {
                context.error = Some(MonitorError::InvalidTopology(error));
                0
            }
        }
    }

    fn decode_device_name(units: &[u16]) -> Result<String, MonitorError> {
        let length = units
            .iter()
            .position(|unit| *unit == 0)
            .unwrap_or(units.len());
        if length == 0 {
            return Err(MonitorError::InvalidDeviceName);
        }
        String::from_utf16(&units[..length]).map_err(|_| MonitorError::InvalidDeviceName)
    }

    fn convert_rect(rect: WinRect) -> Result<Rect, MonitorError> {
        let width = i64::from(rect.right) - i64::from(rect.left);
        let height = i64::from(rect.bottom) - i64::from(rect.top);
        if width <= 0 || height <= 0 {
            return Err(MonitorError::InvalidMonitorInfo);
        }
        Rect::new(
            f64::from(rect.left),
            f64::from(rect.top),
            width as f64,
            height as f64,
        )
        .map_err(|_| MonitorError::InvalidMonitorInfo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(windows))]
    use wl_wlde::{Monitor, Rect};

    #[test]
    fn refresh_filter_accepts_only_topology_related_notifications() {
        assert!(should_refresh_topology(WM_DISPLAYCHANGE, 0));
        assert!(should_refresh_topology(
            WM_DEVICECHANGE,
            DBT_DEVNODES_CHANGED
        ));
        assert!(!should_refresh_topology(WM_DEVICECHANGE, 0));
        assert!(!should_refresh_topology(0x0111, DBT_DEVNODES_CHANGED));
    }

    #[cfg(not(windows))]
    #[test]
    fn non_windows_enumeration_is_explicitly_unsupported_and_preserves_snapshot() {
        let monitor = Monitor::new(
            "DISPLAY1",
            Rect::new(0.0, 0.0, 1920.0, 1080.0).unwrap(),
            Rect::new(0.0, 0.0, 1920.0, 1040.0).unwrap(),
            true,
        )
        .unwrap();
        let mut topology = MonitorTopology::new(vec![monitor]).unwrap();

        assert_eq!(enumerate_monitors(), Err(MonitorError::UnsupportedPlatform));
        assert_eq!(
            refresh_topology(&mut topology),
            Err(MonitorError::UnsupportedPlatform)
        );
        assert_eq!(topology.len(), 1);
        assert_eq!(topology.primary().unwrap().id(), "DISPLAY1");
    }
}

/// Stable-enough identity for the Explorer process that owns this session's desktop.
///
/// A PID alone is not a durable process identity because Windows can recycle it.
/// The creation timestamp distinguishes a restarted Explorer process that happens
/// to receive the same PID. No process handle is retained by this record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DesktopHostIdentity {
    process_id: u32,
    session_id: u32,
    creation_time_100ns: u64,
    image_path: std::path::PathBuf,
}

impl DesktopHostIdentity {
    pub fn process_id(&self) -> u32 {
        self.process_id
    }

    pub fn session_id(&self) -> u32 {
        self.session_id
    }

    /// FILETIME ticks (100 ns since 1601-01-01 UTC), suitable for identity comparison.
    pub fn creation_time_100ns(&self) -> u64 {
        self.creation_time_100ns
    }

    pub fn image_path(&self) -> &std::path::Path {
        &self.image_path
    }
}

/// Errors reported while discovering the current session's desktop-owning Shell process.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DesktopHostError {
    UnsupportedPlatform,
    ShellWindowUnavailable,
    ShellProcessUnavailable,
    OpenProcess {
        code: u32,
    },
    QueryImagePath {
        code: u32,
    },
    InvalidImagePath,
    UnexpectedImageName(std::path::PathBuf),
    QuerySession {
        code: u32,
    },
    SessionMismatch {
        desktop_session: u32,
        host_session: u32,
    },
    QueryProcessTimes {
        code: u32,
    },
    WaitForProcess {
        code: u32,
    },
}

impl fmt::Display for DesktopHostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => {
                f.write_str("desktop-host discovery is available only on Windows")
            }
            Self::ShellWindowUnavailable => {
                f.write_str("Windows has not exposed a Shell desktop window in this session")
            }
            Self::ShellProcessUnavailable => {
                f.write_str("Windows did not return a process ID for the Shell desktop window")
            }
            Self::OpenProcess { code } => {
                write!(f, "could not query the Shell desktop process: Win32 error {code:#x}")
            }
            Self::QueryImagePath { code } => {
                write!(f, "could not read the Shell desktop process image path: Win32 error {code:#x}")
            }
            Self::InvalidImagePath => {
                f.write_str("Windows returned an invalid Shell desktop process image path")
            }
            Self::UnexpectedImageName(path) => write!(
                f,
                "the Shell desktop window is owned by {}, not explorer.exe",
                path.display()
            ),
            Self::QuerySession { code } => {
                write!(f, "could not verify the interactive desktop session: Win32 error {code:#x}")
            }
            Self::SessionMismatch {
                desktop_session,
                host_session,
            } => write!(
                f,
                "Shell desktop belongs to session {desktop_session}, but this BGW process belongs to session {host_session}"
            ),
            Self::QueryProcessTimes { code } => {
                write!(f, "could not read the Shell process creation time: Win32 error {code:#x}")
            }
            Self::WaitForProcess { code } => {
                write!(f, "waiting for the Shell process failed: Win32 result {code:#x}")
            }
        }
    }
}

impl std::error::Error for DesktopHostError {}

/// Checks only the executable basename, independent of path separator style.
pub fn is_explorer_image_path(path: &str) -> bool {
    path.rsplit(['\\', '/'])
        .next()
        .is_some_and(|name| name.eq_ignore_ascii_case("explorer.exe"))
}

/// Discovers the Explorer process that owns the current interactive desktop.
///
/// The returned identity is scoped to the caller's Windows session. BGW should
/// re-run discovery after Shell restart notifications and compare all identity
/// fields before reattaching. This function only observes the host; it does not
/// inject code, install hooks, change Explorer state or retain process handles.
#[cfg(windows)]
pub fn discover_desktop_host() -> Result<DesktopHostIdentity, DesktopHostError> {
    desktop_host_platform::discover()
}

/// Non-Windows builds expose the same API but never guess a desktop host.
#[cfg(not(windows))]
pub fn discover_desktop_host() -> Result<DesktopHostIdentity, DesktopHostError> {
    Err(DesktopHostError::UnsupportedPlatform)
}

/// Returns true only when a fresh discovery resolves to the same process instance.
pub fn is_same_desktop_host(identity: &DesktopHostIdentity) -> Result<bool, DesktopHostError> {
    let current = discover_desktop_host()?;
    Ok(current == *identity)
}

/// A waitable reference to the current desktop-owning Explorer process.
///
/// The process handle is opened for query/synchronization only and is closed
/// automatically when this monitor is dropped. It does not keep Explorer alive.
pub struct DesktopHostMonitor {
    identity: DesktopHostIdentity,
    #[cfg(windows)]
    process: desktop_host_platform::ProcessMonitor,
}

impl DesktopHostMonitor {
    /// Process identity captured when this monitor was created.
    pub fn identity(&self) -> &DesktopHostIdentity {
        &self.identity
    }

    /// Waits for this Explorer process to exit.
    ///
    /// Returns true when the process handle is signaled, false on timeout.
    /// The caller should also re-check the Shell window owner after a timeout,
    /// because the Shell desktop window can be replaced without a process exit.
    pub fn wait_for_exit(&self, timeout: Duration) -> Result<bool, DesktopHostError> {
        #[cfg(windows)]
        {
            let timeout_ms = timeout.as_millis().min(u128::from(u32::MAX - 1)) as u32;
            self.process.wait_for_exit(timeout_ms)
        }
        #[cfg(not(windows))]
        {
            let _ = timeout;
            Err(DesktopHostError::UnsupportedPlatform)
        }
    }
}

/// Discovers and opens a waitable handle to the current desktop-owning Explorer process.
///
/// The monitor keeps only PROCESS_QUERY_INFORMATION and SYNCHRONIZE rights.
/// It does not request memory access, thread creation, termination or write access.
#[cfg(windows)]
pub fn monitor_desktop_host() -> Result<DesktopHostMonitor, DesktopHostError> {
    let identity = discover_desktop_host()?;
    let process = desktop_host_platform::monitor(&identity)?;
    if !is_same_desktop_host(&identity)? {
        return Err(DesktopHostError::ShellProcessUnavailable);
    }
    Ok(DesktopHostMonitor { identity, process })
}

/// Non-Windows builds expose the same API but never guess a desktop host.
#[cfg(not(windows))]
pub fn monitor_desktop_host() -> Result<DesktopHostMonitor, DesktopHostError> {
    Err(DesktopHostError::UnsupportedPlatform)
}

#[cfg(windows)]
mod desktop_host_platform {
    use std::ffi::c_void;
    use std::mem::MaybeUninit;
    use std::os::windows::ffi::OsStringExt;
    use std::path::PathBuf;

    use crate::{is_explorer_image_path, DesktopHostError, DesktopHostIdentity};

    type Handle = *mut c_void;

    // PROCESS_QUERY_INFORMATION is required by ProcessIdToSessionId and GetProcessTimes.
    const PROCESS_QUERY_INFORMATION: u32 = 0x0400;
    const SYNCHRONIZE: u32 = 0x0010_0000;
    const IMAGE_PATH_CAPACITY: usize = 32_768;
    const WAIT_OBJECT_0: u32 = 0x0000_0000;
    const WAIT_TIMEOUT: u32 = 0x0000_0102;
    const WAIT_FAILED: u32 = 0xFFFF_FFFF;

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct FileTime {
        low: u32,
        high: u32,
    }

    impl FileTime {
        fn as_u64(self) -> u64 {
            (u64::from(self.high) << 32) | u64::from(self.low)
        }
    }

    struct OwnedProcess(Handle);

    impl Drop for OwnedProcess {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    CloseHandle(self.0);
                }
            }
        }
    }

    #[link(name = "user32")]
    extern "system" {
        fn GetShellWindow() -> Handle;
        fn GetWindowThreadProcessId(hwnd: Handle, process_id: *mut u32) -> u32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn OpenProcess(access: u32, inherit_handle: i32, process_id: u32) -> Handle;
        fn QueryFullProcessImageNameW(
            process: Handle,
            flags: u32,
            image_path: *mut u16,
            size: *mut u32,
        ) -> i32;
        fn ProcessIdToSessionId(process_id: u32, session_id: *mut u32) -> i32;
        fn GetCurrentProcessId() -> u32;
        fn GetProcessTimes(
            process: Handle,
            creation_time: *mut FileTime,
            exit_time: *mut FileTime,
            kernel_time: *mut FileTime,
            user_time: *mut FileTime,
        ) -> i32;
        fn CloseHandle(handle: Handle) -> i32;
        fn GetLastError() -> u32;
        fn WaitForSingleObject(handle: Handle, milliseconds: u32) -> u32;
    }

    fn process_creation_time(process: Handle) -> Result<u64, DesktopHostError> {
        let mut creation_time = MaybeUninit::<FileTime>::uninit();
        let mut exit_time = FileTime::default();
        let mut kernel_time = FileTime::default();
        let mut user_time = FileTime::default();
        let times_ok = unsafe {
            GetProcessTimes(
                process,
                creation_time.as_mut_ptr(),
                &mut exit_time,
                &mut kernel_time,
                &mut user_time,
            )
        };
        if times_ok == 0 {
            return Err(DesktopHostError::QueryProcessTimes {
                code: unsafe { GetLastError() },
            });
        }
        Ok(unsafe { creation_time.assume_init() }.as_u64())
    }

    pub(super) struct ProcessMonitor(OwnedProcess);

    impl ProcessMonitor {
        pub(super) fn wait_for_exit(&self, timeout_ms: u32) -> Result<bool, DesktopHostError> {
            match unsafe { WaitForSingleObject(self.0 .0, timeout_ms) } {
                WAIT_OBJECT_0 => Ok(true),
                WAIT_TIMEOUT => Ok(false),
                WAIT_FAILED => Err(DesktopHostError::WaitForProcess {
                    code: unsafe { GetLastError() },
                }),
                code => Err(DesktopHostError::WaitForProcess { code }),
            }
        }
    }

    pub(super) fn monitor(
        identity: &DesktopHostIdentity,
    ) -> Result<ProcessMonitor, DesktopHostError> {
        let raw_process = unsafe {
            OpenProcess(
                PROCESS_QUERY_INFORMATION | SYNCHRONIZE,
                0,
                identity.process_id,
            )
        };
        if raw_process.is_null() {
            return Err(DesktopHostError::OpenProcess {
                code: unsafe { GetLastError() },
            });
        }
        let process = OwnedProcess(raw_process);
        let creation_time_100ns = process_creation_time(process.0)?;
        if creation_time_100ns != identity.creation_time_100ns {
            return Err(DesktopHostError::ShellProcessUnavailable);
        }
        Ok(ProcessMonitor(process))
    }

    pub(super) fn discover() -> Result<DesktopHostIdentity, DesktopHostError> {
        let shell_window = unsafe { GetShellWindow() };
        if shell_window.is_null() {
            return Err(DesktopHostError::ShellWindowUnavailable);
        }

        let mut process_id = 0u32;
        let thread_id = unsafe { GetWindowThreadProcessId(shell_window, &mut process_id) };
        if thread_id == 0 || process_id == 0 {
            return Err(DesktopHostError::ShellProcessUnavailable);
        }

        let raw_process = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION, 0, process_id) };
        if raw_process.is_null() {
            return Err(DesktopHostError::OpenProcess {
                code: unsafe { GetLastError() },
            });
        }
        let process = OwnedProcess(raw_process);

        let mut image_buffer = vec![0u16; IMAGE_PATH_CAPACITY];
        let mut image_length = image_buffer.len() as u32;
        let image_ok = unsafe {
            QueryFullProcessImageNameW(process.0, 0, image_buffer.as_mut_ptr(), &mut image_length)
        };
        if image_ok == 0 {
            return Err(DesktopHostError::QueryImagePath {
                code: unsafe { GetLastError() },
            });
        }
        if image_length == 0 || image_length as usize > image_buffer.len() {
            return Err(DesktopHostError::InvalidImagePath);
        }
        image_buffer.truncate(image_length as usize);
        let image_path = PathBuf::from(std::ffi::OsString::from_wide(&image_buffer));
        let image_path_text = image_path.to_string_lossy();
        if !is_explorer_image_path(&image_path_text) {
            return Err(DesktopHostError::UnexpectedImageName(image_path));
        }

        let mut desktop_session = 0u32;
        if unsafe { ProcessIdToSessionId(process_id, &mut desktop_session) } == 0 {
            return Err(DesktopHostError::QuerySession {
                code: unsafe { GetLastError() },
            });
        }
        let current_process_id = unsafe { GetCurrentProcessId() };
        let mut host_session = 0u32;
        if unsafe { ProcessIdToSessionId(current_process_id, &mut host_session) } == 0 {
            return Err(DesktopHostError::QuerySession {
                code: unsafe { GetLastError() },
            });
        }
        if desktop_session != host_session {
            return Err(DesktopHostError::SessionMismatch {
                desktop_session,
                host_session,
            });
        }

        let creation_time_100ns = process_creation_time(process.0)?;

        // Avoid returning an identity after the Shell window has already been replaced.
        let verified_shell_window = unsafe { GetShellWindow() };
        if verified_shell_window.is_null() || verified_shell_window != shell_window {
            return Err(DesktopHostError::ShellWindowUnavailable);
        }
        let mut verified_process_id = 0u32;
        let verified_thread_id =
            unsafe { GetWindowThreadProcessId(verified_shell_window, &mut verified_process_id) };
        if verified_thread_id == 0 || verified_process_id != process_id {
            return Err(DesktopHostError::ShellProcessUnavailable);
        }

        // Keep raw HWND/handles out of the public snapshot; they are transient.
        Ok(DesktopHostIdentity {
            process_id,
            session_id: desktop_session,
            creation_time_100ns,
            image_path,
        })
    }
}

#[cfg(test)]
mod desktop_host_tests {
    use super::*;

    #[test]
    fn explorer_image_path_matching_is_case_insensitive_and_separator_agnostic() {
        assert!(is_explorer_image_path(r"C:\Windows\explorer.exe"));
        assert!(is_explorer_image_path(r"c:\windows\EXPLORER.EXE"));
        assert!(is_explorer_image_path("/usr/compat/Explorer.exe"));
        assert!(!is_explorer_image_path(r"C:\Windows\explorer.exe.bak"));
        assert!(!is_explorer_image_path(r"C:\Windows\not-explorer.exe"));
    }

    #[cfg(not(windows))]
    #[test]
    fn desktop_host_discovery_and_monitor_are_unsupported_off_windows() {
        assert_eq!(
            discover_desktop_host(),
            Err(DesktopHostError::UnsupportedPlatform)
        );
        assert!(matches!(
            monitor_desktop_host(),
            Err(DesktopHostError::UnsupportedPlatform)
        ));
    }
}
