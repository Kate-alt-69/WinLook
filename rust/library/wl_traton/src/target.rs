//! Windows target identity and release matching.

use std::fmt;

/// The Windows product family detected for the current process.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WindowsFamily {
    Windows10,
    Windows11,
    WindowsServer,
}

/// CPU architecture reported for a process or the native operating system.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Architecture {
    X86,
    X64,
    Arm32,
    Arm64,
    Arm64Ec,
    Itanium,
    Unknown(u16),
}

impl Architecture {
    #[cfg(windows)]
    pub(crate) fn from_image_machine(machine: u16) -> Self {
        match machine {
            0x014c => Self::X86,
            0x8664 => Self::X64,
            0x01c0 | 0x01c4 => Self::Arm32,
            0xaa64 => Self::Arm64,
            0xa641 => Self::Arm64Ec,
            0x0200 => Self::Itanium,
            other => Self::Unknown(other),
        }
    }
}

impl fmt::Display for Architecture {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::X86 => formatter.write_str("x86"),
            Self::X64 => formatter.write_str("x64"),
            Self::Arm32 => formatter.write_str("arm32"),
            Self::Arm64 => formatter.write_str("arm64"),
            Self::Arm64Ec => formatter.write_str("arm64ec"),
            Self::Itanium => formatter.write_str("itanium"),
            Self::Unknown(value) => write!(formatter, "unknown-{value:04x}"),
        }
    }
}

/// All OS and process facts gathered before any native provider is selected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowsTargetInfo {
    pub product_name: Option<String>,
    pub display_version: Option<String>,
    pub edition: Option<String>,
    pub installation_type: Option<String>,
    pub build_number: u32,
    pub update_build_revision: Option<u32>,
    pub process_architecture: Architecture,
    pub system_architecture: Architecture,
    pub process_id: u32,
    pub session_id: u32,
}

/// Exact Windows target identity for one running WinLook process/session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowsTarget {
    family: WindowsFamily,
    release_selector: String,
    product_name: Option<String>,
    display_version: Option<String>,
    edition: Option<String>,
    installation_type: Option<String>,
    build_number: u32,
    update_build_revision: Option<u32>,
    process_architecture: Architecture,
    system_architecture: Architecture,
    process_id: u32,
    session_id: u32,
}

impl WindowsTarget {
    /// Detects the current Windows installation and the calling process context.
    ///
    /// Detection is read-only. Non-Windows hosts return UnsupportedPlatform.
    pub fn detect() -> Result<Self, TargetDetectionError> {
        #[cfg(windows)]
        {
            let info = super::windows::detect_target_info()?;
            Self::from_info(info)
        }
        #[cfg(not(windows))]
        {
            Err(TargetDetectionError::UnsupportedPlatform)
        }
    }

    /// Builds a validated target from platform facts. Exposed for deterministic tests.
    pub fn from_info(info: WindowsTargetInfo) -> Result<Self, TargetDetectionError> {
        if info.build_number < 16_299 {
            return Err(TargetDetectionError::UnsupportedBuild(info.build_number));
        }

        let is_server = info
            .product_name
            .as_deref()
            .is_some_and(|name| name.to_ascii_lowercase().contains("server"));

        let family = if is_server {
            WindowsFamily::WindowsServer
        } else if info.build_number >= 22_000 {
            WindowsFamily::Windows11
        } else {
            WindowsFamily::Windows10
        };

        let release_token = normalise_release_token(info.display_version.as_deref());
        let release_selector = match family {
            WindowsFamily::WindowsServer => {
                format!("10-server-build{}", info.build_number)
            }
            WindowsFamily::Windows10 => match release_token {
                Some(token) => format!("10-{token}"),
                None => format!("10-build{}", info.build_number),
            },
            WindowsFamily::Windows11 => match release_token {
                Some(token) => format!("11-{token}"),
                None => format!("11-build{}", info.build_number),
            },
        };

        Ok(Self {
            family,
            release_selector,
            product_name: info.product_name,
            display_version: info.display_version,
            edition: info.edition,
            installation_type: info.installation_type,
            build_number: info.build_number,
            update_build_revision: info.update_build_revision,
            process_architecture: info.process_architecture,
            system_architecture: info.system_architecture,
            process_id: info.process_id,
            session_id: info.session_id,
        })
    }

    pub fn family(&self) -> WindowsFamily {
        self.family
    }

    /// Canonical release selector, such as 10-22h2 or 11-24h2.
    pub fn release_selector(&self) -> &str {
        &self.release_selector
    }

    /// Returns true for the canonical selector or a family alias (10, 11, server).
    pub fn matches_selector(&self, selector: &str) -> bool {
        selector == self.release_selector
            || matches!(
                (selector, self.family),
                ("10", WindowsFamily::Windows10)
                    | ("11", WindowsFamily::Windows11)
                    | ("server", WindowsFamily::WindowsServer)
            )
    }

    pub fn product_name(&self) -> Option<&str> {
        self.product_name.as_deref()
    }

    pub fn display_version(&self) -> Option<&str> {
        self.display_version.as_deref()
    }

    pub fn edition(&self) -> Option<&str> {
        self.edition.as_deref()
    }

    pub fn installation_type(&self) -> Option<&str> {
        self.installation_type.as_deref()
    }

    pub fn build_number(&self) -> u32 {
        self.build_number
    }

    pub fn update_build_revision(&self) -> Option<u32> {
        self.update_build_revision
    }

    pub fn process_architecture(&self) -> Architecture {
        self.process_architecture
    }

    pub fn system_architecture(&self) -> Architecture {
        self.system_architecture
    }

    pub fn process_id(&self) -> u32 {
        self.process_id
    }

    pub fn session_id(&self) -> u32 {
        self.session_id
    }
}

fn normalise_release_token(value: Option<&str>) -> Option<String> {
    let value = value?.trim();
    if value.is_empty() {
        return None;
    }

    let token: String = value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect();

    if token.is_empty()
        || !token.as_bytes()[0].is_ascii_digit()
        || token.len() > 24
        || !token
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    {
        return None;
    }

    Some(token)
}

/// Failure while detecting or validating the current Windows target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetDetectionError {
    UnsupportedPlatform,
    UnsupportedBuild(u32),
    RegistryOpenFailed(u32),
    RegistryQueryFailed(&'static str, u32),
    MissingRegistryValue(&'static str),
    InvalidRegistryValue(&'static str),
    NativeCallFailed(&'static str, u32),
}

impl fmt::Display for TargetDetectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => {
                formatter.write_str("Windows target detection is only available on Windows")
            }
            Self::UnsupportedBuild(build) => write!(
                formatter,
                "Windows build {build} is below the Windows 10 version 1709 target-probe baseline"
            ),
            Self::RegistryOpenFailed(code) => write!(
                formatter,
                "could not open the read-only Windows version registry key (error {code})"
            ),
            Self::RegistryQueryFailed(name, code) => write!(
                formatter,
                "could not read Windows registry value {name} (error {code})"
            ),
            Self::MissingRegistryValue(name) => write!(
                formatter,
                "required Windows registry value {name} is missing"
            ),
            Self::InvalidRegistryValue(name) => write!(
                formatter,
                "Windows registry value {name} has an unexpected type or value"
            ),
            Self::NativeCallFailed(name, code) => write!(
                formatter,
                "{name} failed while detecting the current Windows target (error {code})"
            ),
        }
    }
}

impl std::error::Error for TargetDetectionError {}

#[cfg(test)]
mod tests {
    use super::{
        Architecture, TargetDetectionError, WindowsFamily, WindowsTarget, WindowsTargetInfo,
    };

    fn info(build: u32, display_version: &str) -> WindowsTargetInfo {
        WindowsTargetInfo {
            product_name: Some("Windows 10 Pro".to_owned()),
            display_version: Some(display_version.to_owned()),
            edition: Some("Professional".to_owned()),
            installation_type: Some("Client".to_owned()),
            build_number: build,
            update_build_revision: Some(7000),
            process_architecture: Architecture::X64,
            system_architecture: Architecture::X64,
            process_id: 100,
            session_id: 1,
        }
    }

    #[test]
    fn identifies_windows_10_22h2_without_discarding_ubr() {
        let target = WindowsTarget::from_info(info(19045, "22H2")).unwrap();
        assert_eq!(target.family(), WindowsFamily::Windows10);
        assert_eq!(target.release_selector(), "10-22h2");
        assert_eq!(target.update_build_revision(), Some(7000));
        assert!(target.matches_selector("10"));
        assert!(!target.matches_selector("11"));
    }

    #[test]
    fn identifies_windows_11_from_build_boundary() {
        let target = WindowsTarget::from_info(info(26100, "24H2")).unwrap();
        assert_eq!(target.family(), WindowsFamily::Windows11);
        assert_eq!(target.release_selector(), "11-24h2");
        assert!(target.matches_selector("11-24h2"));
    }

    #[test]
    fn does_not_guess_a_release_when_display_version_is_missing() {
        let target = WindowsTarget::from_info(info(19045, "")).unwrap();
        assert_eq!(target.release_selector(), "10-build19045");
    }

    #[test]
    fn classifies_server_separately() {
        let mut server = info(20348, "21H2");
        server.product_name = Some("Windows Server 2022 Datacenter".to_owned());
        let target = WindowsTarget::from_info(server).unwrap();
        assert_eq!(target.family(), WindowsFamily::WindowsServer);
        assert_eq!(target.release_selector(), "10-server-build20348");
    }

    #[test]
    fn rejects_builds_before_the_native_architecture_api_baseline() {
        assert_eq!(
            WindowsTarget::from_info(info(9600, "8.1")),
            Err(TargetDetectionError::UnsupportedBuild(9600))
        );
        assert_eq!(
            WindowsTarget::from_info(info(15_063, "1703")),
            Err(TargetDetectionError::UnsupportedBuild(15_063))
        );
    }
}
