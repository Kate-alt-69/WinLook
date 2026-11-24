# Traton — `wl_traton`

Traton is WinLook's version-aware Windows object, capability and provider layer.

## Namespace

```text
windows.<release>.<subsystem>.<object>[.<sub-property>...][(<typed-argument>)]
```

Examples:

```text
windows.10-22h2.shell.taskbar
windows.10-22h2.shell.taskbar.surface.anchor
windows.10-22h2.shell.taskbar.items.visibility(pinned-shortcut:item-42)
windows.10-22h2.display.monitor.topology
```

The dotted suffix selects a registered member. The optional argument is a
typed opaque object reference, never a raw HWND, PID, COM pointer, registry path
or command. A property value is supplied separately in a typed operation.

## Implemented core

- Canonical path parsing and bounded typed object-reference parsing.
- Read-only Windows target discovery: release selector, build, UBR, edition,
  installation type, process/native architecture and interactive session.
- Typed capability states. Missing evidence remains `Unknown`, not `Available`.
- Object/member registry with build ranges, action permissions, argument-kind
  policies, value schemas and explicit capability gates.
- Typed operation validation and structured operation-result types.
- Provider descriptors, compatibility selection order and a provider trait
  exposing read/capture/apply/inspect/ownership/verify/restore operations.
- Bounded normalized event envelopes and a generation-aware queue that
  coalesces high-frequency state updates while preserving discrete lifecycle
  events where capacity permits.

The first registry catalogue is deliberately small: taskbar surface anchor,
item visibility, read-only taskbar window count and monitor topology. The
Windows 10 22H2 taskbar window-count provider enumerates only Shell taskbar
windows owned by BGW's verified Explorer process. It reports bounds and
WS_VISIBLE style state only; it does not identify taskbar items or mutate
Explorer. The stock taskbar item-visibility member remains experimental and
has no native mutation provider.

## Native Windows boundary

The Windows target adapter uses `windows-sys 0.61.2` for read-only registry,
process-architecture and session queries. The probe requires Windows 10 version
1709 / build 16299 or later. Registry access requests `KEY_QUERY_VALUE` and
the explicit 64-bit view only; the flag is ignored on 32-bit Windows. It never
writes registry values.

BGW installs a narrow set of out-of-context `SetWinEventHook` subscriptions
on its own dedicated message-pump thread. Traton supplies the exact hook ranges
and a pure classifier mapping selected native IDs into `WinEventCategory`
and `TratonEventKind` values. The observer covers foreground activation,
window-object create/destroy/show, geometry-change and minimize/restore.
It excludes BGW's own process and uses an atomic category mask; HWND/PID
values are not retained or exported.

For foreground and object-level events, the classifier requires
`OBJID_WINDOW` / `CHILDID_SELF`. This identifies the window object itself,
not necessarily a root/top-level HWND. Explicitly resized child windows can
also produce `EVENT_OBJECT_LOCATIONCHANGE`. Minimize/restore are classified
by their system event ID and non-null HWND. The callback does not query window
metadata, call COM, perform I/O or block. The installing thread removes every
hook during shutdown.

BGW converts coalesced event categories into handle-free Traton events through
the bounded queue. The shared auto-reset wake interrupts BGW's maintenance
wait, but the current host loop consumes the normalized signals for aggregate
diagnostics only; it does not route per-window state to WLDE or wl_animate.
Stable window identity, feature-owner dispatch and Shell ChangeNotify
subscriptions are not implemented yet.

## Read-only Windows 10 22H2 taskbar snapshot

On exact build 19045, BGW can query the primary `Shell_TrayWnd` and secondary
`Shell_SecondaryTrayWnd` top-level windows after it has verified the Explorer
process/session. Traton rechecks each returned HWND's owner PID against that
verified Explorer PID before taking a snapshot. The public result contains only
role, screen-space bounds and `WS_VISIBLE` style state; HWNDs never leave the
Win32 adapter. The class names are Shell implementation details, so the
adapter requires the exact `10-22h2` selector and build 19045, and reports
partial capability coverage. BGW revalidates the same Explorer process
generation immediately before and after the snapshot. This does not enumerate
pinned shortcuts, running-window buttons or notification-area icons.

## Safety and ownership

- Parsing does not imply that an object or operation exists.
- Every operation resolves against the detected target and a registered member.
- Unknown capabilities fail closed; experimental members require explicit opt-in.
- Providers must declare exact ownership, operation and build support.
- Any write-capable provider must connect to the existing BGW restoration
  journal and compare-before-restore rules before being enabled.
- WLDE, WLWM, WLDM, WLOSDE and other feature owners retain their own policy and
  persistent state; Traton translates native object access, not subsystem policy.

The portable core is safe Rust. The small Windows FFI boundary is isolated in
`src/windows.rs`; non-Windows builds expose an explicit unsupported-platform
result for target detection.
