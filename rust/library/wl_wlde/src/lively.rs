//! Safe out-of-process integration with an existing Lively Wallpaper install.
//!
//! This adapter uses Lively's documented command-line interface. Lively must
//! already be running; the command utility is preferred over its main UI
//! executable for shorter command dispatch latency. WinLook does not embed
//! Lively, use its private named-pipe protocol, or launch arbitrary app
//! wallpapers through this adapter.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

/// Validated local media or project that Lively can apply.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LivelyTarget {
    path: PathBuf,
    kind: LivelyTargetKind,
}

/// Deliberately limited to projects and ordinary media/web assets. Executable
/// wallpapers are excluded from this first integration surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LivelyTargetKind {
    Project,
    Media,
    Web,
}

/// A selected Lively command-line executable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LivelyInstall {
    executable: PathBuf,
}

/// Scope used by Lively's documented closewp command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LivelyCloseTarget {
    Monitor(u32),
    All,
}

/// Errors returned by the Lively external-process adapter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LivelyIntegrationError {
    ExecutableNotFound,
    InvalidExecutable,
    TargetNotFound,
    MissingProjectManifest,
    UnsupportedTarget,
    InvalidPropertyName,
    InvalidPropertyValue,
    VolumeOutOfRange,
    LaunchFailed(String),
}

impl std::fmt::Display for LivelyIntegrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExecutableNotFound => f.write_str("Lively executable was not found"),
            Self::InvalidExecutable => f.write_str(
                "select lively.exe, livelywpf.exe, or the official livelycu.exe command utility",
            ),
            Self::TargetNotFound => f.write_str("the selected Lively wallpaper target does not exist"),
            Self::MissingProjectManifest => {
                f.write_str("select a Lively project folder containing LivelyInfo.json")
            }
            Self::UnsupportedTarget => f.write_str(
                "select a Lively project folder or supported media/web file; executable wallpapers are not enabled",
            ),
            Self::InvalidPropertyName => {
                f.write_str("Lively property names must use ASCII letters, digits, '.', '_' or '-'")
            }
            Self::InvalidPropertyValue => {
                f.write_str("Lively property value is empty, too long, or contains a line break")
            }
            Self::VolumeOutOfRange => f.write_str("Lively volume must be between 0 and 100"),
            Self::LaunchFailed(error) => write!(f, "Lively command could not be started: {error}"),
        }
    }
}

impl std::error::Error for LivelyIntegrationError {}

fn classify_target(path: &Path) -> Result<LivelyTargetKind, LivelyIntegrationError> {
    if path.is_dir() {
        return if path.join("LivelyInfo.json").is_file() {
            Ok(LivelyTargetKind::Project)
        } else {
            Err(LivelyIntegrationError::MissingProjectManifest)
        };
    }

    if !path.is_file() {
        return Err(LivelyIntegrationError::TargetNotFound);
    }

    let extension = path
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or_default()
        .to_ascii_lowercase();

    match extension.as_str() {
        "wmv" | "avi" | "flv" | "m4v" | "mkv" | "mov" | "mp4" | "mp4v" | "mpeg4" | "mpg"
        | "webm" | "ogm" | "ogv" | "ogx" | "jpg" | "jpeg" | "png" | "bmp" | "tif"
        | "tiff" | "webp" | "jfif" | "gif" => Ok(LivelyTargetKind::Media),
        "html" => Ok(LivelyTargetKind::Web),
        _ => Err(LivelyIntegrationError::UnsupportedTarget),
    }
}

impl LivelyTarget {
    /// Validates and canonicalizes a Lively project folder or supported file.
    ///
    /// Project folders must contain LivelyInfo.json. Executable app wallpapers
    /// are intentionally excluded; WinLook does not auto-launch arbitrary apps.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, LivelyIntegrationError> {
        let path = path.as_ref();
        let kind = classify_target(path)?;
        let canonical = path
            .canonicalize()
            .map_err(|_| LivelyIntegrationError::TargetNotFound)?;
        Ok(Self {
            path: canonical,
            kind,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn kind(&self) -> LivelyTargetKind {
        self.kind
    }
}

impl LivelyInstall {
    /// Validates a user-selected executable from an existing Lively installation.
    ///
    /// This is a path/name check, not a publisher-signature check. The Manager
    /// should clearly identify the selected executable and let the user choose
    /// a trusted installation location.
    pub fn from_executable(path: impl AsRef<Path>) -> Result<Self, LivelyIntegrationError> {
        let path = path.as_ref();
        if !path.is_file() {
            return Err(LivelyIntegrationError::ExecutableNotFound);
        }

        let file_name = path
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or_default();
        if !["lively.exe", "livelywpf.exe", "livelycu.exe"]
            .iter()
            .any(|candidate| file_name.eq_ignore_ascii_case(candidate))
        {
            return Err(LivelyIntegrationError::InvalidExecutable);
        }

        let executable = path
            .canonicalize()
            .map_err(|error| LivelyIntegrationError::LaunchFailed(error.to_string()))?;
        Ok(Self { executable })
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }

    /// Builds setwp --file <path> [--monitor <index>].
    pub fn command_set_wallpaper(
        &self,
        target: &LivelyTarget,
        monitor: Option<u32>,
    ) -> Command {
        let mut command = Command::new(&self.executable);
        command.arg("setwp").arg("--file").arg(&target.path);
        if let Some(index) = monitor {
            command.arg("--monitor").arg(index.to_string());
        }
        command
    }

    /// Applies a project/media target through Lively's documented CLI.
    ///
    /// Lively must already be running. This starts the selected Lively
    /// executable/command utility as a client; it does not start the wallpaper
    /// runtime if Lively is not active.
    pub fn set_wallpaper(
        &self,
        target: &LivelyTarget,
        monitor: Option<u32>,
    ) -> Result<Child, LivelyIntegrationError> {
        self.command_set_wallpaper(target, monitor)
            .spawn()
            .map_err(|error| LivelyIntegrationError::LaunchFailed(error.to_string()))
    }

    /// Builds closewp --monitor <index|-1>.
    pub fn command_close_wallpaper(&self, target: LivelyCloseTarget) -> Command {
        let monitor = match target {
            LivelyCloseTarget::Monitor(index) => index.to_string(),
            LivelyCloseTarget::All => "-1".to_owned(),
        };
        let mut command = Command::new(&self.executable);
        command.arg("closewp").arg("--monitor").arg(monitor);
        command
    }

    pub fn close_wallpaper(
        &self,
        target: LivelyCloseTarget,
    ) -> Result<Child, LivelyIntegrationError> {
        self.command_close_wallpaper(target)
            .spawn()
            .map_err(|error| LivelyIntegrationError::LaunchFailed(error.to_string()))
    }

    /// Builds app --play true|false.
    pub fn command_set_playback(&self, enabled: bool) -> Command {
        let mut command = Command::new(&self.executable);
        command
            .arg("app")
            .arg("--play")
            .arg(if enabled { "true" } else { "false" });
        command
    }

    pub fn set_playback(&self, enabled: bool) -> Result<Child, LivelyIntegrationError> {
        self.command_set_playback(enabled)
            .spawn()
            .map_err(|error| LivelyIntegrationError::LaunchFailed(error.to_string()))
    }

    /// Builds app --volume <0..100>.
    pub fn command_set_volume(&self, volume: u8) -> Result<Command, LivelyIntegrationError> {
        if volume > 100 {
            return Err(LivelyIntegrationError::VolumeOutOfRange);
        }
        let mut command = Command::new(&self.executable);
        command.arg("app").arg("--volume").arg(volume.to_string());
        Ok(command)
    }

    pub fn set_volume(&self, volume: u8) -> Result<Child, LivelyIntegrationError> {
        self.command_set_volume(volume)?
            .spawn()
            .map_err(|error| LivelyIntegrationError::LaunchFailed(error.to_string()))
    }

    /// Builds setprop --property <name=value> with a bounded, typed key.
    pub fn command_set_property(
        &self,
        name: &str,
        value: &str,
    ) -> Result<Command, LivelyIntegrationError> {
        if name.is_empty()
            || name.len() > 64
            || !name
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
        {
            return Err(LivelyIntegrationError::InvalidPropertyName);
        }
        if value.is_empty()
            || value.len() > 512
            || value.chars().any(|ch| matches!(ch, '\0' | '\r' | '\n'))
        {
            return Err(LivelyIntegrationError::InvalidPropertyValue);
        }

        let property = format!("{name}={value}");
        let mut command = Command::new(&self.executable);
        command.arg("setprop").arg("--property").arg(property);
        Ok(command)
    }

    pub fn set_property(&self, name: &str, value: &str) -> Result<Child, LivelyIntegrationError> {
        self.command_set_property(name, value)?
            .spawn()
            .map_err(|error| LivelyIntegrationError::LaunchFailed(error.to_string()))
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
    fn target_accepts_project_media_and_web_but_not_executable_wallpapers() {
        let root =
            std::env::temp_dir().join(format!("winlook-wlde-lively-target-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();

        let project = root.join("Project");
        fs::create_dir_all(&project).unwrap();
        fs::write(project.join("LivelyInfo.json"), b"{}").unwrap();
        assert_eq!(
            LivelyTarget::new(&project).unwrap().kind(),
            LivelyTargetKind::Project
        );

        let video = root.join("loop.MP4");
        fs::write(&video, b"fixture").unwrap();
        assert_eq!(
            LivelyTarget::new(&video).unwrap().kind(),
            LivelyTargetKind::Media
        );

        let page = root.join("index.html");
        fs::write(&page, b"<html></html>").unwrap();
        assert_eq!(
            LivelyTarget::new(&page).unwrap().kind(),
            LivelyTargetKind::Web
        );

        let executable = root.join("unknown.exe");
        fs::write(&executable, b"fixture").unwrap();
        assert_eq!(
            LivelyTarget::new(&executable),
            Err(LivelyIntegrationError::UnsupportedTarget)
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn project_folder_without_lively_manifest_is_rejected() {
        let root = std::env::temp_dir().join(format!(
            "winlook-wlde-lively-project-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        assert_eq!(
            LivelyTarget::new(&root),
            Err(LivelyIntegrationError::MissingProjectManifest)
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn selected_executable_name_is_validated() {
        let root =
            std::env::temp_dir().join(format!("winlook-wlde-lively-exe-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let accepted = root.join("LivelyCu.exe");
        fs::write(&accepted, b"fixture").unwrap();
        assert!(LivelyInstall::from_executable(&accepted).is_ok());

        let rejected = root.join("anything.exe");
        fs::write(&rejected, b"fixture").unwrap();
        assert_eq!(
            LivelyInstall::from_executable(&rejected),
            Err(LivelyIntegrationError::InvalidExecutable)
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn commands_keep_values_as_separate_arguments() {
        let install = LivelyInstall {
            executable: PathBuf::from("livelycu.exe"),
        };
        let target = LivelyTarget {
            path: PathBuf::from(r"C:\Wallpapers\My wallpaper"),
            kind: LivelyTargetKind::Media,
        };
        assert_eq!(
            arguments(&install.command_set_wallpaper(&target, Some(2))),
            vec![
                "setwp",
                "--file",
                r"C:\Wallpapers\My wallpaper",
                "--monitor",
                "2"
            ]
        );
        assert_eq!(
            arguments(&install.command_close_wallpaper(LivelyCloseTarget::All)),
            vec!["closewp", "--monitor", "-1"]
        );
        assert_eq!(
            arguments(&install.command_set_playback(false)),
            vec!["app", "--play", "false"]
        );
        assert_eq!(
            arguments(&install.command_set_volume(75).unwrap()),
            vec!["app", "--volume", "75"]
        );
    }

    #[test]
    fn property_arguments_are_validated_and_not_shell_concatenated() {
        let install = LivelyInstall {
            executable: PathBuf::from("livelycu.exe"),
        };
        let command = install.command_set_property("hue", "-20").unwrap();
        assert_eq!(
            arguments(&command),
            vec!["setprop", "--property", "hue=-20"]
        );
        assert_eq!(
            install.command_set_property("bad name", "1").err(),
            Some(LivelyIntegrationError::InvalidPropertyName)
        );
        assert_eq!(
            install.command_set_property("hue", "1\n!closewp").err(),
            Some(LivelyIntegrationError::InvalidPropertyValue)
        );
        assert_eq!(
            install.command_set_volume(101).err(),
            Some(LivelyIntegrationError::VolumeOutOfRange)
        );
    }
}
