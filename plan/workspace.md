# WinLook Workspace Memory

> Durable project context for Research Companion sessions and other coding agents. Last verified: 2026-11-08. Current user instructions and repository source take precedence.

## Project identity
WinLook is a Windows customization platform bringing deep desktop and appearance customization to Windows 95 and later. It draws inspiration from Linux desktop/window-manager communities but is its own Windows product, not a Hyprland port or Linux config importer. Repository: Kate-alt-69/WinLook. Living plan: [Plan.md](Plan.md). Final-Plan.md has not been created.

## Agreed architecture decisions
- Modern implementation: Rust. Legacy implementation: separate C implementation/toolchain per OS family. Shared conceptual APIs/contracts, not one binary.
- Windows windowing and DWM remain intact. WLWM augments Windows; it does not replace DWM.
- WLWM may inject/hook compatible apps. Main BGW coordinates process detection, compatibility checks/cache, injection and module health.
- WLWM classifies applications before injection and caches compatibility results locally in a binary format. Cache identity/invalidation includes target binary/version/hash, architecture, OS/build, relevant mitigations, WLWM module and compatibility DB versions. Cache hits never skip runtime safety checks.
- Explorer.exe must not be modified/replaced on disk. Explorer-targeted injection is excluded by default pending a separate reversible policy. DWM and authentication/security-critical processes are excluded.
- WLWM Settings are in WinLook Manager and can be included in WLT; no standalone WLWM config language required.
- wl_animate is the reusable Rust animation crate/API for modern WinLook and third-party integrations. wl_Lanimate is the C counterpart for legacy systems. The suite serves WLWM, WLDE, WLOSDE and Manager UI.
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


## Initial implementation checkpoint — animation primitives
- Added `crates/wl_animate`: renderer-independent Rust primitives for interpolation, easing, caller-driven tweens, playback state, keyframes and sampling. It is intended as a reusable crate for WinLook and third-party integrations. Initial version is alpha; API is not stable.
- Added `legacy/wl_Lanimate`: C89-compatible legacy library with corresponding concepts, caller-owned storage and no per-frame heap allocation in the library core.
- Exact C host test: `cd legacy/wl_Lanimate && make clean && make test`; passed with GCC 14.2.0 and `-std=c89 -Wall -Wextra -Werror -pedantic -O2`.
- Rust verification: not run because `rustc` and `cargo` are unavailable in the current execution environment.
- Windows/legacy runtime verification: not run. In particular, no Open Watcom build, Windows 95 VM test, Win32/DWM/GDI backend, BGW integration or injection is included yet.
- No compiled artifacts are committed. See [animation-suite.md](architecture/animation-suite.md) for implementation scope and next gates.

## Repository state
- Root-level Plan.md was moved to plan/Plan.md in commit bd1b7281af700e7efa4dcda67f994ab79dde43f5.
- Per-OS compatibility pages, subsystem architecture docs, source ledger and workspace index/memory were added in that documentation change.
- Follow-up plan/source/index pass: 35f8deab45f354cce2e96ebce31ba6874e081f3c.
- Workspace state refresh: 613b46bf0faff10a59686320e92ab5385944818c.
- Architecture detail pages cover appearance/icon customization, standby/session security, extension/community formats, WLDE, WLWM, WLT, WLOSDE/WLDM, animations and restoration.
- Technical citations are maintained in [references/SOURCES.md](references/SOURCES.md); per-OS research is under [compatibility/](compatibility/README.md).
- User explicitly authorized direct commits to main for this work.
- These documents are planning/research; they do not claim that code is implemented or tested.

## Detailed planning checkpoint — 2026-11-08

- User confirmed this task remains in planning/specification; do not implement engines, UI or move source files yet.
- Proposed source layout: parallel C/ and rust/ source trees; uppercase acronym engine directories; a dedicated, split UI/ under each language; public Rust engine crates under rust/crate/; C public headers plus a proposed C/index.c umbrella; reusable WinLook libraries under root lib/.
- Existing animation prototypes are planned to move from crates/wl_animate and legacy/wl_Lanimate to lib/wl_animate and lib/wl_Lanimate only after structure approval and a separate migration plan.
- The exact purpose of the root wllook/ directory is unresolved; current proposal reserves it for product/packaging metadata and shared assets, not duplicate engine source.
- Manager UI direction: WinToys-inspired clear left navigation and focused pages, familiar/native controls, split pages and reusable components. Do not copy its implementation or commit to WinUI 3 for legacy targets.
- Added [repository layout](architecture/repository-layout.md), [UI architecture](architecture/ui-architecture.md), [documentation standards](architecture/documentation-standard.md), [runtime contracts](architecture/runtime-contracts.md), [roadmap](roadmap.md) and [compatibility test strategy](compatibility/test-strategy.md).
- Expanded detailed contracts for WLWM target lifecycle/cache/injection policy, WLDE wallpaper/resource/Game Mode/Groups, WLT import/export/security, WLOSDE/WLDM lifecycle, restoration journaling and compatibility planning.
- No source files were moved and no product implementation was added in this planning update.
- The installed Research Companion skills and GitHub connector were used. Dedicated local Research Companion helper scripts are not exposed in this session; no claim is made that they ran.

