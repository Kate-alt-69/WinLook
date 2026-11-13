# WinLook

**Customize Windows itself. Keep Windows' native components in charge.**

WinLook is a planned Windows customization platform for desktop appearance, windows, displays, notifications, media controls, File Explorer and user-defined profiles. It is designed to extend Windows rather than replace its Shell, filesystem, window manager, compositor or security infrastructure.

The project aims to cover Windows 95 and later, but that ambition is not a release-support claim. Each feature must use an OS-appropriate adapter, report partial or unsupported capabilities honestly, and retain a safe Windows-native fallback.

## Project status

**Early implementation; shared libraries are in place and the Win10/Win11 Manager has an initial native UI prototype.**

This repository is not yet a complete Windows application. It contains the living architecture/compatibility plan, portable shared-library cores and an initial native WinUI 3 Manager shell. The BGW background runtime and OS-specific system-integration implementations remain future work. It does **not** yet contain a complete WinLook Manager, desktop runtime, Explorer integration, installer or supported Windows release.

The first implementation work is focused on the paired animation libraries, both using **Windows 10 as their initial target baseline**:
- [`wl_animate`](rust/library/wl_animate/README.md) — Rust animation, vector math and SVG path-data core.
- [`wl_Lanimate`](C/library/wl_Lanimate/README.md) — C89-compatible animation and vector-math core.
- [`wl_cpuv`](rust/library/wl_cpuv/README.md) / [`wl_Lcpuv`](C/library/wl_Lcpuv/README.md) — CPU work admission/accounting cores.
- [`wl_gpuv`](rust/library/wl_gpuv/README.md) / [`wl_Lgpuv`](C/library/wl_Lgpuv/README.md) — GPU memory/submission admission cores.

Both cores are renderer-independent: no Windows drawing backend, timer, scheduler, window hook or process injection is included yet. C89 is a source-language constraint, not a claim of support on pre-Windows-10 releases. See the [Animation Suite plan](plan/architecture/animation-suite.md) for implementation status and remaining gates.

## Design principles

- Windows remains responsible for filesystem operations, Shell activation, DWM composition, authentication and secure desktop.
- Explorer remains Explorer. WinLook customizes its visible interface without replacing the file manager or modifying `explorer.exe` on disk.
- Prefer documented APIs and isolated adapters. Deeper per-build customization is opt-in, independently disableable and recoverable.
- Profiles coordinate existing engines; Templates (`.wl.t`) package selected portable settings and assets.
- Modern and legacy builds use separate implementation/toolchain paths. Similar APIs do not imply one binary or identical feature coverage.
- Security, accessibility, performance, privacy and restoration are architectural requirements.

## Planned feature areas

| Area | Responsibility |
|---|---|
| Appearance and Icons | Themes, system/item icons and supported Shell appearance |
| WLDE | Desktop presentation, wallpapers, widgets, Groups and resource-aware rendering |
| WLWM | Window layouts, rules, workspaces and eligible target integrations |
| WLDM | Monitor discovery, display profiles and per-display settings |
| WLOSDE | WinLook notifications, popups and OSD |
| Audio & Media | Endpoint/session audio controls and provider-backed Now Playing |
| File Explorer | Customization of the existing Explorer interface |
| Clipboard | Privacy-aware interface layered on Windows Clipboard History |
| Profiles and WLT | Coordinated configurations and portable settings/assets |

RGB/peripheral lighting is intended to integrate with existing applications where documented interfaces exist; WinLook is not planning to reimplement vendor hardware support. Startup-application management is out of scope.

## Windows and toolchain strategy

The Rust standard MSVC target now requires Windows 10 or later, but WinLook keeps older Win32 generations on deliberately separate backends:

- **Windows 11:** Rust/MSVC + native WinUI 3, built with Visual Studio 2026 and Windows SDK 10.0.26100.
- **Windows 10:** Rust/MSVC + lightweight native Win32/User32, built with Visual Studio 2022 and Windows SDK 10.0.19041. It does not link Windows App SDK, Windows Reactor, WebView2 or the Tauri adapter.
- **Windows 7/8/8.1:** experimental classic Win32 backend. Rust 1.77 is the last stable toolchain with built-in Windows 7 support; newer Rust can use the Tier 3 `*-win7-windows-msvc` custom target, which is not distributed as a prebuilt rustup component.
- **Windows 95 through Vista:** separate C89 `wl_Lui` Win32 library and OS-specific compiler/runtime adapters.
- **macOS/Linux and cross-platform UI:** optional Tauri 2 adapter, using each platform's system WebView. Tauri 2.12 raised its MSRV to 1.90 and dropped official Windows 7 support.

These are code/toolchain routes, not OS-support claims. Every adapter still requires real runtime testing on its oldest declared Windows build.

Sources: [Rust MSVC targets](https://doc.rust-lang.org/rustc/platform-support/windows-msvc.html) · [Rust Windows 7 MSVC target](https://doc.rust-lang.org/rustc/platform-support/win7-windows-msvc.html) · [Tauri 2.12 release](https://v2.tauri.app/blog/tauri-2.12/).

## Runtime architecture

WinLook uses two principal executables: `winlook.exe` is the Manager UI, and `winlook.bg.exe` is the startup-launched BGW background runtime. BGW hosts and coordinates WLDE, WLWM, WLDM, WLOSDE and the other background subsystems, and remains active when the Manager closes. The Manager communicates with BGW through authenticated, versioned local IPC. See the [Background Runtime architecture](plan/architecture/background-runtime.md).

CPUv/GPUv are core resource-governance libraries for WinLook-managed CPU and GPU work. Under pressure, WinLook defers or pauses its own work, freezes the last displayed frame where appropriate and releases recoverable resources; it does not automatically lower wallpaper resolution or degrade visual quality. See [Resource Governors](plan/architecture/resource-governors.md).

## Documentation

Start with these canonical documents:

- [Living project plan](plan/Plan.md) — accepted direction, open decisions and checklist.
- [Workspace index](plan/index.md) — navigation across the planning set.
- [Detailed roadmap](plan/roadmap.md) — dependency order and acceptance gates.
- [Architecture overview](plan/architecture/overview.md) — engines and ownership boundaries.
- [Compatibility matrix](plan/compatibility/matrix.md) — OS-specific capability baseline.
- [Compatibility test strategy](plan/compatibility/test-strategy.md) — evidence labels and test requirements.
- [Technical source ledger](plan/references/SOURCES.md) — primary API/toolchain sources and their limits.

Detailed specifications live in [`plan/architecture/`](plan/architecture/overview.md); per-version research lives in [`plan/compatibility/`](plan/compatibility/README.md).

## Manager UI prototype

The public [wl_ui crate](rust/library/wl_ui/README.md) exposes backend modules from one import path: `wl_ui::windows10`, `wl_ui::windows11`, `wl_ui::windows7` and `wl_ui::tauri2`. Shared application/page types stay at the crate root; the OS renderer crates remain private optional dependencies. The Windows 10 executable uses Win32/User32, while Windows 11 uses native WinUI 3. `wl_Lui` is the separate C89 baseline for legacy Win32 applications. The Manager pages are still preview scaffolds and do not apply system settings until BGW and IPC are implemented. See [UI Architecture](plan/architecture/ui-architecture.md).

## Building WinLook

The root Python runner is the common entry point for building, testing and packaging. It requires Python 3.9+ and uses only the standard library. Running it without an action performs a read-only environment diagnostic:

The same `python build.py ...` commands work from **Windows PowerShell 5.1, PowerShell 7, CMD, and Git Bash**; no shell-specific wrapper or escaping is required. WinLook starts native tools with argument arrays and uses the Windows `COMSPEC` command processor only when it must load Visual Studio `.bat` setup scripts. Windows MSVC/WinUI builds still require Windows; Bash on Linux/macOS can run the host-supported build and verification commands.

```sh
python build.py
python build.py doctor
python build.py doctor --json
python build.py setup --with-rust --apply
python build.py setup --with-c --apply
python build.py verify libraries --sanitizers
python build.py windows.95
python build.py windows.vista
python build.py windows.7
python build.py windows.10
python build.py windows.11
python build.py package win10

# Focused build and Visual Studio Installer diagnostics
python build.py build win10 --debug
python build.py build win10 --all-debug
```

Use the `windows.<version>` selectors for OS-specific application builds: `windows.95`, `windows.98`, `windows.me`, `windows.nt4`, `windows.2000`, `windows.xp`, `windows.vista`, `windows.7`, `windows.8`, `windows.8.1`, `windows.10` and `windows.11`. Run `python build.py --help` for the complete list; terminal help uses ANSI colors when supported and respects `NO_COLOR`.

Each selector checks that generation's application manifest **before** probing or installing a compiler. An unimplemented generation reports the expected source path and exits without downloading anything. Windows 95/98/Me use the Open Watcom legacy C path; NT 4 through Windows 8.1 use their own C/CMake target and a native MinGW-w64 GCC toolchain where the target source is implemented; Windows 10/11 use the existing Rust/WinUI 3 implementation and require the MSVC linker plus a Windows SDK.

`python build.py doctor --json` reports MSVC tools and Windows SDK discovery separately, including a resolved SDK root/version when found.



By default, successful subprocess output and raw command lines are kept quiet while high-level build stages remain visible. Add `--debug` before or after a command (for example, `python build.py build win10 --debug`) to show the command line and live Visual Studio Installer activity such as channel checks, package downloads, package installation and warnings/errors. `--all-debug` includes all captured subprocess output and raw Visual Studio setup-log lines, and writes the complete run to the repository-root `debug.log` (overwritten on each run and ignored by Git).

Compiler discovery reuses what is already installed. For legacy C, the runner checks GCC's `-dumpmachine` output and requires a MinGW target plus Ninja, MinGW Make or MSYS Make; a generic Linux GCC executable is not accepted as a Windows compiler. For Win10/Win11, it scans the current environment, Visual Studio Installer metadata (`vswhere` and the registry), and normal installation locations, then loads the existing developer environment and verifies `cl`, `link`, `rc`, Windows SDK headers and libraries. If a Visual Studio instance exists but lacks the C++ toolchain, WinLook reads that exact instance's channel ID from `vswhere` and modifies it through the installed `setup.exe` using a hidden, elevated UAC operation. The request explicitly adds the x64 MSVC tools and Windows 10 SDK 19041 rather than relying on recommended workload defaults. If no instance exists, or an existing non-Build-Tools instance still cannot provide the compiler/linker/SDK after modification, it makes at most one standalone Build Tools install attempt. It uses WinGet when available; otherwise it downloads the official [Visual Studio 2022 Build Tools bootstrapper](https://aka.ms/vs/17/release/vs_buildtools.exe), caches it under `build/.rcache/installers/`, and verifies its Microsoft Authenticode signature before execution. An existing Build Tools instance is never installed over, a failed first install is not immediately retried, and declining UAC does not start a second installer. Existing MinGW GCC is reported but cannot replace the MSVC ABI/linker and Windows SDK needed by the current WinUI app. CMake is installed only when its executable is missing. Visual Studio Build Tools are machine-wide and may require administrator approval. The Windows SDK probe does not depend only on `WindowsSdkDir`: it also checks the Windows Kits registry root and the standard `Windows Kits/10` locations, validates versioned headers/import libraries, and fills missing SDK/MSVC `PATH`, `INCLUDE` and `LIB` entries in the build process environment. After setup exits, WinLook verifies the actual compiler/linker/SDK again; a zero process exit with `NoUpdate`, canceled channel feeds and no package-apply activity is reported as an unconfirmed/no-op modification rather than a successful toolchain install.

Win10/Win11 builds first check the Rust installations already available through rustup. If Rust 1.95.0 is installed globally, WinLook selects it explicitly without changing your configured default toolchain (for example, `stable` can remain on 1.94). If it is missing but global rustup exists, WinLook installs 1.95.0 side-by-side through that CLI. Only when no usable rustup is available does it bootstrap an isolated copy under `build/.rcache/`. On a Windows machine with the matching Windows App Runtime installed, the Win10/Win11 Manager targets can then be launched through Cargo.

Generated files share one ignored build root:

- Rust outputs: build/rust/
- CMake/Open Watcom outputs: build/c/
- Rust/Cargo/sccache cache: build/.rcache/
- C compiler cache: build/.Ccache/
- Preview packages and reports: build/artifacts/

The C and Rust builds therefore share one build footprint without sharing compiler outputs or cache formats. Run python build.py clean --yes to remove compiled outputs while keeping caches; add --all only when you intentionally want to discard both caches.

See [Build System and CI](plan/architecture/build-and-ci.md) for all commands, Rust toolchain selection and setup, C/MSVC prerequisites, workflow inventory and release gates. The Win95 and BGW workflows remain source-gated until those application targets exist.

## Development status

Planning documents distinguish accepted product scope, proposals, documented API facts and tested implementation behaviour. Do not mark a feature supported from an SDK reference alone. Before implementing a feature, read its per-OS page, confirm toolchain/API minimums, and define fallback, recovery, privacy and acceptance tests.

WinLook's target source organization is now defined in [repository-layout.md](plan/architecture/repository-layout.md): complete Windows 10 and Windows 11 applications under `rust/win10/` and `rust/win11/`, the complete Windows 95 application under `C/win95/`, and shared libraries under each language's `library/` directory. Historical prototype paths may remain until a dedicated migration updates source, manifests, workflows, tests and links together.
