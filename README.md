# WinLook

**Customize Windows itself. Keep Windows' native components in charge.**

WinLook is a planned Windows customization platform for desktop appearance, windows, displays, notifications, media controls, File Explorer and user-defined profiles. It is designed to extend Windows rather than replace its Shell, filesystem, window manager, compositor or security infrastructure.

The project aims to cover Windows 95 and later, but that ambition is not a release-support claim. Each feature must use an OS-appropriate adapter, report partial or unsupported capabilities honestly, and retain a safe Windows-native fallback.

## Project status

**Early planning and portable prototypes.**

This repository is not yet a complete Windows application. It currently contains the living architecture/compatibility plan and the first renderer-independent animation-library prototypes. It does **not** yet contain a complete WinLook Manager, desktop runtime, Explorer integration, installer or supported Windows release.

The animation prototypes are intentionally narrow:
- [`wl_animate`](crates/wl_animate/README.md) — Rust timing, interpolation and keyframe primitives; no Windows renderer/backend.
- [`wl_Lanimate`](legacy/wl_Lanimate/README.md) — C89-compatible counterpart with caller-owned storage; no OS-specific drawing or scheduler.

See the [Animation Suite plan](plan/architecture/animation-suite.md) for recorded verification status and remaining gates.

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

## Working with the current prototypes

There is no root application build command yet. The two portable library prototypes can be tested independently when the required tools are installed:

```sh
cargo test --manifest-path crates/wl_animate/Cargo.toml
make -C legacy/wl_Lanimate test
```

These commands test portable library code on the host. They do not build a WinLook application or prove compatibility with Windows 95–11. The C Makefile requires `make`, a C89-capable compiler and a shell environment compatible with its POSIX-style commands; Open Watcom/Windows runtime verification is a separate gate.

## Development status

Planning documents distinguish accepted product scope, proposals, documented API facts and tested implementation behaviour. Do not mark a feature supported from an SDK reference alone. Before implementing a feature, read its per-OS page, confirm toolchain/API minimums, and define fallback, recovery, privacy and acceptance tests.

The target repository layout and public documentation structure are still proposals. Do not move the animation prototypes into `lib/` or create the complete `C/`, `rust/` engine trees until the layout gates in [repository-layout.md](plan/architecture/repository-layout.md) are accepted.
