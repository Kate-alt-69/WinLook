# Windows 10 Compatibility Notes

> Status: research baseline, not a release-support claim.  
> OS family: Windows NT. Version scope: NT 10.0; record exact release/build.  
> Last reviewed: 2026-11-08.

## 1. Purpose
This page describes what WinLook must consider on Windows 10 before selecting APIs, compiler/linker flags, runtime dependencies, process techniques, rendering paths or installer behaviour. Read it before writing platform-specific code.

## 2. Verified platform facts
- IVirtualDesktopManager is documented for Windows 10 desktop apps and supports querying a window's virtual desktop and moving a window to a specified desktop.
- The interface does not provide every operation WLWM may want, such as a documented general-purpose create/switch API.
- DWM composition is always on since Windows 8.
- PROCESS_MITIGATION_DYNAMIC_CODE_POLICY is available from Windows 8.1 and can prohibit dynamic code generation/modification in target processes.

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
Use IVirtualDesktopManager where its public contract fits. For create/switch/current-desktop selection, keep separate adapters and do not treat undocumented interfaces as stable. BGW cache includes OS build and target binary identity. Respect mitigations, architecture and exclusions.

WLWM's local binary compatibility cache is an analysis cache. Every launch still validates current target identity, architecture, OS/build and relevant mitigation state. Unknown or partially supported targets must retain native Windows behaviour unless an explicitly enabled, reviewed experimental route is used.

## 5. Animation suite
wl_animate Rust implementation can use DWM, D3D11 and DirectComposition backends selected by runtime/driver capability. Injection/hook integrations remain target-specific and may be blocked by mitigations/security software.

wl_animate is the modern Rust crate; wl_Lanimate is the legacy C library. Their conceptual APIs should align where possible, but backends and implementation details are version-specific. Do not assume API availability from headers alone.

## 6. WLDE desktop and wallpaper
Modern DWM desktop composition; implement exposed-region clipping only after profiling and verifying occlusion knowledge. Video decoding cost remains separate from final composition.

Desktop Groups are virtual presentation proxies, never filesystem folders. Original Shell items remain at their real locations and keep their targets. Group support stays disabled until original icon visibility is restored after renderer crash, disable and uninstall.

## 7. WLDM and WLOSDE
WLDM investigates DisplayConfig, per-monitor DPI, refresh and modern monitor APIs; guard by build and driver.

WLOSDE uses WinLook-owned ordinary UI surfaces. It must not imitate or replace native authentication, UAC or secure-desktop UI. WLDM exposes only display functions verified for this OS/driver combination.

## 8. WLT and restoration
WLT import must validate the archive, reject unsafe paths/collisions, cap file counts/sizes/compression ratios, apply only supported components and report skipped items. Fonts/assets require licensing and user consent. The uninstaller must restore WinLook-owned changes and preserve original user files/shortcuts.

## Audio & Media and File Explorer notes

- Core Audio provides endpoint and per-application session controls.
- GSMTC is available only on Windows 10 version 1809 (build 17763) and later. Earlier Windows 10 releases use player adapters; the Manager must report that the system media-session API is unavailable.
- Media playback type, title, provider thumbnail, timeline and transport commands are optional/provider-controlled.
- `IExplorerCommand` and other documented Shell handlers are the native integration layer. Deep styling of navigation/address/search/tabs/command bar requires an opt-in build-specific adapter.
- Adapter signature failure or Explorer crash must quarantine the adapter and preserve native Explorer.

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
