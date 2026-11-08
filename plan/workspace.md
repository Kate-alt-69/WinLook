# WinLook Workspace Memory

> Durable project context for Research Companion sessions and other coding agents. Last verified: 2026-11-09. Current user instructions and repository source take precedence.

## Project identity
WinLook is a Windows customization platform bringing deep desktop and appearance customization to Windows 95 and later. It draws inspiration from Linux desktop/window-manager communities but is its own Windows product, not a Hyprland port or Linux config importer. Repository: Kate-alt-69/WinLook. Living plan: [Plan.md](Plan.md). Final-Plan.md has not been created.

## Agreed architecture decisions
- Modern implementation: Rust. Legacy implementation: separate C implementation/toolchain per OS family. Shared conceptual APIs/contracts, not one binary.
- Windows windowing and DWM remain intact. WLWM augments Windows; it does not replace DWM.
- WLWM may inject/hook compatible apps. Main BGW coordinates process detection, compatibility checks/cache, injection and module health.
- WLWM classifies applications before injection and caches compatibility results locally in a binary format. Cache identity/invalidation includes target binary/version/hash, architecture, OS/build, relevant mitigations, WLWM module and compatibility DB versions. Cache hits never skip runtime safety checks.
- Explorer.exe must never be patched/replaced on disk. Deep Explorer customization is accepted scope: documented Shell extensions plus a separate opt-in, per-build presentation adapter for UI surfaces without public APIs. This is not general WLWM injection; Windows retains native Shell/filesystem operations. DWM and authentication/security-critical processes are excluded.
- WLWM Settings are in WinLook Manager and can be included in WLT; no standalone WLWM config language required.
- wl_animate is the reusable Rust animation crate/API for modern WinLook and third-party integrations. wl_Lanimate is its C counterpart. The current implementation baseline for both libraries is Windows 10; C89 is a source-language constraint, not evidence of pre-Windows-10 runtime support. The suite is intended to serve WLWM, WLDE, WLOSDE and Manager UI.
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

## Detailed planning checkpoint — 2026-11-08 (historical; superseded for animation libraries)

- At that point the user confirmed the task remained in planning/specification; the later explicit implementation request supersedes that restriction for the two animation libraries only. Do not infer permission to implement the other engines/UI or to migrate the source layout.
- Proposed source layout: parallel C/ and rust/ source trees; uppercase acronym engine directories; a dedicated, split UI/ under each language; public Rust engine crates under rust/crate/; C public headers plus a proposed C/index.c umbrella; reusable WinLook libraries under root lib/.
- Existing animation prototypes are planned to move from crates/wl_animate and legacy/wl_Lanimate to lib/wl_animate and lib/wl_Lanimate only after structure approval and a separate migration plan.
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
