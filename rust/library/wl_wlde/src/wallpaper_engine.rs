//! Safe hand-off adapter for an existing Wallpaper Engine installation.
//!
//! Steam remains responsible for Workshop subscriptions, purchases, downloads
//! and updates. This module discovers Wallpaper Engine's executable, validates
//! a user-selected local wallpaper path, then invokes the program's documented
//! command-line control using argument APIs (never a command shell).

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

const STEAM_APP_ID: &str = "431960";

/// A validated Wallpaper Engine executable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WallpaperEngineInstall {
    executable: PathBuf,
}

/// A local file accepted by Wallpaper Engine's documented openWallpaper command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WallpaperTarget {
    path: PathBuf,
    kind: WallpaperTargetKind,
}

/// Kind of local wallpaper source handed to Wallpaper Engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WallpaperTargetKind {
    SceneProject,
    Media,
}

/// Errors returned by the Wallpaper Engine bridge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WallpaperEngineError {
    ExecutableNotFound,
    InvalidExecutable,
    TargetNotFound,
    UnsupportedTarget,
    LaunchFailed(String),
}

impl std::fmt::Display for WallpaperEngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExecutableNotFound => f.write_str("Wallpaper Engine executable was not found"),
            Self::InvalidExecutable => {
                f.write_str("select wallpaper32.exe or wallpaper64.exe from Wallpaper Engine")
            }
            Self::TargetNotFound => f.write_str("the selected wallpaper file does not exist"),
            Self::UnsupportedTarget => {
                f.write_str("select a Wallpaper Engine project.json or supported media file")
            }
            Self::LaunchFailed(error) => write!(f, "Wallpaper Engine could not be started: {error}"),
        }
    }
}

impl std::error::Error for WallpaperEngineError {}

fn classify_target(path: &Path) -> Result<WallpaperTargetKind, WallpaperEngineError> {
    let file_name = path
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or_default()
        .to_ascii_lowercase();

    if file_name == "project.json" {
        return Ok(WallpaperTargetKind::SceneProject);
    }
    match path.extension().and_then(OsStr::to_str).unwrap_or_default().to_ascii_lowercase().as_str() {
        "mp4" | "webm" | "avi" | "mov" | "m4v" | "wmv" | "gif"
        | "png" | "jpg" | "jpeg" | "webp" | "bmp" => Ok(WallpaperTargetKind::Media),
        _ => Err(WallpaperEngineError::UnsupportedTarget),
    }
}

impl WallpaperTarget {
    /// Validates and canonicalizes a supported local Wallpaper Engine source.
    ///
    /// Scene source folders should be selected at their project.json file.
    /// Steam Workshop files remain untouched; WLDE does not unpack them.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, WallpaperEngineError> {
        let path = path.as_ref();
        if !path.is_file() {
            return Err(WallpaperEngineError::TargetNotFound);
        }
        let kind = classify_target(path)?;
        let canonical = path
            .canonicalize()
            .map_err(|error| WallpaperEngineError::LaunchFailed(error.to_string()))?;
        Ok(Self {
            path: canonical,
            kind,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn kind(&self) -> WallpaperTargetKind {
        self.kind
    }
}

impl WallpaperEngineInstall {
    /// Validates an executable selected by the user through the Browse dialog.
    pub fn from_executable(path: impl AsRef<Path>) -> Result<Self, WallpaperEngineError> {
        let path = path.as_ref();
        if !path.is_file() {
            return Err(WallpaperEngineError::ExecutableNotFound);
        }

        let file_name = path
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or_default()
            .to_ascii_lowercase();
        if file_name != "wallpaper32.exe" && file_name != "wallpaper64.exe" {
            return Err(WallpaperEngineError::InvalidExecutable);
        }
        let parent_name = path
            .parent()
            .and_then(Path::file_name)
            .and_then(OsStr::to_str)
            .unwrap_or_default();
        if !parent_name.eq_ignore_ascii_case("wallpaper_engine") {
            return Err(WallpaperEngineError::InvalidExecutable);
        }

        let executable = path
            .canonicalize()
            .map_err(|error| WallpaperEngineError::LaunchFailed(error.to_string()))?;
        Ok(Self { executable })
    }

    /// Searches common Steam locations and each library listed in libraryfolders.vdf.
    ///
    /// This is intentionally bounded: it checks known Steam library roots rather
    /// than recursively scanning disks. Use Browse when Steam is installed elsewhere.
    pub fn discover() -> Option<Self> {
        let roots = steam_roots();
        let mut libraries = roots.clone();

        for root in &roots {
            let vdf = root.join("steamapps").join("libraryfolders.vdf");
            if let Ok(contents) = fs::read_to_string(vdf) {
                libraries.extend(parse_library_paths(&contents));
            }
        }

        for library in unique_paths(libraries) {
            let install = library.join("steamapps").join("common").join("wallpaper_engine");
            for name in ["wallpaper64.exe", "wallpaper32.exe"] {
                let candidate = install.join(name);
                if let Ok(found) = Self::from_executable(candidate) {
                    return Some(found);
                }
            }
        }
        None
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }

    /// Returns Steam's Workshop content folder for Wallpaper Engine when its
    /// executable is in the standard steamapps/common/wallpaper_engine layout.
    pub fn workshop_directory(&self) -> Option<PathBuf> {
        let install = self.executable.parent()?;
        let common = install.parent()?;
        let steamapps = common.parent()?;
        Some(steamapps.join("workshop").join("content").join(STEAM_APP_ID))
    }

    /// Sends Wallpaper Engine its documented openWallpaper control command.
    ///
    /// Wallpaper Engine should already be running. Steam subscriptions/downloads
    /// are not initiated here. The optional monitor index is zero-based.
    pub fn open_wallpaper(
        &self,
        target: &WallpaperTarget,
        monitor_index: Option<u32>,
    ) -> Result<Child, WallpaperEngineError> {
        if !self.executable.is_file() {
            return Err(WallpaperEngineError::ExecutableNotFound);
        }
        if !target.path.is_file() {
            return Err(WallpaperEngineError::TargetNotFound);
        }

        let mut command = Command::new(&self.executable);
        command
            .arg("-control")
            .arg("openWallpaper")
            .arg("-file")
            .arg(&target.path);
        if let Some(index) = monitor_index {
            command.arg("-monitor").arg(index.to_string());
        }
        command
            .spawn()
            .map_err(|error| WallpaperEngineError::LaunchFailed(error.to_string()))
    }
}

fn steam_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for variable in ["STEAM_DIR", "STEAM_PATH"] {
        if let Some(value) = std::env::var_os(variable) {
            roots.push(PathBuf::from(value));
        }
    }
    for variable in ["ProgramFiles(x86)", "ProgramFiles", "LOCALAPPDATA"] {
        if let Some(value) = std::env::var_os(variable) {
            let base = PathBuf::from(value);
            roots.push(base.join("Steam"));
            if variable == "LOCALAPPDATA" {
                roots.push(base.join("Programs").join("Steam"));
            }
        }
    }
    roots
}

fn unique_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut unique = Vec::new();
    for path in paths {
        if !unique.iter().any(|existing: &PathBuf| {
            existing
                .to_string_lossy()
                .eq_ignore_ascii_case(&path.to_string_lossy())
        }) {
            unique.push(path);
        }
    }
    unique
}

fn parse_library_paths(contents: &str) -> Vec<PathBuf> {
    let tokens = tokenize_vdf(contents);
    let mut paths = Vec::new();

    // Current Steam VDF format stores each library as a numbered object with
    // a path key. Older formats stored the path directly as the numeric value.
    for (index, token) in tokens.iter().enumerate() {
        if token.eq_ignore_ascii_case("path") {
            if let Some(value) = tokens.get(index + 1) {
                if is_path_value(value) {
                    paths.push(PathBuf::from(value));
                }
            }
        }
    }
    for pair in tokens.windows(2) {
        if pair[0].parse::<u32>().is_ok() && is_path_value(&pair[1]) {
            paths.push(PathBuf::from(&pair[1]));
        }
    }
    paths
}

fn is_path_value(value: &str) -> bool {
    value.contains(":\\") || value.starts_with("\\\\") || value.starts_with('/')
}

#[allow(clippy::while_let_on_iterator)]
fn tokenize_vdf(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                let mut value = String::new();
                while let Some(ch) = chars.next() {
                    match ch {
                        '"' => break,
                        '\\' => match chars.next() {
                            Some('"') => value.push('"'),
                            Some('\\') => value.push('\\'),
                            Some(other) => {
                                value.push('\\');
                                value.push(other);
                            }
                            None => value.push('\\'),
                        },
                        other => value.push(other),
                    }
                }
                tokens.push(value);
            }
            '/' if chars.peek() == Some(&'/') => {
                for ch in chars.by_ref() {
                    if ch == '\n' {
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steam_vdf_library_paths_are_parsed_and_unescaped() {
        let input = r#"
            "libraryfolders"
            {
                "0" { "path" "C:\\Program Files (x86)\\Steam" }
                "1" { "path" "D:\\Games\\SteamLibrary" }
            }
        "#;
        let paths = parse_library_paths(input);
        assert!(paths.contains(&PathBuf::from(r"C:\Program Files (x86)\Steam")));
        assert!(paths.contains(&PathBuf::from(r"D:\Games\SteamLibrary")));
    }

    #[test]
    fn older_steam_vdf_numeric_path_values_are_parsed() {
        let input = r#""1" "E:\\SteamLibrary""#;
        assert_eq!(
            parse_library_paths(input),
            vec![PathBuf::from(r"E:\SteamLibrary")]
        );
    }

    #[test]
    fn target_kind_is_classified_without_touching_the_filesystem() {
        assert_eq!(
            classify_target(Path::new("project.json")),
            Ok(WallpaperTargetKind::SceneProject)
        );
        assert_eq!(
            classify_target(Path::new("loop.mp4")),
            Ok(WallpaperTargetKind::Media)
        );
        assert_eq!(
            classify_target(Path::new("index.html")),
            Err(WallpaperEngineError::UnsupportedTarget)
        );
        assert_eq!(
            classify_target(Path::new("scene.pkg")),
            Err(WallpaperEngineError::UnsupportedTarget)
        );
        assert_eq!(
            classify_target(Path::new("random.exe")),
            Err(WallpaperEngineError::UnsupportedTarget)
        );
    }

    #[test]
    fn executable_and_workshop_paths_follow_the_standard_steam_layout() {
        let root = std::env::temp_dir().join(format!(
            "winlook-wlde-wallpaper-engine-{}",
            std::process::id()
        ));
        let install_directory = root
            .join("steamapps")
            .join("common")
            .join("wallpaper_engine");
        fs::create_dir_all(&install_directory).unwrap();
        let executable_path = install_directory.join("wallpaper64.exe");
        fs::write(&executable_path, b"test fixture").unwrap();

        let install = WallpaperEngineInstall::from_executable(&executable_path).unwrap();
        assert_eq!(
            install.workshop_directory().unwrap(),
            root.join("steamapps")
                .join("workshop")
                .join("content")
                .join(STEAM_APP_ID)
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn local_target_requires_an_existing_supported_file() {
        let root = std::env::temp_dir().join(format!(
            "winlook-wlde-wallpaper-target-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();

        let project = root.join("project.json");
        fs::write(&project, b"{}").unwrap();
        let target = WallpaperTarget::new(&project).unwrap();
        assert_eq!(target.kind(), WallpaperTargetKind::SceneProject);
        assert!(target.path().is_absolute());

        let unsupported = root.join("readme.txt");
        fs::write(&unsupported, b"not a wallpaper").unwrap();
        assert_eq!(
            WallpaperTarget::new(&unsupported),
            Err(WallpaperEngineError::UnsupportedTarget)
        );

        fs::remove_dir_all(root).unwrap();
    }
}
