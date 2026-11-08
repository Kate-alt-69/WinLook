# Windows Millennium Edition Compatibility Notes

> Status: research baseline, not a release-support claim.  
> OS family: Win9x. Version scope: 4.90 family.  
> Last reviewed: 2026-11-08.

## 1. Purpose
This page describes what WinLook must consider on Windows Millennium Edition before selecting APIs, compiler/linker flags, runtime dependencies, process techniques, rendering paths or installer behaviour. Read it before writing platform-specific code.

## 2. Verified platform facts
- Microsoft lists built-in Active Accessibility version 1.3 for Windows Me.
- SetWinEventHook still has documented Windows 2000 minimum; do not infer availability from MSAA version alone.
- AnimateWindow minimum is Windows 2000.
- Open Watcom's Win95 target is a toolchain candidate for 9x, not proof of a Windows Me binary/runtime combination.

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
Keep WinMe in the Win9x C adapter. Probe exact API exports and installed accessibility runtime. Do not treat it as NT or assume modern process APIs.

WLWM's local binary compatibility cache is only an analysis cache. Every launch still validates current target identity, architecture, relevant OS/build and mitigation state. Unknown or partially-supported targets must retain native Windows behaviour unless a reviewed experimental feature is explicitly enabled.

## 5. Animation suite
wl_Lanimate / GDI; test timer and driver differences from 95/98. No DWM/DirectComposition.

The modern library is wl_animate; the legacy C library is wl_Lanimate. They share conceptual capabilities where the OS permits, not necessarily the same implementation. Do not call a Win32 API before its documented minimum/runtime requirement has been verified.

## 6. WLDE desktop and wallpaper
GDI-backed desktop presentation; video/interactive wallpaper needs independent codec/runtime testing. Groups disabled pending recovery proof.

WLDE Groups are virtual presentation proxies, never filesystem folders. Keep original desktop items and their targets intact. Group support remains disabled until its OS adapter can restore the original icons after renderer crash, disable and uninstall.

## 7. WLDM and WLOSDE
Legacy display APIs only after test. No modern DPI/HDR assumptions.

WLOSDE must use WinLook-owned ordinary UI surfaces. It must not imitate or replace native authentication, UAC or secure-desktop UI. WLDM exposes only display functions verified for this OS/driver combination.

## 8. WLT, assets and restoration
WLT import must validate the archive, reject unsafe paths/collisions, cap file counts/sizes/compression ratios, avoid modern runtime requirements, apply only supported components and report skipped items. Fonts/assets require licensing and user consent. The uninstaller must restore WinLook-owned changes and preserve original user files/shortcuts.

## Audio & Media and File Explorer notes

- Windows Me predates Vista Core Audio and GSMTC. Provide WinLook's own interface but expose only verified global/device mixer functionality; no generic per-process volume promise.
- Player metadata/artwork/transport requires explicit adapters for named player versions.
- Use only classic Shell extension routes verified on Windows Me. XP `IShellView`, Vista+ `IExplorerCommand` and Windows 11 menu registration are not assumed.
- Deeper Explorer styling requires an isolated OS-specific prototype, safe fallback and no modification/replacement of `explorer.exe`.

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
