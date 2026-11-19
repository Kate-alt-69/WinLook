//! Read-only native Windows target detection.

use std::mem::size_of;
use std::ptr;

use windows_sys::Win32::Foundation::{GetLastError, HWND, RECT};
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_QUERY_VALUE,
    KEY_WOW64_64KEY, REG_DWORD, REG_EXPAND_SZ, REG_SZ,
};
use windows_sys::Win32::System::RemoteDesktop::ProcessIdToSessionId;
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, GetCurrentProcessId, IsWow64Process2,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    FindWindowExW, FindWindowW, GetWindowRect, GetWindowThreadProcessId, IsWindowVisible,
};

use crate::target::{Architecture, TargetDetectionError, WindowsTarget, WindowsTargetInfo};
use crate::taskbar::{
    TaskbarDiscoveryError, TaskbarRect, TaskbarSnapshot, TaskbarWindow, TaskbarWindowRole,
    MAX_TASKBAR_WINDOWS,
};

const ERROR_SUCCESS: u32 = 0;
const ERROR_FILE_NOT_FOUND: u32 = 2;
const IMAGE_FILE_MACHINE_UNKNOWN: u16 = 0;
const MAX_REGISTRY_VALUE_BYTES: u32 = 4096;
const WINDOWS_VERSION_KEY: &str = "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion";

struct RegistryKey(HKEY);

impl RegistryKey {
    fn open_current_version() -> Result<Self, TargetDetectionError> {
        let path: Vec<u16> = WINDOWS_VERSION_KEY.encode_utf16().chain(Some(0)).collect();
        let mut key: HKEY = ptr::null_mut();
        let status = unsafe {
            RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                path.as_ptr(),
                0,
                KEY_QUERY_VALUE | KEY_WOW64_64KEY,
                &mut key,
            )
        };
        if status != ERROR_SUCCESS {
            return Err(TargetDetectionError::RegistryOpenFailed(status));
        }
        Ok(Self(key))
    }
}

impl Drop for RegistryKey {
    fn drop(&mut self) {
        unsafe {
            RegCloseKey(self.0);
        }
    }
}

pub(super) fn detect_target_info() -> Result<WindowsTargetInfo, TargetDetectionError> {
    let key = RegistryKey::open_current_version()?;

    let build_text = query_string(key.0, "CurrentBuildNumber")?
        .or(query_string(key.0, "CurrentBuild")?)
        .ok_or(TargetDetectionError::MissingRegistryValue(
            "CurrentBuildNumber",
        ))?;
    let build_number = build_text
        .trim()
        .parse::<u32>()
        .map_err(|_| TargetDetectionError::InvalidRegistryValue("CurrentBuildNumber"))?;

    let display_version =
        query_string(key.0, "DisplayVersion")?.or(query_string(key.0, "ReleaseId")?);
    let product_name = query_string(key.0, "ProductName")?;
    let edition = query_string(key.0, "EditionID")?;
    let installation_type = query_string(key.0, "InstallationType")?;
    let update_build_revision = query_dword(key.0, "UBR")?;

    let current_process = unsafe { GetCurrentProcess() };
    let mut process_machine = IMAGE_FILE_MACHINE_UNKNOWN;
    let mut native_machine = IMAGE_FILE_MACHINE_UNKNOWN;
    let detected =
        unsafe { IsWow64Process2(current_process, &mut process_machine, &mut native_machine) };
    if detected == 0 {
        return Err(TargetDetectionError::NativeCallFailed(
            "IsWow64Process2",
            unsafe { GetLastError() },
        ));
    }

    let system_architecture = Architecture::from_image_machine(native_machine);
    let process_architecture =
        Architecture::from_image_machine(if process_machine == IMAGE_FILE_MACHINE_UNKNOWN {
            native_machine
        } else {
            process_machine
        });

    let process_id = unsafe { GetCurrentProcessId() };
    let mut session_id = 0u32;
    let session_detected = unsafe { ProcessIdToSessionId(process_id, &mut session_id) };
    if session_detected == 0 {
        return Err(TargetDetectionError::NativeCallFailed(
            "ProcessIdToSessionId",
            unsafe { GetLastError() },
        ));
    }

    Ok(WindowsTargetInfo {
        product_name,
        display_version,
        edition,
        installation_type,
        build_number,
        update_build_revision,
        process_architecture,
        system_architecture,
        process_id,
        session_id,
    })
}

fn query_string(key: HKEY, name: &'static str) -> Result<Option<String>, TargetDetectionError> {
    let wide_name: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
    let mut value_type = 0u32;
    let mut byte_count = 0u32;
    let status = unsafe {
        RegQueryValueExW(
            key,
            wide_name.as_ptr(),
            ptr::null(),
            &mut value_type,
            ptr::null_mut(),
            &mut byte_count,
        )
    };
    if status == ERROR_FILE_NOT_FOUND {
        return Ok(None);
    }
    if status != ERROR_SUCCESS {
        return Err(TargetDetectionError::RegistryQueryFailed(name, status));
    }
    if value_type != REG_SZ && value_type != REG_EXPAND_SZ {
        return Err(TargetDetectionError::InvalidRegistryValue(name));
    }
    if byte_count == 0 || byte_count > MAX_REGISTRY_VALUE_BYTES || byte_count % 2 != 0 {
        return Err(TargetDetectionError::InvalidRegistryValue(name));
    }

    let mut value = vec![0u16; (byte_count as usize) / size_of::<u16>()];
    let mut actual_bytes = byte_count;
    let mut actual_type = value_type;
    let status = unsafe {
        RegQueryValueExW(
            key,
            wide_name.as_ptr(),
            ptr::null(),
            &mut actual_type,
            value.as_mut_ptr().cast(),
            &mut actual_bytes,
        )
    };
    if status != ERROR_SUCCESS {
        return Err(TargetDetectionError::RegistryQueryFailed(name, status));
    }
    if (actual_type != REG_SZ && actual_type != REG_EXPAND_SZ)
        || actual_bytes > byte_count
        || actual_bytes % 2 != 0
    {
        return Err(TargetDetectionError::InvalidRegistryValue(name));
    }

    let actual_units = (actual_bytes as usize) / size_of::<u16>();
    value.truncate(actual_units);
    while value.last() == Some(&0) {
        value.pop();
    }
    let decoded =
        String::from_utf16(&value).map_err(|_| TargetDetectionError::InvalidRegistryValue(name))?;
    Ok(Some(decoded))
}

fn query_dword(key: HKEY, name: &'static str) -> Result<Option<u32>, TargetDetectionError> {
    let wide_name: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
    let mut value_type = 0u32;
    let mut value = 0u32;
    let mut byte_count = size_of::<u32>() as u32;
    let status = unsafe {
        RegQueryValueExW(
            key,
            wide_name.as_ptr(),
            ptr::null(),
            &mut value_type,
            (&mut value as *mut u32).cast(),
            &mut byte_count,
        )
    };
    if status == ERROR_FILE_NOT_FOUND {
        return Ok(None);
    }
    if status != ERROR_SUCCESS {
        return Err(TargetDetectionError::RegistryQueryFailed(name, status));
    }
    if value_type != REG_DWORD || byte_count != size_of::<u32>() as u32 {
        return Err(TargetDetectionError::InvalidRegistryValue(name));
    }
    Ok(Some(value))
}


const PRIMARY_TASKBAR_CLASS: &str = "Shell_TrayWnd";
const SECONDARY_TASKBAR_CLASS: &str = "Shell_SecondaryTrayWnd";

/// Enumerates only taskbar windows owned by the Explorer process already
/// verified by the BGW host monitor. The class names are build-specific Shell
/// implementation details; this adapter is intentionally locked to build 19045.
pub(super) fn discover_taskbar_windows(
    target: &WindowsTarget,
    expected_explorer_process_id: u32,
) -> Result<TaskbarSnapshot, TaskbarDiscoveryError> {
    let primary_class: Vec<u16> = PRIMARY_TASKBAR_CLASS
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let secondary_class: Vec<u16> = SECONDARY_TASKBAR_CLASS
        .encode_utf16()
        .chain(Some(0))
        .collect();

    let primary = unsafe { FindWindowW(primary_class.as_ptr(), ptr::null()) };
    if primary.is_null() {
        return Err(TaskbarDiscoveryError::PrimaryTaskbarNotFound);
    }

    let mut windows = Vec::with_capacity(MAX_TASKBAR_WINDOWS);
    windows.push(read_taskbar_window(
        primary,
        TaskbarWindowRole::Primary,
        expected_explorer_process_id,
    )?);

    let mut previous: HWND = ptr::null_mut();
    loop {
        let secondary = unsafe {
            FindWindowExW(
                ptr::null_mut(),
                previous,
                secondary_class.as_ptr(),
                ptr::null(),
            )
        };
        if secondary.is_null() {
            break;
        }
        if windows.len() >= MAX_TASKBAR_WINDOWS {
            return Err(TaskbarDiscoveryError::TooManyTaskbarWindows);
        }

        windows.push(read_taskbar_window(
            secondary,
            TaskbarWindowRole::Secondary,
            expected_explorer_process_id,
        )?);
        previous = secondary;
    }

    TaskbarSnapshot::from_native(target, expected_explorer_process_id, windows)
}

fn read_taskbar_window(
    hwnd: HWND,
    role: TaskbarWindowRole,
    expected_explorer_process_id: u32,
) -> Result<TaskbarWindow, TaskbarDiscoveryError> {
    let mut owner_process_id = 0u32;
    let thread_id = unsafe { GetWindowThreadProcessId(hwnd, &mut owner_process_id) };
    if thread_id == 0 || owner_process_id == 0 {
        return Err(TaskbarDiscoveryError::NativeCallFailed {
            function: "GetWindowThreadProcessId",
            code: unsafe { GetLastError() },
        });
    }
    if owner_process_id != expected_explorer_process_id {
        return Err(TaskbarDiscoveryError::ShellHostMismatch {
            expected_process_id: expected_explorer_process_id,
            observed_process_id: owner_process_id,
        });
    }

    let mut rect: RECT = unsafe { std::mem::zeroed() };
    if unsafe { GetWindowRect(hwnd, &mut rect) } == 0 {
        return Err(TaskbarDiscoveryError::NativeCallFailed {
            function: "GetWindowRect",
            code: unsafe { GetLastError() },
        });
    }

    let bounds = TaskbarRect {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    };
    if !bounds.is_valid() {
        return Err(TaskbarDiscoveryError::InvalidBounds);
    }

    Ok(TaskbarWindow {
        role,
        bounds,
        style_visible: unsafe { IsWindowVisible(hwnd) != 0 },
    })
}
