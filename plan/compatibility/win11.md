# Windows 11 Compatibility Notes

> Status: research baseline, not a release-support claim.  
> OS family: Windows NT. Version scope: NT 10.0 build family; record exact feature build.  
> Last reviewed: 2026-11-08.

## 1. Purpose
This page describes what WinLook must consider on Windows 11 before selecting APIs, compiler/linker flags, runtime dependencies, process techniques, rendering paths or installer behaviour. Read it before writing platform-specific code.

## 2. Verified platform facts
- Windows 11 continues the DWM-composed desktop model; DWM remains the Windows compositor and is not a replacement target.
- IVirtualDesktopManager's Windows 10 desktop-app contract remains relevant for window desktop membership/movement, but does not provide every operation WLWM may want.
- Process mitigation policies such as ProhibitDynamicCode can constrain dynamic code and some in-process modification techniques.
- Modern Shell surfaces and per-build changes require adapter/version testing; undocumented internal layouts are not stable contracts.

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
The current official Rust/MSVC target documents Windows 10 or later as the minimum client OS, so it is the planned Rust route for this target. This target-level fact is not a WinLook runtime test: verify the exact Windows build, architecture, API imports, dependencies, graphics drivers and process mitigations. The C implementation may also be built where a shared legacy path is useful.

## 4. WLWM compatibility
Full modern WLWM target candidate. BGW compatibility classification includes exact OS build, target binary hash/version/architecture and module versions. Process injection is allowed only for eligible targets; games, security software, protected processes, DWM and authentication processes need appropriate exclusions.

WLWM's local binary compatibility cache is an analysis cache. Every launch still validates current target identity, architecture, OS/build and relevant mitigation state. Unknown or partially supported targets must retain native Windows behaviour unless an explicitly enabled, reviewed experimental route is used.

## 5. Animation suite
wl_animate Rust backends use DWM/D3D11/DirectComposition according to measured capability. Window effects must not require modifying DWM or Explorer binaries. Reduced-motion and per-app animation settings are required.

wl_animate is the modern Rust crate; wl_Lanimate is the legacy C library. Their conceptual APIs should align where possible, but backends and implementation details are version-specific. Do not assume API availability from headers alone.

## 6. WLDE desktop and wallpaper
Modern WLDE targets include occlusion-aware rendering, adaptive resource policy, Groups and integrated Store. Measure video decode separately from composition. Ensure desktop Group recovery on crash/uninstall.

Desktop Groups are virtual presentation proxies, never filesystem folders. Original Shell items remain at their real locations and keep their targets. Group support stays disabled until original icon visibility is restored after renderer crash, disable and uninstall.

## 7. WLDM and WLOSDE
WLDM investigates current DisplayConfig, DPI, HDR and monitor APIs. Device/driver support is required, not just Windows 11 version.

WLOSDE uses WinLook-owned ordinary UI surfaces. It must not imitate or replace native authentication, UAC or secure-desktop UI. WLDM exposes only display functions verified for this OS/driver combination.

## 8. WLT and restoration
WLT import must validate the archive, reject unsafe paths/collisions, cap file counts/sizes/compression ratios, apply only supported components and report skipped items. Fonts/assets require licensing and user consent. The uninstaller must restore WinLook-owned changes and preserve original user files/shortcuts.

## Audio & Media and File Explorer notes

- GSMTC exposes Windows media sessions; show playback type, metadata, provider thumbnails, timeline and controls only when the session supplies them.
- Keep Core Audio session-volume controls distinct from GSMTC transport controls. Browser media-session granularity is provider-controlled; do not assume one session per tab.
- Modern File Explorer context-menu commands use `IExplorerCommand` with package-identity/manifest registration requirements.
- Deep styling of navigation, command bar, tabs, address/search and other Explorer chrome is an opt-in per-build adapter, not a universal public Shell API. Retest after feature and cumulative updates.
- Explorer remains the actual Shell/file-operation host. Never modify or replace `explorer.exe` on disk.

## Required WLDE Explorer Desktop Bridge

The Explorer Desktop Bridge is mandatory BGW/WLDE runtime infrastructure on
this OS when Explorer owns the interactive desktop. It is not a Windows 10/11
only feature and has no per-feature user opt-in. BGW must identify the actual
desktop-owning Shell process and select this OS/build's adapter; do not assume
that every Explorer.exe instance hosts the desktop.

The adapter must use this OS's documented Shell interfaces where they satisfy
the feature contract and may use a controlled, version-specific runtime hook
inside the desktop-owning Explorer process where those interfaces do not
expose required desktop icon/Group behaviour. This does not permit modifying
Explorer.exe on disk, replacing DWM, or attaching indiscriminately to other
Explorer folder-window processes. Shell activation and filesystem operations
remain Windows-owned; expensive work remains in BGW/WLDE.

On an unknown or unsupported build, keep native desktop icons visible and
report the affected WLDE capability as unavailable. Before suppressing any
native icon representation, persist enough restore state to recover after
Explorer/BGW crash, Explorer restart, logoff, update and uninstall. This is
a mandatory integration contract; exact support is established only by
per-OS runtime tests, not by API-name or source-compatibility assumptions.

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
