//! Modern Win32 monitor enumeration for the portable WLDE scene core.
//!
//! The adapter returns owned snapshots and never retains HMONITOR handles.
//! Hosts own their event loops and decide when to refresh.

#![deny(unsafe_op_in_unsafe_fn)]

use std::fmt;

use wl_wlde::{MonitorTopology, TopologyError};

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
    message == WM_DISPLAYCHANGE
        || (message == WM_DEVICECHANGE && w_param == DBT_DEVNODES_CHANGED)
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
    type MonitorEnumCallback =
        unsafe extern "system" fn(HMonitor, Hdc, *mut WinRect, isize) -> i32;

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

        MonitorTopology::new(context.monitors)
            .map_err(MonitorError::InvalidTopology)
    }

    fn attached_display_devices() -> Result<Vec<String>, MonitorError> {
        let mut attached = Vec::with_capacity(MAX_MONITORS);
        let mut index = 0;

        loop {
            if index >= MAX_DISPLAY_ADAPTER_SCAN {
                let mut probe: DisplayDeviceW = unsafe { zeroed() };
                probe.cb = size_of::<DisplayDeviceW>() as u32;
                let more_devices = unsafe {
                    EnumDisplayDevicesW(ptr::null(), index, &mut probe, 0)
                };
                if more_devices != 0 {
                    return Err(MonitorError::TooManyMonitors);
                }
                break;
            }

            let mut device: DisplayDeviceW = unsafe { zeroed() };
            device.cb = size_of::<DisplayDeviceW>() as u32;
            let found = unsafe {
                EnumDisplayDevicesW(ptr::null(), index, &mut device, 0)
            };
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
        if !context.attached_devices.iter().any(|device| device == &name) {
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
