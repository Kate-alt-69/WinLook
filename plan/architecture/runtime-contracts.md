# WinLook Runtime and Module Contracts

> Status: detailed design proposal. This defines intended boundaries; it is not an implemented process model or a promise that every contract exists on every Windows release.

## 1. Design invariants

- Windows owns logon, authentication, secure desktop, native input/focus fundamentals, Shell activation and DWM composition.
- Manager and desktop-facing components run in the interactive user's session.
- A privileged Service is optional and must be justified operation by operation.
- No core operation requires modifying Explorer or DWM binaries.
- No injected module is required for Windows or a target application to keep working.
- A disabled, stopped or incompatible WinLook component has a deterministic fallback.
- Every system-wide change has an ownership record and restoration strategy before application.
- Legacy C and modern Rust remain distinct targets with OS-specific adapters.

## 2. Two-executable process topology

The principal application topology is fixed at two cooperating executables:

| Executable | Responsibility | Session/privilege | Failure expectation |
|---|---|---|---|
| `winlook.exe` | Manager UI: settings, previews, templates, catalogue/library UI, diagnostics and user controls | Interactive user; ordinary privilege | May restart independently; does not own the background runtime |
| `winlook.bg.exe` | BGW background runtime: hosts/co-ordinates background subsystems, applies configuration and monitors their lifecycle | Interactive user session; ordinary privilege by default | Manager-independent; recovers from Manager disconnect and restores last valid configuration |

WLDE, WLWM, WLDM, WLOSDE, Appearance/Icon, Audio & Media, CPUv/GPUv coordination and other background capabilities are modules hosted/coordinated by BGW, not additional required application executables. Internal threads or bounded workers do not change this two-executable product model. A separately isolated helper may be introduced only for a specific reviewed security, compatibility or crash boundary.

An optional privileged Service, if a later feature proves it necessary, is separate from these two principal executables and must expose only typed, allowlisted operations. It is not the BGW runtime.

For the full startup, IPC, configuration-acknowledgement, subsystem-registration and recovery design, see [background-runtime.md](background-runtime.md).

## 2A. Logical module topology

The following are logical modules/ownership roles inside the two-executable architecture, not a mandate to launch one process per row.

| Role | Responsibility | Session/privilege | Failure expectation |
|---|---|---|---|
| WinLook Manager | Settings, previews, templates, catalogue UI, diagnostics and consent | Interactive user; ordinary privilege | UI may restart without losing committed configuration |
| WinLook Agent | Per-user startup/shutdown, session coordination and user state | Interactive user session | Native desktop remains usable if it exits |
| Main BGW | WLWM process observation, classification/cache, policy and optional injection orchestration | Same user session as targets by default | Stops optional integrations; never blocks application launch |
| WLDE Runtime | Wallpapers, widgets, desktop presentation, Groups and resource policy | Interactive user session | Falls back to static/native desktop presentation |
| WLOSDE surfaces | WinLook notifications, popups and OSD | Interactive user session | Surfaces disappear; native security UI is unaffected |
| WLDM adapter | Monitor discovery/topology and approved display changes | User context by default | Retains or restores safe display configuration |
| Audio & Media adapter | Audio endpoints/sessions and provider-backed media metadata/transport | User context; ordinary controls do not require elevation | Native audio/player operation continues; unsupported controls are omitted |
| Explorer presentation adapter | Optional Explorer styling/Shell extension coordination | Separate opt-in, build-specific component with out-of-process coordinator | Quarantine/disable adapter; native Explorer and file operations continue |
| Optional Service | Only narrowly scoped operations requiring elevated privilege | Service session | Privileged actions become unavailable; interactive UI continues |
| In-process module | Optional target-specific WLWM integration | Target process security context | Module failure must not terminate/corrupt target |
| Extension host | Untrusted widgets and interactive content | Restricted process/capability sandbox | Extension can be stopped independently |

## 3. Ownership boundaries

| State or operation | Authoritative owner |
|---|---|
| User preferences and profile selection | Manager configuration service |
| Session lifecycle | Agent |
| Target policy and compatibility cache | BGW |
| Window observations, rules and layout decisions | WLWM |
| Desktop presentation and wallpaper playback | WLDE |
| Monitor topology and display-profile state | WLDM |
| Audio endpoint/session control state | Audio & Media adapter |
| Media metadata and transport requests | OS media-session provider or explicit player adapter |
| Explorer theme and presentation policy | Appearance Engine / Explorer customization subsystem |
| Notification queue and presentation policy | WLOSDE |
| Animation primitives/timing semantics | wl_animate and wl_Lanimate |
| Original values and change journal | Restoration subsystem |
| Template contents and import transaction | WLT service in Manager |

A consumer may request an operation; it may not silently become the owner of another subsystem's persistent state.

## 4. IPC contract

The IPC transport is undecided and must be evaluated per OS family. Do not assume modern named pipes, COM, RPC or shared-memory options are universal back to Windows 95.

Every message should contain:
- protocol major/minor version;
- message type and bounded payload schema;
- request/correlation ID;
- sender role and intended recipient;
- user/session scope where relevant;
- deadline/cancellation semantics for blocking operations;
- explicit result status and structured error code;
- capability/permission requirement for state-changing requests.

### IPC safety

1. Authenticate the peer using the strongest mechanism available on that OS; verify process/session identity rather than trusting a role string.
2. Restrict endpoints to the current user/session unless a narrowly reviewed privileged operation requires otherwise.
3. Bound message size, nesting, string lengths, queue depth and outstanding requests.
4. Reject malformed messages and unsupported required protocol versions.
5. Never accept executable code, arbitrary library paths or unrestricted command lines as generic IPC actions.
6. Make retries safe through idempotency keys or durable transaction IDs.
7. Define timeout, cancellation, sender exit, receiver restart and partial completion behaviour.
8. Never wait on another process while holding loader, global UI or render locks.
9. Minimize private window titles, paths and process details in ordinary logs.

Minor protocol versions may add optional fields with safe defaults. Major mismatches require a compatibility path or refusal. Unknown configuration data should be preserved where safe rather than silently discarded.

## 5. Capability and operation result model

Proposed capability states:

| State | Meaning | Manager behaviour |
|---|---|---|
| Available | API, runtime and adapter probes passed | Offer normal controls |
| Partial | A documented subset is available | Explain limitations |
| OptionalDependency | Separately installed and verified component required | Show dependency and consent |
| Experimental | Narrow tested configuration only | Require opt-in and recovery path |
| Unsupported | Requirement absent or policy forbids operation | Disable with reason |
| Unknown | Evidence insufficient | Do not auto-enable or inject |

Capability availability is separate from the result of one operation. Proposed operation results: Success, Partial, Rejected, Unsupported, Cancelled, TimedOut and Failed. Each result carries a stable reason code and recovery guidance.

A capability report should include OS family/version/build, edition/service pack where relevant, process and OS architecture, API exports/interfaces, runtime dependencies, graphics/driver state, session context and adapter version. Store only what is needed for correct decisions and diagnosis.

## 6. Configuration ownership and precedence

Use one validated, versioned conceptual configuration model with platform-specific adapters. File format and storage paths remain open decisions; do not force a modern parser/runtime onto legacy targets.

Proposed precedence, strongest first:
1. Explicit temporary user action for the selected window/surface.
2. Explicit per-item or per-application override.
3. Active workspace/profile rule.
4. Active WinLook theme/template setting.
5. User global preference.
6. Safe platform default.

This precedence is a proposal and requires approval. Manager should explain which rule won and why. Templates must not silently override machine-local security, session or device state. Invalid required values reject the affected component, not unrelated settings.

## 7. Lifecycle and health

Each long-lived component defines startup prerequisites, capability probes, ready signal, health status, bounded graceful shutdown, dependency-loss behaviour, restart backoff, restart-loop protection, cleanup/restoration ownership and state persistence.

Proposed health states: Starting, Ready, Degraded, Stopping, Stopped, Unresponsive and Quarantined.

A supervisor may restart a crashed non-critical component but must not loop on a deterministic compatibility/safety rejection. Quarantine retains a reason and user-visible recovery action.

## 8. Privilege and trust model

- Manager, Agent, BGW, WLDE and WLOSDE run as ordinary user unless a specific operation proves otherwise.
- Optional Service exposes typed allowlisted operations, not a general elevated command runner.
- WinLook never collects passwords, PINs or secure-desktop content.
- Community content is untrusted even when downloaded from a popular catalogue.
- Native code is not a normal WLT customization payload.
- In-process modules obey process mitigations and may not bypass them.
- DWM, authentication/security-critical and protected processes are excluded.
- Explorer is never modified on disk or replaced. Documented Shell extensions and any deep presentation adapter follow the dedicated Explorer policy; the latter is opt-in, per-build, isolated and recoverable, not general WLWM injection.
- Logs and crash reports require user control and data minimization.

## 9. Cross-version adapter contract

Each adapter declares exact OS family/version range, APIs/exports/interfaces, runtime/redistributables, architecture/bitness, capability probes, supported and omitted operations, apply/observe/restore/shutdown behaviour, fallback and exact test evidence.

Do not infer runtime support from a modern SDK header. Use late binding or separate binaries where missing imports would prevent startup.

## 10. Acceptance gates

- [ ] Every role has an owner, session, privilege and crash boundary.
- [ ] IPC transport and peer authentication are selected per OS family.
- [ ] Protocol versioning, message bounds, cancellation and idempotency are specified.
- [ ] Capability states and result codes are ratified.
- [ ] Configuration precedence and unknown-field handling are ratified.
- [ ] Every privileged operation is enumerated and justified.
- [ ] Restart, quarantine and recovery behaviour is tested in a prototype harness.
- [ ] Windows-native fallback is demonstrated when Agent, BGW, WLDE or WLOSDE exits.

## 11. Open decisions

- Exact modern and legacy process split.
- IPC transport per OS family and cross-integrity authentication.
- Configuration format, location and profile layering.
- Stable capability IDs and error-code catalogue.
- Whether a Service is needed for the first release.
- Restart ownership and update ordering.
- Diagnostics retention and privacy controls.
