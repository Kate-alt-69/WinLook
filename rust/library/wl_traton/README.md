# Traton — `wl_traton`

Traton is WinLook's modern Windows translation and object-addressing layer.

It provides one version-aware namespace for addressing Windows targets and
their typed subsystem objects. Native providers can then resolve those paths
against the exact Windows release/build, report capabilities, observe native
events and perform only operations for which a verified provider exists.

## Namespace shape

```text
windows.<release>.<subsystem>.<object>[.<sub-property>...][(<typed-argument>)]
```

The path is split into:

- **Release selector** — for example `10-22h2`. Runtime matching still checks
  the actual Windows build, UBR, edition, architecture, session and host
  generation; the selector is not a compatibility bypass.
- **Subsystem and object** — for example `shell.taskbar`.
- **Sub-property path** — one or more dotted selectors under that object.
- **Optional typed argument** — an opaque item/object reference, not a raw
  HWND, PID, COM pointer, registry path or command.

Examples:

```text
windows.10-22h2.shell.taskbar
windows.10-22h2.shell.taskbar.surface.anchor
windows.10-22h2.shell.taskbar.items.visibility(pinned-shortcut:item-42)
windows.10-22h2.shell.taskbar.items.icon(running-window:window-group-9)
windows.10-22h2.display.monitor.topology
```

For item-scoped taskbar paths, the argument identifies the item category and
opaque stable reference. Pinned shortcuts, running-window buttons, system
buttons and notification-area icons remain distinct object categories.

## Current implementation

The initial crate implements the platform-neutral canonical path parser and
its validation tests. It does **not** yet discover Windows targets, enumerate
Shell objects, install WinEvent/Shell notifications, mutate native taskbars,
or claim any Windows capability. Those responsibilities are specified in
[the Traton architecture](../../../plan/architecture/traton.md) and will be
implemented through explicit providers with BGW-owned lifecycle/event
coordination.

## Safety boundaries

- Parsing and resolving are separate operations; parsing a path never implies
  that the target exists or is supported.
- A provider must match the exact target/build and required capabilities.
- Native callbacks are bounded; BGW routes normalized events to feature owners.
- Any write-capable provider must use the existing BGW restoration contract.
- Unsupported or unverified native operations report unsupported/degraded
  status instead of changing a nearby but different Windows property.

Traton is a Rust 2021 crate with no third-party dependencies and no unsafe
code in this initial parser slice. It can be tested independently of Windows.
