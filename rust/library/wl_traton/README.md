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
item visibility and monitor topology. The stock taskbar item-visibility member
is marked experimental and cannot validate unless capability evidence and
explicit consent are supplied. No native taskbar mutation provider is shipped.

## Native Windows boundary

The Windows target adapter uses `windows-sys 0.61.2` for read-only registry,
process-architecture and session queries. The probe requires Windows 10 version
1709 / build 16299 or later. Registry access requests `KEY_QUERY_VALUE` and
the explicit 64-bit view only; the flag is ignored on 32-bit Windows. It never
writes registry values.

Native event observation is not enabled yet. In particular, Traton does not
install `SetWinEventHook`: its calling thread must own a message loop, and
BGW's current host loop has not yet been extended with that pump. BGW remains
the owner of lifecycle, future event-pump scheduling, routing and backpressure.

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
