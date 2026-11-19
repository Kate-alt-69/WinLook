//! Read-only, build-scoped discovery of native taskbar windows.

use std::fmt;

use crate::capability::CapabilityStatus;
use crate::provider::{ProviderDescriptor, ProviderError, ProviderKind, TranslationProvider};
use crate::registry::{ObjectOwnership, OperationAction, PropertyValue, ValidatedOperation};
use crate::target::{WindowsFamily, WindowsTarget};

pub const MAX_TASKBAR_WINDOWS: usize = 16;
pub const TASKBAR_WINDOW_COUNT_MEMBER: &str = "windows.count";
pub const TASKBAR_WINDOW_DISCOVERY_CAPABILITY_ID: &str = "shell.taskbar.window-discovery";
pub const TASKBAR_WINDOW_DISCOVERY_PROVIDER_ID: &str = "shell.taskbar.discovery.win10-22h2";
const WINDOWS_10_22H2_BUILD: u32 = 19_045;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskbarWindowRole {
    Primary,
    Secondary,
}

/// Screen-space bounds returned by GetWindowRect.
///
/// Windows virtualizes this rectangle according to the calling thread's DPI
/// awareness and it can include invisible resize borders.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskbarRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl TaskbarRect {
    pub fn is_valid(self) -> bool {
        self.right > self.left && self.bottom > self.top
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskbarWindow {
    pub role: TaskbarWindowRole,
    pub bounds: TaskbarRect,
    /// WS_VISIBLE style state, not a guarantee that the window is unobscured.
    pub style_visible: bool,
}

/// A point-in-time snapshot. Native HWNDs are intentionally not exposed or retained.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskbarSnapshot {
    release_selector: String,
    build_number: u32,
    session_id: u32,
    explorer_process_id: u32,
    windows: Vec<TaskbarWindow>,
}

impl TaskbarSnapshot {
    /// Discover the Windows 10 22H2 Explorer taskbar windows belonging to the
    /// Explorer process already verified by the BGW host monitor.
    ///
    /// The adapter is read-only and deliberately build-scoped because the
    /// Shell window class names are implementation details rather than a
    /// documented taskbar enumeration contract.
    pub fn discover(
        target: &WindowsTarget,
        expected_explorer_process_id: u32,
    ) -> Result<Self, TaskbarDiscoveryError> {
        Self::validate_target(target, expected_explorer_process_id)?;

        #[cfg(windows)]
        {
            super::windows::discover_taskbar_windows(target, expected_explorer_process_id)
        }
        #[cfg(not(windows))]
        {
            let _ = target;
            Err(TaskbarDiscoveryError::UnsupportedPlatform)
        }
    }

    fn validate_target(
        target: &WindowsTarget,
        expected_explorer_process_id: u32,
    ) -> Result<(), TaskbarDiscoveryError> {
        if target.family() != WindowsFamily::Windows10
            || target.build_number() != WINDOWS_10_22H2_BUILD
        {
            return Err(TaskbarDiscoveryError::UnsupportedTarget {
                build_number: target.build_number(),
            });
        }
        if expected_explorer_process_id == 0 {
            return Err(TaskbarDiscoveryError::InvalidExplorerProcessId);
        }
        Ok(())
    }

    #[cfg(windows)]
    pub(super) fn from_native(
        target: &WindowsTarget,
        explorer_process_id: u32,
        windows: Vec<TaskbarWindow>,
    ) -> Result<Self, TaskbarDiscoveryError> {
        if windows.len() > MAX_TASKBAR_WINDOWS {
            return Err(TaskbarDiscoveryError::TooManyTaskbarWindows);
        }
        if windows.first().map(|window| window.role) != Some(TaskbarWindowRole::Primary) {
            return Err(TaskbarDiscoveryError::PrimaryTaskbarNotFound);
        }
        if windows.iter().any(|window| !window.bounds.is_valid()) {
            return Err(TaskbarDiscoveryError::InvalidBounds);
        }

        Ok(Self {
            release_selector: target.release_selector().to_owned(),
            build_number: target.build_number(),
            session_id: target.session_id(),
            explorer_process_id,
            windows,
        })
    }

    pub fn release_selector(&self) -> &str {
        &self.release_selector
    }

    pub fn build_number(&self) -> u32 {
        self.build_number
    }

    pub fn session_id(&self) -> u32 {
        self.session_id
    }

    pub fn explorer_process_id(&self) -> u32 {
        self.explorer_process_id
    }

    pub fn windows(&self) -> &[TaskbarWindow] {
        &self.windows
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskbarDiscoveryError {
    UnsupportedPlatform,
    UnsupportedTarget {
        build_number: u32,
    },
    InvalidExplorerProcessId,
    PrimaryTaskbarNotFound,
    TooManyTaskbarWindows,
    ShellHostMismatch {
        expected_process_id: u32,
        observed_process_id: u32,
    },
    NativeCallFailed {
        function: &'static str,
        code: u32,
    },
    InvalidBounds,
}

impl fmt::Display for TaskbarDiscoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => {
                formatter.write_str("native taskbar discovery is available only on Windows")
            }
            Self::UnsupportedTarget { build_number } => write!(
                formatter,
                "taskbar discovery is only verified for Windows 10 22H2 build {WINDOWS_10_22H2_BUILD}; detected build {build_number}"
            ),
            Self::InvalidExplorerProcessId => {
                formatter.write_str("the verified Explorer process ID is invalid")
            }
            Self::PrimaryTaskbarNotFound => {
                formatter.write_str("the Windows 10 22H2 primary taskbar window was not found")
            }
            Self::TooManyTaskbarWindows => write!(
                formatter,
                "taskbar window count exceeds the Traton limit of {MAX_TASKBAR_WINDOWS}"
            ),
            Self::ShellHostMismatch {
                expected_process_id,
                observed_process_id,
            } => write!(
                formatter,
                "taskbar window belongs to process {observed_process_id}, not the verified Explorer process {expected_process_id}"
            ),
            Self::NativeCallFailed { function, code } => {
                write!(formatter, "{function} failed with Win32 error {code:#x}")
            }
            Self::InvalidBounds => {
                formatter.write_str("Windows returned invalid taskbar window bounds")
            }
        }
    }
}

impl std::error::Error for TaskbarDiscoveryError {}

/// Read-only provider for the Windows 10 22H2 Shell taskbar window model.
///
/// The Shell class names are implementation details; the provider is build-
/// scoped and reports partial coverage rather than a universal taskbar API.
#[derive(Clone, Debug)]
pub struct Win10_22H2TaskbarDiscoveryProvider {
    target: WindowsTarget,
    explorer_process_id: u32,
    descriptor: ProviderDescriptor,
}

impl Win10_22H2TaskbarDiscoveryProvider {
    pub fn new(target: WindowsTarget, explorer_process_id: u32) -> Self {
        Self {
            target,
            explorer_process_id,
            descriptor: Self::descriptor(),
        }
    }

    pub fn descriptor() -> ProviderDescriptor {
        ProviderDescriptor {
            id: TASKBAR_WINDOW_DISCOVERY_PROVIDER_ID.to_owned(),
            kind: ProviderKind::ExperimentalAdapter,
            ownership: ObjectOwnership::Windows,
            actions: [OperationAction::Read].into_iter().collect(),
            minimum_build: Some(WINDOWS_10_22H2_BUILD),
            maximum_build: Some(WINDOWS_10_22H2_BUILD),
            requires_consent: false,
        }
    }

    pub fn discover_snapshot(&self) -> Result<TaskbarSnapshot, TaskbarDiscoveryError> {
        TaskbarSnapshot::discover(&self.target, self.explorer_process_id)
    }
}

impl TranslationProvider for Win10_22H2TaskbarDiscoveryProvider {
    fn descriptor(&self) -> &ProviderDescriptor {
        &self.descriptor
    }

    fn probe(&self, target: &WindowsTarget) -> CapabilityStatus {
        if target.family() == WindowsFamily::Windows10
            && target.build_number() == WINDOWS_10_22H2_BUILD
        {
            CapabilityStatus::Partial
        } else {
            CapabilityStatus::Unsupported
        }
    }

    fn read(&self, operation: &ValidatedOperation) -> Result<PropertyValue, ProviderError> {
        if operation.action != OperationAction::Read
            || operation.capability_id != TASKBAR_WINDOW_DISCOVERY_CAPABILITY_ID
            || operation.path.subsystem() != "shell"
            || operation.path.object() != "taskbar"
            || operation.path.sub_properties().join(".") != TASKBAR_WINDOW_COUNT_MEMBER
            || operation.argument.is_some()
            || operation.target_build != self.target.build_number()
        {
            return Err(ProviderError::Unsupported);
        }

        let snapshot = self
            .discover_snapshot()
            .map_err(|error| ProviderError::Failed {
                code: "TRATON_TASKBAR_DISCOVERY".to_owned(),
                message: error.to_string(),
            })?;
        Ok(PropertyValue::Integer(snapshot.windows().len() as i64))
    }
}

#[cfg(test)]
mod tests {
    use super::{TaskbarDiscoveryError, TaskbarSnapshot, Win10_22H2TaskbarDiscoveryProvider};
    use crate::provider::ProviderKind;
    use crate::registry::OperationAction;
    use crate::target::{Architecture, WindowsTarget, WindowsTargetInfo};

    fn target(build_number: u32) -> WindowsTarget {
        WindowsTarget::from_info(WindowsTargetInfo {
            product_name: Some("Windows 10 Pro".to_owned()),
            display_version: Some("22H2".to_owned()),
            edition: Some("Professional".to_owned()),
            installation_type: Some("Client".to_owned()),
            build_number,
            update_build_revision: Some(1),
            process_architecture: Architecture::X64,
            system_architecture: Architecture::X64,
            process_id: 100,
            session_id: 1,
        })
        .unwrap()
    }

    #[test]
    fn validates_the_exact_supported_build_and_host_process() {
        assert_eq!(
            TaskbarSnapshot::validate_target(&target(19_045), 200),
            Ok(())
        );
        assert_eq!(
            TaskbarSnapshot::validate_target(&target(19_045), 0),
            Err(TaskbarDiscoveryError::InvalidExplorerProcessId)
        );
        assert_eq!(
            TaskbarSnapshot::validate_target(&target(19_044), 200),
            Err(TaskbarDiscoveryError::UnsupportedTarget {
                build_number: 19_044
            })
        );
    }

    #[test]
    fn rejects_windows_11_for_the_windows_10_shell_adapter() {
        let windows_11 = WindowsTarget::from_info(WindowsTargetInfo {
            product_name: Some("Windows 11 Pro".to_owned()),
            display_version: Some("24H2".to_owned()),
            edition: Some("Professional".to_owned()),
            installation_type: Some("Client".to_owned()),
            build_number: 26_100,
            update_build_revision: Some(1),
            process_architecture: Architecture::X64,
            system_architecture: Architecture::X64,
            process_id: 100,
            session_id: 1,
        })
        .unwrap();

        assert_eq!(
            TaskbarSnapshot::validate_target(&windows_11, 200),
            Err(TaskbarDiscoveryError::UnsupportedTarget {
                build_number: 26_100
            })
        );
    }

    #[test]
    fn provider_descriptor_is_read_only_and_build_scoped() {
        let descriptor = Win10_22H2TaskbarDiscoveryProvider::descriptor();
        assert_eq!(descriptor.kind, ProviderKind::ExperimentalAdapter);
        assert_eq!(descriptor.minimum_build, Some(19_045));
        assert_eq!(descriptor.maximum_build, Some(19_045));
        assert_eq!(
            descriptor.actions,
            [OperationAction::Read].into_iter().collect()
        );
    }
}
