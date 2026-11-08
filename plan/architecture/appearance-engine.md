# Appearance Engine and Universal Icon Engine

> Status: architecture design. This document describes intended WinLook behaviour; it does not claim that every customization is available through a supported API on every Windows release.

## Purpose and ownership

The Appearance Engine manages Windows' user-facing visual identity. It applies complete or partial WinLook Templates, adapts each customization to the installed Windows generation, tracks what it changes, and restores prior values when a theme is removed or WinLook is uninstalled.

It is not a replacement Windows shell and does not own window composition. WLWM manages window behaviour, WLDE manages desktop presentation, WLDM manages displays, and WLOSDE manages WinLook-owned notification surfaces.

## Design principles

1. **Adapter per platform generation.** Windows 95/98/ME, older NT releases, Vista/7, 8/8.1 and 10/11 expose different Shell, USER/GDI, theme, composition and Settings capabilities. A modern API's existence in a current SDK does not make it available on an old system.
2. **Capability-based feature selection.** A component must declare its required OS/API/runtime capabilities. WinLook reports unsupported or partially supported components instead of silently substituting unrelated behaviour.
3. **User-owned state remains user-owned.** Save original values and apply changes transactionally. Detect external changes before restoration; do not overwrite a later user change blindly.
4. **Explorer remains the native Shell.** WinLook must never patch, replace, rewrite or depend on a modified \`explorer.exe\`. Supported Shell extensions are the baseline; deep Explorer presentation customization belongs to a separate opt-in, build-specific adapter with its own compatibility and recovery policy. See [File Explorer Customization](explorer-customization.md).
5. **No DWM replacement.** WLWM and WLDE coexist with the existing Windows composition stack. DWM remains intact wherever present.
6. **Independent failure and rollback.** A failed taskbar adapter must not prevent the user from restoring icons or opening WinLook Manager.

## Customization domains

### Desktop and window appearance

- Wallpaper and desktop background behaviour belongs to WLDE.
- Window border, shadow, transparency, blur and animation requests must be mapped to the capabilities available on the current OS and graphics driver.
- Vista and later may expose DWM-backed effects; availability and individual attributes differ by release. See [DWM overview](https://learn.microsoft.com/en-us/windows/win32/dwm/dwm-overview) and [DWM window attributes](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwmwindowattribute).
- Pre-Vista systems need independent USER/GDI-era implementations. Do not import DWM APIs into legacy binaries.

### Taskbar and Start Menu

- Research and use documented personalization or Shell interfaces where they meet the requirement.
- Taskbar background, sizing, position, icon layout, pinned shortcut icons and running-window icons are separate capabilities; do not assume one setting changes all of them.
- A pinned shortcut's icon can be different from the icon supplied by an already-running application's window. Running taskbar icons may need a target-specific integration or WLWM adapter.
- If native customization is insufficient, an independent WinLook-owned surface may be evaluated. It must not require replacing Explorer or DWM and must have a reliable native fallback.
- Start Menu implementations differ substantially between classic directory-backed menus and later Shell/packaged-app experiences. Define adapters rather than one universal implementation.
- Source entry points: [Windows Shell interfaces](https://learn.microsoft.com/en-us/windows/win32/shell/interfaces) and [Shell links/shortcuts](https://learn.microsoft.com/en-us/windows/win32/shell/links).

### File Explorer customization

The Appearance Engine owns the visible Explorer customization layer: Shell handlers and item presentation, plus a separately isolated optional per-build presentation adapter for Explorer chrome that documented APIs do not expose. The goal is to customize the existing Explorer interface deeply without replacing its Shell or file-operation responsibilities. See [File Explorer Customization](explorer-customization.md).

### Universal Icon Engine

The Icon Engine supplies a theme's icon mappings and asks the appropriate OS adapter to apply them. It covers:

- Windows system and Shell items;
- file types and registered associations;
- folders and drives where supported;
- application shortcuts and pinned Start/taskbar entries;
- icons exposed by running applications, only where a safe target-specific integration exists;
- WLDE's virtual desktop Group icons.

Keep original icon sources and mapping metadata. Do not rewrite application executable resources as the normal method. An icon may have different presentation routes in File Explorer, the Start Menu, pinned taskbar shortcuts and running taskbar buttons; report those differences.

The desktop Group icon is a WLDE-owned visual proxy. It does not turn its members into a filesystem folder or move/delete the original Shell items. Group presentation has a separate recovery requirement in [WLDE](wlde.md).

### Fonts, cursors, sounds and other resources

- Templates may bundle assets in their designated directories, including the proposed \`wl/&FONT_FILE&/\` directory.
- Before installing a font system-wide, verify its licence, ask for permission where needed, record the prior state and provide a restore path.
- Cursor and sound schemes must be applied through OS-specific mechanisms. Keep original resource identifiers and user customizations.
- Do not assume a resource format or registry route is universal from Windows 95 through 11.

## Theme application transaction

A theme or partial customization should follow this sequence:

1. Parse and validate its manifest, asset paths, compatibility requirements and permissions.
2. Resolve the requested components against the current capability report.
3. Show components that will apply, be skipped, or require an experimental adapter.
4. Snapshot the exact current value and ownership/scope of every setting WinLook will modify.
5. Apply changes in small, ordered stages.
6. Verify the observable result after each stage.
7. Record the applied value and transaction state in WinLook's change journal.
8. On failure, roll back completed stages and report any incomplete restoration.

See [Restoration and clean uninstallation](restoration.md).

## Injection and running applications

WLWM's main background worker (BGW) owns process detection, target classification, compatibility-cache lookup, injection orchestration and module health. The Appearance Engine may request an application-specific icon or appearance integration, but it must go through WLWM's approved target policy.

Injection is an optional implementation technique, not blanket permission to modify every process. Exclude DWM, authentication/security-critical processes and protected targets. Explorer presentation customization is not routed through ordinary WLWM injection: any in-process Explorer adapter is governed by the separate File Explorer policy, opt-in and per-build verification. Cache hits do not replace per-launch identity checks or runtime safety validation.

## Template integration

WLT may include appearance settings, icon mappings and their supporting assets. A Template must declare target OS capabilities and optional components. Import must not install fonts, replace associations, or perform machine-wide changes without explicit user consent.

Local desktop Groups are not exported in WLT. Fancy/portable Groups are a separate concept and require their own approved manifest and item-resolution rules.

## Required research and verification

- [ ] Build a per-version matrix for each appearance domain.
- [ ] Identify the precise API/registry/Shell contract before implementing each adapter.
- [ ] Verify minimum supported Windows version and runtime requirements for every API.
- [ ] Test 32-bit and 64-bit target/application combinations where relevant.
- [ ] Test icon cache refresh and shell restart behaviour without modifying Explorer on disk.
- [ ] Test partial apply, external changes, crash recovery, uninstall and rollback.
- [ ] Verify fonts, cursors and sound assets' redistribution licences.
- [ ] Record evidence and tested OS builds in [the source ledger](../references/SOURCES.md) and the relevant [compatibility page](../compatibility/README.md).

## References

- [Microsoft: Windows Shell interfaces](https://learn.microsoft.com/en-us/windows/win32/shell/interfaces)
- [Microsoft: Shell links and shortcuts](https://learn.microsoft.com/en-us/windows/win32/shell/links)
- [Microsoft: Desktop Window Manager overview](https://learn.microsoft.com/en-us/windows/win32/dwm/dwm-overview)
- [Microsoft: DWM window attributes](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwmwindowattribute)
- [Microsoft: DWM is always on in Windows 8](https://learn.microsoft.com/en-us/windows/compatibility/desktop-window-manager-is-always-on)
