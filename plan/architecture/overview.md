# WinLook Architecture Overview

## Purpose
WinLook is an additive Windows customization platform. It extends the user-facing appearance, desktop experience, window management and application integration while leaving Windows responsible for its native responsibilities.

## Platform split
- Modern builds: Rust, using version-appropriate Win32/COM bindings and graphics APIs.
- Legacy builds: separate C implementation selected for the exact OS family/compiler/runtime.
- Share conceptual APIs and portable data contracts where possible; do not force one binary or library set across Win9x and NT systems.

## Subsystems

| Component | Responsibility |
|---|---|
| WinLook Manager | Settings, previews, import/export, apply and restore |
| Main BGW / Agent | Session/process monitoring, WLWM compatibility cache, injection orchestration and health |
| Appearance Engine | Taskbar, Start Menu, File Explorer presentation/Shell integration, window surfaces and supported system appearance |
| Audio & Media Experience | Global/per-app audio controls plus provider-backed Now Playing metadata, thumbnails, transport and timeline |
| Icon Engine | System/file/folder/drive/application/shortcut icons |
| WLDE | Desktop, wallpaper runtime/library/Store, widgets, Groups and adaptive rendering |
| WLWM | Window policy/layouts/rules/workspaces, compatibility analysis and injection |
| WLDM | Monitor enumeration, display profiles and configuration |
| WLOSDE | WinLook notifications, popups and OSD |
| wl_animate | Modern Rust animation crate/API |
| wl_Lanimate | Legacy C animation library |
| Accessibility | Reduced motion, contrast, scaling, keyboard support and other accommodations |
| Application Integration Bridge | Public API and community adapters |

## DWM and Windows Shell
DWM APIs are documented from Windows Vista. Vista/7 composition can vary by theme/driver/session; query it and provide fallback. Windows 8 and later keep DWM composition always on and apps cannot disable it. Windows 95 through XP have no DWM and use USER/GDI/Shell adapters.

DWM is not the application launcher. Windows Shell/application activation launches apps; WLWM manages windows after creation. Do not patch or replace Explorer.exe on disk. Supported Shell APIs are the baseline; deep Explorer presentation customization is a separate opt-in, per-build adapter with independent recovery, not ordinary WLWM injection. Windows retains filesystem and Shell operations. DWM and credential/security-critical processes are never injection targets.

Sources: [DwmIsCompositionEnabled](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmiscompositionenabled), [Windows 8 DWM always on](https://learn.microsoft.com/en-us/windows/win32/w8cookbook/desktop-window-manager-is-always-on).

## Process/session model
WinLook Manager and the interactive Agent/BGW run in the user session. Any optional Service is limited to operations requiring service privileges. Starting with Windows Vista, Session 0 is reserved for services/non-interactive processes; services must not host the interactive WLDE/Manager UI. Source: [Service changes for Windows Vista](https://learn.microsoft.com/en-us/windows/win32/services/service-changes-for-windows-vista).

## BGW injection flow
1. Detect process creation through a per-OS mechanism.
2. Identify target file/binary, architecture and version.
3. Read or compute the compatibility record.
4. Re-check identity, current OS, mitigation state, allow/deny policy and module requirements.
5. Select only compatible in-process integrations.
6. Apply injection/hook through a version-specific adapter.
7. Verify initialization and report health.
8. On failure, unload or quarantine when safe and preserve native Windows behaviour.

A cache hit is not permission to skip current safety checks. Do not run slow analysis or network operations while holding loader locks.

## Further reading
[Appearance Engine](appearance-engine.md) · [File Explorer Customization](explorer-customization.md) · [Audio & Media Experience](audio-experience.md) · [WLWM](wlwm.md) · [Animation suite](animation-suite.md) · [WLDE](wlde.md) · [WLT](wlt.md) · [WLOSDE/WLDM](wlosde-wldm.md) · [Standby/security](standby-security.md) · [Extensions](extensions.md) · [Restoration](restoration.md) · [Compatibility](../compatibility/README.md) · [Sources](../references/SOURCES.md)

## Repository, UI and documentation architecture

The proposed source layout is documented in [repository-layout.md](repository-layout.md). C and Rust use parallel feature directories, reusable WinLook libraries live under the repository's lib directory, and each language has a dedicated UI tree. Public APIs are separated from engine implementation.

The Manager UI direction is a clear, native-feeling navigation shell with focused pages and reusable controls; see [ui-architecture.md](ui-architecture.md). Public API and application documentation requirements are in [documentation-standard.md](documentation-standard.md). These are planning proposals and do not imply that the source tree has been moved or that a UI toolkit has been selected.

