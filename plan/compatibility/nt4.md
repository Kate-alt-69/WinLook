# Windows NT 4.0 Compatibility Notes

> Status: research baseline, not a release-support claim.  
> OS family: NT. Version scope: NT 4.0; service pack matters.  
> Last reviewed: 2026-11-08.

## 1. Purpose
This page describes what WinLook must consider on Windows NT 4.0 before selecting APIs, compiler/linker flags, runtime dependencies, process techniques, rendering paths or installer behaviour. Read it before writing platform-specific code.

## 2. Verified platform facts
- NT4 has a distinct NT kernel/process/security model and is not Win9x.
- Microsoft says MSAA 1.3 is built in beginning with SP6; earlier NT4 needs an RDK.
- SetWinEventHook has a separate RDK route on NT4 SP6+.
- NT4 has no DWM; use GDI/USER/Shell mechanisms.

Primary references:
- [Active Accessibility supported platforms](https://learn.microsoft.com/en-us/windows/win32/winauto/supported-platforms)
- [SetWinEventHook](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwineventhook)
- [AnimateWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-animatewindow)
- [DwmIsCompositionEnabled](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmiscompositionenabled)
- [Open Watcom Linker Guide](https://open-watcom.github.io/open-watcom-v2-wikidocs/lguide.html)

A source link supports only the claim it documents. It does not prove a WinLook adapter works on this OS.

## 3. Toolchain and binary strategy
Use a dedicated NT-family C adapter for this legacy target. Verify the exact compiler/linker/runtime, service pack, imports, access rights and target architecture. Do not reuse Win9x assumptions or modern Rust runtime requirements without proof.

- Choose compiler/linker/runtime after checking the target OS loader, imported DLLs, calling conventions, CRT assumptions and CPU baseline.
- Keep modern Rust and legacy C builds separate.
- Do not require modern Unicode-only exports or a modern CRT on an old OS without a tested compatibility route.
- Use capability checks/late binding for optional APIs so missing imports cannot prevent startup.
- Put OS-specific calls behind the corresponding adapter.

## 4. WLWM compatibility
Use a dedicated NT4 adapter; validate SP6/service pack, access controls, process APIs and hooks. Do not reuse Win9x assumptions. Injection requires separate tests and strict recovery.

WLWM's local binary compatibility cache is only an analysis cache. Every launch still validates current target identity, architecture, relevant OS/build and mitigation state. Unknown or partially-supported targets must retain native Windows behaviour unless a reviewed experimental feature is explicitly enabled.

## 5. Animation suite
wl_Lanimate GDI/USER effects. Do not call AnimateWindow as a required system API; custom animation of other apps requires independent evidence.

The modern library is wl_animate; the legacy C library is wl_Lanimate. They share conceptual capabilities where the OS permits, not necessarily the same implementation. Do not call a Win32 API before its documented minimum/runtime requirement has been verified.

## 6. WLDE desktop and wallpaper
GDI wallpaper/widget surface. Desktop item handling and shell integration require NT4-specific adapter.

WLDE Groups are virtual presentation proxies, never filesystem folders. Keep original desktop items and their targets intact. Group support remains disabled until its OS adapter can restore the original icons after renderer crash, disable and uninstall.

## 7. WLDM and WLOSDE
Basic display driver APIs only; multi-monitor must be proven.

WLOSDE must use WinLook-owned ordinary UI surfaces. It must not imitate or replace native authentication, UAC or secure-desktop UI. WLDM exposes only display functions verified for this OS/driver combination.

## 8. WLT, assets and restoration
WLT import must validate the archive, reject unsafe paths/collisions, cap file counts/sizes/compression ratios, avoid modern runtime requirements, apply only supported components and report skipped items. Fonts/assets require licensing and user consent. The uninstaller must restore WinLook-owned changes and preserve original user files/shortcuts.

## Audio & Media and File Explorer notes

- NT 4.0 predates Vista Core Audio and GSMTC. WinLook still needs its own mixer/media interface, but only expose controls provided by verified NT 4.0 device/driver APIs; no generic per-process session mixer is assumed.
- Media metadata and commands require explicit supported-player adapters.
- Explorer customization is limited to classic Shell capabilities established for the exact NT 4.0 service pack. Do not assume XP-era `IShellView` or Vista+ `IExplorerCommand`.
- Deep visual adapters require a separate NT 4.0-specific investigation and must never modify/replace `explorer.exe`.

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
