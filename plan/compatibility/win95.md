# Windows 95 Compatibility Notes

> Status: research baseline, not a release-support claim.  
> OS family: Win9x. Version scope: 4.00 family; record retail/OSR revision.  
> Last reviewed: 2026-11-08.

## 1. Purpose
This page describes what WinLook must consider on Windows 95 before selecting APIs, compiler/linker flags, runtime dependencies, process techniques, rendering paths or installer behaviour. Read it before writing platform-specific code.

## 2. Verified platform facts
- Windows 95 has no built-in Microsoft Active Accessibility; Microsoft's platform table says it does not support version 2.0 and documents Active Accessibility 1.3 RDK as an install route.
- SetWinEventHook is documented with Windows 2000 minimum; Microsoft separately lists the 1.3 RDK for Windows 95. Do not assume it exists on a clean installation.
- AnimateWindow is documented with Windows 2000 minimum; it is not a Windows 95 dependency.
- Open Watcom documents a 32-bit x86 win95 executable/DLL target distinct from NT/Win32 targets.

Primary references:
- [Active Accessibility supported platforms](https://learn.microsoft.com/en-us/windows/win32/winauto/supported-platforms)
- [SetWinEventHook](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwineventhook)
- [AnimateWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-animatewindow)
- [DwmIsCompositionEnabled](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmiscompositionenabled)
- [Open Watcom Linker Guide](https://open-watcom.github.io/open-watcom-v2-wikidocs/lguide.html)

A source link supports only the claim it documents. It does not prove a WinLook adapter works on this OS.

## 3. Toolchain and binary strategy
Use the legacy C build with a Win9x-compatible target and runtime. Open Watcom documents a distinct 32-bit x86 win95 target; verify the exact linker, CRT, DLL imports, calling conventions and binary load on this release. Do not assume modern Rust or modern system DLL availability.

- Choose compiler/linker/runtime after checking the target OS loader, imported DLLs, calling conventions, CRT assumptions and CPU baseline.
- Keep modern Rust and legacy C builds separate.
- Do not require modern Unicode-only exports or a modern CRT on an old OS without a tested compatibility route.
- Use capability checks/late binding for optional APIs so missing imports cannot prevent startup.
- Put OS-specific calls behind the corresponding adapter.

## 4. WLWM compatibility
Use a conservative 32-bit C/Win9x adapter. Verify every USER/GDI/Shell API against the target SDK and actual OS. Avoid imports of newer DLL exports and modern CRT assumptions. Process injection is an independent high-risk research item: prove the mechanism, target behaviour and unload/recovery on Windows 95 before claiming support.

WLWM's local binary compatibility cache is only an analysis cache. Every launch still validates current target identity, architecture, relevant OS/build and mitigation state. Unknown or partially-supported targets must retain native Windows behaviour unless a reviewed experimental feature is explicitly enabled.

## 5. Animation suite
wl_Lanimate only: GDI/USER, timers/message loop and bounded memory. Do not depend on AnimateWindow, DWM, DirectComposition or D3D11. Implement simple WinLook-owned surfaces first; profile handle leaks and old GPU drivers.

The modern library is wl_animate; the legacy C library is wl_Lanimate. They share conceptual capabilities where the OS permits, not necessarily the same implementation. Do not call a Win32 API before its documented minimum/runtime requirement has been verified.

## 6. WLDE desktop and wallpaper
WLDE begins with static wallpaper and simple GDI motion. Desktop icon Group suppression is high risk and must stay off until original native icon visibility can be restored after forced process termination. Video/web/3D/audio-reactive formats are not assumed.

WLDE Groups are virtual presentation proxies, never filesystem folders. Keep original desktop items and their targets intact. Group support remains disabled until its OS adapter can restore the original icons after renderer crash, disable and uninstall.

## 7. WLDM and WLOSDE
WLDM should begin with primary-display geometry; investigate legacy multi-monitor separately. WLOSDE should use ordinary WinLook windows; no secure-desktop emulation.

WLOSDE must use WinLook-owned ordinary UI surfaces. It must not imitate or replace native authentication, UAC or secure-desktop UI. WLDM exposes only display functions verified for this OS/driver combination.

## 8. WLT, assets and restoration
WLT import must validate the archive, reject unsafe paths/collisions, cap file counts/sizes/compression ratios, avoid modern runtime requirements, apply only supported components and report skipped items. Fonts/assets require licensing and user consent. The uninstaller must restore WinLook-owned changes and preserve original user files/shortcuts.

## Audio & Media and File Explorer notes

- This OS predates Vista Core Audio and GSMTC. WinLook still supplies its mixer/media UI, but only offer global/device mixer controls actually exposed by the selected driver. Do not promise generic per-process volume.
- Media metadata, artwork and transport require adapters for named players and versions; there is no system-wide Now Playing API equivalent to GSMTC.
- Explorer customization must use classic Shell capabilities verified on Windows 95. Do not assume XP-era `IShellView`, Windows 7 `IExplorerCommand` or Windows 11 context-menu registration.
- Any deep visual adapter is experimental until tested on the exact Shell build and must not patch or replace `explorer.exe`.

## 9. Required verification
- [ ] Exact OS build/service pack/edition recorded.
- [ ] Compiler/linker/runtime and imported DLLs recorded.
- [ ] API/redistributable probes recorded.
- [ ] WLWM window/event/injection/fallback tests run.
- [ ] wl_Lanimate backend profiled.
- [ ] WLDE desktop/wallpaper/Group recovery tested.
- [ ] WLDM/WLOSDE capability and fallback tested.
- [ ] WLT import and clean uninstall tested.
- [ ] Real hardware tested where practical; VM-only evidence is labelled as such.

## 10. Open questions
- Which exact edition/build is the baseline?
- Which compiler/runtime combination loads reliably?
- Which APIs are present without optional redistributables?
- Which injection methods are safe, observable and unloadable?
- Which graphics/display paths work with actual drivers?
- Can desktop icon presentation be safely changed and restored?
- Which WLT, wallpaper and uninstall features can be supported without unsupported system modification?

Update this page with test evidence as compatibility is established; do not mark this OS supported by inference.
