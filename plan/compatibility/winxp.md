# Windows XP Compatibility Notes

> Status: research baseline, not a release-support claim.  
> OS family: NT. Version scope: NT 5.1; XP x64/NT 5.2 separately.  
> Last reviewed: 2026-11-08.

## 1. Purpose
This page describes what WinLook must consider on Windows XP before selecting APIs, compiler/linker flags, runtime dependencies, process techniques, rendering paths or installer behaviour. Read it before writing platform-specific code.

## 2. Verified platform facts
- Microsoft lists Active Accessibility 2.0 built into XP.
- SetWinEventHook and AnimateWindow are within their documented Windows 2000+ minimum.
- Windows XP predates DWM and DirectComposition; use the GDI/USER path.
- XP x64 and 32-bit XP are separate compatibility targets for binary architecture and driver testing.

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
Use NT process/window APIs; implement logical workspaces instead of modern virtual desktop APIs. Cache includes process architecture and XP variant. Test injection with service packs and DEP state.

WLWM's local binary compatibility cache is only an analysis cache. Every launch still validates current target identity, architecture, relevant OS/build and mitigation state. Unknown or partially-supported targets must retain native Windows behaviour unless a reviewed experimental feature is explicitly enabled.

## 5. Animation suite
wl_Lanimate C baseline. GDI/USER animation; optional graphics APIs require per-driver checks. No DWM/DirectComposition.

The modern library is wl_animate; the legacy C library is wl_Lanimate. They share conceptual capabilities where the OS permits, not necessarily the same implementation. Do not call a Win32 API before its documented minimum/runtime requirement has been verified.

## 6. WLDE desktop and wallpaper
Wallpaper rendering can use GDI and optional legacy graphics backends selected after driver research. Keep Groups disabled until XP Shell view adapter is proven.

WLDE Groups are virtual presentation proxies, never filesystem folders. Keep original desktop items and their targets intact. Group support remains disabled until its OS adapter can restore the original icons after renderer crash, disable and uninstall.

## 7. WLDM and WLOSDE
XP-era display APIs; no modern HDR/DisplayConfig assumptions.

WLOSDE must use WinLook-owned ordinary UI surfaces. It must not imitate or replace native authentication, UAC or secure-desktop UI. WLDM exposes only display functions verified for this OS/driver combination.

## 8. WLT, assets and restoration
WLT import must validate the archive, reject unsafe paths/collisions, cap file counts/sizes/compression ratios, avoid modern runtime requirements, apply only supported components and report skipped items. Fonts/assets require licensing and user consent. The uninstaller must restore WinLook-owned changes and preserve original user files/shortcuts.

## Audio & Media and File Explorer notes

- XP has no Vista Core Audio session model and no GSMTC. WinLook still provides its own UI, but only verified global/device mixer controls are promised; no generic per-process volume.
- Media metadata, thumbnails and transport require adapters for explicitly supported player/version combinations.
- `IShellView` is documented from Windows XP for a custom namespace folder view; it does not reskin all standard Explorer folders/panes. `IExplorerCommand` is not the XP command route.
- Deep chrome styling requires a separate XP-specific research/prototype path. Never modify or replace `explorer.exe`.

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
