# Windows Vista Compatibility Notes

> Status: research baseline, not a release-support claim.  
> OS family: Windows NT. Version scope: NT 6.0; edition/driver/session.  
> Last reviewed: 2026-11-08.

## 1. Purpose
This page describes what WinLook must consider on Windows Vista before selecting APIs, compiler/linker flags, runtime dependencies, process techniques, rendering paths or installer behaviour. Read it before writing platform-specific code.

## 2. Verified platform facts
- DWM APIs such as DwmIsCompositionEnabled are documented from Vista.
- Vista changed the service/session model: Session 0 is reserved for services and non-interactive processes; interactive users log into another session.
- Vista composition availability is conditional; do not assume it is enabled with every theme, edition, driver or remote session.
- IVirtualDesktopManager is a Windows 10 interface and is not available here.

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
Use the C implementation as the planned default for Vista. The current official Rust/MSVC target requires Windows 10 or later for client installs; an older pinned Rust toolchain is research-only until its compiler, standard library, linker/runtime and Vista VM behaviour are verified. Avoid imports newer than the target and detect optional APIs at runtime.

## 4. WLWM compatibility
The current Rust/MSVC target is not the supported Vista route. Use the C build unless a separately reviewed historical Rust toolchain proves Vista runtime compatibility. Process APIs and hook backends need their own minimum checks. WLWM should retain logical workspaces. Use DWM only when composition state says available; maintain non-DWM fallback.

WLWM's local binary compatibility cache is an analysis cache. Every launch still validates current target identity, architecture, OS/build and relevant mitigation state. Unknown or partially supported targets must retain native Windows behaviour unless an explicitly enabled, reviewed experimental route is used.

## 5. Animation suite
wl_animate may use Vista DWM capabilities after runtime detection; D3D backends depend on GPU/driver. Keep layered-window/GDI fallback. IDCompositionDesktopDevice is not assumed; its documented minimum is Windows 8.1.

wl_animate is the modern Rust crate; wl_Lanimate is the legacy C library. Their conceptual APIs should align where possible, but backends and implementation details are version-specific. Do not assume API availability from headers alone.

## 6. WLDE desktop and wallpaper
Use a desktop/overlay integration that works both with DWM composition enabled and fallback conditions. Occlusion data and layered-window behaviour need tests. WLDE must not disable DWM.

Desktop Groups are virtual presentation proxies, never filesystem folders. Original Shell items remain at their real locations and keep their targets. Group support stays disabled until original icon visibility is restored after renderer crash, disable and uninstall.

## 7. WLDM and WLOSDE
WLDM probes display capabilities individually; Aero/DWM state and display-driver capabilities vary.

WLOSDE uses WinLook-owned ordinary UI surfaces. It must not imitate or replace native authentication, UAC or secure-desktop UI. WLDM exposes only display functions verified for this OS/driver combination.

## 8. WLT and restoration
WLT import must validate the archive, reject unsafe paths/collisions, cap file counts/sizes/compression ratios, apply only supported components and report skipped items. Fonts/assets require licensing and user consent. The uninstaller must restore WinLook-owned changes and preserve original user files/shortcuts.

## Audio & Media and File Explorer notes

- Vista has Core Audio base session/endpoint interfaces, including `IAudioSessionManager` and `ISimpleAudioVolume`; controls are possible where shared-mode session identity is available.
- Do not use `IAudioSessionManager2` as a Vista baseline: Microsoft documents that interface from Windows 7. Full live per-app enumeration needs separate validation.
- GSMTC is unavailable. Now Playing metadata, artwork and transport use explicit supported-player adapters.
- `IShellView` supports custom namespace views from XP onward. `IExplorerCommand` has a documented Vista minimum; verify the appropriate Vista Shell registration route. Deep Explorer chrome styling is a separate build-specific experimental adapter.
- Never modify or replace `explorer.exe`; preserve native Shell and file-operation ownership.

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
