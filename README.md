# WinLook

**Customize Windows itself. Keep Windows' native components in charge.**

WinLook is a planned Windows customization platform for desktop appearance, windows, displays, notifications, media controls, File Explorer and user-defined profiles. It is designed to extend Windows rather than replace its Shell, filesystem, window manager, compositor or security infrastructure.

The project aims to cover Windows 95 and later, but that ambition is not a release-support claim. Each feature must use an OS-appropriate adapter, report partial or unsupported capabilities honestly, and retain a safe Windows-native fallback.

## Project status

**Early product planning; animation libraries in active alpha implementation.**

This repository is not yet a complete Windows application. It currently contains the living architecture/compatibility plan and the first renderer-independent animation-library prototypes. It does **not** yet contain a complete WinLook Manager, desktop runtime, Explorer integration, installer or supported Windows release.

The first implementation work is focused on the paired animation libraries, both using **Windows 10 as their initial target baseline**:
- [`wl_animate`](crates/wl_animate/README.md) — Rust animation primitives and public crate API.
- [`wl_Lanimate`](legacy/wl_Lanimate/README.md) — C89-compatible counterpart with caller-owned storage and static-library builds.

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

## Building the animation libraries

There is no root application build command yet. Build and test the Rust crate with:

```sh
cargo fmt --manifest-path crates/wl_animate/Cargo.toml -- --check
cargo test --manifest-path crates/wl_animate/Cargo.toml
cargo clippy --manifest-path crates/wl_animate/Cargo.toml --all-targets --all-features -- -D warnings
```

Build the C static library and run its host tests with:

```sh
make -C legacy/wl_Lanimate clean
make -C legacy/wl_Lanimate all
make -C legacy/wl_Lanimate test
```

The C library also provides a CMake build for MSVC; see its [build instructions](legacy/wl_Lanimate/README.md). A dedicated GitHub Actions workflow checks Rust/MSVC and C/MSVC builds. Host C tests validate the portable core, not a Windows 10 client runtime. The Windows CI runner is a build/test environment, not a substitute for the planned Windows 10 client VM/hardware verification.

## Development status

Planning documents distinguish accepted product scope, proposals, documented API facts and tested implementation behaviour. Do not mark a feature supported from an SDK reference alone. Before implementing a feature, read its per-OS page, confirm toolchain/API minimums, and define fallback, recovery, privacy and acceptance tests.

WinLook's target source organization is now defined in [repository-layout.md](plan/architecture/repository-layout.md): complete Windows 10 and Windows 11 applications under `rust/win10/` and `rust/win11/`, the complete Windows 95 application under `C/win95/`, and shared libraries under each language's `library/` directory. Historical prototype paths may remain until a dedicated migration updates source, manifests, workflows, tests and links together.
