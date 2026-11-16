//! Safe out-of-process integration with an existing Rainmeter installation.
//!
//! This adapter sends only documented Rainmeter command-line bangs. It does
//! not parse/execute skins, load Rainmeter plugin DLLs, extract .rmskin archives
//! or silently install packages. Rainmeter and its own SkinInstaller remain
//! responsible for the runtime and package installation UI.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

/// A validated Rainmeter configuration path (for example, illustro\Clock).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RainmeterConfig {
    value: String,
}

/// A validated .ini file name within a Rainmeter configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RainmeterIni {
    value: String,
}

/// A validated local .rmskin package. WinLook never unpacks it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RainmeterSkinPackage {
    path: PathBuf,
}

/// A selected Rainmeter installation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RainmeterInstall {
    executable: PathBuf,
}

/// Errors returned by the Rainmeter external-process adapter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RainmeterIntegrationError {
    ExecutableNotFound,
    InvalidExecutable,
    PackageNotFound,
    UnsupportedPackage,
    InvalidConfig,
    InvalidIniFile,
    SkinInstallerNotFound,
    LaunchFailed(String),
}

impl std::fmt::Display for RainmeterIntegrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExecutableNotFound => f.write_str("Rainmeter executable was not found"),
            Self::InvalidExecutable => {
                f.write_str("select Rainmeter.exe from an installed Rainmeter folder")
            }
            Self::PackageNotFound => f.write_str("the selected Rainmeter package does not exist"),
            Self::UnsupportedPackage => f.write_str("select a .rmskin package"),
            Self::InvalidConfig => {
                f.write_str("Rainmeter config path is empty or contains an unsafe component")
            }
            Self::InvalidIniFile => {
                f.write_str("Rainmeter skin file must be a simple .ini file name")
            }
            Self::SkinInstallerNotFound => {
                f.write_str("SkinInstaller.exe was not found beside Rainmeter.exe")
            }
            Self::LaunchFailed(error) => {
                write!(f, "Rainmeter command could not be started: {error}")
            }
        }
    }
}

impl std::error::Error for RainmeterIntegrationError {}

impl RainmeterConfig {
    /// Validates and normalizes a Rainmeter config path.
    ///
    /// Path traversal, empty path components, quotes and control characters are
    /// rejected before the value reaches Rainmeter's bang parser.
    pub fn new(value: impl AsRef<str>) -> Result<Self, RainmeterIntegrationError> {
        let value = value.as_ref();
        if value.is_empty() || value.len() > 512 || value.starts_with(['\\', '/']) {
            return Err(RainmeterIntegrationError::InvalidConfig);
        }

        let mut segments = Vec::new();
        for segment in value.split(['\\', '/']) {
            if segment.is_empty()
                || segment == "."
                || segment == ".."
                || segment.chars().any(|ch| {
                    matches!(
                        ch,
                        '\0' | '\r' | '\n' | '"' | ':' | '<' | '>' | '|' | '?' | '*'
                    )
                })
            {
                return Err(RainmeterIntegrationError::InvalidConfig);
            }
            segments.push(segment);
        }

        Ok(Self {
            value: segments.join("\\"),
        })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl RainmeterIni {
    /// Validates a simple .ini file name; directory selection is via the config.
    pub fn new(value: impl AsRef<str>) -> Result<Self, RainmeterIntegrationError> {
        let value = value.as_ref();
        if value.is_empty()
            || value.len() > 255
            || value.contains(['\\', '/'])
            || value.chars().any(|ch| {
                matches!(
                    ch,
                    '\0' | '\r' | '\n' | '"' | ':' | '<' | '>' | '|' | '?' | '*'
                )
            })
            || Path::new(value).file_name().and_then(OsStr::to_str) != Some(value)
            || !value.to_ascii_lowercase().ends_with(".ini")
        {
            return Err(RainmeterIntegrationError::InvalidIniFile);
        }

        Ok(Self {
            value: value.to_owned(),
        })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl RainmeterSkinPackage {
    /// Validates a selected .rmskin file without reading or extracting its contents.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, RainmeterIntegrationError> {
        let path = path.as_ref();
        if !path.is_file() {
            return Err(RainmeterIntegrationError::PackageNotFound);
        }
        let extension = path.extension().and_then(OsStr::to_str).unwrap_or_default();
        if !extension.eq_ignore_ascii_case("rmskin") {
            return Err(RainmeterIntegrationError::UnsupportedPackage);
        }
        let path = path
            .canonicalize()
            .map_err(|_| RainmeterIntegrationError::PackageNotFound)?;
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl RainmeterInstall {
    /// Validates a Rainmeter executable selected by the user through Browse.
    ///
    /// This checks the path and executable name only, not its digital signature.
    pub fn from_executable(path: impl AsRef<Path>) -> Result<Self, RainmeterIntegrationError> {
        let path = path.as_ref();
        if !path.is_file() {
            return Err(RainmeterIntegrationError::ExecutableNotFound);
        }
        let file_name = path.file_name().and_then(OsStr::to_str).unwrap_or_default();
        if !file_name.eq_ignore_ascii_case("Rainmeter.exe") {
            return Err(RainmeterIntegrationError::InvalidExecutable);
        }
        let executable = path
            .canonicalize()
            .map_err(|error| RainmeterIntegrationError::LaunchFailed(error.to_string()))?;
        Ok(Self { executable })
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }

    /// Builds the documented Rainmeter.exe !ActivateConfig <config> [file] call.
    pub fn command_activate_config(
        &self,
        config: &RainmeterConfig,
        ini: Option<&RainmeterIni>,
    ) -> Command {
        let mut command = Command::new(&self.executable);
        command.arg("!ActivateConfig").arg(config.as_str());
        if let Some(ini) = ini {
            command.arg(ini.as_str());
        }
        command
    }

    pub fn activate_config(
        &self,
        config: &RainmeterConfig,
        ini: Option<&RainmeterIni>,
    ) -> Result<Child, RainmeterIntegrationError> {
        self.command_activate_config(config, ini)
            .spawn()
            .map_err(|error| RainmeterIntegrationError::LaunchFailed(error.to_string()))
    }

    pub fn command_deactivate_config(&self, config: &RainmeterConfig) -> Command {
        let mut command = Command::new(&self.executable);
        command.arg("!DeactivateConfig").arg(config.as_str());
        command
    }

    pub fn deactivate_config(
        &self,
        config: &RainmeterConfig,
    ) -> Result<Child, RainmeterIntegrationError> {
        self.command_deactivate_config(config)
            .spawn()
            .map_err(|error| RainmeterIntegrationError::LaunchFailed(error.to_string()))
    }

    pub fn command_refresh_config(&self, config: &RainmeterConfig) -> Command {
        let mut command = Command::new(&self.executable);
        command.arg("!Refresh").arg(config.as_str());
        command
    }

    pub fn refresh_config(
        &self,
        config: &RainmeterConfig,
    ) -> Result<Child, RainmeterIntegrationError> {
        self.command_refresh_config(config)
            .spawn()
            .map_err(|error| RainmeterIntegrationError::LaunchFailed(error.to_string()))
    }

    pub fn command_refresh_all(&self) -> Command {
        let mut command = Command::new(&self.executable);
        command.arg("!RefreshApp");
        command
    }

    pub fn refresh_all(&self) -> Result<Child, RainmeterIntegrationError> {
        self.command_refresh_all()
            .spawn()
            .map_err(|error| RainmeterIntegrationError::LaunchFailed(error.to_string()))
    }

    /// Locates the normal SkinInstaller.exe companion for a portable/installed copy.
    ///
    /// The official workflow opens this GUI and asks the user to browse to the
    /// .rmskin file. The package path is therefore not passed as an undocumented
    /// command-line argument.
    pub fn skin_installer_executable(&self) -> Result<PathBuf, RainmeterIntegrationError> {
        let installer = self
            .executable
            .parent()
            .ok_or(RainmeterIntegrationError::SkinInstallerNotFound)?
            .join("SkinInstaller.exe");
        if installer.is_file() {
            Ok(installer)
        } else {
            Err(RainmeterIntegrationError::SkinInstallerNotFound)
        }
    }

    pub fn launch_skin_installer(&self) -> Result<Child, RainmeterIntegrationError> {
        Command::new(self.skin_installer_executable()?)
            .spawn()
            .map_err(|error| RainmeterIntegrationError::LaunchFailed(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn arguments(command: &Command) -> Vec<String> {
        command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn config_paths_are_normalized_and_traversal_is_rejected() {
        assert_eq!(
            RainmeterConfig::new("illustro/Clock").unwrap().as_str(),
            r"illustro\Clock"
        );
        for invalid in [
            "",
            r"..\Outside",
            r"illustro\..\Outside",
            r"\Absolute",
            r"C:\Absolute",
            "bad\"config",
        ] {
            assert_eq!(
                RainmeterConfig::new(invalid).err(),
                Some(RainmeterIntegrationError::InvalidConfig)
            );
        }
    }

    #[test]
    fn ini_file_must_be_a_simple_ini_name() {
        assert_eq!(
            RainmeterIni::new("Clock.ini").unwrap().as_str(),
            "Clock.ini"
        );
        for invalid in [
            "",
            "Clock.txt",
            r"..\Clock.ini",
            r"C:Clock.ini",
            "bad\"name.ini",
        ] {
            assert_eq!(
                RainmeterIni::new(invalid).err(),
                Some(RainmeterIntegrationError::InvalidIniFile)
            );
        }
    }

    #[test]
    fn commands_use_documented_bangs_and_separate_arguments() {
        let install = RainmeterInstall {
            executable: PathBuf::from("Rainmeter.exe"),
        };
        let config = RainmeterConfig::new("illustro\\Clock").unwrap();
        let ini = RainmeterIni::new("Clock.ini").unwrap();

        assert_eq!(
            arguments(&install.command_activate_config(&config, Some(&ini))),
            vec!["!ActivateConfig", r"illustro\Clock", "Clock.ini"]
        );
        assert_eq!(
            arguments(&install.command_deactivate_config(&config)),
            vec!["!DeactivateConfig", r"illustro\Clock"]
        );
        assert_eq!(
            arguments(&install.command_refresh_config(&config)),
            vec!["!Refresh", r"illustro\Clock"]
        );
        assert_eq!(
            arguments(&install.command_refresh_all()),
            vec!["!RefreshApp"]
        );
    }

    #[test]
    fn skin_package_is_validated_without_unpacking() {
        let root = std::env::temp_dir().join(format!(
            "winlook-wlde-rainmeter-package-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        let package = root.join("skin.rmskin");
        fs::write(&package, b"fixture").unwrap();
        assert_eq!(
            RainmeterSkinPackage::new(&package).unwrap().path(),
            package.canonicalize().unwrap()
        );
        let unsupported = root.join("skin.zip");
        fs::write(&unsupported, b"fixture").unwrap();
        assert_eq!(
            RainmeterSkinPackage::new(&unsupported).err(),
            Some(RainmeterIntegrationError::UnsupportedPackage)
        );
        fs::remove_dir_all(root).unwrap();
    }
}
