# WinLook Workspace Memory

> Durable project context for Research Companion sessions and other coding agents. Last verified: 2026-11-09. Current user instructions and repository source take precedence.

## Project identity
WinLook is a Windows customization platform bringing deep desktop and appearance customization to Windows 95 and later. It draws inspiration from Linux desktop/window-manager communities but is its own Windows product, not a Hyprland port or Linux config importer. Repository: Kate-alt-69/WinLook. Living plan: [Plan.md](Plan.md). Final-Plan.md has not been created.

## Agreed architecture decisions
- Modern implementation: Rust. Legacy implementation: separate C implementation/toolchain per OS family. Shared conceptual APIs/contracts, not one binary.
- Windows windowing and DWM remain intact. WLWM augments Windows; it does not replace DWM.
- WLWM may inject/hook compatible apps through separately gated, version-specific integrations. BGW is the complete background runtime/worker, not merely a WLWM coordinator; it hosts/co-ordinates the background subsystems and owns their lifecycle, policy and health.
- WLWM classifies applications before injection and caches compatibility results locally in a binary format. Cache identity/invalidation includes target binary/version/hash, architecture, OS/build, relevant mitigations, WLWM module and compatibility DB versions. Cache hits never skip runtime safety checks.
- Explorer.exe must never be patched/replaced on disk. Deep Explorer customization is accepted scope: documented Shell extensions plus a separate opt-in, per-build presentation adapter for UI surfaces without public APIs. This is not general WLWM injection; Windows retains native Shell/filesystem operations. DWM and authentication/security-critical processes are excluded.
- WLWM Settings are in WinLook Manager and can be included in WLT; no standalone WLWM config language required.
- wl_animate is the reusable Rust animation crate/API for modern WinLook and third-party integrations. wl_Lanimate is its C counterpart. The current implementation baseline for both libraries is Windows 10; C89 is a source-language constraint, not evidence of pre-Windows-10 runtime support. The intended vector/SVG and alpha contract is specified in [vector-animation.md](architecture/vector-animation.md); it is not yet a complete public API.
- CPUv/GPUv are planned as shared WinLook-owned resource governors: Rust wl_cpuv/wl_gpuv and C wl_Lcpuv/wl_Lgpuv. They coordinate only WinLook-managed CPU tasks and GPU allocations/submissions; Windows/driver remains authoritative for process scheduling and GPU memory. See [resource-governors.md](architecture/resource-governors.md).
- WLDE (WinLook Desktop Engine, WALDE) owns wallpaper playback, library/community Store, widgets, Rainmeter compatibility, desktop icon presentation, virtual Groups, occlusion-aware rendering, adaptive resource control and Game Mode.
- WLDE Groups are virtual visual objects, never filesystem folders. Original Shell items remain in place and launch normally. A custom Group icon opens a borderless popup with original items. Grouping is disabled by default. Local Groups are machine-specific and never exported in WLT. Fancy Groups are separate and not yet specified.
- WLT is WinLook Template (.wl.t): proposed TAR.GZ-style archive with config and assets. ZIP containing multiple .wl.t files is a Template Group, imported independently. Proposed subdirs include rmskin/, wl/taskbar/, wl/&FONT_FILE&/, wl/wallpaper/, wl/icons/, wl/cursors/, wl/sounds/, wl/desktop/, wl/wlwm/ and wl/display/.
- .wl.wp is normal wallpaper asset/package; .wl.i.wp is an interactive wallpaper archive/package containing media and supporting resources. Preserve source quality.
- WLOSDE is configurable WinLook notifications/popups/OSD. WLDM manages displays. WLDE renders; WLWM manages windows across monitors.
- Accessibility is required throughout.
- Application Integration Bridge is a public API plus community adapters; do not assume big vendors will integrate.
- Store/catalogues should support independent community sources, self-hosted catalogues, GitHub releases and local imports rather than requiring one central WinLook server.
- No WinCheck, WLCC, custom Launcher for now, separate Scene Engine, separate Creator Studio or generic Input Customization Engine. Creation tools are built into WinLook Manager.

## Compatibility facts source-verified in documentation
- SetWinEventHook: minimum documented client Windows 2000; separate Active Accessibility 1.3 RDK route for Windows 95 and NT4 SP6.
- Active Accessibility table: Windows 95 had none built in; Windows 98 initially 1.1; Windows Me/2000 1.3; NT4 from SP6 1.3; XP/Vista 2.0; Windows 7 3.0.
- AnimateWindow minimum documented client Windows 2000.
- DWM APIs minimum Vista; Vista/7 composition may be disabled; Windows 8 makes DWM composition always on.
- IDCompositionDesktopDevice minimum Windows 8.1.
- IVirtualDesktopManager documented for Windows 10 and supports window desktop queries/moves; not the whole create/switch functionality WLWM may want.
- PROCESS_MITIGATION_DYNAMIC_CODE_POLICY documented from Windows 8.1; ProhibitDynamicCode blocks dynamic code generation or executable code modification.
- WTSRegisterSessionNotification is Vista+ only; legacy lock/session detection must use separately verified adapters.
- Session 0 becomes reserved for services/non-interactive processes from Vista; before Vista first user also shared Session 0.
- Winlogon desktop has restricted access; WinLook must leave authentication and secure desktop UI to Windows.
- Open Watcom documents separate Win95 and NT/Win32 targets; no guarantee every runtime works on every release.
- Rainmeter .rmskin is a modified ZIP with package metadata; a normal renamed ZIP is not an equivalent package.

## Open questions
- Support tiers/release sequence per OS.
- Exact legacy compiler/linker/runtime and SDK combinations.
- Shared settings schema/versioning.
- wl_animate public API and wl_Lanimate C ABI.
- Injection target policy, compatibility scoring and cache invalidation.
- Per-OS workspace strategy.
- Desktop icon suppression/recovery adapter per OS.
- WLT manifest and archive security/signing.
- Wallpaper formats and rendering backends.
- WLDM capabilities and rollback by OS/driver.
- Rainmeter compatibility level and licensing.
- Legacy pre-Vista session/lock detection mechanisms.
- Fancy Groups and cross-machine item resolution.


## Initial implementation checkpoint — alpha.1 prototype (historical)
- Added `crates/wl_animate`: renderer-independent Rust primitives for interpolation, easing, caller-driven tweens, playback state, keyframes and sampling. It is intended as a reusable crate for WinLook and third-party integrations. Initial version is alpha; API is not stable.
- Added `legacy/wl_Lanimate`: C89-compatible legacy library with corresponding concepts, caller-owned storage and no per-frame heap allocation in the library core.
- Exact C host test: `cd legacy/wl_Lanimate && make clean && make test`; passed with GCC 14.2.0 and `-std=c89 -Wall -Wextra -Werror -pedantic -O2`.
- Rust verification: not run because `rustc` and `cargo` are unavailable in the current execution environment.
- Windows/legacy runtime verification: not run. In particular, no Open Watcom build, Windows 95 VM test, Win32/DWM/GDI backend, BGW integration or injection is included yet.
- No compiled artifacts from that prototype checkpoint were committed. See [animation-suite.md](architecture/animation-suite.md) for current implementation scope and next gates.

## Repository state
- Root-level Plan.md was moved to plan/Plan.md in commit bd1b7281af700e7efa4dcda67f994ab79dde43f5.
- Per-OS compatibility pages, subsystem architecture docs, source ledger and workspace index/memory were added in that documentation change.
- Follow-up plan/source/index pass: 35f8deab45f354cce2e96ebce31ba6874e081f3c.
- Workspace state refresh: 613b46bf0faff10a59686320e92ab5385944818c.
- Architecture detail pages cover appearance/icon customization, standby/session security, extension/community formats, WLDE, WLWM, WLT, WLOSDE/WLDM, animations and restoration.
- Technical citations are maintained in [references/SOURCES.md](references/SOURCES.md); per-OS research is under [compatibility/](compatibility/README.md).
- User explicitly authorized direct commits to main for this work.
- These documents are planning/research; they do not claim that code is implemented or tested.

## Detailed planning checkpoint — 2026-11-08 (historical; superseded by the 2026-11-10 repository layout)

- At that point the user confirmed the task remained in planning/specification; the later explicit implementation request supersedes that restriction for the two animation libraries only. Do not infer permission to implement the other engines/UI or to migrate the source layout.
- Historical proposal only (superseded): parallel acronym-engine trees, dedicated UI trees, `rust/crate/`, C umbrella headers and root `lib/`. The current authoritative layout is recorded in the 2026-11-10 section below.
- Historical prototype locations were `crates/wl_animate` and `legacy/wl_Lanimate`. Their future migration must target the current `rust/library/` and `C/library/` layout, not the superseded root `lib/` proposal.
- The exact purpose of the root wllook/ directory is unresolved; current proposal reserves it for product/packaging metadata and shared assets, not duplicate engine source.
- Manager UI direction: WinToys-inspired clear left navigation and focused pages, familiar/native controls, split pages and reusable components. Do not copy its implementation or commit to WinUI 3 for legacy targets.
- Added [repository layout](architecture/repository-layout.md), [UI architecture](architecture/ui-architecture.md), [documentation standards](architecture/documentation-standard.md), [runtime contracts](architecture/runtime-contracts.md), [roadmap](roadmap.md) and [compatibility test strategy](compatibility/test-strategy.md).
- Expanded detailed contracts for WLWM target lifecycle/cache/injection policy, WLDE wallpaper/resource/Game Mode/Groups, WLT import/export/security, WLOSDE/WLDM lifecycle, restoration journaling and compatibility planning.
- No source files were moved and no product implementation was added in this planning update.
- The installed Research Companion skills and GitHub connector were used. Dedicated local Research Companion helper scripts are not exposed in this session; no claim is made that they ran.

## Audio & Media and Explorer scope — 2026-11-08

- Audio & Media is one accepted WinLook-owned experience: endpoint controls, per-app audio sessions where supported, and Now Playing metadata/artwork/video thumbnails/transport/timeline where the provider exposes them.
- Audio-session controls and media-session controls are distinct. Vista has the base Core Audio session interface; `IAudioSessionManager2` enumeration starts with Windows 7. GSMTC media sessions begin with Windows 10 version 1809 (build 17763).
- Vista and older systems still receive the WinLook UI. Systems without GSMTC use explicit supported-player adapters. Pre-Vista generic per-process volume is not promised; label legacy global/device controls and player-owned volume separately.
- Explorer deep customization is accepted product scope for the actual File Explorer interface, including context menus, navigation, address/search, command bar, tabs, views, panes, icons and thumbnails. Windows keeps filesystem, Shell namespace, file-operation, activation and security responsibilities.
- Never modify/replace `explorer.exe` on disk and never build a replacement file manager. Shell extensions and namespace views have limited documented scopes. Deep chrome styling without public APIs is a separate opt-in, per-build adapter, independent from WLWM injection, with out-of-band disable, quarantine and recovery.
- Profiles can coordinate Audio & Media preferences and Explorer presentation presets. WLT stores selected portable settings/assets only, not active media, clipboard content, process handles, cache state, hardware serials or Explorer implementation details.
- RGB/peripheral lighting remains delegated to external apps; startup-app management remains out of scope.
- Detailed specs: [audio-experience.md](architecture/audio-experience.md), [explorer-customization.md](architecture/explorer-customization.md), [profiles-and-optimization.md](architecture/profiles-and-optimization.md).

## Toolchain and API baseline refinement — 2026-11-08

- Current official Rust/MSVC Windows targets document Windows 10 or later as the minimum client OS. The Rust GNU Windows target page is currently a stub and does not establish a pre-Windows-10 support baseline.
- Planned product route: Rust/MSVC for Windows 10+; separate C implementation/toolchain adapters for Windows 95 through 8.1. Older pinned Rust toolchains remain research-only until exact compiler, standard library, linker/runtime and OS tests pass.
- A crate's `rust-version` is a Rust compiler/language MSRV, not a Windows OS minimum.
- `IExplorerCommand` has a documented Vista minimum. Windows 11's modern Explorer context-menu registration adds app-identity/manifest requirements; keep those two facts separate.
- Vista's `IAudioSessionManager` can retrieve controls for known sessions but does not provide the later `IAudioSessionManager2` live enumerator. Windows 7+ session lists should combine initial enumeration with notification callbacks; do not treat one enumeration snapshot as complete.
- GSMTC media-session access requires the `globalMediaControl` app capability and Windows 10 version 1809+.
- Canonical source ledger: [references/SOURCES.md](references/SOURCES.md); toolchain/OS facts also appear in [compatibility/README.md](compatibility/README.md) and [compatibility/matrix.md](compatibility/matrix.md).


## Animation library implementation pass — 2026-11-09

- User authorized starting implementation of the two paired libraries: Rust `wl_animate` and C `wl_Lanimate`. The first target baseline for **both** is Windows 10. C89 remains a language/source constraint only; no pre-Windows-10 compatibility claim is made.
- Kept existing paths `crates/wl_animate` and `legacy/wl_Lanimate`; the planned `lib/` migration was not part of this task.
- Alpha `0.1.0-alpha.2` adds delay-aware tweens, pause/resume/cancel/restart, progress/state inspection, 13 easing curves, Point2/RGBA interpolation, strict keyframe validation and sampled tracks. Rust track lookup uses binary search. C now builds a static archive via Make and a native static library via CMake/MSVC. Fixed the C RGBA alpha-channel interpolation defect.
- Added focused Rust unit tests and C coverage for tween lifecycle/delay, easing, all RGBA channels, keyframe order and invalid timestamps.
- Local C verification: GCC 14.2 C89 Make build + tests passed; GCC/CMake 3.31.6 build + CTest passed (1/1); Clang C89 build with AddressSanitizer and UndefinedBehaviorSanitizer passed. Build artifacts were cleaned and not committed.
- Local Rust verification is unavailable in this environment because `rustc`, `cargo` and `rustfmt` are absent. Added `.github/workflows/animation-libraries.yml` to verify Rust formatting/tests/Clippy/docs and C MSVC/CMake/CTest on a Windows runner. Workflow success and actual Windows 10 client runtime tests remain pending.
- No rendering backend, scheduler, Win32/GDI/DWM integration, process injection, manager or engine code was added. The core remains renderer-independent. Follow-up API scope includes timelines, sequences, parallel groups, interruption policies, callbacks and reduced-motion/resource controls.


## Vector animation and CPUv/GPUv planning — 2026-11-09

- User paused further source implementation and requested more architecture planning before resuming implementation.
- Added [vector-animation.md](architecture/vector-animation.md): typed vector/matrix/shape model, vector document/item hierarchy, SVG path-data contract, supported SVG profile, color-space/alpha/group-opacity rules, timelines/sequences/parallel groups, CPU/GPU renderer separation, and a "does X do X?" conformance/stress matrix.
- Added [resource-governors.md](architecture/resource-governors.md): intended responsibilities and limits of CPUv/GPUv, tickets/admission, scheduler and memory-budget contracts, activity modes (interactive/gaming/idle/power saver), Windows adapters, cross-language parity, bypass audit and stress tests.
- Proposed library identities: Rust `wl_cpuv`/`wl_gpuv`; C `wl_Lcpuv`/`wl_Lgpuv`. Their target homes are now `rust/library/` and `C/library/`, respectively.
- Important enforcement boundary: the governors can require all WinLook-owned managed work to request permits; they cannot intercept Windows, unrelated processes, arbitrary code bypasses or GPU drivers. DXGI reports per-process video-memory budgets; reservations are OS hints, not hardware partitions. Job Object hard CPU caps can pause all threads in the contained job after its interval allocation and are not the default interactive-app throttle.
- Current main branch has a `0.2.0-beta.1` Cargo manifest label and initial `vector.rs`/`svg.rs` files, but `lib.rs` has not yet wired or exported them. The label is not a completed/released beta. The source parser and vector API remain gated on the new plan's tests and public contract review.
- No CPUv/GPUv implementation files, Windows rendering backend, worker scheduler, DXGI adapter or machine-code optimizer have been committed in this planning pass. Planned features must not be advertised as implemented.


## Two-executable runtime and current repository layout — 2026-11-10

- The two principal application executables are `winlook.exe` (interactive Manager UI) and `winlook.bg.exe` (startup-launched BGW background runtime/worker). Do not introduce a third mandatory Agent/desktop-runtime executable or one executable per subsystem.
- BGW hosts/co-ordinates WLDE, WLWM, WLDM, WLOSDE, Appearance/Icon, Audio & Media and other enabled background modules. These are logical modules and may use internal threads/bounded workers.
- Manager↔BGW communication uses local IPC with protocol versioning, peer/session authentication, bounded typed messages, request IDs, explicit acknowledgements and reconnect/recovery semantics. BGW is authoritative for applied runtime state; Manager drafts are not applied until BGW acknowledges them.
- BGW continues after the Manager closes, using the last valid committed configuration. It initializes modules after capability checks, monitors module health and retains a minimal control/recovery path under critical pressure.
- Subsystems register capabilities, dependencies, owned work and safe response actions with BGW. BGW observes supported system/session signals; users do not need to register every external game/window for general pressure policy.
- CPUv/GPUv are core libraries for WinLook-managed CPU work and GPU allocations/submissions. BGW coordinates policy; the governors cannot control unrelated processes or replace Windows/driver scheduling and residency.
- Automatic Dynamic Quality Scaling is explicitly rejected. Never silently lower wallpaper resolution, source quality or visual fidelity. Defer/pause optional work, freeze the last displayed frame, stop unnecessary decoding/updates and release recoverable resources instead.
- Target repository layout is user-defined: `.github/workflows/`, `rust/win10/`, `rust/win11/`, `rust/library/`, `C/win95/`, `C/library/`, `plan/` and root `README.md`. This supersedes earlier `crates/`, `legacy/`, root `lib/`, `docs/`, `wllook/` and acronym-engine layout proposals. Historical source may remain in old paths until a dedicated migration is performed.
- FFmpeg integration direction: narrow wrapper libraries `wl_ffmpeg` and `wl_Lffmpeg`, coordinated with CPUv/GPUv. Codec/build configuration, licensing and Windows 95 support remain open research decisions.
- New detailed source: [background-runtime.md](architecture/background-runtime.md). The design is documentation/planning only; no new executable, IPC transport, resource governor or FFmpeg wrapper is claimed implemented.


## Shared library implementation checkpoint — 2026-11-10

- Current implementation branch: `feature/library-foundation`, based on `main` at the 2026-11-10 architecture-doc commit.
- Target source homes are now populated on the feature branch: `rust/library/wl_animate`, `rust/library/wl_cpuv`, `rust/library/wl_gpuv`, `C/library/wl_Lanimate`, `C/library/wl_Lcpuv`, `C/library/wl_Lgpuv`.
- Root `Cargo.toml` defines a workspace for the three Rust crates; `C/library/CMakeLists.txt` aggregates the three C libraries.
- Rust `wl_animate` now publicly exports its existing vector/SVG path-data modules. Its SVG parser's arc flags were changed to consume single-character flags so compact flag sequences can be parsed.
- C `wl_Lanimate` now includes C89 vector3/vector4/rect, affine matrix composition/point transform, opacity and shortest-arc Transform2D interpolation helpers.
- CPUv/GPUv Rust and C cores are portable policy/admission primitives only. They do not implement OS scheduling, DXGI, hardware resource creation, BGW integration or system-wide controls.
- FFmpeg wrappers remain deferred pending codec/toolchain/licensing and Windows 95 compatibility decisions.
- CI workflow: .github/workflows/library-tests.yml. Full run [42832087050](https://github.com/Kate-alt-69/WinLook/actions/runs/42832087050) passed: Rust formatting/tests/Clippy/docs for all three crates and workspace integration; C89 MSVC plus GCC ASan/UBSan for all three C libraries; combined CMake/CTest passed 3/3.
- Next sequence: run/fix all library CI; review API/contract parity and fill test gaps; then implement and test the WinToys-style Manager UI; only after UI verification proceed to application compilation and optimization.


## Manager UI research checkpoint — 2026-11-10

- Modern Win10/Win11 UI prototype direction: Windows Reactor + native WinUI 3, using the pinned windows-reactor-pre 0.107.1 preview package and Rust 1.95+; its public API may change.
- Windows App SDK/WinUI 3 minimum documented client is Windows 10 1809 (build 17763). The exact Win10 product baseline remains open; the current prototype targets 1809+ only and cannot be claimed to support earlier Win10 builds.
- Start with framework-dependent deployment for the prototype; compare self-contained runtime staging after measuring package size and install experience.
- No Electron or WebView2 for the core settings UI.
- Shared UI components/page models/design tokens belong in a future rust/library/wl_ui/; Win10 and Win11 application entry points remain in their own platform trees.
- Win95 remains a separate C/native-control implementation; no WinUI 3/Windows App SDK dependency.
- Evidence and caveats: [UI Architecture](architecture/ui-architecture.md), [Source Ledger](references/SOURCES.md).
- Prototype acceptance: cold start, idle CPU, private bytes/working set, navigation latency, mount/unmount memory, UI Automation, keyboard/high contrast/text scaling/reduced motion and constrained-window layout. No performance claim until measured on representative low-end hardware.


## WinToys-style Manager UI implementation checkpoint — 2026-11-10

- Stacked UI branch: feature/wintoys-manager-ui, based on feature/library-foundation.
- Added wl_ui using the pinned windows-reactor-pre 0.107.1 preview package / WinUI 3 and Win10/Win11 Manager entry points.
- Current shell includes persistent left navigation, search, back/pane controls, System/Light/Dark window theme, Home quick actions and reusable status cards.
- All non-Home pages are explicit scaffolds. They do not claim to apply system settings; BGW/IPC and OS adapters are not implemented yet.
- Windows 95 UI remains out of this branch and will use a separate C-native implementation.
- Build/test gates are defined in `.github/workflows/library-tests.yml`; the Windows UI job verifies workspace formatting, UI tests/Clippy, Win10/Win11 application builds and preview artifacts. The UI branch CI passed after the `pages.rs` rustfmt corrections (run 43057947732). Actual Windows App Runtime desktop smoke testing and low-end performance measurements remain required.
## Unified build runner and workflow implementation — 2026-11-11

- The staged implementation sequence is complete: shared-library foundation PR #1, Manager UI PR #2 and build/workflow PR #3 are all merged into `main`. PR #3 merged at commit `ebc853e894e88d3e863c5b8034a41128dfbbaead`.
- Root build entry point: `build.py` (Python 3.9+, standard library only). `workflow-lint.yml` validates the runner/tests and all workflow YAML with pinned actionlint.
- Build output contract: Rust under `build/rust/`; CMake/Open Watcom under `build/c/`; package/report output under `build/artifacts/`.
- Cache contract: `CARGO_HOME` and optional sccache under `build/.rcache/`; ccache under `build/.Ccache/`. Root `.gitignore` excludes the full `build/` tree. `clean --yes` keeps caches; `clean --yes --all` removes them.
- Canonical workflow set: `workflow-lint.yml`, `libraries.yml`, `manager-ui.yml`, `win10.yml`, `win11.yml`, `bgw.yml`, `win95.yml`, `docs.yml`, `security.yml`, `build-all.yml` and `release.yml`. The earlier monolithic `library-tests.yml` is removed.
- Workflow lint uses pinned actionlint 1.7.12 (SHA-256 verified); `.github/actionlint.yaml` declares the self-hosted `open-watcom` label.
- Security workflow uses CodeQL for C/C++, Rust, Actions and Python, plus a full-history Gitleaks scan with redacted report output. Dependency review remains conditional on enabling the repository Dependency Graph and setting Actions variable `WINLOOK_DEPENDENCY_GRAPH_ENABLED=true`.
- Win95 remains source-gated until `C/win95/Makefile` exists. Its CI path requires a self-hosted Linux x64 runner labelled `open-watcom`, repository variable `WINLOOK_ENABLE_WIN95_RUNNER=true` and a manually installed Open Watcom compiler. The Python helper verifies/downloads the upstream installer but does not assume undocumented silent switches.
- BGW remains source-gated until a `winlook.bg` / `winlook-bg` / `winlook_bg` Cargo binary target is explicitly declared.
- Keep source-detection skips distinct from actual builds: Win95 and BGW executables are not built until their source targets exist.
- The PR #3 merge commit's combined-status endpoint returned no status entries in this inspection; that is not evidence of a passing or failing hosted Actions run. Check the individual workflow runs before treating the merged head as fully CI-verified.


## Build runner CLI and workflow correction — 2026-11-12

- User reproduced `py build.py` failing because argparse required an action. The intended default is now the read-only `doctor` preflight; no arguments must never trigger setup, compilation, downloads or cleanup.
- Add regression coverage for the no-argument dispatch and parser acceptance of every workflow-used CLI command.
- `workflow-lint.yml` and `docs.yml` smoke-test `python build.py` in addition to syntax compilation and unit tests.
- `build-all.yml` runs the default preflight and a strict toolchain doctor check before `verify all`; on Windows this verification includes actual supported library/UI checks and Win10/Win11 app compilation where manifests are present.
- Distinguish workflow lint and unit tests from real compiler evidence. The latter must come from `libraries.yml` and `build-all.yml`; BGW and Win95 remain explicitly source/toolchain gated.


## Project-local toolchain provisioning — 2026-11-12

- User reproduced Win10 build failure because the process selected Rust 1.94.0 while the WinUI/Windows Reactor crates and WinLook packages declare Rust 1.95 minimum.
- `build.py` now pins Rust 1.95.0 and bootstraps official `rustup-init` with its official `.sha256` sidecar verified before execution.
- Rustup state/toolchains and Cargo cache/proxies are scoped to `build/.rcache/rustup-home` and `build/.rcache/cargo-home`; build subprocesses put those proxies first and select `RUSTUP_TOOLCHAIN=1.95.0`. Global Rust installs and persistent PATH are not modified.
- Rust-dependent build/test/verify/fmt commands provision the local toolchain automatically. `setup --with-rust --apply` performs the same setup explicitly.
- Windows C/MSVC preparation loads an existing Visual Studio C++ environment using `vswhere` and `VsDevCmd`; if absent, WinGet installs Visual Studio 2022 Build Tools with the Desktop development with C++ workload. CMake is installed through WinGet when missing. Visual Studio is necessarily machine-wide and may require elevation; Rust remains project-local.
- CI workflows now invoke the repository setup command and use the pinned Rust toolchain instead of the floating `stable` action.
- Regression tests cover local Rust/Cargo/Rustup paths, Windows MSVC host selection, and setup CLI flags. CI must still confirm the installer, exact toolchain, and actual Win10/Win11 builds on the next runs.


## Reuse existing Rust 1.95 installation — 2026-11-12

- User showed `rustup show` with `1.95.0-x86_64-pc-windows-msvc` already installed alongside the existing default `stable` toolchain. The build runner's previous unconditional project-local `CARGO_HOME`/`RUSTUP_HOME` redirection hid that installation and caused an unnecessary rustup-init download.
- `Runner` now applies a selected Rust environment only after toolchain detection; it no longer forces empty project-local Rust homes on global rustup commands.
- `ensure_rust_toolchain` probes global rustup first with `rustup run 1.95.0 rustc --version` and `cargo --version`. If present, build subprocesses receive `RUSTUP_TOOLCHAIN=1.95.0`; the user's global default and PATH are untouched.
- If 1.95.0 is missing but global rustup exists, the CLI installs it side-by-side without changing the default. Existing project-local rustup is the next fallback; rustup-init download is now the final fallback only.
- Regression tests cover global-toolchain reuse, preserving a pre-existing `RUSTUP_TOOLCHAIN=stable`, side-by-side installation, and not forcing project-local `CARGO_HOME`/`RUSTUP_HOME` for the global path.


## Generation-aware Windows build selectors — 2026-11-12

- Added canonical CLI selectors for every documented Windows generation: `windows.95`, `windows.98`, `windows.me`, `windows.nt4`, `windows.2000`, `windows.xp`, `windows.vista`, `windows.7`, `windows.8`, `windows.8.1`, `windows.10` and `windows.11`.
- Each selector resolves its own source manifest before compiler setup. A missing target now exits with the exact expected path and does not invoke WinGet, Visual Studio Installer, rustup or another compiler installer.
- Win9x selectors use the Open Watcom Makefile path. NT-family legacy selectors use an independent CMake target and verify that GCC is actually a native MinGW-w64 compiler via `gcc -dumpmachine`, with Ninja/MinGW Make/MSYS Make available as the CMake generator backend. A generic GCC binary alone does not prove it can produce Windows binaries.
- Win10/Win11 stay on the Rust MSVC/WinUI route. MSVC discovery checks the active environment, Visual Studio Installer metadata, registry and conventional installation roots; it loads the existing developer environment and checks `cl`, `link`, `rc`, Windows SDK headers and libraries. Existing Visual Studio instances are modified for missing C++ components; WinGet installs Build Tools only if no instance exists. MinGW is never treated as a substitute for WinUI's MSVC linker/SDK requirements.
- Windows CMake library builds reuse installed MSVC or verified MinGW GCC. CMake setup is attempted only when its executable is absent. Terminal help has optional ANSI styling and respects `NO_COLOR`.
- Regression coverage added for every selector (including dotted `windows.8.1`), source-gate-before-installer behavior, generic-GCC rejection, MinGW selection, Visual Studio non-reinstallation, and color opt-out. Hosted workflow results are checked separately; this checkpoint does not claim a local Windows application build.


## Visual Studio Installer elevation and diagnostics — 2026-11-12

- The local user's failed Build Tools modification showed `isadmin=true` but `iselevated=false`; the installer explicitly requires `--quiet`/`--passive` operations to be launched elevated. Exit code 5007 should be paired with the installer log context rather than treated as a compiler error.
- Existing-instance modifications now call the installed `setup.exe modify` through the Windows `runas` UAC verb, wait on the returned process handle, use `--quiet --norestart` (no transient Installer UI), and report UAC cancellation distinctly.
- The runner no longer passes `--wait` to installed `setup.exe` (that flag belongs to a bootstrapper). Windows process API signatures are declared explicitly for 64-bit-safe handle waits and exit-code retrieval.
- On failure, the runner maps known Visual Studio installer exit codes and extracts recent error/warning lines from `dd_setup_*.log`, `dd_client_*.log` and `dd_bootstrapper_*.log` files in the temp directory.
- Regression tests cover quiet/elevated routing, omission of `--passive`/`--wait`, error code explanations and log inclusion. UAC and actual installer behaviour still require a Windows interactive test.


## One-time Visual Studio Build Tools fallback — 2026-11-12

- Existing Visual Studio workload changes run through the hidden, elevated UAC helper and wait for the actual installer process; `setup.exe modify` uses `--quiet --norestart`, not the bootstrapper-only `--wait` option.
- If a pre-existing Enterprise/Professional/Community instance cannot be modified or still fails to expose `cl.exe`, `link.exe`, `rc.exe` and Windows SDK headers/libraries, the runner can make one fallback attempt to install the standalone Visual Studio 2022 Build Tools SKU.
- The standalone Build Tools SKU uses WinGet when available; otherwise it downloads `https://aka.ms/vs/17/release/vs_buildtools.exe` into `build/.rcache/installers/`, checks the Microsoft Authenticode signature with PowerShell, then invokes the bootstrapper with `--quiet --wait --norestart`. Cached bootstrapper files are signature-checked again before reuse.
- An existing Build Tools instance is never reinstalled. If the standalone install was the first setup attempt and fails, the error is returned without an immediate second installer call. A user-declined UAC operation is also not retried through another installer.
- The MinGW path is independent: native `x86_64-w64-mingw32` GCC plus Ninja/Make can build the shared C libraries, but it does not satisfy the Win10/11 Rust MSVC + WinUI linker/SDK gate.
- Regression tests cover each single-attempt and cancellation boundary; the Windows hosted build still needs to confirm the installer can actually provide the required workload in that runner image.


## Repeated build stress workflow — 2026-11-12

- Added `.github/workflows/build-stress.yml` as a reproducible build gate for relevant pushes and pull requests, with a manual dispatch entry point.
- The portable job runs Rust formatting/tests/Clippy/docs and C89 CMake tests with ASan/UBSan twice, cleaning generated outputs between runs while retaining toolchain caches.
- The Windows matrix builds Win10 and Win11 independently on `windows-2022`. It locates the installed VS instance with `Microsoft.VisualStudio.Component.VC.Tools.x86.x64`, imports the `VsDevCmd.bat` environment across GitHub Actions steps, and verifies the MSVC compiler/linker/resource compiler plus Windows SDK headers/libraries before strict preflight. Each target then runs the Manager UI tests/Clippy and performs clean, incremental and clean-rebuild passes.
- Each Windows pass packages the executable and validates the ZIP structure and non-empty executable; the final package is uploaded as a workflow artifact.
- `doctor --strict` now fails when a Win10/Win11 project exists on Windows but the complete MSVC compiler/linker/resource-compiler + Windows SDK environment cannot be loaded. It uses explicit executable paths for MSVC probes so incomplete `PATHEXT` settings cannot hide valid tool binaries. The SDK environment checker normalizes copied environment-variable keys so Windows' case-insensitive semantics are preserved.
- The workflow validates the preinstalled hosted-runner toolchain and actual builds; it does not test the interactive Visual Studio installer/UAC flow or prove runtime launch on a clean desktop. Hosted results remain pending until the new workflow completes.



## Visual Studio modify channel and explicit component selection — 2026-11-12

- The local Win10 build log showed that the Visual Studio 2022 Build Tools instance accepted a quiet `setup.exe modify` operation with exit code 0, but the following environment probe still could not find `cl.exe`, `link.exe`, `rc.exe` or a usable Windows SDK.
- The modify invocation omitted the required `--channelId` argument. WinLook now reads the exact selected instance's `channelId` from `vswhere -format json`, matching the installation path case-insensitively. If metadata is missing or cannot identify the instance/channel, it fails with a specific diagnostic rather than guessing.
- The C++ setup request explicitly adds `Microsoft.VisualStudio.Component.VC.Tools.x86.x64` and `Microsoft.VisualStudio.Component.Windows10SDK.19041` alongside `Microsoft.VisualStudio.Workload.VCTools`, rather than relying on optional/recommended workload defaults.
- The Windows stress workflow now runs the build-runner Python regression suite on both hosted Windows matrix jobs before checking the preinstalled MSVC environment and performing clean/incremental builds.
- Regression tests cover exact channel discovery, missing metadata, setup modify arguments and explicit compiler/SDK components. Actual UAC/installer execution still requires a Windows interactive build.


## Rust verification components on reused toolchains — 2026-11-12

- Windows CI reused the global Rust 1.95.0 compiler correctly, but the existing toolchain did not contain `cargo-fmt`; UI verification stopped before Clippy.
- Rust toolchain selection now audits the selected 1.95.0 installation for `rustfmt` and `clippy`. It adds only missing components through that explicit toolchain and leaves the configured rustup default untouched. Read-only setup reports missing components and the planned action without installing them.
- Regression coverage exercises the already-installed compiler path with and without the required support components.


## wl_ui backend facade and legacy C UI — 2026-11-13

- Public Rust crate API is consolidated under `wl_ui`. Consumers import a backend submodule such as `wl_ui::windows10`, `wl_ui::windows11`, or `wl_ui::tauri2`, while common types (`UiApplication`, `UiPage`, `UiWindowConfig`, `UiError`) remain at the crate root.
- Backend implementation crates are private optional dependencies. Cargo features only control compilation; backend selection is explicit in source, not inferred from the current OS at runtime.
- `wl_ui::windows10` uses the shared `wl_ui_win32` User32 renderer and intentionally excludes Windows App SDK / WinUI / Tauri dependencies.
- `wl_ui::windows11` is the native WinUI 3 / Windows Reactor renderer, isolated from the Win10 dependency graph.
- `wl_ui::windows7`, `windows8`, and `windows81` expose the same classic Win32 backend. Rust 1.77 is the last stable release with built-in Win7 support; current Rust requires the custom Tier 3 Win7 target for that OS baseline. No legacy OS runtime support is claimed until VM tests pass.
- `wl_ui::tauri2` attaches the shared model to an existing Tauri 2 builder and remains opt-in. Tauri 2.12 MSRV is 1.90 and its official Win7 support was dropped, so it is not used as the Win7 adapter.
- Added the independent C89 `C/library/wl_Lui` library as a baseline Win32 window lifecycle API, keeping pre-Vista and legacy compiler work outside the modern Rust UI dependency graph.
- Win10 app profile: Visual Studio 2022 + Windows SDK 10.0.19041.0. Win11 app profile: Visual Studio 2026 + Windows SDK 10.0.26100.0. CI uses separate hosted runner images for both.
- The Windows 10 Manager entry point explicitly calls `wl_ui::windows10::run()`; the Windows 11 Manager entry point explicitly calls `wl_ui::windows11::run()`.
- UI remains a scaffold: navigation and page descriptions are rendered, but BGW IPC and real system-setting operations are not implemented. Runtime smoke tests, accessibility checks, low-end performance measures, installer/runtime validation, and Windows 7 VM tests remain pending.
- Research Companion evidence is recorded in [references/SOURCES.md](references/SOURCES.md); this workspace entry is a design/implementation checkpoint, not a support declaration.


## wl_ui design system and styling upgrade — 2026-11-13

- The previously merged UI backend split is the base. New work is isolated on `feat/wl-ui-styling-studio-api`; it must not append commits to the already-merged PR #9 branch.
- User-reported smoke test: the Windows 10 application now launches. This confirms the user's local startup test only; it does not replace CI or runtime coverage across other Windows versions.
- The shared Rust core is extended with `UiTheme`, `UiPalette`, `UiTypography`, `UiMetrics`, `UiEffects`, semantic `UiColor`, a typed `UiStyle`/selector/stylesheet model, visual states, CSS variable generation, typed linear gradients, responsive media rules, flex/layout/sizing, transforms and transitions. `UiApplication` now carries a theme and style sheet and validates them before renderer setup.
- Tauri 2 receives generated theme variables and selector CSS at document start and publishes it in `window.__WL_UI_BOOTSTRAP__`. Its host continues to own frontend markup, controls, JS, plugins, capabilities and permissions.
- Win10 uses native Win32/GDI resources for the supported shell/theme properties and adds GDI32 linkage without introducing a webview or Windows App SDK dependency. Win11 applies its shared palette/typography/spacing/surface to the native WinUI shell and re-exports Windows Reactor controls from `wl_ui::windows11` on Windows.
- Legacy C adds versioned C89 theme/style structs, initializer/setter APIs and a style capability query. The baseline reports only background, foreground and font as implemented native style fields; other flags are rejected as unsupported.
- Public docs are under `docs/wl_ui/` and `docs/wl_ui/C/`. Rust compile/runtime verification must be done by CI and target builds; strict C89, CTest, sanitizer, and real Windows/legacy-OS GUI checks remain distinct gates.

## CSS compatibility audit — 2026-11-13

- Extended the style-range audit beyond the earlier negative-margin and percentage-length fixes. CSS transforms now accept finite negative scale values for mirrored layouts; the serializer already emits the signed values directly.
- Box-shadow offsets and spread may be signed, while blur radii remain finite and non-negative. Text shadows accept signed X/Y offsets and non-negative blur, and reject non-zero spread because CSS `text-shadow` has no spread component.
- Position insets (`top`, `right`, `bottom`, `left`) now validate finite values while allowing signed pixel/percentage offsets; outline offsets may be negative while outline widths remain non-negative.
- Added regression coverage for mirrored transforms, signed box-shadow geometry, signed text-shadow offsets, invalid text-shadow spread / negative blur, signed insets / outline offsets, and non-finite insets.
- Updated `docs/wl_ui/styling.md` to document these CSS-compatible ranges.
- Commits for this audit: `65172cd` (initial validation changes), `86bd982` (iterator correction), `2396f4c` (text-shadow validation), `b1f4ddc` (Rust formatting), `587d274` (shadow/transform documentation), `2b4ad12` (signed inset validation), `8ed3104` (inset/outline documentation), and `599f6eb` (separate valid text-shadow test fixture).
- Verification status: the workflow run created for the earlier `65172cd` merge candidate failed at `cargo fmt --check` on one test-call layout, before Rust compilation/tests. The formatting correction was committed as `b1f4ddc`; subsequent text-shadow and docs changes are not yet covered by a fresh confirmed CI result. Do not treat the earlier clean CI run for `48438aa` as verification of these later commits.

## Portable animation runtime and shared frame clock — 2026-11-14

- Added a platform-neutral Rust `wl_animate` runtime and its C89 `wl_Lanimate` counterpart. Both are caller-owned and host-driven: no timer, worker thread, message pump, OS window API or renderer is created by an animation or track.
- The Rust `wl_ui` facade re-exports `FrameClock`, `FrameTick` and the full runtime as `wl_ui::animate`. Generic `UiBackend` implementations own their event loop and can use the same clock on non-Windows hosts. Tauri continues to use validated CSS keyframes and the webview compositor path.
- Rust and C frame clocks share the same 60 FPS target, 100 ms delta cap, pause/resume baseline reset and fractional phase handling. Regression tests cover 90 Hz/144 Hz host polling and a maximum delta smaller than the target cadence. The C frame-clock hot path no longer calls floating modulo; the Rust path uses bounded subtraction rather than remainder.
- Keyframe tracks are capped at 64 frames in both runtimes. Rust keyframe storage is immutable and Arc-backed, so cloned playback tracks share one allocation; playback cursors are independent and advance at exact keyframe boundaries. Sequential samples stay on the cached segment; reverse seeks use binary search.
- The Rust and C easing paths use the degree-15 polynomial sine approximation; tests sweep the C and Rust curves against trigonometric references. Built-in Rust scalar/point/colour interpolation and C sampling do not allocate during playback. Custom Rust `Interpolate` implementations own their own clone/interpolation allocation behavior.
- Added matching 1,024-animation / 600-frame CPU workload benchmarks for Rust and C89. `build.py verify libraries` runs both and prints timing. The benchmarks assert all 600 clock ticks were emitted; they do not include GPU rendering and do not promise universal sub-1% utilization.
- Moved all Rust UI implementation crates beneath `rust/library/wl_ui/{core,backends}`. A tree audit confirms the old top-level `rust/library/wl_ui_core` path and the old separate `wl_ui_*` source roots are gone; Cargo package identifiers remain internal and the public consumer entry point is `wl_ui`.
- Added [docs/wl_ui/animation.md](../docs/wl_ui/animation.md) and matching Rust/C library documentation. WLDE's architecture points to the same clock/runtime; the WLDE renderer itself is not part of this PR.
- Relevant follow-up commits: `2ff16a6` (bounded Rust tracks), `1360b73` / `7e283b4` (exact-boundary cursors), `b4dbdc7` (C workload benchmark), `9dced0e` (verification integration), `aec6c87` / `f13a83e` / `c9684be` (C frame-count assertion and diagnostic), `7c69b40` / `4591946` (shared benchmark docs and timing distinction), `3f92a49` (Rust emitted-frame assertion), and `caf4dfb` / `392460b` (integer parity in the alternating animation hot path).
- Earlier source-equivalent verification passed Rust tests/Clippy/docs, GCC C89 with ASan/UBSan, C89 MSVC, Win10/Win11 packaging, Manager UI tests and repeated Rust/C sanitizers. The *latest* full matrix for the benchmark, track-limit, cursor-boundary, timing-label and integer-parity changes is queued against head `c9684be`; do not treat earlier green jobs as verification of this final head.



## WLDE portable scene core — 2026-11-14

- Started a separate WLDE implementation branch from the merged PR #10 baseline; PR #10 remains closed and is not being rewritten.
- Added Rust package wl_wlde and C89 companion wl_Lwlde. Both consume the existing animation frame clock and CPUv/GPUv admission contracts rather than introducing another timer or resource governor.
- Implemented bounded monitor topology snapshots, Fill/Fit/Stretch/Center placement math, conservative visibility scheduling, 60/30 FPS Normal/Optimize policies, Suspend handling and explicit presentation capability gates.
- C uses caller-owned storage for a maximum of 16 monitors and holds non-owning pointers to already-initialized wl_Lcpuv/wl_Lgpuv governor objects. Its placement/condition/tick paths do not allocate.
- Local validation completed for the C89 WLDE source with GCC 14 strict flags and focused contract tests using a small frame-clock test stub. This validates the new C contract/source syntax, not full integration with the repository C archives or MSVC.
- Cargo is not installed in the current implementation environment. Rust formatting, compilation, tests, Clippy and documentation must therefore be established by hosted CI; they are not claimed locally.
- This is an initial portable policy/scene layer, not a completed desktop presentation engine. OS monitor enumeration, actual static/animated desktop surfaces, decoders, GPU renderer/device-loss recovery, package/Store/widget/Rainmeter runtimes and recoverable Groups remain open. The full WLDE acceptance checklist stays open.


## WLDE Wallpaper Store provider hub — 2026-11-14

- Added a nested Manager navigation branch: Wallpaper → Wallpaper Store → Wallpaper Engine / Free Catalogues / Wallpaper Library.
- Added Rust provider metadata in wl_wlde::store for Wallpaper Engine, Openverse, Wikimedia Commons and NASA Image and Video Library.
- Added matching C89 provider metadata API wl_Lwlde_store_provider_get(); provider records are static/immutable and allocation-free.
- Added a Rust Wallpaper Engine bridge: bounded Steam root discovery, libraryfolders.vdf parsing, executable Browse validation, local target validation and the documented openWallpaper command launched with argument APIs.
- The Win10 Win32 picker defaults to the detected Workshop directory steamapps/workshop/content/431960; Win11 uses the same shared file-picker helper and provider metadata.
- Steam retains subscriptions/downloads/purchases. The bridge does not modify, unpack, delete or redistribute Workshop content.
- The free provider hub links to official Openverse, Commons and NASA catalogue pages. Each asset still requires exact license/credit verification; the app does not scrape or mirror these catalogues.
- Scope remains intentionally bounded: there is no hosted catalogue API, in-app search/cards/previews, direct download manager, attribution sidecar, signing/moderation, or complete offline wallpaper library.
- Local Rust/Cargo is not available in the authoring environment. Run Rust formatting/tests/Clippy/docs, Win10/Win11 package builds and strict C89+sanitizers in hosted CI. The C89 additions are included in the existing WLDE test target.


## WLDE modern Win32 monitor snapshot adapter — 2026-11-14

- Added a separate `wl_wlde_win32` package. The portable `wl_wlde` core retains `#![forbid(unsafe_code)]`; Win32 FFI is isolated in the platform adapter.
- The adapter enumerates attached desktop GDI display devices, filters mirroring-driver pseudo-devices, then reads monitor bounds, work area, device name and primary state via EnumDisplayMonitors / GetMonitorInfoW(MONITORINFOEXW).
- Snapshots are capped at WLDE's 16-monitor limit. `refresh_topology` replaces the current topology only after a full successful enumeration/validation; a failure preserves the old snapshot.
- Exposes a narrow host message predicate for WM_DISPLAYCHANGE and WM_DEVICECHANGE + DBT_DEVNODES_CHANGED. No timer, hook, polling worker or window is created. Win10/Win11 backend modules re-export the API.
- GDI device names are transient snapshot identifiers, not durable monitor profile IDs. DPI, orientation, remote-session reconciliation and actual message-loop wiring remain open.
- C89 `wl_Lwlde` remains platform-neutral; a C Win32 enumeration adapter is a separate task.
- Hosted CI is required for the MSVC target and Windows-specific FFI compile. Do not infer runtime hot-plug correctness from portable Linux tests alone.


## WLDE resource-pressure controller — 2026-11-14

- Added matching Rust and C89 allocation-free controller APIs for caller-supplied CPU, GPU, RAM and VRAM percentages.
- Evaluates the maximum of all four domains (no averaging that could hide a saturated GPU or memory domain).
- Requires explicit recovery < Optimize < Suspend thresholds and non-zero consecutive enter/recovery windows; rejects out-of-range samples and invalid policies.
- Resulting Normal/Optimize/Suspend pressure can be applied to the shared scene: Optimize selects 30 FPS, Suspend suppresses animation requests while preserving the last presented frame.
- CPUv/GPUv admission tickets remain distinct from system telemetry. No OS sampling provider, polling thread, sampling cadence or resource attribution has been added yet.
- The plan's 90% severe-pressure threshold remains a host policy/configuration choice, not a hardcoded default.


## WLDE Manager UI integration — 2026-11-14

- Expanded the shared Manager navigation from 18 to 23 pages and added dedicated WLDE destinations for Widgets, Resource & Performance, Desktop Groups, Rainmeter and Game Mode alongside Wallpaper, Store, Engine, Free Catalogues, Library and Displays.
- `manager_app()` now selects the WinLook Graphite dark theme and provides semantic Window, Navigation, Surface, Card and Button styles; it no longer starts in the generic white system palette.
- Windows 11 uses reusable bordered info/status cards for each WLDE area and exposes a manual display snapshot refresh. It clearly reports that the Windows Reactor display/device event bridge is still pending.
- Windows 10 uses the same dark palette, shows module-specific status content and has a manual monitor refresh action. Its native message loop now refreshes the topology summary on `WM_DISPLAYCHANGE` and `WM_DEVICECHANGE/DBT_DEVNODES_CHANGED`.
- The UI does not invent telemetry values: CPU/GPU/RAM/VRAM are shown as unavailable until a trusted sampler is integrated. Wallpaper Library import, native wallpaper rendering, widget execution and Rainmeter runtime remain unimplemented. Desktop Groups remain visibly disabled until crash-safe Shell restoration exists.
- Hosted CI must verify Rust formatting, manager unit tests, Win10/Win11 UI compilation/package builds, and the full build matrix on the new UI head.


## WLDE per-application Game Mode resolver — 2026-11-14

- Added matching Rust/C89 bounded per-application resolvers keyed by host-supplied executable SHA-256 bytes, not executable names.
- Resolver accepts foreground ownership, fullscreen state and an optional OS Game Mode signal; it evaluates up to 64 unique profiles without allocating.
- Precedence is exact user profile (including explicit Normal override), optional OS Game Mode fallback, optional fullscreen fallback, then Normal. Non-foreground applications always resolve to Normal.
- The resulting mode can be applied to the existing scene mode setter. Rust exposes `Scene::set_game_mode`; C exposes `wl_Lwlde_scene_set_game_mode`.
- The portable core does not enumerate processes, inspect or hash executable files, install hooks or poll Windows. Win10/Win11 signal collection, profile persistence and Manager editing remain open.


## WLDE virtual Groups, adaptive app icons and mandatory Explorer Desktop Bridge — 2026-11-14

- Added portable Rust wl_wlde desktop models for opaque desktop item IDs and typed File/Folder/Application references, local virtual Groups and transactional replacement validation.
- Group membership is bounded (64 Groups, 128 items per Group), rejects unknown/duplicate members and never creates physical directories or changes source paths.
- Added adaptive application-icon metadata with foreground/background layers, optional monochrome layer, Circle/RoundedSquare/Squircle masks and a 48–72% safe-zone range (61% default). Source icon artwork, shortcuts and executables are not rewritten.
- Added matching Rust/C89 type-aware context action contracts for Apps, Files, Folders and Groups. App-only Customize Icon and grouped-item Remove From Group are explicit; native Open/Properties verbs remain Shell-owned.
- Added matching C89 wl_Lwlde_desktop_groups_* and wl_Lwlde_adaptive_app_icon_validate() APIs using caller-owned bounded storage and non-owning asset/member references.
- Added Manager Icons and Desktop Groups cards describing the portable models, Shell integration and current implementation boundaries.
- Explorer Desktop Bridge is required BGW/WLDE runtime infrastructure wherever Explorer owns the desktop, across supported Windows generations; it is not a per-feature user opt-in and is not Windows 10/11-only.
- BGW identifies the actual desktop-owning Explorer process and chooses a generation-specific adapter. Supported Shell COM integration is preferred; controlled in-process runtime hooks are permitted where public interfaces do not expose the required desktop view behaviour.
- Explorer.exe on disk is never modified/replaced. DWM remains the compositor. Shell retains filesystem operations, activation, permissions and native item semantics.
- Expensive rendering, decoding and persistence stay out of Explorer callbacks. BGW monitors bridge/Explorer restarts, validates adapters and drives idempotent restore after failure/update/uninstall.
- Live desktop inventory, durable local persistence, icon extraction/rendering, native Group proxies, custom popup windows and full context-menu interception are not implemented in this slice. Groups remain disabled until restoration is proven.
- Official design references: Android adaptive icon layout, Microsoft Shell extensions, namespace extensions and separate Explorer desktop-process testing. See plan/references/SOURCES.md WLDE-007–010.
- Hosted CI must run Rust formatting/tests/Clippy, C89 GCC sanitizer and MSVC jobs, plus Win10/Win11 Manager builds. Legacy Shell adapters still need their own OS VMs and tests; modern CI cannot prove Windows 95–8.1 behaviour.


## WLDE Explorer desktop-host discovery — 2026-11-14

- Added DesktopHostIdentity, DesktopHostError, discover_desktop_host() and is_same_desktop_host() to rust/library/wl_wlde_win32.
- The adapter resolves the Shell desktop window with GetShellWindow, obtains the owning PID with GetWindowThreadProcessId, verifies the executable basename is explorer.exe, checks the process belongs to BGW's interactive session and records the process creation FILETIME alongside the PID.
- Rechecks the Shell desktop window and its owning PID before returning so an Explorer restart during discovery is rejected; BGW can re-probe before reconnecting and compare the full identity rather than PID alone.
- Opens the target process with PROCESS_QUERY_INFORMATION only, as required by ProcessIdToSessionId; this also supplies limited-query access for QueryFullProcessImageNameW and GetProcessTimes. It does not request process memory, thread creation, termination or modification rights.
- Exported the discovery API from the shared Win32 backend and the Windows 10/11 facades. The Windows 11 public module already re-exports its backend's public symbols.
- API constraint: GetShellWindow is documented from Windows 2000, GetProcessTimes from XP, while ProcessIdToSessionId and QueryFullProcessImageNameW are Vista+. The composed modern Rust probe is therefore Vista+ (the Rust Win10/Win11 targets are newer still); legacy OS targets need their own classic C adapter.
- This is host discovery only. The independent BGW executable now hosts a waitable monitor, but the probe still does not inject a DLL, register a Shell extension, enumerate desktop PIDLs, suppress native icons or draw Groups. Those remain separate implementation gates with restoration requirements.
- Focused checks: portable path-basename matching and non-Windows unsupported-platform behaviour are unit-tested. Windows runtime discovery still needs Win10/11 interactive-session testing and Explorer restart validation.
- Official references WLDE-011–015: plan/references/SOURCES.md.


## WLDE BGW initial Explorer host monitor — 2026-11-15

- Added independent binary package `rust/bgw` (`winlook-bgw`, binary target `winlook-bg`) and shared `rust/library/wl_bgw_runtime`; neither Manager package depends on BGW, preserving the two-executable process split.
- The BGW build runner selects only the dedicated `winlook-bgw` package, validates the manifest and entry point, builds the Cargo binary, and copies it to the required Windows filename `winlook.bg.exe`.
- Updated Cargo workspace membership, Cargo.lock, BGW CI path filters and artifact upload.
- `wl_wlde_win32::monitor_desktop_host()` opens Explorer with only PROCESS_QUERY_INFORMATION | SYNCHRONIZE, verifies the process creation FILETIME against the discovered identity, and waits with WaitForSingleObject.
- BGW revalidates the Shell desktop owner every five seconds, detects Explorer termination or owner replacement, drops the old process handle and rediscovers. It retries every two seconds when no compatible Explorer-owned desktop exists.
- Windows status diagnostics use OutputDebugStringW only; there is no persistent log, Manager status IPC or UI surface yet.
- This implementation is not the complete Explorer Desktop Bridge. It does not inject/load Shell code, register COM extensions, enumerate PIDLs, hide native desktop icons, install Groups, store restoration state, render wallpaper, register Windows startup or accept Manager commands.
- It is the modern Rust implementation used by current Win10/Win11 targets. Windows 95–XP and legacy C targets still need generation-specific host-process discovery and BGW/runtime equivalents; the Explorer-owned desktop contract remains mandatory wherever the Shell supports it.
- Build target: `python build.py build bgw`; expected output: `build/rust/bgw/winlook-bgw/release/winlook.bg.exe`.
- Windows runtime validation remains pending in CI. Portable host-discovery tests and build-runner tests cover the non-Windows stubs, package selection and executable rename.


## WLDE adaptive icon reusable output buffer — 2026-11-15

- Added Rust `render_adaptive_app_icon_into()` as a non-allocating companion to the existing `render_adaptive_app_icon()` helper. Both use the same compositor; the convenience helper allocates one vector and delegates to the caller-buffer path.
- The caller-buffer path validates the output size/capacity before writing, fills only the requested packed RGBA8 tile, and leaves extra trailing capacity untouched. It performs no allocation in the render function.
- Rust and C89 retain matching background cover-crop, foreground safe-zone contain, premultiplied-alpha bilinear sampling, source-over blend and 2×2 mask coverage for Circle/RoundedSquare/Squircle.
- BGW's rendering worker should reuse a tile buffer/cache and call the `*_into` function; the compositor must remain outside the Explorer in-process callback.
- The optional monochrome asset remains metadata only; no theme tinting, image decoding/extraction, icon registration or live Explorer tile display is implemented by this change.


## WLDE adaptive app-icon reusable output — 2026-11-15

- The shared C89 core already exposes caller-owned RGBA output through `wl_Lwlde_adaptive_app_icon_render_rgba()`.
- Added Rust `render_adaptive_app_icon_into()` to match that no-allocation contract. It writes into a caller-owned reusable RGBA buffer, validates capacity before writing and leaves any trailing capacity unchanged. The existing `render_adaptive_app_icon()` allocating convenience helper now delegates to the same compositor.
- Rust/C behavior matches: 16–512 square output, source dimensions bounded to 4096, centered background cover crop, foreground safe-zone contain, premultiplied-alpha bilinear sampling, source-over composition and 2×2 mask coverage for Circle/RoundedSquare/Squircle.
- The compositor is a bitmap operation only. Hosts still need Shell icon extraction/identity, monochrome tinting, DPI-aware cache policy and native icon registration. BGW must run it outside Explorer callbacks; live Shell presentation and crash-safe restore remain open.
- Added caller-buffer parity tests, output-capacity/no-partial-write tests and docs clarifying the distinction between bitmap composition and live desktop presentation.


## WLDE transactional desktop Group commands — 2026-11-15

- Extended Rust `DesktopGroups` with transactional `add_group`, `rename_group`, `add_members`, `remove_members` and `ungroup` operations. Every changed collection passes the existing full-set validator before commit; invalid actions retain the last known-good model.
- Removing the final member automatically removes the empty Group record; the referenced file, folder or app shortcut is never moved, copied, renamed or deleted.
- Added stable group lookup and explicit membership queries. Unknown Groups and attempts to remove a non-member return typed errors.
- Extended the right-click model with a Desktop Background target and WLDE actions Create Group / Open Desktop Settings. Native Shell entries and file operations remain Shell-owned.
- Added matching C89 desktop-background action values plus read-only Group lookup/membership helpers. C89 editing continues through caller-owned candidate arrays and transactional `wl_Lwlde_desktop_groups_replace()`.
- Re-exported `render_adaptive_app_icon_into()` from the Rust crate root so BGW can use the allocation-free compositor API described in the README.
- Added Rust/C89 tests for the new group actions, transactional failure behaviour, lookup and desktop-background menu actions. The BGW-owned binary Group/layout store is implemented in a later 2026-11-15 checkpoint; the native Explorer presenter, popup window and crash-safe icon restoration remain unimplemented.


## WLDE modern Desktop filesystem inventory — 2026-11-15

- Added `wl_wlde_win32::enumerate_desktop_items()` as a bounded, read-only snapshot of the current user's Desktop and Public Desktop Known Folders, resolved with `SHGetKnownFolderPath`.
- The snapshot is one level deep and returns a portable `DesktopItemRef` plus an ephemeral local path for immediate Shell operations.
- IDs prefer the volume-serial/file-index identity returned by `GetFileInformationByHandle`, with creation time included as an additional reuse discriminator. `DesktopInventoryItem::identity_stability()` distinguishes filesystem-backed IDs from path-hash fallbacks.
- Ordinary same-volume NTFS renames preserve filesystem-backed IDs. Missing/partial identity queries, reported hard links or duplicate identities in the visible inventory use distinct path-hash fallbacks; fallback IDs are not rename-stable.
- Capacity is fixed at 4096 entries. Missing Public Desktop is treated as empty; access or iteration failures abort the snapshot rather than returning partial data.
- Common launcher extensions are currently classified as Applications; .lnk targets are not yet resolved, so shortcuts to documents/folders need a later Shell-link resolver.
- The pass does not traverse folders, follow directory links, read file contents or mutate Shell items. It does not include virtual Shell namespace items, hidden/visibility state or the Explorer view's current filtered membership. A future IShellFolder/IFolderView adapter is required for exact desktop view integration.
- Raised the portable display-label limit to 1023 UTF-8 bytes (C89 buffer to 1024 bytes) so long Unicode filenames can be represented without truncation.
- sha2 0.10.9 remains a direct Win32-adapter dependency for the explicitly marked path fallback.
- Official references WLDE-021–026 are recorded in plan/references/SOURCES.md. Windows runtime enumeration and cross-user/Public Desktop behaviour remain CI/device verification tasks.

## WLDE themed monochrome adaptive app icons — 2026-11-15

- Added matching Rust and C89 monochrome render paths. The optional monochrome layer's alpha channel is treated as the glyph mask; its RGB values are ignored and replaced with the caller's theme tint.
- Background cover crop, foreground/glyph safe-zone contain, premultiplied-alpha sampling, source-over blending and the Circle/RoundedSquare/Squircle 2×2 coverage mask match the color renderer.
- Added allocating and caller-buffer Rust APIs plus an allocation-free C89 API. Caller-buffer paths retain the same bounded 16–512 px output contract and do not modify source assets.
- Added Rust/C89 tests for tint application, source immutability, safe-zone and output parity.
- This is a bitmap operation only. Shell icon extraction, app-profile color selection, DPI cache management, icon registration/presentation and crash-safe restore remain separate native integration gates.


## WLDE Manager Icons and Desktop Groups status pages — 2026-11-15

- Updated the existing Win10 and Win11 Icons pages to state that the shared color and themed-monochrome bitmap composers are ready. The monochrome alpha-mask/theme-tint behaviour is now reflected accurately; Shell extraction, DPI cache policy and icon registration remain adapter work.
- Updated both Desktop Groups pages to describe transactional Group commands and the BGW read-only User/Public Desktop filesystem inventory.
- Added explicit boundaries for work not yet wired: Manager/BGW status and configuration IPC and a Manager-driven save/edit route, exact Shell-view membership, virtual namespace entries, Explorer context-menu registration, native icon presentation, live Group tiles and crash restoration. BGW can load and validate an existing local Groups/layout snapshot, but Manager cannot yet submit edits through IPC.
- Kept Groups disabled for live presentation until native icon restoration can be demonstrated across BGW crash, Explorer restart, disable, session exit and uninstall.


## WLDE normalized desktop layout — 2026-11-15

- Added paired Rust/C89 top-level placement models for ordinary desktop items and Group tiles.
- Each entry stores an Item/Group target, a current monitor ID, normalized x/y anchors in 0–10,000 basis points against the monitor work area, and a stable z-order hint.
- Every ungrouped inventory item and every Group must have exactly one placement. Group members cannot also have standalone desktop-icon positions.
- Replacement validates inventory identity, Group membership/uniqueness, monitor membership, coordinate bounds, duplicate targets and completeness before committing. Failed replacement preserves the last valid layout.
- Bounded the model at 4,096 inventory records plus 64 Groups (4,160 layout placements); C89 uses caller-owned fixed storage and no allocation.
- Added Rust and C89 tests for valid multi-monitor placement, Group tile membership, duplicate target rejection, unknown monitor rejection, invalid coordinate rejection, incomplete layout rejection and transactional preservation.
- This remains data only. There is no live desktop tile presenter, native icon suppression, Shell context-menu hook, persistent layout storage or verified crash-safe restore yet.


## WLDE filesystem change-notification evidence — 2026-11-15

- Added primary Microsoft Learn references WLDE-023/024 for FindFirstChangeNotificationW, FindNextChangeNotification, FindCloseChangeNotification and WaitForMultipleObjects.
- The modern watcher waits on non-recursive User/Public Desktop directory handles. A signal is only a coarse indication that something changed, so BGW re-enumerates the bounded snapshot rather than assuming which item changed.
- The documented minimum client is Windows XP; the Win10/11 Rust adapter must not be copied into the Windows 95/98/Me/NT4/2000 C targets without a separate compatibility design.
- UNC/Desktop redirects use a five-second refresh fallback because remote change notification delivery is not guaranteed. Long paths receive an extended-length prefix where required.


### Desktop watcher resource policy — 2026-11-15

The User/Public Desktop watcher selects only file-name and directory-name
notifications because those are the changes that can add, remove or rename a
top-level desktop entry. It does not wake for ordinary content writes. When
the path is UNC/remote, or a Known Folder is temporarily absent, BGW uses a
finite five-second inventory refresh fallback; local existing folders remain
event-driven. The wait owns at most two handles and re-arms each signaled
notification before waiting again.


## WLDE stable Desktop filesystem identities — 2026-11-15

- The modern Win32 inventory now prefers the 64-bit volume serial + 128-bit `FILE_ID_128` returned by `GetFileInformationByHandleEx(FileIdInfo)`, with creation FILETIME as an additional key discriminator. It deliberately ignores the legacy 64-bit file index because Microsoft does not guarantee uniqueness on ReFS.
- The handle requests only `FILE_READ_ATTRIBUTES`, permits concurrent read/write/delete sharing, opens directories with `FILE_FLAG_BACKUP_SEMANTICS`, and opens reparse points themselves with `FILE_FLAG_OPEN_REPARSE_POINT`. It does not read file contents or follow a reparse target.
- `DesktopInventoryItem::identity_stability()` distinguishes `FilesystemFileId` from `PathFallback`; persistent Group storage must not treat path fallbacks as rename-stable.
- File IDs are used only when the identity is unique in the current inventory and Windows reports exactly one hard link. This keeps hard-link names as separate entries and avoids changing a hard-link's ID between stable and ambiguous states.
- Windows documentation notes filesystem variation: file IDs may be reused after deletion and remote filesystems can return partial data. The 128-bit file ID avoids ReFS's documented legacy 64-bit index limitation; a zero/unsupported ID or missing creation timestamp becomes a path fallback. Creation time reduces accidental reuse but is not a formal guarantee.
- Focused portable tests cover identity construction and fallback selection. Windows-only tests cover same-volume rename continuity and hard-link disambiguation; hosted Windows CI must execute them.
- Official sources WLDE-025–027 are recorded in `plan/references/SOURCES.md`. Windows-only tests now exercise same-volume rename continuity and hard-link fallback on CI. This modern Rust adapter work does not implement legacy C adapters for Windows 95–8.1; those still need generation-specific identity providers.


## WLDE local Group/layout persistence foundation — 2026-11-15

- Added `rust/library/wl_bgw_runtime/src/desktop_store.rs` as the BGW-owned machine-local store. Its default Windows location is `%LOCALAPPDATA%/WinLook/WLDE/desktop-state.v1.bin`.
- The version-1 binary stores Group IDs/labels/member IDs, per-member identity kind and normalized layout entries. It stores no filesystem path, file content, executable path, source artwork or portable-template data.
- The format has a 4 MiB cap, strict field/count/string bounds, exact-length parsing and CRC32 corruption detection. CRC32 is not an authenticity mechanism.
- Saves validate against the complete current inventory and monitor topology, write a unique temporary file in the same directory, sync it, and rename it into place. BGW is the single-writer owner.
- Loads revalidate Groups, inventory identity kinds, complete placements and monitor IDs. Stale fallback IDs, missing/new inventory entries, changed monitor IDs, corrupt data or incompatible format versions are rejected without rewriting the saved file.
- BGW loads the snapshot after verifying the Explorer host and refreshes the validation after Desktop inventory notifications. This only restores the in-memory configuration model; it does not display Groups or suppress native icons.
- The Manager-driven edit/save path is not implemented because authenticated Manager/BGW IPC is still open. The store API supports save, but the running Manager does not yet call it.
- Current GDI monitor IDs are snapshot keys rather than durable physical display identities; a changed key safely rejects layout restoration until WLDM monitor mapping/reconciliation exists.
- This local configuration store is separate from the crash-safe native icon/Shell restoration journal. Groups remain presentation-gated.


## WLDE read-only Shell desktop icon extraction — 2026-11-15

- Added `wl_wlde_win32::extract_desktop_shell_icon()` using `SHGetFileInfoW` for existing User/Public Desktop filesystem paths. Supports small and Shell-sized large icon requests.
- The returned `DesktopShellIcon` owns the HICON and destroys it on drop. The HICON metadata path uses `GetIconInfo` and `GetObjectW`; any HBITMAPs created by `GetIconInfo` are released with `DeleteObject`.
- This is handle + dimensions only, not RGBA pixel extraction. The next renderer boundary is a bounded BGRA/RGBA rasterizer with alpha/mask handling and a reusable cache, then the existing portable adaptive icon compositor.
- Shell icon extraction may invoke a Shell handler and is documented for a BGW worker, never the Manager UI thread. Missing/unsupported icons return a typed error; no native desktop icon registration changes occur.
- The API remains path-based, enforces the documented SHGetFileInfoW MAX_PATH input bound and does not enumerate virtual Shell namespace items or the Explorer view's exact filtered membership. PIDL/IFolderView integration remains part of the Explorer Desktop Bridge.
- Microsoft sources WLDE-028–030 are recorded in `plan/references/SOURCES.md`.


## WLDE Shell icon RGBA rasterization — 2026-11-15

- Added `rasterize_desktop_shell_icon_rgba()` after the owned HICON acquisition step. Output is tightly packed straight-alpha RGBA suitable for `wl_wlde::AdaptiveIconImageView`.
- The rasterizer is bounded to 512×512 pixels and uses a top-down 32-bpp DIB selected into a memory DC. It draws the same icon onto opaque black and white probe backgrounds and reconstructs alpha and straight color from their channel differences.
- The dual-background method handles standard source-over alpha icons and classic AND/XOR icon masks where the result is representable as a static RGBA image. Screen-dependent inversion pixels are rejected with `UnsupportedRasterOperation`; no background-dependent operation is silently converted into a guessed color.
- The DIB/DC/selected-object lifetime is RAII-managed. `GdiFlush` runs before reading the directly accessible DIB bits, and the original selected object is restored before the DIB is deleted. Output allocation uses checked lengths and `try_reserve_exact`; invalid dimensions, oversized icons, allocation failure and Win32 drawing failures return typed errors.
- Added portable pixel-math tests for opaque/transparent, half-alpha and inversion cases. Windows smoke coverage still needs to execute the HICON-to-RGBA path on hosted CI.
- BGW now uses the bounded identity/DPI/theme/size-aware in-memory cache and Shell worker described below. Feeding cached pixels into the portable adaptive compositor and live presentation remain later work. No native icon registration, Explorer view object, live Group tile or restoration behaviour is changed.
- Microsoft sources WLDE-031–033 are recorded in `plan/references/SOURCES.md`.


## WLDE BGW memory-only Shell icon pipeline — 2026-11-15

- Added rust/library/wl_bgw_runtime/src/desktop_icon_pipeline.rs: one dedicated Shell worker, bounded 16-entry work/result queues, an 8 MiB / 256-entry LRU RGBA cache, and generation/identity checks that discard stale results.
- Worker calls SHGetFileInfoW only after initializing an STA COM apartment; CoUninitialize is paired through an RAII guard. It pumps the STA message queue while idle and during result-channel backpressure. The BGW inventory/Explorer monitor loop only queues and polls, never waits on the Shell handler. It polls every 250 ms while icon requests are pending and returns to the five-second host cadence when the queue drains.
- Cache identity is opaque item ID + identity stability + Shell icon size + DPI bucket + theme generation. Initial context is 96 DPI/theme generation 0 pending actual BGW DPI/appearance events.
- Inventory refresh removes stale identities and invalidates path-fallback artwork even when a path-derived ID string remains the same. Filesystem-ID artwork is retained for identities still present. Host/context generation changes drop old pixels and stale worker results.
- The cache is memory-only and is not part of desktop-state persistence. Stable filesystem-ID failures are suppressed for the current host/context; path-fallback failures retry after inventory refresh. The cache exposes RGBA as a borrowed WLDE AdaptiveIconImageView without copying. It does not draw tiles or change Explorer/native icon state; Group presentation remains gated on the separate crash-safe restoration journal.
- Microsoft COM initialization and STA message-dispatch evidence is recorded as WLDE-035–036 in plan/references/SOURCES.md.

## WLDE crash-safe Explorer restoration journal contract — 2026-11-15

- Refined `plan/architecture/restoration.md` into a write-ahead contract for the future Explorer Desktop Bridge. The current Groups/layout state file remains configuration only and is explicitly not a restoration journal.
- Required a separate, protected, append-only, bounded journal with typed allowlisted adapter/resource IDs, versioned records, monotonic sequence numbers, original/desired values and adapter ownership evidence. No arbitrary commands, paths, handles, pixel buffers or credentials may be replayed.
- Each reversible OS mutation must have durable `Prepared` and `Applying` transitions before the adapter call; after mutation, a fresh readback plus ownership proof is required before `Committed`. Restoration is itself journaled before applying, verified after, and idempotent.
- Recovery distinguishes Prepared (no side effect started), Applying (inspect and prove ownership before any restore), Committed (active WinLook-owned state) and Restoring (inspect/complete or retry). External changes or missing ownership become Conflict, never blind rollback.
- Windows write-ahead persistence requires the native journal backend to flush the writable handle with `FlushFileBuffers` before mutation. `ReplaceFileW(REPLACEFILE_WRITE_THROUGH)` is not a valid substitute because Microsoft documents that flag as unsupported; `MoveFileExW(MOVEFILE_WRITE_THROUGH)` is documented for file moves, not as a record-level append protocol.
- This remains design only: no native Shell mutation, icon suppression, Shell registration, Group presenter or hook was enabled. Sources WLDE-037–039 were added. Implementation must pass fault injection at every write/flush/native-operation boundary and per-build ownership/restoration tests before the gate opens.


## Portable WLDE restoration record core — 2026-11-15

- Added `wl_wlde::DesktopRestoreLedger` in `rust/library/wl_wlde/src/restore_journal.rs`: bounded, versioned length-delimited record codec; CRC32; monotonic global sequence; immutable adapter/resource/original/desired/ownership snapshots; strictly validated transaction state transitions; explicit Aborted/Conflict outcomes.
- The decoder distinguishes an incomplete final append from a complete corrupt frame and exposes `valid_bytes` for a future storage writer. The pure recovery planner makes no restoration recommendation unless the observed desired value and adapter ownership proof both match.
- This is a reusable portable codec/state-machine only. It does not open or lock a file, inspect user ACLs, enforce the runtime adapter registry, flush durable storage or call native APIs. The actual BGW journal writer and platform adapters are still required before any desktop mutation can be enabled.
- Added public exports and library README/catalog guidance. Portable unit tests cover round-trip histories, truncation at every byte position, checksum corruption, transaction/sequence rules, resource bounds and safe recovery decisions; hosted Rust CI must verify them.
