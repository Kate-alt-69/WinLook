# WinLook File Explorer Customization

> Status: accepted product direction; detailed implementation strategy and per-version surface coverage remain under research. Explorer remains the Windows Shell/file-operation host, while BGW/WLDE provides a required runtime integration for desktop presentation on sessions where Explorer owns the desktop.

## 1. Product requirement

WinLook should deeply customize the actual Explorer interface: file/folder/background right-click menus; navigation/tree pane; address/breadcrumb bar; search and results presentation; command bar, toolbar and ribbon-era surfaces; tabs; folder/file views and layout; Details columns, status/details/preview panes; icons, thumbnails, overlays and previews; window appearance; and presentation of file-operation progress/confirmation surfaces where safely customizable.

Full visible-interface customization is the product goal. Exact coverage is OS/build-dependent: documented Shell extension points cover selected functions, while other visual surfaces require build-specific adapters. Separately, the Explorer Desktop Bridge used by WLDE Groups and app icons is a required BGW subsystem, not an optional feature toggle. This document does not claim every surface has a stable public API.

## 2. Boundary: Explorer remains Explorer

Windows Explorer/Shell remains responsible for filesystem and namespace operations, enumeration, copy/move/rename/delete, file associations and activation, search/index integration, cloud-file providers, permissions/UAC/security decisions, drag/drop, clipboard and native Shell contracts.

WinLook customizes presentation, layout, visual themes and selected interactions. It does not implement a second general-purpose file manager, replace the Shell, take ownership of core file-operation logic, or replace Windows security surfaces.

**Never patch, rewrite, binary-modify or replace `explorer.exe` on disk.** WinLook must not depend on a modified Explorer executable. DWM, Winlogon, credential UI, secure desktops and protected/security-critical processes are outside the customization target.

## 3. Layered architecture

### Layer A — documented Shell integration

Use supported Shell handlers where they match the target OS/feature: context-menu commands; property pages/handlers; icons, thumbnails, preview handlers and infotips; Details columns/metadata; icon/thumbnail overlays; and drag/drop handlers only when ownership/failure behaviour is understood.

`IExplorerCommand` is documented with a Windows Vista minimum client. The API minimum does not mean registration or visual context-menu behaviour is identical across releases. Windows 11 modern context-menu integration additionally requires app identity and package-manifest registration (including a sparse-package route for unpackaged Win32 apps); older releases use their corresponding classic Shell registration path.

Shell extensions may execute in Explorer's process or UI path. Keep discovery/state/icon callbacks fast and deterministic. Never block Explorer on network access, package parsing or slow filesystem work. Hand long operations to an out-of-process broker after explicit invocation.

### Layer B — namespace extensions and custom folder views

A Shell namespace extension can provide its own virtual folder inside Explorer. Its folder object can provide a custom `IShellView` for that extension's own contents and may contribute folder-specific menus, toolbar and status behaviour.

This is useful for WinLook-managed virtual content and custom views of that namespace. It is **not** a universal reskin API for ordinary filesystem folders. Microsoft's guidance describes significant control over the extension's own folder view, but much more limited control over the general tree/navigation view. A custom view may lose behaviours supplied by Explorer's default view.

Do not claim that `IShellView` replaces the view, navigation pane, address bar, tabs or menus for all user folders.

### Layer C — Explorer presentation adapters

Some requested visual controls—especially the standard navigation pane,
address/search controls, tabs/command bar styling and overall Explorer layout—
are not fully configurable through documented extension points.

There are two distinct adapter responsibilities:

1. The **Explorer Desktop Bridge** is mandatory BGW/WLDE infrastructure on
   every supported Windows session where Explorer owns the desktop. It connects
   desktop item identity, icon presentation, Group proxies and right-click
   routing to the WLDE presenter. BGW selects a native adapter for the detected
   Windows generation and actual desktop-owning Explorer process.
2. Additional **Explorer chrome styling** is per-build and capability-based.
   Where documented APIs do not expose a requested visual surface, a narrow
   runtime UI/resource adapter or hook may be used after build/signature checks.
   These chrome changes are not a universal public API and may be unavailable
   on a particular build without disabling the required WLDE Desktop Bridge.

Requirements:
- target the actual desktop-owning Explorer process for desktop features; target
  other Explorer hosts only for separately specified File Explorer features;
- identify the OS build, architecture, process identity and control/framework
  signatures before using version-specific hooks;
- preserve Shell command/filesystem ownership and never patch files on disk;
- use documented APIs first and a runtime hook only for behaviour the public
  interfaces do not expose;
- have BGW coordinate health, heartbeat, restart and recovery;
- fail closed on unknown builds, changed signatures, process mitigations or
  failed probes; keep native icons visible and preserve native Shell behaviour;
- auto-quarantine a failing adapter and expose exactly which capability is
  unavailable rather than claiming universal support;
- keep expensive work outside Explorer callbacks and avoid blocking the Shell UI.

### Layer D — Manager, Profiles and Templates

WinLook Manager owns a dedicated **File Explorer** page for presentation themes/presets, per-surface controls, capability state, adapter diagnostics and safe recovery. The required Desktop Bridge has no end-user opt-in switch; per-build unsupported capabilities are handled by BGW's automatic safe fallback. Additional experimental Explorer chrome styling may have a separate user-controlled setting.

Keep theme data separate from executable integration code. Profiles may select Explorer presentation presets. WLT may carry portable style settings/assets but not private control addresses, process state, binary offsets or local caches.

## 4. Right-click menus

Context-menu work has two jobs:

1. **Commands:** add, group, rename or conditionally expose WinLook commands through the appropriate Shell command-extension interface. Do not remove required native commands or break commands registered by other applications.
2. **Presentation:** customize spacing, typography, icons, separators, surfaces, flyouts and visual arrangement. Use documented styling only where exposed; otherwise route through the separately gated presentation adapter.

Support file, folder, folder-background, drive/namespace and multi-selection contexts. Preserve keyboard invocation, Shift+F10, accessible names, focus and native fallback commands. Menu discovery must be fast; long tasks run outside Explorer's UI path.

## 5. Theme model

Explorer customization is declarative data: color/typography/icon/spacing/density tokens; supported layouts for each surface; context-menu command visibility/order; folder-view preferences and metadata columns; and per-build selectors restricted to adapter versions that explicitly support them.

Do not put raw pointers, control addresses, private process state or arbitrary scripts in a theme. Native Shell extensions/adapters are separately installed, signed/versioned and permissioned. A visual theme must not silently install executable code.

## 6. Compatibility and support states

| Windows generation | Planned route | Important limit |
|---|---|---|
| 95 / 98 / Me / NT4 / 2000 / XP | Research classic Shell extension points and test each OS independently; `IExplorerCommand` is not available | Do not assume Vista+ command interfaces or modern menu registration |
| Vista | Legacy Shell handlers, `IExplorerCommand` API, custom namespace `IShellView` from XP | API availability does not provide a universal Explorer redesign |
| 7 / 8 / 8.1 | Supported Shell handlers including `IExplorerCommand` | Deep chrome styling remains per-build research |
| 10 | Supported Shell handlers plus build-specific presentation research | Do not assume Windows 11 XAML/WinUI control structures |
| 11 | Modern context-menu integration plus supported handlers; optional build-specific adapter | Internal control trees change with feature/cumulative updates |

`IExplorerCommand` is documented from Vista; Windows 11 modern context-menu registration adds package-identity/manifest requirements. Neither fact implies all Explorer chrome is publicly themeable. See [the compatibility matrix](../compatibility/matrix.md) and [source ledger](../references/SOURCES.md).

## 7. Failure isolation and recovery

- Shell handlers stay small and fast; long operations belong in an out-of-process broker.
- The mandatory Desktop Bridge is owned by BGW/WLDE and is separate from Manager UI; it targets only the actual desktop-owning Shell process.
- Additional Explorer chrome-styling hooks remain separate from WLWM and cannot be used as a reason to skip the required WLDE bridge.
- Keep an independently readable disable flag outside Explorer's UI/configuration path.
- On repeated Explorer crashes, adapter initialization failure or signature mismatch, quarantine the adapter and allow native Explorer to restart normally.
- WinLook safe mode must disable experimental hooks without opening Explorer settings.
- Record only WinLook-owned registrations/configuration and undo only those changes.
- Test Explorer restart, profile switching, Windows updates, sign-out, crash, partial unload, disable and uninstall.

## 8. Security and privacy

No credential, secure-desktop, UAC or protected-process interaction. Do not inspect or transmit Explorer paths, selected items, search terms or contents unless an explicit user action requires it. Treat Shell extensions as code in a sensitive process context; themes are data-only. Respect process mitigations and OS access-control decisions; never bypass them.

## 9. Acceptance gates

- [ ] Product scope covers the visible Explorer surfaces listed above.
- [ ] Windows retains filesystem, namespace, Shell activation, security and file-operation ownership.
- [ ] Documented Shell routes are mapped to exact OS minimums.
- [ ] Context commands work for files, folders, background and multi-select without blocking Explorer.
- [ ] Namespace-extension claims are limited to their own virtual namespace/view.
- [ ] The mandatory Desktop Bridge is detected and coordinated by BGW for every verified Explorer-owned desktop session.
- [ ] Per-version adapters target named builds; unknown builds preserve native desktop icons and report unsupported WLDE presentation.
- [ ] Explorer crashes trigger bridge recovery/quarantine and native fallback.
- [ ] Any extra Explorer chrome-styling adapter remains separately disableable without disabling the required desktop bridge.
- [ ] Manager safe mode works when Explorer customization is broken.
- [ ] Profiles/WLT contain portable configuration, not Explorer implementation state.
- [ ] Accessibility, keyboard navigation, high contrast, scaling and localization are considered.
- [ ] Windows 95–11 surface coverage is backed by per-OS evidence.

## 10. Open decisions

- Keep this as an Appearance Engine submodule or later assign a dedicated engine name.
- Exact legacy Shell extension strategies and oldest versions for each surface.
- First modern build for a presentation-adapter proof of concept.
- Which deep Explorer surfaces are safely customizable on each tested build.
- Adapter technology, broker boundary and relationship to BGW health reporting.
- Theme schema and conditional per-build preset strategy.
