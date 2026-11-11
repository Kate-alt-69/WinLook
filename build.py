#!/usr/bin/env python3
"""WinLook's cross-platform build, test, packaging, and toolchain entry point.

Requires Python 3.9+. Uses only the Python standard library and invokes native
tools with argument arrays (never through a shell). It does not install
system-wide compilers or change PATH unless an explicit setup option is used.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import shlex
import shutil
import subprocess
import tarfile
import sys
import urllib.request
import zipfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Sequence, Tuple

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
APP_RUNTIME_URL = (
    "https://learn.microsoft.com/en-us/windows/apps/windows-app-sdk/downloads"
)
BGW_TARGET_NAMES = {"winlook.bg", "winlook-bg", "winlook_bg"}
WINDOWS_APPS = {
    "win10": ("winlook-win10", "target/win10"),
    "win11": ("winlook-win11", "target/win11"),
}


class BuildError(RuntimeError):
    """An actionable build or toolchain error."""


@dataclass
class Runner:
    root: Path = ROOT
    dry_run: bool = False
    verbose: bool = False

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
        rendered = subprocess.list2cmdline(args) if os.name == "nt" else shlex.join(args)
        print("\n==> " + label)
        print("$ " + rendered)
        if self.dry_run:
            return subprocess.CompletedProcess(args, 0, stdout="", stderr="")
        try:
            result = subprocess.run(
                args, cwd=str(location), env=env, check=False, text=True,
                capture_output=capture,
            )
        except FileNotFoundError as exc:
            raise BuildError(
                "Required command was not found: " + args[0]
                + ". Run python build.py doctor for toolchain guidance."
            ) from exc
        if result.returncode != 0:
            detail = ""
            if capture:
                detail = (result.stderr or result.stdout or "").strip()
            raise BuildError(
                label + " failed with exit code " + str(result.returncode)
                + (("\n" + detail[-5000:]) if detail else "")
            )
        if capture and self.verbose and result.stdout:
            print(result.stdout)
        return result


def command_path(name: str) -> Optional[str]:
    return shutil.which(name)


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


def print_skip(message: str) -> None:
    print("[SKIPPED] " + message)


def command_doctor(args: argparse.Namespace, runner: Runner) -> int:
    checks = {
        "python": sys.version.split()[0],
        "platform": platform.platform(),
        "architecture": platform.machine(),
        "git": command_path("git"),
        "rustup": command_path("rustup"),
        "rustc": command_path("rustc"),
        "cargo": command_path("cargo"),
        "rustfmt": command_path("rustfmt"),
        "clippy-driver": command_path("clippy-driver"),
        "cmake": command_path("cmake"),
        "cmake_generator": os.environ.get("CMAKE_GENERATOR"),
        "msvc_cl": command_path("cl"),
        "gcc": command_path("gcc"),
        "clang": command_path("clang"),
        "open_watcom_root": os.environ.get("WATCOM"),
        "open_watcom_wcc386": command_path("wcc386"),
        "open_watcom_wmake": command_path("wmake"),
        "manager_ui_sources": check_ui_sources(runner.root),
        "win10_project": check_windows_app(runner.root, "win10"),
        "win11_project": check_windows_app(runner.root, "win11"),
        "win95_makefile": project_has_manifest(runner.root, "C/win95/Makefile"),
    }
    missing = []
    if sys.version_info < PYTHON_MINIMUM:
        missing.append("Python 3.9 or newer")
    for tool in ("cargo", "rustc", "cmake"):
        if not checks[tool]:
            missing.append(tool)
    if args.json:
        print(json.dumps({"root": str(runner.root), "checks": checks, "missing": missing}, indent=2))
    else:
        print("WinLook build environment")
        print("Repository: " + str(runner.root))
        for key, value in checks.items():
            print("  {:30} {}".format(key + ":", "not found" if value is None else str(value)))
        if missing:
            print("\nMissing baseline tools: " + ", ".join(missing))
        print("\nWindows UI builds require Visual Studio C++ Build Tools.")
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


def install_open_watcom(runner: Runner, apply: bool) -> None:
    if not sys.platform.startswith("linux") or platform.machine().lower() not in {"x86_64", "amd64"}:
        raise BuildError(
            "Automated Open Watcom setup is pinned for Linux x86_64. "
            "Install it manually on other hosts and set WATCOM."
        )
    destination = runner.root / ".toolchains" / ("open-watcom-" + WATCOM_RELEASE)
    compiler = destination / "binl64" / "wcc386"
    if compiler.is_file():
        print("Open Watcom already exists at " + str(destination))
    elif not apply:
        print("Plan: download the official Open Watcom asset:")
        print("  " + WATCOM_ASSET_URL)
        print("Verify SHA-256: " + WATCOM_ASSET_SHA256)
        print("Install into: " + str(destination))
        print("No files were downloaded. Repeat with --apply.")
        return
    else:
        destination.parent.mkdir(parents=True, exist_ok=True)
        installer = runner.root / ".toolchains" / "open-watcom-installer"
        request = urllib.request.Request(
            WATCOM_ASSET_URL, headers={"User-Agent": "WinLook-build.py"}
        )
        print("Downloading pinned Open Watcom 2.0 toolchain asset...")
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
        installer.chmod(0o755)
        destination.mkdir(parents=True, exist_ok=True)
        runner.run(
            "Install verified Open Watcom 2.0",
            [str(installer), "-s", "--install-dir=" + str(destination)],
        )
        installer.unlink(missing_ok=True)
    if not compiler.is_file():
        raise BuildError(
            "Open Watcom installer did not produce " + str(compiler)
            + ". Inspect installer output before retrying."
        )
    include = destination / "h"
    append_github_environment("WATCOM", str(destination))
    append_github_environment("INCLUDE", str(include) + os.pathsep + str(include / "nt"))
    append_github_environment("EDPATH", str(destination / "eddat"))
    append_github_path(str(destination / "binl64"))
    print("Open Watcom verified at " + str(destination))
    print("WATCOM, INCLUDE, EDPATH and PATH exported when running in GitHub Actions.")


def command_setup(args: argparse.Namespace, runner: Runner) -> int:
    actions: List[List[str]] = []
    if not command_path("rustup"):
        if args.install_rustup:
            if os.name != "nt" or not command_path("winget"):
                raise BuildError(
                    "Automatic rustup installation uses WinGet on Windows. "
                    "Install Rust from https://rustup.rs/ and retry."
                )
            actions.append(["winget", "install", "--id", "Rustlang.Rustup", "--exact"])
        else:
            print("Rustup is missing. Install it from https://rustup.rs/.")
    else:
        actions.append([
            "rustup", "toolchain", "install", "stable", "--profile", "minimal",
            "--component", "rustfmt", "--component", "clippy",
        ])
    if args.with_cmake and not command_path("cmake"):
        if os.name != "nt" or not command_path("winget"):
            raise BuildError(
                "CMake is missing. Install it from https://cmake.org/download/ "
                "or use your OS package manager."
            )
        actions.append(["winget", "install", "--id", "Kitware.CMake", "--exact"])
    if args.open_watcom:
        install_open_watcom(runner, args.apply and not runner.dry_run)
    for action in actions:
        print("Planned setup: " + shlex.join(action))
    if not args.apply:
        print("Nothing was installed. Repeat with --apply to execute the listed setup.")
        return 0
    for action in actions:
        if action[0] == "winget":
            action.extend(["--accept-package-agreements", "--accept-source-agreements"])
        runner.run("Install requested build prerequisite", action)
    print("Setup commands completed. Run python build.py doctor to verify.")
    return 0


def rust_verify(runner: Runner) -> None:
    require_command("cargo", "Install Rust through https://rustup.rs/.")
    commands = [
        ("Rust formatting", ["cargo", "fmt", "--all", "--", "--check"]),
        ("Rust workspace tests", ["cargo", "test", "--workspace", "--all-targets", "--all-features"]),
        ("Rust Clippy", ["cargo", "clippy", "--workspace", "--all-targets", "--all-features", "--", "-D", "warnings"]),
        ("Rust API documentation", ["cargo", "doc", "--workspace", "--no-deps"]),
    ]
    for label, command in commands:
        runner.run(label, command)


def rust_tests(runner: Runner) -> None:
    require_command("cargo", "Install Rust through https://rustup.rs/.")
    runner.run("Rust workspace unit tests", ["cargo", "test", "--workspace", "--all-targets", "--all-features"])


def cmake_c_test(runner: Runner, sanitizers: bool = False) -> None:
    require_command("cmake", "Install CMake from https://cmake.org/download/.")
    source = runner.root / "C" / "library"
    if not (source / "CMakeLists.txt").is_file():
        raise BuildError("C/library/CMakeLists.txt is missing.")
    build_dir = runner.root / "build" / "c-library"
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
    runner.run("Configure all C89 libraries", configure)
    runner.run("Build all C89 libraries", ["cmake", "--build", str(build_dir), "--config", "Release", "--parallel"])
    runner.run("Run all C89 tests", ["ctest", "--test-dir", str(build_dir), "-C", "Release", "--output-on-failure"])


def ui_verify(runner: Runner) -> None:
    require_windows("Windows Reactor / WinUI 3")
    if not check_ui_sources(runner.root):
        raise BuildError(
            "Manager UI manifests are not present. Merge the WinToys Manager UI implementation first."
        )
    require_command("cargo", "Install Rust through https://rustup.rs/.")
    runner.run("Manager UI tests", ["cargo", "test", "-p", "wl_ui", "--all-targets"])
    runner.run("Manager UI Clippy", ["cargo", "clippy", "-p", "wl_ui", "--all-targets", "--", "-D", "warnings"])


def build_windows_app(runner: Runner, target: str) -> None:
    require_windows(target + " application")
    if target not in WINDOWS_APPS:
        raise BuildError("Unknown modern Windows application target: " + target)
    require_manifest(runner.root, "rust/" + target + "/Cargo.toml", "Windows " + target + " application")
    require_command("cargo", "Install Rust through https://rustup.rs/.")
    package, target_dir = WINDOWS_APPS[target]
    runner.run(
        "Build " + target + " Manager executable",
        ["cargo", "build", "--release", "-p", package, "--target-dir", target_dir],
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
        target_dir = runner.root / "target" / "bgw" / package
        runner.run(
            "Build BGW executable " + binary,
            ["cargo", "build", "--release", "-p", package, "--bin", binary, "--target-dir", target_dir],
        )


def build_win95(runner: Runner) -> None:
    makefile = runner.root / "C" / "win95" / "Makefile"
    if not makefile.is_file():
        raise BuildError(
            "Windows 95 application source is not implemented. Expected C/win95/Makefile; "
            "no legacy binary will be fabricated."
        )
    if not os.environ.get("WATCOM"):
        raise BuildError(
            "WATCOM is not configured. Run python build.py setup --open-watcom --apply "
            "on Linux x86_64, or install Open Watcom manually and set WATCOM."
        )
    require_command("wmake", "Install Open Watcom 2.0 and add its bin directory to PATH.")
    runner.run("Build Windows 95 C application with Open Watcom", ["wmake", "-f", "Makefile"], cwd=makefile.parent)


def build_libraries(runner: Runner) -> None:
    require_command("cargo", "Install Rust through https://rustup.rs/.")
    runner.run("Build Rust shared libraries", ["cargo", "build", "--workspace", "--release"])
    cmake_c_test(runner)


def test_scope(runner: Runner, scope: str, sanitizers: bool = False) -> None:
    if scope in ("rust", "libraries", "all"):
        rust_tests(runner)
        if scope in ("libraries", "all"):
            runner.run("Rust formatting", ["cargo", "fmt", "--all", "--", "--check"])
            runner.run("Rust Clippy", ["cargo", "clippy", "--workspace", "--all-targets", "--all-features", "--", "-D", "warnings"])
            runner.run("Rust API docs", ["cargo", "doc", "--workspace", "--no-deps"])
    if scope in ("c", "libraries", "all"):
        cmake_c_test(runner, sanitizers=sanitizers)
    if scope in ("ui", "all"):
        if check_ui_sources(runner.root):
            ui_verify(runner)
        elif scope == "ui":
            raise BuildError("Manager UI is not available in this checkout.")
        else:
            print_skip("Manager UI manifests are not present.")
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
        if not runner.dry_run and not select_bgw_targets(cargo_metadata(runner)):
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
    output_dir = runner.root / "dist"
    output_dir.mkdir(parents=True, exist_ok=True)
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
    with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED) as package:
        package.write(executable, "winlook.exe")
        package.writestr("RUNTIME-REQUIREMENTS.txt", note)
    print("Created " + str(archive))
    return archive


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
    targets = [runner.root / "build", runner.root / "target", runner.root / "dist"]
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
    parser = argparse.ArgumentParser(description="WinLook's Python build and verification entry point.")
    parser.add_argument("--root", default=str(ROOT), help="Repository root; defaults to the directory containing build.py.")
    sub = parser.add_subparsers(dest="action", required=True)

    doctor = sub.add_parser("doctor", help="Inspect installed build tools and available project targets.")
    doctor.add_argument("--json", action="store_true")
    doctor.add_argument("--strict", action="store_true")

    setup = sub.add_parser("setup", help="Plan or install user-scoped build prerequisites.")
    setup.add_argument("--apply", action="store_true", help="Execute setup; otherwise show the plan only.")
    setup.add_argument("--install-rustup", action="store_true", help="Allow WinGet to install rustup if missing.")
    setup.add_argument("--with-cmake", action="store_true", help="Allow WinGet to install CMake if missing.")
    setup.add_argument("--open-watcom", action="store_true", help="Install the pinned Open Watcom Linux x64 package.")
    setup.add_argument("--dry-run", action="store_true")

    fmt = sub.add_parser("fmt", help="Format Rust code or check formatting.")
    fmt.add_argument("--check", action="store_true")
    fmt.add_argument("--dry-run", action="store_true")

    test = sub.add_parser("test", help="Run tests for a project area.")
    test.add_argument("scope", choices=("rust", "c", "libraries", "ui", "all"))
    test.add_argument("--sanitizers", action="store_true")
    test.add_argument("--dry-run", action="store_true")

    build = sub.add_parser("build", help="Build a selected project area.")
    build.add_argument("target", choices=("libraries", "ui", "win10", "win11", "win95", "bgw", "all"))
    build.add_argument("--dry-run", action="store_true")

    verify = sub.add_parser("verify", help="Run the full verification gate for a selected area.")
    verify.add_argument("target", choices=("rust", "c", "libraries", "ui", "all"))
    verify.add_argument("--sanitizers", action="store_true")
    verify.add_argument("--dry-run", action="store_true")

    status = sub.add_parser("status", help="Report whether a build target has source files.")
    status.add_argument("target", choices=("libraries", "ui", "win10", "win11", "win95", "bgw"))
    status.add_argument("--github-output", action="store_true", help="Write an available output for workflow gates.")

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
    return parser


def main(argv: Optional[Sequence[str]] = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    root = Path(args.root).expanduser().resolve()
    if not (root / "Cargo.toml").is_file() or not (root / "README.md").is_file():
        parser.error("--root must point to the WinLook repository root.")
    runner = Runner(
        root=root,
        dry_run=bool(getattr(args, "dry_run", False)),
        verbose=bool(os.environ.get("WINLOOK_BUILD_VERBOSE")),
    )
    try:
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
        if args.action == "clean":
            return command_clean(args, runner)
    except BuildError as exc:
        print("\n[BUILD ERROR] " + str(exc), file=sys.stderr)
        return 1
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
