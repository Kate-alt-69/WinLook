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
