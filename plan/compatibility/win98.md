# Windows 98 / 98 SE Compatibility Notes

> Status: research baseline, not a release-support claim.  
> OS family: Win9x. Version scope: 4.10 family; distinguish first release and SE.  
> Last reviewed: 2026-11-08.

## 1. Purpose
This page describes what WinLook must consider on Windows 98 / 98 SE before selecting APIs, compiler/linker flags, runtime dependencies, process techniques, rendering paths or installer behaviour. Read it before writing platform-specific code.

## 2. Verified platform facts
- Microsoft says the initial Windows 98 release included Active Accessibility 1.1; the Active Accessibility 2.0 RDK can upgrade it.
- SetWinEventHook minimum remains Windows 2000; detect whether any relevant redistributable/API is present rather than inferring from MSAA version.
- AnimateWindow minimum is Windows 2000; do not assume it on 98.
- Open Watcom documents a Win9x 32-bit target; each runtime/library combination needs OS testing.

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
Use a Win9x C adapter and test first release and 98 SE separately. Use verified USER APIs and polling/legacy event mechanisms where required. Injection, loader hooks and recovery remain research tasks.

WLWM's local binary compatibility cache is only an analysis cache. Every launch still validates current target identity, architecture, relevant OS/build and mitigation state. Unknown or partially-supported targets must retain native Windows behaviour unless a reviewed experimental feature is explicitly enabled.

## 5. Animation suite
wl_Lanimate with GDI/timers; measure separately from Win95 due to driver/timer differences. Native AnimateWindow unavailable per documented minimum.

The modern library is wl_animate; the legacy C library is wl_Lanimate. They share conceptual capabilities where the OS permits, not necessarily the same implementation. Do not call a Win32 API before its documented minimum/runtime requirement has been verified.

## 6. WLDE desktop and wallpaper
Static and light animation candidates. Keep Groups disabled until native icon representation can be suppressed/restored safely. No DWM.

WLDE Groups are virtual presentation proxies, never filesystem folders. Keep original desktop items and their targets intact. Group support remains disabled until its OS adapter can restore the original icons after renderer crash, disable and uninstall.

## 7. WLDM and WLOSDE
Primary display baseline; multi-monitor depends on driver/extension. OSD via own windows.

WLOSDE must use WinLook-owned ordinary UI surfaces. It must not imitate or replace native authentication, UAC or secure-desktop UI. WLDM exposes only display functions verified for this OS/driver combination.

## 8. WLT, assets and restoration
WLT import must validate the archive, reject unsafe paths/collisions, cap file counts/sizes/compression ratios, avoid modern runtime requirements, apply only supported components and report skipped items. Fonts/assets require licensing and user consent. The uninstaller must restore WinLook-owned changes and preserve original user files/shortcuts.

## Audio & Media and File Explorer notes

- Windows 98 predates Vista Core Audio and GSMTC. Keep the WinLook-owned UI, but provide only global/device mixer controls proven for the exact edition and driver. Do not promise generic per-process volume.
- Media metadata, artwork and playback controls require named player adapters; there is no system-wide Now Playing API equivalent to GSMTC.
- Explorer customization must use classic Shell capabilities verified on this OS. Do not assume XP-era `IShellView`, Windows 7 `IExplorerCommand` or Windows 11 modern context-menu registration.
- Deep Explorer presentation work must remain a separately tested compatibility adapter and must never patch/replace `explorer.exe`.

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
