#!/usr/bin/env python3
"""WinLook's cross-platform build, test, packaging, and toolchain entry point.

Requires Python 3.9+. Uses only the Python standard library and invokes native
tools with argument arrays except for the fixed Visual Studio environment bootstrap.
Rust is installed into the repository build cache; optional Visual Studio Build Tools
and CMake setup use the Windows package manager when explicitly requested or needed.
"""
from __future__ import annotations

import argparse
import contextlib
import hashlib
import json
import os
import platform
import re
import shlex
import shutil
import subprocess
import tarfile
import sys
import tempfile
import time
import urllib.request
import zipfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Callable, Dict, Iterable, List, Optional, Sequence, Tuple

ROOT = Path(__file__).resolve().parent
PYTHON_MINIMUM = (3, 9)
WATCOM_RELEASE = "2026-11-01-Build"
WATCOM_ASSET_URL = (
    "https://github.com/open-watcom/open-watcom-v2/releases/download/"
    + WATCOM_RELEASE
    + "/open-watcom-2_0-c-linux-x64"
)
WATCOM_ASSET_SHA256 = (
    "9d9d823c2ee040a38484ad74ad4a594e904672184e4e8fff84b3bac7090c9f6c"
)
RUST_TOOLCHAIN = "1.95.0"
RUSTUP_DIST = "https://static.rust-lang.org/rustup/dist"
CMAKE_WINGET_ID = "Kitware.CMake"
VS_BUILD_TOOLS_WINGET_ID = "Microsoft.VisualStudio.2022.BuildTools"
VS_BUILD_TOOLS_BOOTSTRAPPER_URL = "https://aka.ms/vs/17/release/vs_buildtools.exe"
VS_REQUIRED_COMPONENT_IDS = (
    "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
    "Microsoft.VisualStudio.Component.Windows10SDK.19041",
)
ACTIONLINT_VERSION = "1.7.12"
ACTIONLINT_URL = (
    "https://github.com/rhysd/actionlint/releases/download/v"
    + ACTIONLINT_VERSION
    + "/actionlint_" + ACTIONLINT_VERSION + "_linux_amd64.tar.gz"
)
ACTIONLINT_SHA256 = (
    "8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8"
)
GITLEAKS_VERSION = "8.30.1"
GITLEAKS_URL = (
    "https://github.com/gitleaks/gitleaks/releases/download/v"
    + GITLEAKS_VERSION
    + "/gitleaks_" + GITLEAKS_VERSION + "_linux_x64.tar.gz"
)
GITLEAKS_SHA256 = (
    "551f6fc83ea457d62a0d98237cbad105af8d557003051f41f3e7ca7b3f2470eb"
)
APP_RUNTIME_URL = (
    "https://learn.microsoft.com/en-us/windows/apps/windows-app-sdk/downloads"
)
BGW_TARGET_NAMES = {"winlook.bg", "winlook-bg", "winlook_bg"}
WINDOWS_APPS = {
    "win10": ("winlook-win10", "build/rust/win10"),
    "win11": ("winlook-win11", "build/rust/win11"),
}

# Canonical selectors cover every OS page in plan/compatibility. A selector is
# intentionally available before its implementation: the build then gives a
# source-gate message instead of silently building a different Windows version.
WINDOWS_GENERATIONS = {
    "95": ("Windows 95", "win95", "watcom"),
    "98": ("Windows 98", "win98", "watcom"),
    "me": ("Windows Me", "winme", "watcom"),
    "nt4": ("Windows NT 4.0", "winnt4", "mingw"),
    "2000": ("Windows 2000", "win2000", "mingw"),
    "xp": ("Windows XP", "winxp", "mingw"),
    "vista": ("Windows Vista", "winvista", "mingw"),
    "7": ("Windows 7", "win7", "mingw"),
    "8": ("Windows 8", "win8", "mingw"),
    "8.1": ("Windows 8.1", "win81", "mingw"),
    "10": ("Windows 10", "win10", "msvc"),
    "11": ("Windows 11", "win11", "msvc"),
}
WINDOWS_BUILD_COMMANDS = tuple("windows." + version for version in WINDOWS_GENERATIONS)

ANSI_RESET = "\x1b[0m"
ANSI_BOLD = "\x1b[1m"
ANSI_CYAN = "\x1b[1;36m"
ANSI_GREEN = "\x1b[1;32m"
ANSI_YELLOW = "\x1b[1;33m"
ANSI_MAGENTA = "\x1b[1;35m"


def color_enabled(stream: Any = None) -> bool:
    """Use ANSI colors in terminals; respect the standard NO_COLOR opt-out."""
    if "NO_COLOR" in os.environ or os.environ.get("WINLOOK_NO_COLOR") == "1":
        return False
    forced = os.environ.get("FORCE_COLOR")
    if forced is not None:
        return forced not in {"", "0", "false", "False"}
    output = stream or sys.stdout
    try:
        return bool(output.isatty()) and os.environ.get("TERM", "") != "dumb"
    except (AttributeError, OSError):
        return False


def colorize_help(value: str) -> str:
    """Add restrained ANSI styling to argparse help without changing its text."""
    value = re.sub(r"(?m)^(usage:)", ANSI_BOLD + ANSI_CYAN + r"\1" + ANSI_RESET, value)
    value = re.sub(
        r"(?m)^(positional arguments|options|optional arguments|commands):",
        ANSI_BOLD + ANSI_CYAN + r"\1" + ANSI_RESET,
        value,
    )
    value = re.sub(
        r"(?<![\w-])(--[A-Za-z0-9][\w.-]*|-[A-Za-z])(?=[\s,=\]]|$)",
        lambda match: ANSI_GREEN + match.group(1) + ANSI_RESET,
        value,
    )
    value = re.sub(
        r"(?m)^(\s{2,})(windows\.[\w.]+)(\s{2,})",
        lambda match: match.group(1) + ANSI_MAGENTA + match.group(2) + ANSI_RESET + match.group(3),
        value,
    )
    return value


class ColorArgumentParser(argparse.ArgumentParser):
    """argparse parser with terminal-aware ANSI help styling."""

    def format_help(self) -> str:
        rendered = super().format_help()
        return colorize_help(rendered) if color_enabled() else rendered


def rust_paths(root: Path) -> Dict[str, Path]:
    cache = build_paths(root)["rust_cache"]
    return {
        "cargo_home": cache / "cargo-home",
        "rustup_home": cache / "rustup-home",
        "bootstrap": cache / "bootstrap",
    }


def rustup_host() -> str:
    machine = platform.machine().lower()
    if os.name == "nt":
        if machine in {"arm64", "aarch64"}:
            return "aarch64-pc-windows-msvc"
        return "x86_64-pc-windows-msvc"
    if sys.platform == "darwin":
        return "aarch64-apple-darwin" if machine in {"arm64", "aarch64"} else "x86_64-apple-darwin"
    if sys.platform.startswith("linux"):
        return "aarch64-unknown-linux-gnu" if machine in {"arm64", "aarch64"} else "x86_64-unknown-linux-gnu"
    raise BuildError("Automatic Rust setup is not supported on this operating system: " + sys.platform)


def rust_environment(root: Path) -> Dict[str, str]:
    paths = rust_paths(root)
    env = os.environ.copy()
    env["CARGO_HOME"] = str(paths["cargo_home"])
    env["RUSTUP_HOME"] = str(paths["rustup_home"])
    env["CARGO_TARGET_DIR"] = str(build_paths(root)["rust"] / "shared")
    env["RUSTUP_TOOLCHAIN"] = RUST_TOOLCHAIN
    bin_dir = paths["cargo_home"] / "bin"
    env["PATH"] = str(bin_dir) + os.pathsep + env.get("PATH", "")
    return env


def _download_verified(url: str, destination: Path, label: str) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    request = urllib.request.Request(url, headers={"User-Agent": "WinLook-build.py"})
    with urllib.request.urlopen(request, timeout=120) as response:
        payload = response.read()
    checksum_url = url + ".sha256"
    checksum_request = urllib.request.Request(checksum_url, headers={"User-Agent": "WinLook-build.py"})
    with urllib.request.urlopen(checksum_request, timeout=30) as response:
        checksum_text = response.read(4096).decode("ascii", errors="replace")
    expected_match = __import__("re").search(r"\b([0-9a-fA-F]{64})\b", checksum_text)
    if expected_match is None:
        raise BuildError("Could not read the official SHA-256 checksum for " + label + ".")
    expected = expected_match.group(1).lower()
    actual = hashlib.sha256(payload).hexdigest()
    if actual != expected:
        raise BuildError(label + " SHA-256 mismatch; refusing to execute the downloaded installer.")
    destination.write_bytes(payload)
    print("Verified " + label + " SHA-256: " + actual)


def _rustup_executable(root: Path) -> Path:
    suffix = ".exe" if os.name == "nt" else ""
    return rust_paths(root)["cargo_home"] / "bin" / ("rustup" + suffix)


def _probe_rustup_toolchain(rustup: str, toolchain: str, env: Optional[Dict[str, str]] = None) -> Tuple[bool, str]:
    """Check the requested compiler through a specific rustup installation."""
    rustc = run_subprocess(
        [rustup, "run", toolchain, "rustc", "--version"],
        text=True, capture_output=True, check=False, env=env,
    )
    output = (rustc.stdout or "").strip()
    if rustc.returncode != 0 or not output.startswith("rustc " + toolchain + " "):
        return False, output or (rustc.stderr or "").strip()
    cargo = run_subprocess(
        [rustup, "run", toolchain, "cargo", "--version"],
        text=True, capture_output=True, check=False, env=env,
    )
    if cargo.returncode != 0:
        return False, (cargo.stderr or cargo.stdout or "Cargo is not available for " + toolchain).strip()
    return True, output


def _probe_direct_rust(toolchain: str) -> Tuple[bool, str]:
    """Check a non-rustup compiler already supplied on PATH."""
    rustc = command_path("rustc")
    cargo = command_path("cargo")
    if not rustc or not cargo:
        return False, ""
    result = run_subprocess([rustc, "--version"], text=True, capture_output=True, check=False)
    output = (result.stdout or "").strip()
    if result.returncode == 0 and output.startswith("rustc " + toolchain + " "):
        return True, output
    return False, output


def _installed_rust_components(rustup: str, toolchain: str, env: Optional[Dict[str, str]] = None) -> set:
    """Return the installed rustfmt/clippy components for one toolchain."""
    result = run_subprocess(
        [rustup, "component", "list", "--installed", "--toolchain", toolchain],
        text=True, capture_output=True, check=False, env=env,
    )
    if result.returncode != 0:
        detail = (result.stderr or result.stdout or "rustup returned no diagnostic").strip()
        raise BuildError("Could not inspect Rust components for " + toolchain + ": " + detail[-2000:])
    installed = set()
    for line in (result.stdout or "").splitlines():
        entry = line.strip()
        for component in ("rustfmt", "clippy"):
            if entry == component or entry.startswith(component + "-"):
                installed.add(component)
    return installed


def _ensure_rust_support_components(
    runner: "Runner",
    rustup: str,
    env: Optional[Dict[str, str]] = None,
    apply: bool = True,
) -> None:
    """Ensure the selected compiler can run cargo fmt and cargo clippy."""
    required = ("rustfmt", "clippy")
    installed = _installed_rust_components(rustup, RUST_TOOLCHAIN, env)
    missing = [component for component in required if component not in installed]
    if not missing:
        return
    if not apply:
        print("Rust " + RUST_TOOLCHAIN + " is missing support components: " + ", ".join(missing) + ".")
        print("Plan: add only the missing components to this toolchain; the rustup default stays unchanged.")
        return
    runner.run(
        "Install missing Rust support components for " + RUST_TOOLCHAIN,
        [rustup, "component", "add", "--toolchain", RUST_TOOLCHAIN, *missing],
        env=env,
    )


def _select_global_rustup(runner: "Runner", rustup: str, install_if_missing: bool) -> bool:
    """Use the user's existing rustup home and preserve its default toolchain."""
    available, version = _probe_rustup_toolchain(rustup, RUST_TOOLCHAIN)
    if not available and install_if_missing:
        print("Rust " + RUST_TOOLCHAIN + " is not installed in the existing rustup home.")
        print("Installing it side-by-side; the configured default toolchain will not be changed.")
        runner.run(
            "Install Rust " + RUST_TOOLCHAIN + " through existing rustup",
            [rustup, "toolchain", "install", RUST_TOOLCHAIN, "--profile", "minimal",
             "--component", "rustfmt", "--component", "clippy"],
        )
        available, version = _probe_rustup_toolchain(rustup, RUST_TOOLCHAIN)
    if not available:
        return False
    runner.rust_env = {"RUSTUP_TOOLCHAIN": RUST_TOOLCHAIN}
    append_github_environment("RUSTUP_TOOLCHAIN", RUST_TOOLCHAIN)
    print("Using existing global Rust toolchain: " + version)
    print("Global rustup default is unchanged; WinLook selects " + RUST_TOOLCHAIN + " for its build commands.")
    return True


def _select_project_rustup(runner: "Runner", rustup: Path, install_if_missing: bool) -> bool:
    """Reuse a rustup installation already stored in this repository."""
    env = rust_environment(runner.root)
    available, version = _probe_rustup_toolchain(str(rustup), RUST_TOOLCHAIN, env)
    if not available and install_if_missing:
        runner.run(
            "Install project-local Rust " + RUST_TOOLCHAIN,
            [str(rustup), "toolchain", "install", RUST_TOOLCHAIN, "--profile", "minimal",
             "--component", "rustfmt", "--component", "clippy"],
            env=env,
        )
        available, version = _probe_rustup_toolchain(str(rustup), RUST_TOOLCHAIN, env)
    if not available:
        return False
    runner.rust_env = {
        "CARGO_HOME": str(rust_paths(runner.root)["cargo_home"]),
        "RUSTUP_HOME": str(rust_paths(runner.root)["rustup_home"]),
        "RUSTUP_TOOLCHAIN": RUST_TOOLCHAIN,
        "PATH": str(rust_paths(runner.root)["cargo_home"] / "bin") + os.pathsep + os.environ.get("PATH", ""),
    }
    append_github_environment("CARGO_HOME", runner.rust_env["CARGO_HOME"])
    append_github_environment("RUSTUP_HOME", runner.rust_env["RUSTUP_HOME"])
    append_github_environment("RUSTUP_TOOLCHAIN", RUST_TOOLCHAIN)
    append_github_path(str(rust_paths(runner.root)["cargo_home"] / "bin"))
    print("Using existing WinLook-local Rust toolchain: " + version)
    return True


def ensure_rust_toolchain(runner: "Runner", apply: bool = True) -> bool:
    """Select installed Rust 1.95 first; only bootstrap a new rustup when necessary.

    Preference order:
      1. A global rustup installation that already has 1.95.0.
      2. A direct rustc/cargo installation that is already 1.95.0.
      3. An existing project-local rustup installation.
      4. Install 1.95.0 with an existing global rustup (side-by-side).
      5. Bootstrap an isolated project-local rustup as a last resort.
    """
    global_rustup = command_path("rustup")
    if global_rustup and _select_global_rustup(runner, global_rustup, install_if_missing=False):
        _ensure_rust_support_components(runner, global_rustup, apply=apply)
        return True

    direct_ok, direct_version = _probe_direct_rust(RUST_TOOLCHAIN)
    if direct_ok:
        runner.rust_env = {}
        print("Using existing PATH Rust toolchain: " + direct_version)
        return True

    local_rustup = _rustup_executable(runner.root)
    if local_rustup.is_file() and _select_project_rustup(runner, local_rustup, install_if_missing=False):
        _ensure_rust_support_components(
            runner, str(local_rustup), env=rust_environment(runner.root), apply=apply
        )
        return True

    if not apply:
        print("Rust " + RUST_TOOLCHAIN + " is not currently usable.")
        if global_rustup:
            print("Plan: install " + RUST_TOOLCHAIN + " side-by-side with the existing global rustup; its default will remain unchanged.")
        else:
            print("Plan: bootstrap a project-local rustup under " + str(rust_paths(runner.root)["rustup_home"]))
        return False

    # When rustup is already installed, use its CLI to add a side-by-side toolchain.
    # This is much smaller and avoids downloading a second rustup installer.
    if global_rustup and _select_global_rustup(runner, global_rustup, install_if_missing=True):
        _ensure_rust_support_components(runner, global_rustup, apply=True)
        return True

    if local_rustup.is_file() and _select_project_rustup(runner, local_rustup, install_if_missing=True):
        _ensure_rust_support_components(
            runner, str(local_rustup), env=rust_environment(runner.root), apply=True
        )
        return True

    paths = rust_paths(runner.root)
    env = rust_environment(runner.root)
    installer_suffix = ".exe" if os.name == "nt" else ""
    installer = paths["bootstrap"] / ("rustup-init" + installer_suffix)
    if not local_rustup.is_file():
        host = rustup_host()
        url = RUSTUP_DIST + "/" + host + "/rustup-init" + installer_suffix
        print("No usable Rust 1.95.0 or existing rustup installation was found.")
        print("Last resort: downloading the official rustup installer for " + host + "...")
        _download_verified(url, installer, "rustup-init")
        if os.name != "nt":
            installer.chmod(installer.stat().st_mode | 0o111)
        runner.run(
            "Install project-local rustup",
            [str(installer), "-y", "--no-modify-path", "--default-toolchain", "none", "--profile", "minimal"],
            env=env,
        )
    if not local_rustup.is_file():
        raise BuildError("rustup installer completed but the project-local rustup executable was not created at " + str(local_rustup))
    if not _select_project_rustup(runner, local_rustup, install_if_missing=True):
        raise BuildError("Could not install or verify project-local Rust " + RUST_TOOLCHAIN + ".")
    version_bin = paths["cargo_home"] / "bin" / ("rustc.exe" if os.name == "nt" else "rustc")
    version = run_subprocess(
        [str(version_bin), "--version"], cwd=str(runner.root),
        env=runner.rust_env, text=True, capture_output=True, check=False,
    )
    if version.returncode != 0 or not (version.stdout or "").startswith("rustc " + RUST_TOOLCHAIN + " "):
        raise BuildError("Project-local Rust compiler verification failed: " + (version.stderr or version.stdout or "unknown error"))
    return True


def _vswhere_path() -> Optional[Path]:
    candidates = []
    program_files_x86 = os.environ.get("ProgramFiles(x86)")
    if program_files_x86:
        candidates.append(Path(program_files_x86) / "Microsoft Visual Studio" / "Installer" / "vswhere.exe")
    program_files = os.environ.get("ProgramFiles")
    if program_files:
        candidates.append(Path(program_files) / "Microsoft Visual Studio" / "Installer" / "vswhere.exe")
    candidates.extend([
        Path("C:/Program Files (x86)/Microsoft Visual Studio/Installer/vswhere.exe"),
        Path("C:/Program Files/Microsoft Visual Studio/Installer/vswhere.exe"),
    ])
    found = command_path("vswhere")
    if found:
        candidates.append(Path(found))
    return next((path for path in candidates if path.is_file()), None)


def _visual_studio_installations() -> List[Path]:
    """Find registered VS/Build Tools instances, including partial installs."""
    installations: List[Path] = []
    vswhere = _vswhere_path()
    if vswhere is not None:
        result = run_subprocess(
            [str(vswhere), "-all", "-products", "*", "-property", "installationPath"],
            text=True, capture_output=True, check=False,
        )
        if result.returncode == 0:
            for line in (result.stdout or "").splitlines():
                candidate = Path(line.strip().lstrip("\ufeff"))
                if candidate.is_dir() and candidate not in installations:
                    installations.append(candidate)

    try:
        import winreg
        registry_path = r"SOFTWARE\Microsoft\VisualStudio\Setup\Instances"
        for view in (winreg.KEY_WOW64_64KEY, winreg.KEY_WOW64_32KEY):
            try:
                with winreg.OpenKey(
                    winreg.HKEY_LOCAL_MACHINE, registry_path, 0, winreg.KEY_READ | view
                ) as root_key:
                    for index in range(winreg.QueryInfoKey(root_key)[0]):
                        try:
                            with winreg.OpenKey(root_key, winreg.EnumKey(root_key, index)) as instance:
                                value, _ = winreg.QueryValueEx(instance, "installationPath")
                                candidate = Path(value)
                                if candidate.is_dir() and candidate not in installations:
                                    installations.append(candidate)
                        except OSError:
                            continue
            except OSError:
                continue
    except ImportError:
        pass

    # Registry/vswhere are authoritative; conventional roots are a recovery
    # path for older or damaged Visual Studio Installer metadata.
    roots = [
        Path(os.environ.get("ProgramFiles", "C:/Program Files")),
        Path(os.environ.get("ProgramFiles(x86)", "C:/Program Files (x86)")),
    ]
    for root in roots:
        for year in ("2022", "2019"):
            for edition in ("Enterprise", "Professional", "Community", "BuildTools"):
                candidate = root / "Microsoft Visual Studio" / year / edition
                if candidate.is_dir() and candidate not in installations:
                    installations.append(candidate)
    return installations



def _windows_installation_path_key(value: Any) -> str:
    """Normalize a Windows installation path for case-insensitive comparison."""
    return str(value).replace("/", "\\").rstrip("\\").casefold()


def _visual_studio_channel_id(installation: Path) -> str:
    """Find the selected VS instance's exact channel ID; never guess for modify."""
    vswhere = _vswhere_path()
    if vswhere is None:
        raise BuildError(
            "vswhere.exe is required to read the Visual Studio channel ID for "
            + str(installation) + ". Restore the Visual Studio Installer metadata "
            "or repair the instance through Visual Studio Installer."
        )
    try:
        result = run_subprocess(
            [str(vswhere), "-all", "-products", "*", "-format", "json"],
            text=True, capture_output=True, check=False,
        )
    except OSError as exc:
        raise BuildError("Could not run vswhere.exe to identify the Visual Studio channel: " + str(exc)) from exc
    if result.returncode != 0:
        detail = (result.stderr or result.stdout or "vswhere returned no diagnostic").strip()
        raise BuildError("vswhere.exe could not enumerate Visual Studio channels: " + detail[-2000:])
    try:
        instances = json.loads(result.stdout or "[]")
    except (TypeError, ValueError) as exc:
        raise BuildError("vswhere.exe returned invalid JSON while reading Visual Studio channel metadata.") from exc
    if isinstance(instances, dict):
        instances = [instances]
    if not isinstance(instances, list):
        raise BuildError("vswhere.exe returned an unexpected Visual Studio instance list.")
    requested_path = _windows_installation_path_key(installation)
    for instance in instances:
        if not isinstance(instance, dict):
            continue
        candidate = instance.get("installationPath")
        if not candidate or _windows_installation_path_key(candidate) != requested_path:
            continue
        channel_id = instance.get("channelId") or instance.get("installedChannelId")
        if channel_id:
            return str(channel_id)
        raise BuildError(
            "vswhere.exe found the Visual Studio instance at " + str(installation)
            + " but did not report its channelId. Repair the Visual Studio Installer metadata "
            "before retrying the build."
        )
    raise BuildError(
        "vswhere.exe did not return the selected Visual Studio instance at "
        + str(installation) + ". Refusing to guess a channel ID for setup.exe modify."
    )


def _visual_studio_workload_arguments() -> List[str]:
    """Explicitly request the compiler and SDK required by the current WinUI app."""
    arguments = ["--add", "Microsoft.VisualStudio.Workload.VCTools"]
    for component_id in VS_REQUIRED_COMPONENT_IDS:
        arguments.extend(["--add", component_id])
    return arguments


def _vs_installer_path() -> Optional[Path]:
    candidates = []
    for variable, fallback in (
        ("ProgramFiles(x86)", "C:/Program Files (x86)"),
        ("ProgramFiles", "C:/Program Files"),
    ):
        root = Path(os.environ.get(variable, fallback))
        installer_dir = root / "Microsoft Visual Studio" / "Installer"
        # Microsoft documents setup.exe for modify/update operations. Keep
        # vs_installer.exe as a compatibility fallback for unusual installs.
        candidates.append(installer_dir / "setup.exe")
        candidates.append(installer_dir / "vs_installer.exe")
    found = command_path("setup") or command_path("vs_installer")
    if found:
        candidates.append(Path(found))
    return next((path for path in candidates if path.is_file()), None)


def _windows_sdk_registry_roots() -> List[Path]:
    """Read the registered Windows 10/11 Kits root when registry access is available."""
    try:
        import winreg
    except ImportError:
        return []

    roots: List[Path] = []
    registry_path = r"SOFTWARE\Microsoft\Windows Kits\Installed Roots"
    views = [
        getattr(winreg, "KEY_WOW64_64KEY", 0),
        getattr(winreg, "KEY_WOW64_32KEY", 0),
    ]
    for view in views:
        try:
            with winreg.OpenKey(
                winreg.HKEY_LOCAL_MACHINE,
                registry_path,
                0,
                winreg.KEY_READ | view,
            ) as key:
                value, _ = winreg.QueryValueEx(key, "KitsRoot10")
            if value:
                roots.append(Path(str(value).strip().strip('"')))
        except OSError:
            continue
    return roots


def _windows_sdk_root_candidates(environment: Dict[str, str]) -> List[Path]:
    """Find Windows Kits roots from the active environment, registry and standard paths."""
    normalized = {str(key).casefold(): value for key, value in environment.items()}
    values = [
        normalized.get("windowssdkdir"),
        normalized.get("windowssdkroot"),
        normalized.get("kitsroot10"),
    ]
    candidates: List[Path] = []
    for value in values:
        if not value:
            continue
        candidate = Path(str(value).strip().strip('"'))
        if candidate.name.casefold() in {"include", "lib"}:
            candidate = candidate.parent
        candidates.append(candidate)

    candidates.extend(_windows_sdk_registry_roots())

    program_files_x86 = normalized.get("programfiles(x86)") or "C:/Program Files (x86)"
    program_files = normalized.get("programfiles") or "C:/Program Files"
    candidates.extend([
        Path(program_files_x86) / "Windows Kits" / "10",
        Path(program_files) / "Windows Kits" / "10",
    ])

    unique: List[Path] = []
    seen = set()
    for candidate in candidates:
        key = str(candidate).replace("\\", "/").rstrip("/").casefold()
        if key and key not in seen:
            seen.add(key)
            unique.append(candidate)
    return unique


def _windows_sdk_installation(environment: Dict[str, str]) -> Optional[Tuple[Path, str]]:
    """Return a Windows Kits root/version pair with the headers and x64 import libraries."""
    normalized = {str(key).casefold(): value for key, value in environment.items()}
    requested_version = str(normalized.get("windowssdkversion") or "").strip().strip('"')
    requested_version = requested_version.replace("\\", "/").rstrip("/").split("/")[-1]
    version_pattern = re.compile(r"^\d+(?:\.\d+){3}$")

    for root in _windows_sdk_root_candidates(environment):
        include_root = root / "Include"
        lib_root = root / "Lib"
        versions: List[str] = []
        if requested_version and version_pattern.fullmatch(requested_version):
            versions.append(requested_version)
        try:
            available = [
                path.name for path in include_root.iterdir()
                if path.is_dir() and version_pattern.fullmatch(path.name)
            ]
        except OSError:
            available = []

        def version_key(value: str) -> Tuple[int, ...]:
            try:
                return tuple(int(part) for part in value.split("."))
            except ValueError:
                return (0,)

        for version in sorted(available, key=version_key, reverse=True):
            if version not in versions:
                versions.append(version)

        for version in versions:
            if (
                (include_root / version / "um" / "Windows.h").is_file()
                and (lib_root / version / "um" / "x64" / "kernel32.lib").is_file()
            ):
                return root, version
    return None


def _windows_sdk_is_present(environment: Dict[str, str]) -> bool:
    return _windows_sdk_installation(environment) is not None


def _complete_msvc_environment(
    environment: Dict[str, str],
) -> Tuple[Dict[str, str], Optional[Tuple[Path, str]]]:
    """Fill SDK variables and search paths from the discovered SDK without changing global settings."""
    completed = dict(environment)
    sdk = _windows_sdk_installation(completed)
    if sdk is None:
        return completed, None

    root, version = sdk
    completed["WindowsSdkDir"] = str(root).rstrip("\\/") + os.sep
    completed["WindowsSDKVersion"] = version + os.sep

    include_root = root / "Include" / version
    lib_root = root / "Lib" / version
    include_dirs = [
        include_root / name for name in ("ucrt", "shared", "um", "winrt", "cppwinrt")
    ]
    lib_dirs = [
        lib_root / "ucrt" / "x64",
        lib_root / "um" / "x64",
    ]
    for key, directories in (("INCLUDE", include_dirs), ("LIB", lib_dirs)):
        existing = completed.get(key, "")
        values = [item for item in existing.split(os.pathsep) if item]
        seen = {item.rstrip("\\/").casefold() for item in values}
        for directory in directories:
            if directory.is_dir():
                value = str(directory)
                normalized = value.rstrip("\\/").casefold()
                if normalized not in seen:
                    values.append(value)
                    seen.add(normalized)
        if values:
            completed[key] = os.pathsep.join(values)

    sdk_bin = root / "bin" / version / "x64"
    if not (sdk_bin / "rc.exe").is_file():
        sdk_bin = root / "bin" / "x64"
    if (sdk_bin / "rc.exe").is_file():
        path_value = completed.get("PATH", "")
        path_entries = [item for item in path_value.split(os.pathsep) if item]
        if str(sdk_bin).rstrip("\\/").casefold() not in {
            item.rstrip("\\/").casefold() for item in path_entries
        }:
            path_entries.insert(0, str(sdk_bin))
            completed["PATH"] = os.pathsep.join(path_entries)
    return completed, sdk


def _msvc_environment_issues(environment: Dict[str, str]) -> Tuple[List[str], Dict[str, str]]:
    """Explain exactly which MSVC/SDK prerequisites are absent from a developer environment."""
    completed, sdk = _complete_msvc_environment(environment)
    normalized = {str(key).casefold(): value for key, value in completed.items()}
    search_path = normalized.get("path", "")
    issues = [
        name + ".exe was not found on the Visual Studio developer PATH"
        for name in ("cl", "link", "rc")
        if shutil.which(name + ".exe", path=search_path) is None
    ]
    if sdk is None:
        roots = _windows_sdk_root_candidates(environment)
        existing = [str(root) for root in roots if root.is_dir()]
        if existing:
            issues.append(
                "Windows SDK files were not complete under: " + ", ".join(existing)
                + " (expected Include/<version>/um/Windows.h and "
                "Lib/<version>/um/x64/kernel32.lib)"
            )
        else:
            issues.append(
                "Windows SDK root was not found from WindowsSdkDir, the Windows Kits "
                "registry entry, or the standard Windows Kits installation paths"
            )
    return issues, completed


def _msvc_environment_tools(environment: Dict[str, str]) -> bool:
    issues, _ = _msvc_environment_issues(environment)
    return not issues



def load_msvc_environment(runner: "Runner") -> bool:
    """Load an existing MSVC compiler, linker and Windows SDK without reinstalling."""
    if os.name != "nt":
        return False

    current = dict(os.environ)
    current_issues, current_environment = _msvc_environment_issues(current)
    if not current_issues:
        os.environ.update(current_environment)
        runner.msvc_diagnostics = []
        return True

    last_issues = current_issues
    for installation in _visual_studio_installations():
        scripts = (
            installation / "Common7" / "Tools" / "VsDevCmd.bat",
            installation / "VC" / "Auxiliary" / "Build" / "vcvars64.bat",
        )
        for script in scripts:
            if not script.is_file():
                continue
            arguments = " -no_logo -arch=x64 -host_arch=x64" if script.name.lower() == "vsdevcmd.bat" else ""
            command = 'call "' + str(script) + '"' + arguments + " && set"
            try:
                result = run_subprocess(
                    ["cmd.exe", "/d", "/c", command],
                    text=True, capture_output=True, check=False,
                )
            except OSError as exc:
                last_issues = ["Could not run " + str(script) + ": " + str(exc)]
                continue
            if result.returncode != 0:
                detail = (result.stderr or result.stdout or "").strip()
                last_issues = [
                    str(script) + " exited with code " + str(result.returncode)
                    + (": " + detail[-1200:] if detail else "")
                ]
                continue
            environment = dict(os.environ)
            for line in (result.stdout or "").splitlines():
                if "=" in line:
                    key, value = line.split("=", 1)
                    if key:
                        environment[key] = value
            issues, completed_environment = _msvc_environment_issues(environment)
            if not issues:
                # Do not contaminate this process with a partial environment.
                os.environ.update(completed_environment)
                runner.msvc_diagnostics = []
                print("Using existing Visual Studio C++ Build Tools at " + str(installation))
                return True
            last_issues = [str(script) + ": " + issue for issue in issues]
            if runner.debug:
                runner.debug_log("[MSVC] " + "; ".join(last_issues))

    runner.msvc_diagnostics = last_issues
    return False


def probe_mingw_gcc() -> Optional[Dict[str, str]]:
    """Return a usable native MinGW GCC/CMake toolchain, not just any gcc.exe."""
    gcc = command_path("gcc")
    if not gcc:
        return None
    try:
        result = run_subprocess([gcc, "-dumpmachine"], text=True, capture_output=True, check=False)
    except OSError:
        return None
    target = (result.stdout or "").strip()
    if result.returncode != 0 or "mingw" not in target.lower():
        return None

    ninja = command_path("ninja")
    mingw_make = command_path("mingw32-make")
    msys_make = command_path("make")
    if ninja:
        generator, make_program = "Ninja", ninja
    elif mingw_make:
        generator, make_program = "MinGW Makefiles", mingw_make
    elif msys_make:
        generator, make_program = "MSYS Makefiles", msys_make
    else:
        generator, make_program = "", ""

    return {
        "gcc": gcc,
        "target": target,
        "generator": generator,
        "make_program": make_program,
        "g++": command_path("g++") or "",
    }


def _select_mingw_cmake(runner: "Runner", required: bool = False) -> bool:
    info = probe_mingw_gcc()
    if info is None:
        return False
    if not info["generator"]:
        if required:
            raise BuildError(
                "MinGW GCC was found (" + info["target"] + "), but CMake needs Ninja, "
                "mingw32-make, or MSYS make on PATH. Install one of those build tools."
            )
        return False
    runner.c_compiler = "mingw"
    runner.tool_env = {
        "CC": info["gcc"],
        "CMAKE_GENERATOR": info["generator"],
        "CMAKE_MAKE_PROGRAM": info["make_program"],
    }
    print("Using installed MinGW GCC: " + info["gcc"] + " (" + info["target"] + ")")
    print("CMake generator: " + info["generator"])
    return True


def visual_studio_exit_reason(code: int) -> str:
    """Translate common Visual Studio setup process exit codes."""
    return {
        0: "Operation completed successfully.",
        740: "Elevation is required.",
        1001: "A Visual Studio Installer operation is already running.",
        1003: "Visual Studio is currently in use.",
        1602: "The operation was cancelled.",
        1618: "Another installation is already running.",
        3010: "Installation completed, but Windows must be restarted before the tools can be used.",
        5003: "The bootstrapper could not download the installer.",
        5004: "The operation was cancelled.",
        5005: "The installer rejected its command-line arguments.",
        5007: "The operation was blocked because the machine does not meet an installer requirement.",
        8001: "The installer rejected this ARM machine configuration.",
        8002: "The background-download precheck failed.",
        8003: "One or more selected components are out of support.",
        8004: "The installation target directory was rejected.",
        8005: "The installer could not verify one or more source payloads.",
        8006: "Visual Studio processes are running and block the change.",
        8010: "This Windows version is not supported by the selected Visual Studio workload.",
    }.get(code, "Visual Studio Installer reported an unspecified failure; inspect its setup logs.")



class VisualStudioSetupLogTailer:
    """Read only newly appended lines from Visual Studio's current setup logs."""

    PATTERNS = ("dd_setup_*.log", "dd_client_*.log", "dd_bootstrapper_*.log")

    def __init__(self, started_at: float) -> None:
        self.started_at = started_at
        self._positions: Dict[Path, Tuple[int, bytes]] = {}

    def read_new_lines(self, flush_pending: bool = False) -> List[Tuple[Path, str]]:
        log_root = Path(tempfile.gettempdir())
        candidates = set()
        for pattern in self.PATTERNS:
            try:
                candidates.update(log_root.glob(pattern))
            except OSError:
                continue

        records: List[Tuple[Path, str]] = []
        for path in sorted(candidates, key=lambda item: item.name.casefold()):
            try:
                stat = path.stat()
                if not path.is_file() or stat.st_mtime < self.started_at - 10:
                    continue
                offset, pending = self._positions.get(path, (0, b""))
                if stat.st_size < offset:
                    offset, pending = 0, b""
                with path.open("rb") as stream:
                    stream.seek(offset)
                    chunk = stream.read()
                    next_offset = stream.tell()
                combined = pending + chunk
                pieces = combined.split(b"\n")
                pending = pieces.pop() if pieces else b""
                if flush_pending and pending:
                    pieces.append(pending)
                    pending = b""
                self._positions[path] = (next_offset, pending)
                for piece in pieces:
                    line = piece.rstrip(b"\r").decode("utf-8", errors="replace").strip()
                    if line:
                        records.append((path, line))
            except (OSError, ValueError):
                continue
        return records


def visual_studio_activity(line: str) -> Optional[str]:
    """Reduce noisy VS setup records to actionable package/channel activity."""
    message = re.sub(r"^(?:\[[^\]]+\]){1,4}\s*", "", line.strip())
    if not message:
        return None
    lower = message.casefold()

    if any(token in lower for token in (" error", "error:", "failed", "failure", "exception")):
        prefix = "ERROR"
    elif "warning" in lower or "warn:" in lower:
        prefix = "WARNING"
    elif "download" in lower or "acquir" in lower:
        prefix = "DOWNLOAD"
    elif "install" in lower and ("package" in lower or "component" in lower):
        prefix = "INSTALL"
    elif any(token in lower for token in ("apply package", "applying package", "apply operation")):
        prefix = "APPLY"
    elif "uninstall" in lower or "removing package" in lower:
        prefix = "REMOVE"
    elif "verify" in lower or "authenticode" in lower or "signature" in lower:
        prefix = "VERIFY"
    elif "manifest" in lower:
        prefix = "MANIFEST"
    elif "status changed" in lower or "status:" in lower:
        prefix = "STATUS"
    elif "channel" in lower or "feed" in lower:
        prefix = "CHANNEL"
    elif "package" in lower or "component" in lower:
        prefix = "PACKAGE"
    else:
        return None

    return prefix + ": " + message[:360]


def _wait_for_windows_process(
    kernel32: Any,
    process_handle: Any,
    on_wait: Optional[Callable[[], None]] = None,
    poll_interval_ms: int = 1000,
) -> None:
    """Wait in bounded intervals so callers can report progress while a process runs."""
    interval = max(100, int(poll_interval_ms))
    while True:
        result = int(kernel32.WaitForSingleObject(process_handle, interval))
        if result == 0:  # WAIT_OBJECT_0
            return
        if result == 258:  # WAIT_TIMEOUT
            if on_wait is not None:
                on_wait()
            continue
        raise BuildError(
            "Waiting for the elevated Visual Studio Installer process failed "
            "(Win32 result " + str(result) + ")."
        )


class VisualStudioProgress:
    """A small spinner/heartbeat that never hides the most recent installer activity."""

    FRAMES = "|/-\\"

    def __init__(self, runner: "Runner") -> None:
        self.runner = runner
        self.started = time.monotonic()
        self.last_heartbeat = 0.0
        self.frame = 0
        self.message = "Waiting for Visual Studio Installer activity"
        try:
            self.terminal = bool(sys.stdout.isatty())
        except (AttributeError, OSError):
            self.terminal = False

    def set_activity(self, message: str) -> None:
        self.message = message

    def tick(self) -> None:
        elapsed = max(0.0, time.monotonic() - self.started)
        elapsed_text = time.strftime("%M:%S", time.gmtime(elapsed))
        if self.terminal and not self.runner.debug:
            frame = self.FRAMES[self.frame % len(self.FRAMES)]
            self.frame += 1
            sys.stdout.write(
                "\r[VS Installer] " + frame + " " + self.message + " (" + elapsed_text + ")"
            )
            sys.stdout.flush()
            return
        if elapsed - self.last_heartbeat >= 10:
            if self.runner.debug:
                self.runner.debug_log(
                    "Installer still running after " + elapsed_text
                    + "; latest activity: " + self.message
                )
            else:
                print(
                    "[VS Installer] Still running after " + elapsed_text
                    + "; " + self.message
                )
            self.last_heartbeat = elapsed

    def finish(self, status: str) -> None:
        elapsed = max(0.0, time.monotonic() - self.started)
        elapsed_text = time.strftime("%M:%S", time.gmtime(elapsed))
        if self.terminal and not self.runner.debug:
            sys.stdout.write("\r" + (" " * 140) + "\r")
            sys.stdout.flush()
        print("[VS Installer] " + status + " (" + elapsed_text + ").")



def _run_elevated_windows_command(
    command: Sequence[str],
    cwd: Path,
    on_wait: Optional[Callable[[], None]] = None,
) -> int:
    """Run an elevated command and expose periodic wait callbacks for live progress."""
    if os.name != "nt":
        raise BuildError("UAC elevation is available only on Windows.")
    import ctypes
    from ctypes import wintypes

    class ShellExecuteInfo(ctypes.Structure):
        _fields_ = [
            ("cbSize", wintypes.DWORD),
            ("fMask", wintypes.ULONG),
            ("hwnd", wintypes.HWND),
            ("lpVerb", wintypes.LPCWSTR),
            ("lpFile", wintypes.LPCWSTR),
            ("lpParameters", wintypes.LPCWSTR),
            ("lpDirectory", wintypes.LPCWSTR),
            ("nShow", ctypes.c_int),
            ("hInstApp", wintypes.HINSTANCE),
            ("lpIDList", wintypes.LPVOID),
            ("lpClass", wintypes.LPCWSTR),
            ("hkeyClass", wintypes.HKEY),
            ("dwHotKey", wintypes.DWORD),
            ("hIconOrMonitor", wintypes.HANDLE),
            ("hProcess", wintypes.HANDLE),
        ]

    args = [str(part) for part in command]
    info = ShellExecuteInfo()
    info.cbSize = ctypes.sizeof(info)
    info.fMask = 0x00000040  # SEE_MASK_NOCLOSEPROCESS
    info.lpVerb = "runas"
    info.lpFile = args[0]
    info.lpParameters = subprocess.list2cmdline(args[1:])
    info.lpDirectory = str(cwd)
    info.nShow = 0  # SW_HIDE; the Visual Studio operation itself uses --quiet.

    shell32 = ctypes.WinDLL("shell32", use_last_error=True)
    kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
    shell32.ShellExecuteExW.argtypes = [ctypes.POINTER(ShellExecuteInfo)]
    shell32.ShellExecuteExW.restype = wintypes.BOOL
    kernel32.WaitForSingleObject.argtypes = [wintypes.HANDLE, wintypes.DWORD]
    kernel32.WaitForSingleObject.restype = wintypes.DWORD
    kernel32.GetExitCodeProcess.argtypes = [wintypes.HANDLE, ctypes.POINTER(wintypes.DWORD)]
    kernel32.GetExitCodeProcess.restype = wintypes.BOOL
    kernel32.CloseHandle.argtypes = [wintypes.HANDLE]
    kernel32.CloseHandle.restype = wintypes.BOOL
    if not shell32.ShellExecuteExW(ctypes.byref(info)):
        error = ctypes.get_last_error()
        if error == 1223:  # ERROR_CANCELLED: user declined the UAC prompt.
            raise BuildError("Windows elevation was declined. No Visual Studio changes were made.")
        raise BuildError(
            "Windows could not start the elevated Visual Studio Installer process "
            + "(Win32 error " + str(error) + ")."
        )

    try:
        _wait_for_windows_process(kernel32, info.hProcess, on_wait=on_wait)
        exit_code = wintypes.DWORD()
        if not kernel32.GetExitCodeProcess(info.hProcess, ctypes.byref(exit_code)):
            error = ctypes.get_last_error()
            raise BuildError("Could not read the Visual Studio Installer exit code (Win32 error " + str(error) + ").")
        return int(exit_code.value)
    finally:
        kernel32.CloseHandle(info.hProcess)


def _recent_visual_studio_setup_logs(started_at: float, limit: int = 3) -> str:
    """Collect concise error context from Visual Studio's documented %TEMP% logs."""
    log_root = Path(tempfile.gettempdir())
    candidates: List[Path] = []
    for pattern in ("dd_setup_*.log", "dd_client_*.log", "dd_bootstrapper_*.log"):
        try:
            for path in log_root.glob(pattern):
                try:
                    if path.is_file() and path.stat().st_mtime >= started_at - 10:
                        candidates.append(path)
                except OSError:
                    continue
        except OSError:
            continue
    candidates.sort(key=lambda path: path.stat().st_mtime, reverse=True)
    sections: List[str] = []
    for path in candidates[:limit]:
        try:
            lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
        except OSError as exc:
            sections.append(str(path) + ": unable to read log (" + str(exc) + ")")
            continue
        markers = ("error", "failed", "failure", "warning", "exit code", "blocked")
        relevant = [
            line for line in lines
            if any(marker in line.lower() for marker in markers)
        ]
        excerpt = relevant[-24:] if relevant else lines[-12:]
        sections.append(
            str(path) + "\n" + "\n".join(excerpt)[-5000:]
        )
    if not sections:
        return "No new dd_setup/dd_client/dd_bootstrapper logs were found in " + str(log_root) + "."
    return "\\n\\n".join(sections)[-12000:]


def _visual_studio_setup_log_diagnostics(started_at: float) -> List[str]:
    """Identify successful-looking setup exits that did not actually apply components."""
    log_root = Path(tempfile.gettempdir())
    candidates: List[Path] = []
    for pattern in ("dd_setup_*.log", "dd_client_*.log", "dd_bootstrapper_*.log"):
        try:
            for path in log_root.glob(pattern):
                try:
                    if path.is_file() and path.stat().st_mtime >= started_at - 10:
                        candidates.append(path)
                except OSError:
                    continue
        except OSError:
            continue

    contents: List[str] = []
    for path in candidates:
        try:
            contents.append(path.read_text(encoding="utf-8", errors="replace")[-250000:])
        except OSError:
            continue
    combined = "\n".join(contents)
    if not combined:
        return ["No new Visual Studio setup logs were found; the installer exit code alone does not confirm component installation."]

    notes: List[str] = []
    lower = combined.casefold()
    feed_failed = (
        "didn't find any channel feed" in lower
        or "requestcanceled" in lower
        or "could not update channel" in lower
    )
    if feed_failed:
        notes.append(
            "Visual Studio could not refresh its channel feed (the log reports canceled "
            "requests to aka.ms/vs/.../channel); check access to Microsoft's Visual Studio "
            "channel endpoints before retrying the component modification"
        )

    no_update = "status changed to noupdate" in lower
    apply_activity = re.search(
        r"\b(?:install(?:ing|ed)?|download(?:ing|ed)?|acquir(?:e|ing|ed)|apply(?:ing|ied))\b"
        r".{0,180}\b(?:package|component|workload)\b",
        combined,
        re.IGNORECASE,
    ) is not None
    if no_update and not apply_activity:
        notes.append(
            "The setup log ended in NoUpdate without package download/install/apply activity; "
            "setup.exe returned 0, but the requested C++/SDK components were not confirmed as applied"
        )
    return notes


def run_visual_studio_setup(
    runner: "Runner",
    installer: Path,
    arguments: Sequence[str],
    operation: str = "Modify existing Visual Studio C++ workload",
) -> None:
    """Run Visual Studio setup quietly while reporting live activity from its log files."""
    command = [str(installer)] + [str(part) for part in arguments]
    rendered = subprocess.list2cmdline(command) if os.name == "nt" else shlex.join(command)
    print("\n==> " + operation)
    if runner.debug:
        print("$ " + rendered)
    if runner.dry_run:
        print("[VS Installer] Dry run; installer was not started.")
        return

    started_at = time.time()
    runner.visual_studio_setup_started_at = started_at
    runner.visual_studio_restart_required = False
    tailer = VisualStudioSetupLogTailer(started_at)
    progress = VisualStudioProgress(runner)

    def poll_installer_logs(flush_pending: bool = False) -> None:
        for path, line in tailer.read_new_lines(flush_pending=flush_pending):
            if runner.all_debug:
                runner.debug_log("[VS RAW " + path.name + "] " + line)
            activity = visual_studio_activity(line)
            if activity:
                progress.set_activity(activity)
                if runner.debug:
                    runner.debug_log("[VS Installer] " + activity)
        progress.tick()

    print("[VS Installer] Waiting for Windows administrator approval and installer startup...")
    try:
        exit_code = _run_elevated_windows_command(
            command, runner.root, on_wait=poll_installer_logs
        )
        poll_installer_logs(flush_pending=True)
    except BaseException:
        progress.finish("stopped before a completion status was reported")
        raise

    if exit_code == 0:
        progress.finish("installer process exited with code 0; validating the resulting toolchain")
        return
    if exit_code == 3010:
        runner.visual_studio_restart_required = True
        progress.finish("installer process exited with code 3010; a Windows restart is required")
        return
    progress.finish("failed with exit code " + str(exit_code))
    reason = visual_studio_exit_reason(exit_code)
    logs = _recent_visual_studio_setup_logs(started_at)
    raise BuildError(
        "Visual Studio setup failed with exit code " + str(exit_code) + ".\n"
        + reason + "\n"
        + "Relevant Visual Studio setup log excerpts:\n" + logs
    )



def _has_buildtools_instance(installations: Optional[List[Path]] = None) -> bool:
    candidates = installations if installations is not None else _visual_studio_installations()
    return any(path.name.casefold() == "buildtools" for path in candidates)


def _powershell_executable() -> Optional[str]:
    return command_path("powershell") or command_path("pwsh")


def _verify_microsoft_authenticode(path: Path, powershell: str) -> str:
    """Require a valid Authenticode signature issued to Microsoft Corporation."""
    env = os.environ.copy()
    env["WINLOOK_AUTHENTICODE_FILE"] = str(path)
    script = (
        "$ErrorActionPreference = 'Stop'; "
        "Import-Module Microsoft.PowerShell.Security -Force -ErrorAction Stop; "
        "$sig = Get-AuthenticodeSignature -LiteralPath $env:WINLOOK_AUTHENTICODE_FILE; "
        "if ($sig.Status -ne 'Valid') { Write-Error ('Signature status: ' + $sig.Status); exit 2 }; "
        "$subject = $sig.SignerCertificate.Subject; "
        "if ($subject -notmatch 'Microsoft Corporation') { Write-Error ('Unexpected signer: ' + $subject); exit 3 }; "
        "Write-Output $subject"
    )
    result = run_subprocess(
        [powershell, "-NoProfile", "-NonInteractive", "-Command", script],
        text=True, capture_output=True, check=False, env=env,
    )
    if result.returncode != 0:
        detail = (result.stderr or result.stdout or "Authenticode verification failed").strip()
        raise BuildError(
            "Refusing to execute the Visual Studio Build Tools bootstrapper because its "
            "Microsoft Authenticode signature could not be verified. " + detail[-2000:]
        )
    signer = (result.stdout or "").strip()
    if "Microsoft Corporation" not in signer:
        raise BuildError(
            "Refusing to execute the Visual Studio Build Tools bootstrapper: "
            "the Authenticode signer is not Microsoft Corporation."
        )
    return signer


def download_vs_build_tools_bootstrapper(runner: "Runner") -> Path:
    """Reuse or download the Microsoft bootstrapper and verify its Authenticode signature."""
    destination = (
        build_paths(runner.root)["rust_cache"] / "installers" / "vs_buildtools_2022.exe"
    )
    if runner.dry_run:
        print("Would reuse or download the Microsoft Visual Studio 2022 Build Tools bootstrapper:")
        print("  " + VS_BUILD_TOOLS_BOOTSTRAPPER_URL)
        print("Would verify its Microsoft Authenticode signature before execution.")
        return destination

    powershell = _powershell_executable()
    if not powershell:
        raise BuildError(
            "PowerShell is unavailable, so the downloaded Visual Studio installer cannot be "
            "Authenticode-verified. Install PowerShell or provide WinGet."
        )

    if destination.is_file():
        try:
            signer = _verify_microsoft_authenticode(destination, powershell)
        except BuildError:
            print("Cached Visual Studio bootstrapper failed signature verification; downloading a fresh copy.")
            destination.unlink(missing_ok=True)
        else:
            print("Reusing cached Visual Studio Build Tools bootstrapper signed by " + signer)
            return destination

    destination.parent.mkdir(parents=True, exist_ok=True)
    temporary = destination.with_name(destination.name + ".download")
    request = urllib.request.Request(
        VS_BUILD_TOOLS_BOOTSTRAPPER_URL,
        headers={"User-Agent": "WinLook-build.py"},
    )
    try:
        with urllib.request.urlopen(request, timeout=120) as response:
            with temporary.open("wb") as output:
                shutil.copyfileobj(response, output)
        if temporary.stat().st_size < 64 * 1024:
            raise BuildError("The Visual Studio Build Tools bootstrapper download is unexpectedly small.")
        signer = _verify_microsoft_authenticode(temporary, powershell)
        temporary.replace(destination)
        print("Downloaded and verified Visual Studio Build Tools bootstrapper: " + str(destination))
        print("Authenticode signer: " + signer)
    except BuildError:
        temporary.unlink(missing_ok=True)
        raise
    except Exception as exc:
        temporary.unlink(missing_ok=True)
        raise BuildError(
            "Could not download the official Visual Studio Build Tools bootstrapper from "
            + VS_BUILD_TOOLS_BOOTSTRAPPER_URL + ": " + str(exc)
        ) from exc
    return destination


def _install_visual_studio_build_tools(runner: "Runner") -> None:
    """Install the standalone Build Tools SKU once, using WinGet or Microsoft's bootstrapper."""
    winget = command_path("winget")
    if winget:
        runner.run(
            "Install Visual Studio 2022 C++ Build Tools through WinGet",
            [
                winget, "install", "--id", VS_BUILD_TOOLS_WINGET_ID, "--exact",
                "--accept-package-agreements", "--accept-source-agreements",
                "--override",
                " ".join([
                    "--wait --passive --norestart",
                    "--add Microsoft.VisualStudio.Workload.VCTools",
                    *["--add " + component_id for component_id in VS_REQUIRED_COMPONENT_IDS],
                    "--includeRecommended",
                ]),
            ],
        )
        return

    print("WinGet is unavailable; using Microsoft's official Visual Studio 2022 Build Tools bootstrapper.")
    bootstrapper = download_vs_build_tools_bootstrapper(runner)
    if runner.dry_run:
        return
    install_root = Path(os.environ.get("ProgramFiles(x86)", r"C:\Program Files (x86)"))
    install_path = install_root / "Microsoft Visual Studio" / "2022" / "BuildTools"
    run_visual_studio_setup(
        runner,
        bootstrapper,
        [
            "--quiet", "--wait", "--norestart",
            "--installPath", str(install_path),
            "--add", "Microsoft.VisualStudio.Workload.VCTools",
        ] + [
            value
            for component_id in VS_REQUIRED_COMPONENT_IDS
            for value in ("--add", component_id)
        ] + ["--includeRecommended"],
        operation="Install Visual Studio 2022 C++ Build Tools",
    )


def _install_or_modify_visual_studio(
    runner: "Runner", installations: Optional[List[Path]] = None
) -> None:
    """Modify an existing VS instance; install Build Tools only when none exist."""
    installations = installations if installations is not None else _visual_studio_installations()
    if installations:
        # Prefer a Build Tools instance if one exists; never create another copy
        # just because an Enterprise/Community instance is missing its workload.
        build_tools = [path for path in installations if path.name.casefold() == "buildtools"]
        selected = build_tools[0] if build_tools else installations[0]
        installer = _vs_installer_path()
        if installer is None:
            raise BuildError(
                "A Visual Studio installation already exists at " + str(selected)
                + ", but Visual Studio Installer could not be located. "
                "The existing instance will not be reinstalled."
            )
        channel_id = _visual_studio_channel_id(selected)
        run_visual_studio_setup(
            runner,
            installer,
            [
                "modify", "--installPath", str(selected),
                "--channelId", channel_id,
            ] + _visual_studio_workload_arguments() + ["--includeRecommended", "--quiet", "--norestart"],
        )
        return

    _install_visual_studio_build_tools(runner)


def ensure_windows_linker(runner: "Runner", apply: bool = True) -> None:
    """Prepare the MSVC linker/Windows SDK required by the Rust/WinUI targets."""
    if os.name != "nt":
        return
    if load_msvc_environment(runner):
        return

    gcc = probe_mingw_gcc()
    if gcc:
        print(
            "MinGW GCC is installed (" + gcc["target"] + "), but the current WinUI 3 app "
            "uses the Rust MSVC ABI. GCC alone cannot replace link.exe or the Windows SDK."
        )

    installations = _visual_studio_installations()
    if not apply:
        if installations:
            print("Plan: add the C++ workload to the existing Visual Studio installation.")
            if not _has_buildtools_instance(installations):
                print("If modifying that instance fails, install the standalone Build Tools SKU once.")
        else:
            print("Plan: install Visual Studio 2022 Build Tools through WinGet.")
        return

    had_installations = bool(installations)
    had_buildtools = _has_buildtools_instance(installations)
    standalone_install_attempted = not had_installations
    try:
        _install_or_modify_visual_studio(runner, installations)
    except BuildError as exc:
        message = str(exc).lower()
        if had_buildtools or not had_installations or "declined" in message or "cancelled" in message:
            # Do not retry a failed standalone installation, a Build Tools
            # instance, or an operation the user explicitly cancelled.
            raise
        print(
            "Existing Visual Studio modification failed; falling back to the standalone "
            "Build Tools SKU once instead of retrying the same instance."
        )
        _install_visual_studio_build_tools(runner)
        standalone_install_attempted = True

    if load_msvc_environment(runner):
        return

    # A non-BuildTools VS edition can be present without the C++ workload.
    # If modification returned success but produced no usable environment,
    # install the standalone Build Tools SKU once. Never loop after that attempt.
    current_installations = _visual_studio_installations()
    if (
        had_installations
        and not had_buildtools
        and not standalone_install_attempted
        and not _has_buildtools_instance(current_installations)
    ):
        print("The existing Visual Studio instance is still missing the C++ toolchain.")
        print("Installing the standalone Build Tools SKU once as a fallback.")
        _install_visual_studio_build_tools(runner)
        standalone_install_attempted = True
        if load_msvc_environment(runner):
            return

    diagnostics = runner.msvc_diagnostics or _msvc_environment_issues(dict(os.environ))[0]
    detail = ""
    if diagnostics:
        detail = "\n\nToolchain detection details:\n - " + "\n - ".join(diagnostics)
    setup_detail = ""
    if runner.visual_studio_setup_started_at is not None:
        setup_diagnostics = _visual_studio_setup_log_diagnostics(runner.visual_studio_setup_started_at)
        if setup_diagnostics:
            setup_detail = "\n\nVisual Studio Installer diagnostics:\n - " + "\n - ".join(setup_diagnostics)
    if runner.visual_studio_restart_required:
        raise BuildError(
            "Visual Studio setup returned 3010 and requires a Windows restart before the "
            "MSVC environment can be loaded. Restart Windows, then rerun the build."
            + detail + setup_detail
        )
    raise BuildError(
        "The Visual Studio Installer process returned successfully, but WinLook could not "
        "verify a usable MSVC compiler/linker/resource compiler and Windows SDK. No second "
        "installer was started. Repair the existing Build Tools instance and rerun the build."
        + detail + setup_detail
    )


def ensure_windows_c_toolchain(runner: "Runner", apply: bool = True) -> str:
    """Select an already-installed C compiler; use MSVC or MinGW without duplicate setup."""
    if os.name != "nt":
        return "host"
    if load_msvc_environment(runner):
        runner.c_compiler = "msvc"
        runner.tool_env = {}
        return "msvc"
    if _select_mingw_cmake(runner):
        return "mingw"
    if not apply:
        print("Plan: use installed MSVC or MinGW GCC; install CMake only if it is missing.")
        return "missing"
    ensure_windows_linker(runner, apply=True)
    runner.c_compiler = "msvc"
    runner.tool_env = {}
    return "msvc"


def ensure_cmake(runner: "Runner", apply: bool = True) -> None:
    if command_path("cmake"):
        return
    if os.name == "nt" and command_path("winget"):
        if not apply:
            print("Plan: install CMake via WinGet (" + CMAKE_WINGET_ID + ").")
            return
        runner.run(
            "Install CMake",
            ["winget", "install", "--id", CMAKE_WINGET_ID, "--exact",
             "--accept-package-agreements", "--accept-source-agreements"],
        )
        if command_path("cmake"):
            return
    raise BuildError("CMake is missing. Install it from https://cmake.org/download/ and reopen the terminal.")


def build_paths(root: Path) -> Dict[str, Path]:
    base = root / "build"
    return {
        "rust": base / "rust",
        "c": base / "c",
        "rust_cache": base / ".rcache",
        "c_cache": base / ".Ccache",
        "artifacts": base / "artifacts",
    }


class BuildError(RuntimeError):
    """An actionable build or toolchain error."""


_ALL_DEBUG_ACTIVE = False


def run_subprocess(command: Any, *args: Any, **kwargs: Any) -> subprocess.CompletedProcess:
    """Run a child process and record its captured output during --all-debug."""
    result = subprocess.run(command, *args, **kwargs)
    if _ALL_DEBUG_ACTIVE:
        command_parts = (
            [str(part) for part in command]
            if isinstance(command, (list, tuple))
            else [str(command)]
        )
        rendered = (
            subprocess.list2cmdline(command_parts)
            if os.name == "nt"
            else shlex.join(command_parts)
        )
        print("[SUBPROCESS] " + rendered + " (exit " + str(result.returncode) + ")")
        for stream_name in ("stdout", "stderr"):
            value = getattr(result, stream_name, None)
            if not value:
                continue
            if isinstance(value, bytes):
                value = value.decode("utf-8", errors="replace")
            output = str(value)
            print("[SUBPROCESS " + stream_name.upper() + "]")
            print(output, end="" if output.endswith("\n") else "\n")
    return result


@dataclass
class Runner:
    root: Path = ROOT
    dry_run: bool = False
    verbose: bool = False
    debug: bool = False
    all_debug: bool = False
    msvc_diagnostics: List[str] = field(default_factory=list)
    visual_studio_setup_started_at: Optional[float] = None
    visual_studio_restart_required: bool = False
    rust_env: Dict[str, str] = field(default_factory=dict)
    tool_env: Dict[str, str] = field(default_factory=dict)
    c_compiler: str = ""

    def run(
        self,
        label: str,
        command: Sequence[str],
        cwd: Optional[Path] = None,
        env: Optional[Dict[str, str]] = None,
        capture: bool = False,
    ) -> subprocess.CompletedProcess:
        args = [str(item) for item in command]
        location = cwd or self.root
        process_env = os.environ.copy()
        if env is not None:
            process_env.update(env)
        paths = build_paths(self.root)
        process_env.update(self.rust_env)
        process_env.update(self.tool_env)
        process_env.setdefault("CARGO_TARGET_DIR", str(paths["rust"] / "shared"))
        process_env.setdefault("SCCACHE_DIR", str(paths["rust_cache"] / "sccache"))
        process_env.setdefault("CCACHE_DIR", str(paths["c_cache"]))
        process_env.setdefault("CCACHE_BASEDIR", str(self.root))
        if command_path("sccache"):
            process_env.setdefault("RUSTC_WRAPPER", command_path("sccache") or "sccache")
        rendered = subprocess.list2cmdline(args) if os.name == "nt" else shlex.join(args)
        print("\n==> " + label)
        if self.debug and (not self.all_debug or not _ALL_DEBUG_ACTIVE):
            print("$ " + rendered)
        if self.dry_run:
            return subprocess.CompletedProcess(args, 0, stdout="", stderr="")
        try:
            # Capture child output so normal builds stay readable. Full output is
            # replayed only in --all-debug mode; failures always retain a useful tail.
            result = run_subprocess(
                args, cwd=str(location), env=process_env, check=False, text=True,
                capture_output=True,
            )
            if self.all_debug and not _ALL_DEBUG_ACTIVE:
                if result.stdout:
                    print("[COMMAND STDOUT] " + label)
                    print(result.stdout, end="" if result.stdout.endswith("\n") else "\n")
                if result.stderr:
                    print("[COMMAND STDERR] " + label, file=sys.stderr)
                    print(result.stderr, end="" if result.stderr.endswith("\n") else "\n", file=sys.stderr)
        except FileNotFoundError as exc:
            raise BuildError(
                "Required command was not found: " + args[0]
                + ". Run python build.py doctor for toolchain guidance."
            ) from exc
        if result.returncode != 0:
            details = []
            if result.stderr:
                details.append("stderr:\n" + result.stderr.strip())
            if result.stdout:
                details.append("stdout:\n" + result.stdout.strip())
            detail = "\n".join(details)
            raise BuildError(
                label + " failed with exit code " + str(result.returncode)
                + (("\n" + detail[-5000:]) if detail else "")
            )
        if capture and self.verbose and result.stdout and not self.all_debug:
            print(result.stdout)
        return result

    def debug_log(self, message: str) -> None:
        """Emit a detailed diagnostic only when a debug mode is enabled."""
        if self.debug:
            print(message)


def format_command(command: Sequence[str]) -> str:
    values = [str(item) for item in command]
    return subprocess.list2cmdline(values) if os.name == "nt" else shlex.join(values)


def command_path(name: str) -> Optional[str]:
    found = shutil.which(name)
    if found or os.name != "nt" or name.lower().endswith((".exe", ".bat", ".cmd", ".com")):
        return found
    # Explicit .exe lookup remains reliable even when PATHEXT is missing or
    # incomplete in a process launched from a stripped-down CI environment.
    return shutil.which(name + ".exe")


def project_has_manifest(root: Path, relative: str) -> bool:
    return (root / relative).is_file()


def is_bgw_target(name: str, kinds: Iterable[str]) -> bool:
    return "bin" in set(kinds) and name in BGW_TARGET_NAMES


def select_bgw_targets(metadata: Dict[str, Any]) -> List[Tuple[str, str]]:
    """Return package/binary pairs for explicitly named BGW executables."""
    matches: List[Tuple[str, str]] = []
    for package in metadata.get("packages", []):
        for target in package.get("targets", []):
            if is_bgw_target(target.get("name", ""), target.get("kind", [])):
                matches.append((package["name"], target["name"]))
    return matches


def package_filename(target: str) -> str:
    if target not in WINDOWS_APPS:
        raise ValueError("Unsupported package target: " + target)
    return "winlook-" + target + "-x64-preview.zip"


def require_command(name: str, guidance: str) -> None:
    if command_path(name) is None:
        raise BuildError("Missing " + name + ". " + guidance)


def require_windows(feature: str) -> None:
    if os.name != "nt":
        raise BuildError(
            feature + " requires a Windows host with the MSVC toolchain. "
            "Portable Rust/C library tests can still run on this host."
        )


def require_manifest(root: Path, relative: str, description: str) -> None:
    if not project_has_manifest(root, relative):
        raise BuildError(
            description + " is not present yet (expected " + relative + "). "
            "This target is gated until its source project is added."
        )


def check_ui_sources(root: Path) -> bool:
    required = (
        "rust/library/wl_ui/Cargo.toml",
        "rust/win10/Cargo.toml",
        "rust/win11/Cargo.toml",
    )
    return all(project_has_manifest(root, item) for item in required)


def check_windows_app(root: Path, target: str) -> bool:
    return project_has_manifest(root, "rust/" + target + "/Cargo.toml")


def target_available(root: Path, target: str) -> Tuple[bool, str]:
    if target.startswith("windows."):
        version = target[len("windows."):]
        if version not in WINDOWS_GENERATIONS:
            raise ValueError("Unknown build target: " + target)
        label, directory, toolchain = WINDOWS_GENERATIONS[version]
        if toolchain == "msvc":
            path = "rust/" + directory + "/Cargo.toml"
        elif toolchain == "watcom":
            path = "C/" + directory + "/Makefile"
        else:
            path = "C/" + directory + "/CMakeLists.txt"
        ready = project_has_manifest(root, path)
        return ready, path + (" found" if ready else " is not present")
    if target == "libraries":
        ready = project_has_manifest(root, "Cargo.toml") and project_has_manifest(root, "C/library/CMakeLists.txt")
        return ready, "Rust workspace and C library aggregator found" if ready else "library manifests incomplete"
    if target == "ui":
        ready = check_ui_sources(root)
        return ready, "Manager UI manifests found" if ready else "Manager UI manifests are not present"
    if target in ("win10", "win11"):
        ready = check_windows_app(root, target)
        return ready, target + " application manifest found" if ready else target + " application manifest is not present"
    if target == "win95":
        ready = project_has_manifest(root, "C/win95/Makefile")
        return ready, "C/win95/Makefile found" if ready else "C/win95/Makefile is not present"
    if target == "bgw":
        manifests = (root / "rust/win10/Cargo.toml", root / "rust/win11/Cargo.toml")
        for manifest in manifests:
            if manifest.is_file():
                try:
                    source = manifest.read_text(encoding="utf-8")
                except OSError:
                    continue
                for target_name in BGW_TARGET_NAMES:
                    if target_name in source:
                        return True, "BGW binary declaration found in " + str(manifest.relative_to(root))
        return False, "BGW binary target is not implemented"
    raise ValueError("Unknown build target: " + target)


def print_skip(message: str) -> None:
    print("[SKIPPED] " + message)


def _windows_msvc_requirement_missing(
    checks: Dict[str, Any], windows_host: Optional[bool] = None
) -> bool:
    """Require the complete MSVC + Windows SDK toolchain for available WinUI apps."""
    if windows_host is None:
        windows_host = os.name == "nt"
    has_modern_app = bool(checks.get("win10_project") or checks.get("win11_project"))
    return bool(windows_host and has_modern_app and checks.get("msvc_available") is not True)


def command_doctor(args: argparse.Namespace, runner: Runner) -> int:
    mingw = probe_mingw_gcc()
    msvc_available = load_msvc_environment(runner) if os.name == "nt" else None
    visual_studio_instances = [str(path) for path in _visual_studio_installations()] if os.name == "nt" else []
    checks = {
        "python": sys.version.split()[0],
        "platform": platform.platform(),
        "architecture": platform.machine(),
        "git": command_path("git"),
        "rustup": str(_rustup_executable(runner.root)) if _rustup_executable(runner.root).is_file() else command_path("rustup"),
        "rustc": str(rust_paths(runner.root)["cargo_home"] / "bin" / ("rustc.exe" if os.name == "nt" else "rustc")) if (rust_paths(runner.root)["cargo_home"] / "bin" / ("rustc.exe" if os.name == "nt" else "rustc")).is_file() else command_path("rustc"),
        "cargo": str(rust_paths(runner.root)["cargo_home"] / "bin" / ("cargo.exe" if os.name == "nt" else "cargo")) if (rust_paths(runner.root)["cargo_home"] / "bin" / ("cargo.exe" if os.name == "nt" else "cargo")).is_file() else command_path("cargo"),
        "project_rustup": str(_rustup_executable(runner.root)) if _rustup_executable(runner.root).is_file() else None,
        "project_rustc": str(rust_paths(runner.root)["cargo_home"] / "bin" / ("rustc.exe" if os.name == "nt" else "rustc")) if (rust_paths(runner.root)["cargo_home"] / "bin" / ("rustc.exe" if os.name == "nt" else "rustc")).is_file() else None,
        "required_rust": RUST_TOOLCHAIN,
        "rustfmt": command_path("rustfmt"),
        "clippy-driver": command_path("clippy-driver"),
        "cmake": command_path("cmake"),
        "ccache": command_path("ccache"),
        "sccache": command_path("sccache"),
        "cargo_home": str(build_paths(runner.root)["rust_cache"] / "cargo-home"),
        "rust_output": str(build_paths(runner.root)["rust"]),
        "c_output": str(build_paths(runner.root)["c"]),
        "rust_cache": str(build_paths(runner.root)["rust_cache"]),
        "c_cache": str(build_paths(runner.root)["c_cache"]),
        "cmake_generator": os.environ.get("CMAKE_GENERATOR"),
        "msvc_available": msvc_available,
        "msvc_cl": command_path("cl") if msvc_available else None,
        "msvc_link": command_path("link") if msvc_available else None,
        "msvc_rc": command_path("rc") if msvc_available else None,
        "windows_sdk_dir": os.environ.get("WindowsSdkDir") or os.environ.get("WindowsSDKDir"),
        "windows_sdk_version": os.environ.get("WindowsSDKVersion"),
        "windows_sdk_present": _windows_sdk_is_present(dict(os.environ)) if os.name == "nt" else None,
        "visual_studio_instances": visual_studio_instances,
        "gcc": command_path("gcc"),
        "gcc_target": (mingw or {}).get("target"),
        "mingw_build_system": (mingw or {}).get("generator") or None,
        "ninja": command_path("ninja"),
        "mingw32_make": command_path("mingw32-make"),
        "clang": command_path("clang"),
        "open_watcom_root": os.environ.get("WATCOM"),
        "open_watcom_wcc386": command_path("wcc386"),
        "open_watcom_wmake": command_path("wmake"),
        "manager_ui_sources": check_ui_sources(runner.root),
        "win10_project": check_windows_app(runner.root, "win10"),
        "win11_project": check_windows_app(runner.root, "win11"),
        "win95_makefile": project_has_manifest(runner.root, "C/win95/Makefile"),
        "windows_target_sources": {
            selector: target_available(runner.root, selector)
            for selector in WINDOWS_BUILD_COMMANDS
        },
    }
    missing = []
    if sys.version_info < PYTHON_MINIMUM:
        missing.append("Python 3.9 or newer")
    for tool in ("cargo", "rustc", "cmake"):
        if not checks[tool]:
            missing.append(tool)
    if _windows_msvc_requirement_missing(checks):
        missing.append("MSVC cl.exe, link.exe, rc.exe and a usable Windows SDK (required by Win10/Win11)")
    if args.json:
        print(json.dumps({"root": str(runner.root), "checks": checks, "missing": missing}, indent=2))
    else:
        print("WinLook build environment")
        print("Repository: " + str(runner.root))
        for key, value in checks.items():
            print("  {:30} {}".format(key + ":", "not found" if value is None else str(value)))
        if missing:
            print("\nMissing baseline tools: " + ", ".join(missing))
        print("\nNext steps:")
        print("  python build.py verify libraries")
        print("  python build.py windows.10")
        print("  python build.py windows.11")
        print("  python build.py windows.7    (source-gated C/MinGW target)")
        print("  python build.py windows.95   (source-gated C/Open Watcom target)")
        print("Running without an action is equivalent to: python build.py doctor")
        print("\nWindows target source readiness:")
        for selector in WINDOWS_BUILD_COMMANDS:
            available, reason = checks["windows_target_sources"][selector]
            status = "ready" if available else "source not added"
            print("  {:18} {:16} {}".format(selector, status, reason))
        print("\nWin10/Win11 WinUI builds require the MSVC linker and Windows SDK.")
        print("Legacy NT-family CMake targets use native MinGW GCC; Win9x targets use Open Watcom.")
        print("Running WinUI 3 also requires the matching Windows App Runtime:")
        print("  " + APP_RUNTIME_URL)
        print("Win95 builds require Open Watcom and C/win95/Makefile.")
    return 2 if args.strict and missing else 0


def append_github_environment(name: str, value: str) -> None:
    env_file = os.environ.get("GITHUB_ENV")
    if env_file:
        with open(env_file, "a", encoding="utf-8") as handle:
            handle.write(name + "=" + value + "\n")


def append_github_path(value: str) -> None:
    path_file = os.environ.get("GITHUB_PATH")
    if path_file:
        with open(path_file, "a", encoding="utf-8") as handle:
            handle.write(value + "\n")


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def download_open_watcom(runner: Runner, apply: bool) -> None:
    if not sys.platform.startswith("linux") or platform.machine().lower() not in {"x86_64", "amd64"}:
        raise BuildError(
            "Automated Open Watcom setup is pinned for Linux x86_64. "
            "Install it manually on other hosts and set WATCOM."
        )
    destination = build_paths(runner.root)["rust_cache"] / "toolchains" / ("open-watcom-" + WATCOM_RELEASE)
    compiler = destination / "binl64" / "wcc386"
    installer = destination.parent / "open-watcom-2_0-c-linux-x64"
    if compiler.is_file():
        print("Open Watcom compiler already exists at " + str(destination))
        return
    if not apply:
        print("Plan: download the official Open Watcom installer:")
        print("  " + WATCOM_ASSET_URL)
        print("Verify SHA-256: " + WATCOM_ASSET_SHA256)
        print("Store it under: " + str(installer))
        print("No files were downloaded. Repeat with --apply.")
        return
    destination.parent.mkdir(parents=True, exist_ok=True)
    request = urllib.request.Request(
        WATCOM_ASSET_URL, headers={"User-Agent": "WinLook-build.py"}
    )
    print("Downloading pinned Open Watcom 2.0 installer...")
    with urllib.request.urlopen(request, timeout=120) as response:
        with installer.open("wb") as output:
            shutil.copyfileobj(response, output)
    actual = sha256_file(installer)
    if actual.lower() != WATCOM_ASSET_SHA256:
        installer.unlink(missing_ok=True)
        raise BuildError(
            "Open Watcom SHA-256 mismatch; refusing to execute the installer. "
            "Expected " + WATCOM_ASSET_SHA256 + ", got " + actual
        )
    print("Verified and downloaded Open Watcom installer: " + str(installer))
    print("The upstream docs do not specify unattended installer switches.")
    print("Run the verified installer manually; then set WATCOM to its install directory.")
    print("No installer command was guessed or executed.")


def command_setup(args: argparse.Namespace, runner: Runner) -> int:
    if not args.with_rust and not args.with_c and not args.open_watcom and not args.with_cmake:
        print("Select at least one setup target: --with-rust, --with-c, --with-cmake, or --open-watcom.")
        return 0
    if args.with_rust:
        ensure_rust_toolchain(runner, apply=args.apply)
    if args.with_c:
        if os.name == "nt":
            ensure_windows_c_toolchain(runner, apply=args.apply)
        ensure_cmake(runner, apply=args.apply)
    elif args.with_cmake:
        ensure_cmake(runner, apply=args.apply)
    if args.open_watcom:
        download_open_watcom(runner, args.apply and not runner.dry_run)
    if not args.apply:
        print("Nothing was installed. Review the plan and repeat with --apply.")
    else:
        print("Requested setup completed. Run python build.py doctor to verify.")
    return 0


def rust_verify(runner: Runner) -> None:
    require_command("cargo", "Install Rust through https://rustup.rs/.")
    commands = [
        ("Rust formatting", ["cargo", "fmt", "--all", "--", "--check"]),
        ("Rust shared-library tests", ["cargo", "test", "-p", "wl_animate", "-p", "wl_cpuv", "-p", "wl_gpuv", "--all-targets", "--all-features"]),
        ("Rust shared-library Clippy", ["cargo", "clippy", "-p", "wl_animate", "-p", "wl_cpuv", "-p", "wl_gpuv", "--all-targets", "--all-features", "--", "-D", "warnings"]),
        ("Rust shared-library API documentation", ["cargo", "doc", "-p", "wl_animate", "-p", "wl_cpuv", "-p", "wl_gpuv", "--no-deps"]),
    ]
    for label, command in commands:
        runner.run(label, command)


def rust_tests(runner: Runner) -> None:
    require_command("cargo", "Install Rust through https://rustup.rs/.")
    runner.run("Rust shared-library unit tests", ["cargo", "test", "-p", "wl_animate", "-p", "wl_cpuv", "-p", "wl_gpuv", "--all-targets", "--all-features"])


def cmake_c_test(runner: Runner, sanitizers: bool = False) -> None:
    source = runner.root / "C" / "library"
    if not (source / "CMakeLists.txt").is_file():
        raise BuildError("C/library/CMakeLists.txt is missing; no compiler setup was attempted.")
    if os.name == "nt":
        ensure_windows_c_toolchain(runner, apply=not runner.dry_run)
    ensure_cmake(runner, apply=not runner.dry_run)
    suffix = "-mingw" if runner.c_compiler == "mingw" else ""
    build_dir = build_paths(runner.root)["c"] / (("library-sanitized" if sanitizers else "library-release") + suffix)
    configure = [
        "cmake", "-S", str(source), "-B", str(build_dir),
        "-DCMAKE_BUILD_TYPE=" + ("Debug" if sanitizers else "Release"),
    ]
    if sanitizers and os.name != "nt":
        flags = "-std=c89 -Wall -Wextra -Werror -pedantic -fsanitize=address,undefined -fno-omit-frame-pointer"
        configure.extend([
            "-DCMAKE_C_FLAGS=" + flags,
            "-DCMAKE_EXE_LINKER_FLAGS=-fsanitize=address,undefined",
        ])
    if runner.c_compiler == "mingw":
        configure.extend([
            "-G", runner.tool_env["CMAKE_GENERATOR"],
            "-DCMAKE_C_COMPILER=" + runner.tool_env["CC"],
            "-DCMAKE_MAKE_PROGRAM=" + runner.tool_env["CMAKE_MAKE_PROGRAM"],
        ])
    elif os.name != "nt" and command_path("ccache"):
        configure.append("-DCMAKE_C_COMPILER_LAUNCHER=ccache")
    try:
        runner.run("Configure all C89 libraries", configure, capture=True)
    except BuildError:
        # Compiler discovery is completed before configure; never start an
        # installer as a reaction to a generic CMake error message.
        raise
    config = "Debug" if sanitizers else "Release"
    runner.run("Build all C89 libraries", ["cmake", "--build", str(build_dir), "--config", config, "--parallel"])
    runner.run("Run all C89 tests", ["ctest", "--test-dir", str(build_dir), "-C", config, "--output-on-failure"])


def ui_verify(runner: Runner) -> None:
    require_windows("Windows Reactor / WinUI 3")
    ensure_windows_linker(runner)
    if not check_ui_sources(runner.root):
        raise BuildError(
            "Manager UI manifests are not present. Merge the WinToys Manager UI implementation first."
        )
    require_command("cargo", "Install Rust through https://rustup.rs/.")
    runner.run("Manager UI tests", ["cargo", "test", "-p", "wl_ui", "--all-targets"])
    runner.run("Manager UI Clippy", ["cargo", "clippy", "-p", "wl_ui", "--all-targets", "--", "-D", "warnings"])


def build_windows_app(runner: Runner, target: str) -> None:
    if target not in WINDOWS_APPS:
        raise BuildError("Unknown modern Windows application target: " + target)
    require_manifest(runner.root, "rust/" + target + "/Cargo.toml", "Windows " + target + " application")
    require_windows(target + " application")
    ensure_windows_linker(runner)
    require_command("cargo", "Install Rust through https://rustup.rs/.")
    package, target_dir = WINDOWS_APPS[target]
    runner.run(
        "Build " + target + " Manager executable",
        ["cargo", "build", "--release", "-p", package, "--target-dir", str(runner.root / target_dir)],
    )


def cargo_metadata(runner: Runner) -> Dict[str, Any]:
    require_command("cargo", "Install Rust through https://rustup.rs/.")
    result = runner.run(
        "Read Cargo target metadata",
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        capture=True,
    )
    try:
        return json.loads(result.stdout or "{}")
    except json.JSONDecodeError as exc:
        raise BuildError("Cargo metadata returned invalid JSON.") from exc


def build_bgw(runner: Runner) -> None:
    targets = select_bgw_targets(cargo_metadata(runner))
    if not targets:
        raise BuildError(
            "No BGW binary target is declared. Add a Cargo binary named "
            "winlook.bg, winlook-bg, or winlook_bg before enabling BGW compilation."
        )
    for package, binary in targets:
        target_dir = build_paths(runner.root)["rust"] / "bgw" / package
        runner.run(
            "Build BGW executable " + binary,
            ["cargo", "build", "--release", "-p", package, "--bin", binary, "--target-dir", str(target_dir)],
        )


def build_watcom_application(runner: Runner, version: str) -> None:
    _, directory, _ = WINDOWS_GENERATIONS[version]
    makefile = runner.root / "C" / directory / "Makefile"
    if not makefile.is_file():
        raise BuildError(
            WINDOWS_GENERATIONS[version][0] + " application source is not implemented. "
            "Expected C/" + directory + "/Makefile; the build was stopped before toolchain setup."
        )
    if not os.environ.get("WATCOM"):
        raise BuildError(
            "WATCOM is not configured for " + WINDOWS_GENERATIONS[version][0] + ". "
            "Run python build.py setup --open-watcom --apply to download and verify the pinned "
            "installer, then install it manually and set WATCOM. Unattended installer switches "
            "are not yet verified."
        )
    require_command("wmake", "Install Open Watcom 2.0 and add its bin directory to PATH.")
    output = build_paths(runner.root)["c"] / "windows" / version
    build_env = {"WINLOOK_BUILD_DIR": str(output)}
    runner.run(
        "Build " + WINDOWS_GENERATIONS[version][0] + " C application with Open Watcom",
        ["wmake", "-f", "Makefile"],
        cwd=makefile.parent,
        env=build_env,
    )


def build_win95(runner: Runner) -> None:
    build_watcom_application(runner, "95")


def build_legacy_windows_c(runner: Runner, version: str) -> None:
    """Build an implemented NT-family C target with a detected MinGW toolchain."""
    _, directory, _ = WINDOWS_GENERATIONS[version]
    source = runner.root / "C" / directory
    manifest = source / "CMakeLists.txt"
    if not manifest.is_file():
        raise BuildError(
            WINDOWS_GENERATIONS[version][0] + " application source is not implemented. "
            "Expected C/" + directory + "/CMakeLists.txt; the build was stopped before toolchain setup."
        )
    if not _select_mingw_cmake(runner, required=True):
        raise BuildError(
            "This legacy target uses its OS-specific C implementation. A native MinGW-w64 GCC "
            "toolchain with Ninja, mingw32-make or MSYS make is required; a generic host GCC is not enough."
        )
    ensure_cmake(runner)
    output = build_paths(runner.root)["c"] / "windows" / version
    configure = [
        "cmake", "-S", str(source), "-B", str(output),
        "-G", runner.tool_env["CMAKE_GENERATOR"],
        "-DCMAKE_C_COMPILER=" + runner.tool_env["CC"],
        "-DCMAKE_MAKE_PROGRAM=" + runner.tool_env["CMAKE_MAKE_PROGRAM"],
        "-DCMAKE_BUILD_TYPE=Release",
    ]
    runner.run("Configure " + WINDOWS_GENERATIONS[version][0] + " C application", configure, capture=True)
    runner.run(
        "Build " + WINDOWS_GENERATIONS[version][0] + " C application",
        ["cmake", "--build", str(output), "--config", "Release", "--parallel"],
    )


def build_windows_generation(runner: Runner, version: str) -> None:
    if version not in WINDOWS_GENERATIONS:
        raise BuildError("Unknown Windows generation: " + version)
    selector = "windows." + version
    available, reason = target_available(runner.root, selector)
    if not available:
        label = WINDOWS_GENERATIONS[version][0]
        _, directory, toolchain = WINDOWS_GENERATIONS[version]
        expected = (
            "rust/" + directory + "/Cargo.toml" if toolchain == "msvc"
            else "C/" + directory + ("/Makefile" if toolchain == "watcom" else "/CMakeLists.txt")
        )
        raise BuildError(
            label + " application source is not implemented. Expected " + expected
            + ". No compiler check, installer or download was started."
        )
    _, _, toolchain = WINDOWS_GENERATIONS[version]
    if version in {"10", "11"}:
        build_windows_app(runner, "win" + version)
    elif toolchain == "watcom":
        build_watcom_application(runner, version)
    else:
        build_legacy_windows_c(runner, version)


def build_libraries(runner: Runner) -> None:
    require_command("cargo", "Install Rust through https://rustup.rs/.")
    runner.run("Build Rust shared libraries", ["cargo", "build", "-p", "wl_animate", "-p", "wl_cpuv", "-p", "wl_gpuv", "--release", "--target-dir", str(build_paths(runner.root)["rust"] / "shared")])
    cmake_c_test(runner)


def test_scope(runner: Runner, scope: str, sanitizers: bool = False) -> None:
    if scope in ("rust", "libraries", "all"):
        rust_tests(runner)
        if scope in ("libraries", "all"):
            runner.run("Rust formatting", ["cargo", "fmt", "--all", "--", "--check"])
            runner.run("Rust shared-library Clippy", ["cargo", "clippy", "-p", "wl_animate", "-p", "wl_cpuv", "-p", "wl_gpuv", "--all-targets", "--all-features", "--", "-D", "warnings"])
            runner.run("Rust shared-library API docs", ["cargo", "doc", "-p", "wl_animate", "-p", "wl_cpuv", "-p", "wl_gpuv", "--no-deps"])
    if scope in ("c", "libraries", "all"):
        cmake_c_test(runner, sanitizers=sanitizers)
    if scope in ("ui", "all"):
        if not check_ui_sources(runner.root):
            if scope == "ui":
                raise BuildError("Manager UI is not available in this checkout.")
            print_skip("Manager UI manifests are not present.")
        elif os.name != "nt":
            if scope == "ui":
                raise BuildError("The WinUI 3 Manager UI tests require a Windows host.")
            print_skip("WinUI 3 Manager UI tests require Windows; Rust/C library tests can still run here.")
        else:
            ui_verify(runner)
    if scope == "all":
        if check_ui_sources(runner.root):
            if os.name == "nt":
                build_windows_app(runner, "win10")
                build_windows_app(runner, "win11")
            else:
                print_skip("WinUI 3 builds require Windows.")
        else:
            print_skip("Manager UI source manifests are not present.")
        if not (runner.root / "C/win95/Makefile").is_file():
            print_skip("Win95 source is not present.")
        if not os.environ.get("WATCOM"):
            print_skip("Open Watcom is not configured; Win95 cross-build not run.")
        if runner.dry_run:
            print_skip("BGW target discovery is deferred in dry-run mode.")
        elif select_bgw_targets(cargo_metadata(runner)):
            print_skip("BGW target exists; use python build.py build bgw for its compile gate.")
        else:
            print_skip("No BGW Cargo binary target exists yet.")


def verify_scope(runner: Runner, scope: str, sanitizers: bool = False) -> None:
    if scope == "rust":
        rust_verify(runner)
    elif scope == "c":
        cmake_c_test(runner, sanitizers=sanitizers)
    elif scope == "libraries":
        rust_verify(runner)
        cmake_c_test(runner, sanitizers=sanitizers)
    elif scope == "ui":
        require_windows("Manager UI verification")
        runner.run("Workspace formatting", ["cargo", "fmt", "--all", "--", "--check"])
        ui_verify(runner)
    elif scope == "all":
        verify_scope(runner, "libraries", sanitizers)
        if check_ui_sources(runner.root):
            if os.name == "nt":
                verify_scope(runner, "ui", sanitizers)
                build_windows_app(runner, "win10")
                build_windows_app(runner, "win11")
            else:
                print_skip("WinUI 3 Manager builds require Windows.")
        else:
            print_skip("Manager UI source manifests are not present.")
        if (runner.root / "C/win95/Makefile").is_file() and os.environ.get("WATCOM"):
            build_win95(runner)
        else:
            print_skip("Win95 source/toolchain gate is not satisfied.")
        if not runner.dry_run and select_bgw_targets(cargo_metadata(runner)):
            build_bgw(runner)
        else:
            print_skip("BGW Cargo target is not implemented.")
    else:
        raise BuildError("Unknown verification scope: " + scope)


def package_windows_app(runner: Runner, target: str) -> Path:
    if target not in WINDOWS_APPS:
        raise BuildError("Only win10 and win11 application packages are supported.")
    _, target_dir = WINDOWS_APPS[target]
    executable = runner.root / target_dir / "release" / "winlook.exe"
    if not executable.is_file():
        raise BuildError("Missing " + str(executable) + ". Build the target before packaging.")
    output_dir = build_paths(runner.root)["artifacts"]
    archive = output_dir / package_filename(target)
    note = (
        "WinLook " + target + " Manager preview\n\n"
        "This prototype uses framework-dependent Windows App SDK deployment. "
        "Install the matching Windows App Runtime before launching the EXE:\n"
        + APP_RUNTIME_URL
        + "\n\nThis archive does not contain winlook.bg.exe; BGW is not implemented yet.\n"
    )
    if runner.dry_run:
        print("Would package " + str(executable) + " -> " + str(archive))
        return archive
    output_dir.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED) as package:
        package.write(executable, "winlook.exe")
        package.writestr("RUNTIME-REQUIREMENTS.txt", note)
    print("Created " + str(archive))
    return archive


def command_status(args: argparse.Namespace, runner: Runner) -> int:
    available, reason = target_available(runner.root, args.target)
    print(json.dumps({"target": args.target, "available": available, "reason": reason}))
    output_file = os.environ.get("GITHUB_OUTPUT")
    if args.github_output and output_file:
        with open(output_file, "a", encoding="utf-8") as handle:
            handle.write("available=" + ("true" if available else "false") + "\n")
            handle.write("reason=" + reason.replace("\n", " ") + "\n")
    return 0


def actionlint_path(root: Path, apply: bool) -> Path:
    if not sys.platform.startswith("linux") or platform.machine().lower() not in {"x86_64", "amd64"}:
        raise BuildError("Automatic actionlint bootstrap currently targets Linux x86_64.")
    destination = root / "build" / ".rcache" / "tools" / ("actionlint-" + ACTIONLINT_VERSION)
    executable = destination / "actionlint"
    if executable.is_file():
        return executable
    if not apply:
        raise BuildError(
            "actionlint is not cached. Run python build.py workflow-lint --apply "
            "to download the pinned binary and verify its SHA-256."
        )
    destination.mkdir(parents=True, exist_ok=True)
    archive_path = destination / "actionlint.tar.gz"
    request = urllib.request.Request(ACTIONLINT_URL, headers={"User-Agent": "WinLook-build.py"})
    with urllib.request.urlopen(request, timeout=120) as response:
        with archive_path.open("wb") as output:
            shutil.copyfileobj(response, output)
    actual = sha256_file(archive_path)
    if actual.lower() != ACTIONLINT_SHA256:
        archive_path.unlink(missing_ok=True)
        raise BuildError("actionlint SHA-256 mismatch; refusing to extract the archive.")
    try:
        with tarfile.open(archive_path, "r:gz") as archive:
            member = next(
                (item for item in archive.getmembers() if item.name == "actionlint" and item.isfile()),
                None,
            )
            if member is None:
                raise BuildError("Pinned actionlint archive did not contain the expected binary.")
            source = archive.extractfile(member)
            if source is None:
                raise BuildError("Could not read actionlint from the verified archive.")
            with executable.open("wb") as output:
                shutil.copyfileobj(source, output)
    finally:
        archive_path.unlink(missing_ok=True)
    executable.chmod(0o755)
    return executable


def command_workflow_lint(args: argparse.Namespace, runner: Runner) -> int:
    if runner.dry_run:
        print("Would download/verify actionlint " + ACTIONLINT_VERSION + " if needed.")
        print("Would lint .github/workflows/*.yml")
        return 0
    executable = actionlint_path(runner.root, args.apply)
    files = sorted((runner.root / ".github" / "workflows").glob("*.yml"))
    if not files:
        raise BuildError("No GitHub Actions workflow YAML files were found.")
    runner.run("Lint GitHub Actions workflows", [str(executable)] + [str(path) for path in files])
    return 0


def gitleaks_path(root: Path, apply: bool) -> Path:
    if not sys.platform.startswith("linux") or platform.machine().lower() not in {"x86_64", "amd64"}:
        raise BuildError("The automatic Gitleaks bootstrap currently targets Linux x86_64.")
    destination = build_paths(root)["rust_cache"] / "tools" / ("gitleaks-" + GITLEAKS_VERSION)
    executable = destination / "gitleaks"
    if executable.is_file():
        return executable
    if not apply:
        raise BuildError(
            "Gitleaks is not cached. Run python build.py secret-scan --apply "
            "to download the pinned binary and verify its SHA-256."
        )
    destination.mkdir(parents=True, exist_ok=True)
    archive_path = destination / "gitleaks.tar.gz"
    request = urllib.request.Request(GITLEAKS_URL, headers={"User-Agent": "WinLook-build.py"})
    with urllib.request.urlopen(request, timeout=120) as response:
        with archive_path.open("wb") as output:
            shutil.copyfileobj(response, output)
    actual = sha256_file(archive_path)
    if actual.lower() != GITLEAKS_SHA256:
        archive_path.unlink(missing_ok=True)
        raise BuildError("Gitleaks SHA-256 mismatch; refusing to extract the archive.")
    try:
        with tarfile.open(archive_path, "r:gz") as archive:
            member = next(
                (item for item in archive.getmembers() if item.name == "gitleaks" and item.isfile()),
                None,
            )
            if member is None:
                raise BuildError("Pinned Gitleaks archive did not contain the expected binary.")
            source = archive.extractfile(member)
            if source is None:
                raise BuildError("Could not read Gitleaks from the verified archive.")
            with executable.open("wb") as output:
                shutil.copyfileobj(source, output)
    finally:
        archive_path.unlink(missing_ok=True)
    executable.chmod(0o755)
    return executable


def command_secret_scan(args: argparse.Namespace, runner: Runner) -> int:
    if runner.dry_run:
        print("Would download/verify Gitleaks " + GITLEAKS_VERSION + " if needed.")
        print("Would scan the full Git history with redacted output.")
        return 0
    executable = gitleaks_path(runner.root, args.apply)
    report = build_paths(runner.root)["artifacts"] / "gitleaks.sarif"
    report.parent.mkdir(parents=True, exist_ok=True)
    runner.run(
        "Scan full Git history for exposed secrets",
        [
            str(executable), "git", "--no-banner", "--redact",
            "--report-format", "sarif", "--report-path", str(report),
            "--log-opts=--all", str(runner.root),
        ],
    )
    return 0


def command_docs(args: argparse.Namespace, runner: Runner) -> int:
    required = (
        "README.md", "plan/Plan.md", "plan/index.md", "plan/roadmap.md",
        "plan/references/SOURCES.md", "plan/architecture/build-and-ci.md",
    )
    missing = [path for path in required if not (runner.root / path).is_file()]
    if missing:
        raise BuildError("Required documentation files are missing: " + ", ".join(missing))
    print("Documentation entry-point files are present.")
    return 0


def command_clean(args: argparse.Namespace, runner: Runner) -> int:
    targets = [
        build_paths(runner.root)["rust"],
        build_paths(runner.root)["c"],
        build_paths(runner.root)["artifacts"],
    ]
    if args.all:
        targets.extend([
            build_paths(runner.root)["rust_cache"],
            build_paths(runner.root)["c_cache"],
        ])
    existing = [path for path in targets if path.exists()]
    if not args.yes:
        print("Generated output directories that would be removed:")
        for path in existing:
            print("  " + str(path))
        print("Nothing was removed. Repeat with --yes to continue.")
        return 0
    for path in existing:
        if path.is_dir():
            shutil.rmtree(path)
        else:
            path.unlink()
        print("Removed " + str(path))
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = ColorArgumentParser(description="WinLook's cross-generation Windows build system.")
    parser.add_argument("--root", default=str(ROOT), help="Repository root; defaults to the directory containing build.py.")
    parser.add_argument("--debug", action="store_true", help="Show detailed build and Visual Studio Installer activity.")
    parser.add_argument("--all-debug", action="store_true", help="Show all captured output and save the full session to debug.log.")
    sub = parser.add_subparsers(dest="action", required=True, parser_class=ColorArgumentParser)

    for version, (label, _, toolchain) in WINDOWS_GENERATIONS.items():
        sub.add_parser(
            "windows." + version,
            help="Build " + label + " (toolchain: " + toolchain + ")",
            description=(
                label + " target. The runner checks the target's source manifest before "
                "selecting or installing any compiler."
            ),
        ).add_argument("--dry-run", action="store_true")

    doctor = sub.add_parser("doctor", help="Inspect installed build tools and available project targets.")
    doctor.add_argument("--json", action="store_true")
    doctor.add_argument("--strict", action="store_true")

    setup = sub.add_parser("setup", help="Plan or install user-scoped build prerequisites.")
    setup.add_argument("--apply", action="store_true", help="Execute setup; otherwise show the plan only.")
    setup.add_argument("--with-rust", action="store_true", help="Select existing Rust 1.95.0 or install it side-by-side without changing the default toolchain.")
    setup.add_argument("--with-c", action="store_true", help="Reuse installed MSVC or MinGW GCC, then ensure CMake is available.")
    setup.add_argument("--with-cmake", action="store_true", help="Install CMake if missing.")
    setup.add_argument("--open-watcom", action="store_true", help="Install the pinned Open Watcom Linux x64 package.")
    setup.add_argument("--dry-run", action="store_true")

    fmt = sub.add_parser("fmt", help="Format Rust code or check formatting.")
    fmt.add_argument("--check", action="store_true")
    fmt.add_argument("--dry-run", action="store_true")

    test = sub.add_parser("test", help="Run tests for a project area.")
    test.add_argument("scope", choices=("rust", "c", "libraries", "ui", "all"))
    test.add_argument("--sanitizers", action="store_true")
    test.add_argument("--dry-run", action="store_true")

    build = sub.add_parser("build", help="Build shared libraries, UI, or a legacy target (compatibility aliases).")
    build.add_argument("target", choices=("libraries", "ui", "win10", "win11", "win95", "bgw", "all"))
    build.add_argument("--dry-run", action="store_true")

    verify = sub.add_parser("verify", help="Run the full verification gate for a selected area.")
    verify.add_argument("target", choices=("rust", "c", "libraries", "ui", "all"))
    verify.add_argument("--sanitizers", action="store_true")
    verify.add_argument("--dry-run", action="store_true")

    status = sub.add_parser("status", help="Report whether a build target has source files.")
    status.add_argument("target", choices=("libraries", "ui", "win10", "win11", "win95", "bgw"))
    status.add_argument("--github-output", action="store_true", help="Write an available output for workflow gates.")

    secret_scan = sub.add_parser("secret-scan", help="Download a pinned Gitleaks binary and scan Git history.")
    secret_scan.add_argument("--apply", action="store_true", help="Download and SHA-256 verify Gitleaks if missing.")
    secret_scan.add_argument("--dry-run", action="store_true")

    workflow_lint = sub.add_parser("workflow-lint", help="Download a pinned actionlint binary and lint workflow YAML.")
    workflow_lint.add_argument("--apply", action="store_true", help="Download and SHA-256 verify actionlint if missing.")
    workflow_lint.add_argument("--dry-run", action="store_true")

    package = sub.add_parser("package", help="Package a previously built Windows Manager executable.")
    package.add_argument("target", choices=("win10", "win11"))
    package.add_argument("--dry-run", action="store_true")

    docs = sub.add_parser("docs", help="Check required project documentation entry points.")
    docs.add_argument("--dry-run", action="store_true")

    clean = sub.add_parser("clean", help="Remove generated build outputs only.")
    clean.add_argument("--yes", action="store_true")
    clean.add_argument("--all", action="store_true", help="Also remove Rust and C compiler caches.")
    return parser


def _extract_debug_flags(arguments: Sequence[str]) -> Tuple[List[str], bool, bool]:
    """Accept global debug switches before or after a subcommand."""
    cleaned: List[str] = []
    debug = False
    all_debug = False
    for argument in arguments:
        if argument == "--debug":
            debug = True
        elif argument == "--all-debug":
            all_debug = True
        else:
            cleaned.append(argument)
    return cleaned, debug, all_debug


class TeeTextIO:
    """Mirror text written to the console into a diagnostic file."""

    def __init__(self, primary: Any, mirror: Any) -> None:
        self.primary = primary
        self.mirror = mirror

    @property
    def encoding(self) -> str:
        return getattr(self.primary, "encoding", None) or "utf-8"

    def write(self, value: str) -> int:
        self.primary.write(value)
        self.mirror.write(value)
        return len(value)

    def flush(self) -> None:
        self.primary.flush()
        self.mirror.flush()

    def isatty(self) -> bool:
        try:
            return bool(self.primary.isatty())
        except (AttributeError, OSError):
            return False


def run_with_all_debug_log(root: Path, action: Callable[[], int]) -> int:
    """Capture the complete run in root/debug.log while keeping console output visible."""
    destination = root / "debug.log"
    try:
        log_file = destination.open("w", encoding="utf-8", buffering=1)
    except OSError as exc:
        raise BuildError("Could not create the requested debug log at " + str(destination) + ": " + str(exc)) from exc

    global _ALL_DEBUG_ACTIVE
    previous_debug_state = _ALL_DEBUG_ACTIVE
    _ALL_DEBUG_ACTIVE = True
    try:
        with log_file:
            stdout = TeeTextIO(sys.stdout, log_file)
            stderr = TeeTextIO(sys.stderr, log_file)
            with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
                print("=== WinLook all-debug session: " + time.strftime("%Y-%m-%d %H:%M:%S") + " ===")
                print("Python: " + sys.version.replace("\n", " "))
                print("Arguments: " + shlex.join([str(value) for value in sys.argv[1:]]))
                result = action()
                print("=== WinLook finished with exit code " + str(result) + " ===")
                print("Full diagnostic log: " + str(destination))
                return result
    except OSError as exc:
        raise BuildError("Could not write the requested debug log at " + str(destination) + ": " + str(exc)) from exc
    finally:
        _ALL_DEBUG_ACTIVE = previous_debug_state


def _run_action(args: argparse.Namespace, runner: Runner) -> int:
    try:
        needs_rust = (
            args.action == "fmt"
            or (args.action in {"windows.10", "windows.11"} and check_windows_app(runner.root, "win" + args.action[len("windows.") :]))
            or (args.action == "build" and args.target in {"libraries", "ui", "win10", "win11", "bgw", "all"})
            or (args.action == "verify" and args.target in {"rust", "libraries", "ui", "all"})
            or (args.action == "test" and args.scope in {"rust", "libraries", "ui", "all"})
        )
        if needs_rust:
            ensure_rust_toolchain(runner, apply=True)
        if args.action in WINDOWS_BUILD_COMMANDS:
            build_windows_generation(runner, args.action[len("windows.") :])
            return 0
        if args.action == "doctor":
            return command_doctor(args, runner)
        if args.action == "setup":
            return command_setup(args, runner)
        if args.action == "fmt":
            require_command("cargo", "Install Rust through https://rustup.rs/.")
            suffix = ["--", "--check"] if args.check else []
            runner.run("Check Rust formatting" if args.check else "Format Rust workspace", ["cargo", "fmt", "--all"] + suffix)
            return 0
        if args.action == "test":
            test_scope(runner, args.scope, args.sanitizers)
            return 0
        if args.action == "build":
            if args.target == "libraries":
                build_libraries(runner)
            elif args.target == "ui":
                ui_verify(runner)
                build_windows_app(runner, "win10")
                build_windows_app(runner, "win11")
            elif args.target in WINDOWS_APPS:
                build_windows_app(runner, args.target)
            elif args.target == "win95":
                build_win95(runner)
            elif args.target == "bgw":
                build_bgw(runner)
            elif args.target == "all":
                build_libraries(runner)
                if os.name == "nt" and check_ui_sources(runner.root):
                    build_windows_app(runner, "win10")
                    build_windows_app(runner, "win11")
                else:
                    print_skip("Modern Manager build requires its source manifests and Windows.")
                if (runner.root / "C/win95/Makefile").is_file() and os.environ.get("WATCOM"):
                    build_win95(runner)
                else:
                    print_skip("Win95 source/toolchain gate is not satisfied.")
                if not runner.dry_run and select_bgw_targets(cargo_metadata(runner)):
                    build_bgw(runner)
                else:
                    print_skip("BGW Cargo target is not implemented.")
            return 0
        if args.action == "verify":
            verify_scope(runner, args.target, args.sanitizers)
            return 0
        if args.action == "package":
            package_windows_app(runner, args.target)
            return 0
        if args.action == "docs":
            return command_docs(args, runner)
        if args.action == "status":
            return command_status(args, runner)
        if args.action == "workflow-lint":
            return command_workflow_lint(args, runner)
        if args.action == "secret-scan":
            return command_secret_scan(args, runner)
        if args.action == "clean":
            return command_clean(args, runner)
    except KeyboardInterrupt:
        print("\n[SETUP CANCELLED] The running command was interrupted. No automatic retry will be started.", file=sys.stderr)
        return 130
    except BuildError as exc:
        print("\n[BUILD ERROR] " + str(exc), file=sys.stderr)
        return 1
    return 2


def main(argv: Optional[Sequence[str]] = None) -> int:
    parser = build_parser()
    raw_arguments = list(sys.argv[1:] if argv is None else argv)
    arguments, debug_flag, all_debug_flag = _extract_debug_flags(raw_arguments)
    # Running the entry point without an action should be useful, not an argparse error.
    if not arguments:
        arguments = ["doctor"]
    args = parser.parse_args(arguments)
    args.debug = bool(getattr(args, "debug", False) or debug_flag or all_debug_flag)
    args.all_debug = bool(getattr(args, "all_debug", False) or all_debug_flag)
    root = Path(args.root).expanduser().resolve()
    if not (root / "Cargo.toml").is_file() or not (root / "README.md").is_file():
        parser.error("--root must point to the WinLook repository root.")
    runner = Runner(
        root=root,
        dry_run=bool(getattr(args, "dry_run", False)),
        verbose=bool(os.environ.get("WINLOOK_BUILD_VERBOSE")),
        debug=args.debug,
        all_debug=args.all_debug,
    )
    if runner.all_debug:
        try:
            return run_with_all_debug_log(root, lambda: _run_action(args, runner))
        except BuildError as exc:
            print("\n[BUILD ERROR] " + str(exc), file=sys.stderr)
            return 1
    return _run_action(args, runner)




if __name__ == "__main__":
    raise SystemExit(main())
