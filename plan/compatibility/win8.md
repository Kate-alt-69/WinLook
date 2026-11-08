# Windows 8 Compatibility Notes

> Status: research baseline, not a release-support claim.  
> OS family: Windows NT. Version scope: NT 6.2.  
> Last reviewed: 2026-11-08.

## 1. Purpose
This page describes what WinLook must consider on Windows 8 before selecting APIs, compiler/linker flags, runtime dependencies, process techniques, rendering paths or installer behaviour. Read it before writing platform-specific code.

## 2. Verified platform facts
- Microsoft explicitly documents DWM composition as always on in Windows 8 and not disableable by applications or users.
- Windows 8 has a changed separation/protection model for Store/immersive app experiences; those surfaces are not ordinary desktop windows.
- IDCompositionDesktopDevice is documented for Windows 8.1, not Windows 8.
- IVirtualDesktopManager is documented for Windows 10, not Windows 8.

Primary references:
- [Active Accessibility supported platforms](https://learn.microsoft.com/en-us/windows/win32/winauto/supported-platforms)
- [SetWinEventHook](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwineventhook)
- [AnimateWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-animatewindow)
- [DwmIsCompositionEnabled](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmiscompositionenabled)
- [Windows 8 DWM always on](https://learn.microsoft.com/en-us/windows/win32/w8cookbook/desktop-window-manager-is-always-on)
- [IDCompositionDesktopDevice](https://learn.microsoft.com/en-us/windows/win32/api/dcomp/nn-dcomp-idcompositiondesktopdevice)
- [IVirtualDesktopManager](https://learn.microsoft.com/en-us/windows/desktop/api/shobjidl_core/nn-shobjidl_core-ivirtualdesktopmanager)
- [Dynamic code policy](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-process_mitigation_dynamic_code_policy)

These links support only the claims they document. They do not prove WinLook code has been tested on this OS.

## 3. Toolchain and runtime strategy
Use a Rust build only when the selected compiler, standard library and dependencies officially support the exact Windows release. Otherwise use an approved legacy C implementation. Avoid importing APIs whose minimum supported version exceeds the target. Detect optional DLL exports and capabilities at runtime. Keep architecture/bitness-specific modules separate.

## 4. WLWM compatibility
WLWM manages desktop app windows and treats Store/immersive app surfaces as a distinct category. Use logical workspaces. Do not rely on undocumented desktop-manager COM interfaces.

WLWM's local binary compatibility cache is an analysis cache. Every launch still validates current target identity, architecture, OS/build and relevant mitigation state. Unknown or partially supported targets must retain native Windows behaviour unless an explicitly enabled, reviewed experimental route is used.

## 5. Animation suite
wl_animate may use DWM APIs. Do not assume IDCompositionDesktopDevice is available on Windows 8; use another verified path or fallback.

wl_animate is the modern Rust crate; wl_Lanimate is the legacy C library. Their conceptual APIs should align where possible, but backends and implementation details are version-specific. Do not assume API availability from headers alone.

## 6. WLDE desktop and wallpaper
DWM always composes. Desktop Shell integration still needs OS-specific testing; Store/immersive windows are not ordinary desktop windows. Preserve the separation model.

Desktop Groups are virtual presentation proxies, never filesystem folders. Original Shell items remain at their real locations and keep their targets. Group support stays disabled until original icon visibility is restored after renderer crash, disable and uninstall.

## 7. WLDM and WLOSDE
Use Windows 8 display APIs and test multi-monitor Start/desktop behaviour. Do not assume Windows 8.1 DirectComposition desktop interface.

WLOSDE uses WinLook-owned ordinary UI surfaces. It must not imitate or replace native authentication, UAC or secure-desktop UI. WLDM exposes only display functions verified for this OS/driver combination.

## 8. WLT and restoration
WLT import must validate the archive, reject unsafe paths/collisions, cap file counts/sizes/compression ratios, apply only supported components and report skipped items. Fonts/assets require licensing and user consent. The uninstaller must restore WinLook-owned changes and preserve original user files/shortcuts.

## Audio & Media and File Explorer notes

- Use Core Audio and `IAudioSessionManager2` for available session discovery/control.
- GSMTC is unavailable. Use supported-player adapters for Now Playing metadata and commands.
- Documented Shell handlers and namespace-view extensions are the baseline. Deep navigation/address/search/tab/command-bar styling requires a separate Windows 8-specific adapter.
- Keep Explorer responsible for native Shell and file operations; never patch or replace `explorer.exe`.

## 9. Required verification
- [ ] Exact OS build/service pack/edition recorded.
- [ ] Compiler/linker/runtime and imported DLLs recorded.
- [ ] API/redistributable probes recorded.
- [ ] WLWM window/event/injection/fallback tests run.
- [ ] wl_animate or wl_Lanimate backend profiled.
- [ ] WLDE desktop/wallpaper/Group recovery tested.
- [ ] WLDM/WLOSDE capability and fallback tested.
- [ ] WLT import and clean uninstall tested.
- [ ] Real hardware tested where practical; VM-only evidence is labelled.

## 10. Open questions
- Which exact edition/build is the baseline?
- Which compiler/runtime combination loads reliably?
- Which APIs are present without optional redistributables?
- Which injection methods are safe, observable and unloadable?
- Which graphics/display paths work with actual drivers?
- Can desktop icon presentation be safely changed and restored?
- Which WLT, wallpaper and uninstall features can be supported without unsupported system modification?

Update this page with test evidence as compatibility is established; do not mark this OS supported by inference.
