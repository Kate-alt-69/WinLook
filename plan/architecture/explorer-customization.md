# WinLook File Explorer Customization

> Status: accepted product direction; detailed implementation strategy and per-version surface coverage remain under research. This is a customization layer for the existing Windows File Explorer, not a replacement file manager or Windows Shell.

## 1. Product requirement

WinLook should deeply customize the actual Explorer interface: file/folder/background right-click menus; navigation/tree pane; address/breadcrumb bar; search and results presentation; command bar, toolbar and ribbon-era surfaces; tabs; folder/file views and layout; Details columns, status/details/preview panes; icons, thumbnails, overlays and previews; window appearance; and presentation of file-operation progress/confirmation surfaces where safely customizable.

Full visible-interface customization is the product goal. Exact coverage is OS/build-dependent: documented Shell extension points cover selected functions, while other visual surfaces require an optional build-specific adapter. This document does not claim every surface has a stable public API.

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

### Layer C — Explorer presentation adapter

Some requested visual controls—especially the standard navigation pane, address/search controls, tabs/command bar styling and overall Explorer layout—are not fully configurable through documented extension points.

To meet the product goal, research a separate optional per-Windows-build Explorer presentation adapter. Depending on the Windows generation, a viable prototype may use a narrowly scoped runtime UI/resource adapter or hook. This is not normal WLWM injection and is not enabled by default.

Requirements:
- target only a named, tested OS build/architecture and known control/framework signatures;
- change explicitly supported presentation properties while preserving Shell command/filesystem ownership;
- fail closed on unknown builds, changed signatures or failed probes;
- never patch files on disk, replace Explorer components or bypass process mitigations;
- keep the adapter independently enabled/disabled from normal WLWM features;
- have an out-of-process coordinator and out-of-band disable/recovery route;
- auto-quarantine after repeat crashes, initialization failures or signature mismatch;
- expose partial/unsupported surfaces instead of pretending universal coverage;
- retain native Explorer operation whenever disabled.

Windhawk's Windows 11 File Explorer Styler demonstrates selected Explorer controls can be restyled on particular builds. This is feasibility evidence for selected surfaces, not a Microsoft compatibility contract or proof of complete Explorer coverage.

### Layer D — Manager, Profiles and Templates

WinLook Manager owns a dedicated **File Explorer** page for presentation themes/presets, per-surface controls and capability state, experimental adapter opt-in/recovery, diagnostics and safe reset.

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
- The presentation adapter is separate from Manager UI and normal WLWM target integrations.
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
- [ ] Deep presentation adapters target named builds and are independently disableable.
- [ ] Unknown builds refuse activation and preserve native Explorer.
- [ ] Explorer crashes trigger adapter quarantine and native fallback.
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
