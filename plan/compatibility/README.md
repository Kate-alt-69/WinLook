# Windows Compatibility Research

## Purpose
This directory defines what "Windows 95 and later compatibility" means for WinLook. Read the target OS page before selecting APIs, compiler/linker flags, runtime dependencies, process techniques, graphics paths, installers or display behaviour.

These are research/architecture notes, not release-support claims. A feature is not supported merely because a similar API exists on another Windows generation.

## Capability-based compatibility
Evaluate OS family, exact version/build/service pack/edition, OS/process architecture, API exports, COM interfaces, runtime/redistributables, graphics APIs/driver, DWM state where present, target process mitigations, Shell/window implementation and tested adapter support.

Never infer API availability from a modern header or SDK alone. Late-bind optional APIs or build separate binaries so missing imports cannot prevent startup.

## Families
- Win9x: Windows 95/98/ME; related but distinct releases; C legacy build, USER/GDI/Shell APIs; no DWM.
- Older NT: NT4/2000/XP; not Win9x. Different kernel/process/service model; no DWM before Vista.
- Vista/7: DWM exists but composition can be conditional; query it and provide fallback. Session 0 becomes non-interactive for services starting Vista.
- Windows 8/8.1: DWM composition always on from Windows 8; IDCompositionDesktopDevice documented for 8.1, not 8.
- Windows 10/11: modern DWM/Shell; IVirtualDesktopManager documented for Windows 10 but limited to its published interface; process mitigations and build differences affect injection.

## Evidence labels
- Verified (documentation): primary source explicitly documents a fact.
- Verified (runtime): WinLook was reproduced on the exact OS/build/runtime with a test record.
- Partially verified: source supports the general fact, exact build/runtime untested.
- Research required: plausible approach, not yet supported by adequate source or runtime evidence.
- Unsupported: known unavailable or intentionally excluded.

## Verified API facts
- SetWinEventHook minimum documented client is Windows 2000; Microsoft separately notes Active Accessibility 1.3 RDK for Windows 95 and NT4 SP6.
- Active Accessibility table: Win95 did not include MSAA; Win98 initially 1.1; Me/2000 1.3; NT4 from SP6 1.3; XP/Vista 2.0; Windows 7 3.0.
- AnimateWindow minimum documented client is Windows 2000.
- DWM APIs start with Vista; Vista/7 composition is conditional; Windows 8 makes DWM always on.
- IDCompositionDesktopDevice minimum is Windows 8.1.
- IVirtualDesktopManager is documented for Windows 10 and does not provide every create/switch operation WLWM may want.
- PROCESS_MITIGATION_DYNAMIC_CODE_POLICY is documented from Windows 8.1; ProhibitDynamicCode blocks dynamic code generation/executable-code modification.
- Session 0 becomes reserved for services/non-interactive processes from Vista.
- Open Watcom documents distinct Win95 and NT/Win32 targets; runtime compatibility remains test-dependent.

## Required platform test record
Record exact edition/build/service pack/architecture, hardware or VM, compiler/linker/runtime, API/redistributable probes, feature/module, expected/observed behaviour, evidence/log location, limitations and test date. VM success does not prove all graphics drivers or anti-cheat/hardware combinations.

## Per-version pages
- [Windows 95](win95.md)
- [Windows 98 / SE](win98.md)
- [Windows Me](winme.md)
- [Windows NT 4.0](nt4.md)
- [Windows 2000](win2000.md)
- [Windows XP](winxp.md)
- [Windows Vista](vista.md)
- [Windows 7](win7.md)
- [Windows 8](win8.md)
- [Windows 8.1](win81.md)
- [Windows 10](win10.md)
- [Windows 11](win11.md)

See the [matrix](matrix.md) and [source ledger](../references/SOURCES.md).
