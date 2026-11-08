# WinLook Technical Source Ledger

Last verified: 2026-11-08. These citations support platform or upstream-project claims; they do not prove that WinLook code has been implemented or tested.

## Microsoft Windows / Win32

| ID | Source | Claim supported | Scope / caveat |
|---|---|---|---|
| MSAA-001 | [Active Accessibility supported platforms](https://learn.microsoft.com/en-us/windows/win32/winauto/supported-platforms) | MSAA versions included with Win95/98/Me/NT4/2000/XP/Vista/7; Win95 requires 1.3 RDK | Does not prove every WinEvent API exists in each base OS |
| WIN-001 | [SetWinEventHook](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwineventhook) | Minimum client Windows 2000; Active Accessibility 1.3 RDK for Windows 95 and NT4 SP6+ | RDK route requires separate installation/testing |
| WIN-002 | [AnimateWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-animatewindow) | Show/hide effects; minimum client Windows 2000 | Not available as a Win95 dependency |
| DWM-001 | [DwmIsCompositionEnabled](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmiscompositionenabled) | DWM API minimum Vista; query composition state | Windows 8+ always on; manifest compatibility affects return behaviour |
| DWM-002 | [Windows 8 DWM always on](https://learn.microsoft.com/en-us/windows/compatibility/desktop-window-manager-is-always-on) | DWM cannot be disabled by apps/users in Windows 8; composition is core | Windows 8 platform-specific documentation |
| DCOMP-001 | [IDCompositionDesktopDevice](https://learn.microsoft.com/en-us/windows/win32/api/dcomp/nn-dcomp-idcompositiondesktopdevice) | DirectComposition Win32 desktop interface; minimum Windows 8.1 | Does not imply this interface exists on Windows 8 |
| VDESK-001 | [IVirtualDesktopManager](https://learn.microsoft.com/en-us/windows/desktop/api/shobjidl_core/nn-shobjidl_core-ivirtualdesktopmanager) | Windows 10 interface for window virtual desktop interaction | Not a complete create/switch API |
| VDESK-002 | [MoveWindowToDesktop](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ivirtualdesktopmanager-movewindowtodesktop) | Moving top-level HWND to a desktop; minimum Windows 10 | Method-specific contract |
| MIT-001 | [PROCESS_MITIGATION_DYNAMIC_CODE_POLICY](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-process_mitigation_dynamic_code_policy) | ProhibitDynamicCode prevents dynamic code generation or executable-code modification; minimum Windows 8.1 | One mitigation type; query actual target |
| SVC-001 | [Service changes for Windows Vista](https://learn.microsoft.com/en-us/windows/win32/services/service-changes-for-windows-vista) | Session 0 reserved for services/non-interactive processes from Vista | Critical to BGW/Service/UI placement |
| WTS-001 | [WTSRegisterSessionNotification](https://learn.microsoft.com/en-us/windows/win32/api/wtsapi32/nf-wtsapi32-wtsregistersessionnotification) | Registers a window for session-change notifications delivered as WM_WTSSESSION_CHANGE | Minimum client Vista; unregister before destroying the registered window |
| WTS-002 | [WM_WTSSESSION_CHANGE](https://learn.microsoft.com/en-us/windows/win32/termserv/wm-wtssession_change) | Message delivered to windows registered for session-change events | Not a universal Win95-to-Win11 event source |
| SESSION-001 | [Desktops](https://learn.microsoft.com/en-us/windows/desktop/winstation/desktops) | Default, ScreenSaver and Winlogon desktop roles; Winlogon desktop access is restricted | Security boundary reference; does not provide WinLook access |
| SHELL-001 | [Windows Shell interfaces](https://learn.microsoft.com/en-us/windows/win32/shell/interfaces) | Overview of Shell COM interfaces for file/desktop integration | Each interface has its own minimum OS and behaviour |
| SHELL-002 | [Shell links and shortcuts](https://learn.microsoft.com/en-us/windows/win32/shell/links) | Shell link model and shortcut integration | Does not imply a universal running-taskbar icon replacement API |
| DWM-003 | [DWM overview](https://learn.microsoft.com/en-us/windows/win32/dwm/dwm-overview) | DWM compositor architecture | Vista+; retain fallback on older systems |
| DWM-004 | [DWM window attributes](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwmwindowattribute) | Supported DWM window attributes | Attribute support differs by Windows release |

## Legacy toolchain

| ID | Source | Claim supported | Scope / caveat |
|---|---|---|---|
| OW-001 | [Open Watcom Linker Guide](https://open-watcom.github.io/open-watcom-v2-wikidocs/lguide.html) | Distinct Win95 x86 and NT/Win32 targets | Toolchain target availability is not runtime proof |

## Third-party project references

| ID | Source | Claim supported | Scope / caveat |
|---|---|---|---|
| WINDHAWK-001 | [Windhawk injection targets](https://github.com/ramensoftware/windhawk/wiki/Injection-targets-and-critical-system-processes) | Injection targeting, early-load rationale, exclusion lists and compatibility tradeoffs | Design reference only; do not copy its defaults without review |
| WINDHAWK-002 | [Windhawk repository](https://github.com/ramensoftware/windhawk) | Upstream project source | Pin specific tag/commit before relying on implementation internals |
| HYPR-001 | [Hyprland](https://hypr.land/) | Window manager/compositor project identity | Inspiration only, not Windows compatibility evidence |
| HYPR-002 | [Hyprland window rules](https://wiki.hypr.land/Configuring/Basics/Window-Rules/) | Window rule concepts | Do not copy syntax as WinLook contract |
| HYPR-003 | [Hyprland workspace rules](https://wiki.hypr.land/Configuring/Workspace-Rules/) | Workspace rule concepts | Design reference only |
| RAIN-001 | [Rainmeter documentation](https://docs.rainmeter.net/) | Rainmeter configuration/runtime concepts | Parsing .rmskin is not full runtime compatibility |
| RAIN-002 | [Rainmeter source](https://github.com/rainmeter/rainmeter) | Upstream implementation | GPL-2.0 project; respect project and asset licensing; pin commit when citing internals |
| RAIN-003 | [Distributing Skins (.rmskin)](https://docs.rainmeter.net/manual/distributing-skins/) | .rmskin is a modified ZIP with package metadata/integrity data; renaming a ZIP is not equivalent | Rainmeter package specification; inspect version-specific metadata |
| RAIN-004 | [Installing Skins](https://docs.rainmeter.net/manual/installing-skins/) | Skin packages may contain skins, layouts and architecture-specific plugins | Does not imply WinLook implements the Rainmeter runtime |

## Ledger rules

For every new source record its exact claim, publisher, affected version, last-verified date and confidence. Separate documentation-verified facts from runtime tests. Pin a commit/tag when describing upstream internals. Do not promote an inferred behaviour into fact without evidence.

## Open research

Win9x toolchain/runtime combinations; safe desktop icon suppression/recovery; process creation/injection compatibility; legacy display APIs; desktop wallpaper placement without Explorer modification; modern DWM/DirectComposition differences; WLWM workspace APIs beyond the public IVirtualDesktopManager interface; OS-specific lock/session event mechanisms for pre-Vista systems; exact Rainmeter package metadata and runtime emulation scope.
