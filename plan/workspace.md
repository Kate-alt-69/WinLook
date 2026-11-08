# WinLook Workspace Memory

> Durable project context for Research Companion sessions and other coding agents. Last verified: 2026-11-08. Current user instructions and repository source take precedence.

## Project identity
WinLook is a Windows customization platform bringing deep desktop and appearance customization to Windows 95 and later. It draws inspiration from Linux desktop/window-manager communities but is its own Windows product, not a Hyprland port or Linux config importer. Repository: Kate-alt-69/WinLook. Living plan: plan/Plan.md. Final-Plan.md has not been created.

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
- Session 0 becomes reserved for services/non-interactive processes from Vista; before Vista first user also shared Session 0.
- Open Watcom documents separate Win95 and NT/Win32 targets; no guarantee every runtime works on every release.

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

## Repository state
- Root-level Plan.md was moved to plan/Plan.md in commit bd1b7281af700e7efa4dcda67f994ab79dde43f5.
- Per-OS compatibility pages, subsystem architecture docs, source ledger and workspace index/memory were added to main in that documentation change.
- User explicitly authorized direct commits to main for this work.
- These documents are planning/research; they do not claim that code is implemented or tested.
