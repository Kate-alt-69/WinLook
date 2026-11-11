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

The current official Rust/MSVC target documentation requires Windows 10 or later for client installs. The Rust GNU target page is currently a stub and does not establish an older-Windows support baseline. Accordingly, the planned default is:

- **Windows 10 and later:** Rust/MSVC implementation candidate.
- **Windows 95 through Windows 8.1:** separate C implementation and per-OS compiler/runtime adapters.
- **Older Rust toolchains on pre-Windows-10:** research-only until the exact compiler, standard library, linker/runtime and target OS are verified.

These are toolchain planning boundaries, not WinLook OS support claims. Every WinLook adapter still needs exact build/runtime tests.

Sources: [Rust MSVC target support](https://doc.rust-lang.org/rustc/platform-support/windows-msvc.html) · [Rust GNU target support](https://doc.rust-lang.org/rustc/platform-support/windows-gnu.html).

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
<<<<<<< HEAD
=======

The Win10/Win11 Manager shell is being prototyped with native WinUI 3 through Windows Reactor in [wl_ui](rust/library/wl_ui/README.md). The current shell has navigation/search/theme controls and explicit preview states; it does not apply system settings until BGW and IPC are implemented. See [UI Architecture](plan/architecture/ui-architecture.md).

## Building the shared libraries
>>>>>>> origin/main

The Win10/Win11 Manager shell is being prototyped with native WinUI 3 through Windows Reactor in [wl_ui](rust/library/wl_ui/README.md). The current shell has navigation/search/theme controls and explicit preview states; it does not apply system settings until BGW and IPC are implemented. See [UI Architecture](plan/architecture/ui-architecture.md).

## Building WinLook

The root Python runner is the common entry point for building, testing and packaging. It requires Python 3.9+ and uses only the standard library:

```sh
python build.py doctor
python build.py setup
python build.py verify libraries --sanitizers
python build.py build win10
python build.py build win11
python build.py package win10
```

On a Windows machine with the matching Windows App Runtime installed, the Win10/Win11 Manager targets can be launched through Cargo. Their application builds use the native MSVC toolchain.

Generated files share one ignored build root:

- Rust outputs: build/rust/
- CMake/Open Watcom outputs: build/c/
- Rust/Cargo/sccache cache: build/.rcache/
- C compiler cache: build/.Ccache/
- Preview packages and reports: build/artifacts/

The C and Rust builds therefore share one build footprint without sharing compiler outputs or cache formats. Run python build.py clean --yes to remove compiled outputs while keeping caches; add --all only when you intentionally want to discard both caches.

See [Build System and CI](plan/architecture/build-and-ci.md) for all commands, setup rules, workflow inventory, release gates and the required library → Manager UI → workflow merge order. The Win95 and BGW workflows remain source-gated until those application targets exist.

## Development status

Planning documents distinguish accepted product scope, proposals, documented API facts and tested implementation behaviour. Do not mark a feature supported from an SDK reference alone. Before implementing a feature, read its per-OS page, confirm toolchain/API minimums, and define fallback, recovery, privacy and acceptance tests.

WinLook's target source organization is now defined in [repository-layout.md](plan/architecture/repository-layout.md): complete Windows 10 and Windows 11 applications under `rust/win10/` and `rust/win11/`, the complete Windows 95 application under `C/win95/`, and shared libraries under each language's `library/` directory. Historical prototype paths may remain until a dedicated migration updates source, manifests, workflows, tests and links together.
