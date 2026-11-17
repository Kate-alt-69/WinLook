# wl_translation — Native Windows Translation, Shell Integration and Quirks

> **Status:** Architecture specification and implementation backlog. This document records the intended design; it does not claim that the library, parser, Shell adapters or runtime integrations exist yet.
>
> **Library name:** \`wl_translation\` (Rust, modern Windows implementation).
>
> **Initial native target:** Windows 10, with Windows 10 version/build detection and capability checks. The first fixed compatibility reference is Windows 10 version 22H2, OS build 19045.

## 1. Purpose

\`wl_translation\` is WinLook's native Windows translation layer. It translates stable, version-aware WinLook intents and object references into the native Windows operations that are actually available on the current machine. It also observes native Windows/Shell state and publishes normalized events so WinLook subsystems can react to real desktop activity.

"Translation" here means **platform/API translation**, not human-language localization.

The library is intended to make Windows Shell, Win32, DWM, taskbar, Explorer desktop, native window and related platform features addressable through one typed contract. A caller should be able to express an intent against a named Windows object without hardcoding an HWND, undocumented class name, registry path or one particular Windows build into every subsystem.

The public contract must make three things explicit:

1. **What object is being targeted?** A versioned Windows namespace path resolves to a typed object or capability.
2. **What does the caller want?** A typed, validated intent describes the requested change or observation.
3. **How can this Windows build do it?** A provider chooses a documented native API, a verified compatibility adapter, a WinLook-owned surface, or reports that the operation is unavailable.

The goal is broad extensibility across Windows—not a promise that every visual property can be changed on every build or in every third-party application.

## 2. Relationship to the other libraries

- **\`wl_translation\`** discovers Windows targets and capabilities, exposes versioned Shell/window objects, normalizes platform events, translates intents, applies supported native changes and manages reversible state.
- **\`wl_animate\`** remains the portable motion engine. It owns keyframes, property tracks, timelines, sequences, parallel/staggered playback, event/interaction triggers, cancellation and playback state. It must not contain Win32, DWM or Explorer-specific code.
- **WLDE** owns the desktop experience and its own rendered surfaces, taskbar-like surfaces, widgets, desktop transitions and presentation policy. It consumes the Windows objects/events supplied by \`wl_translation\` and uses \`wl_animate\` to calculate motion.
- **BGW** owns long-running subsystem lifecycle, event coordination, adapter health, persistence and recovery. Native callbacks must not become a second general-purpose application runtime.
- **WLWM** remains responsible for its own window-management policy and any separately approved target-specific application integration. \`wl_translation\` can expose supported window objects and capabilities without taking over WLWM's ownership.

The intended event/data flow is:

\`\`\`text
Windows USER / Shell / Explorer / DWM
                 |
                 v
       wl_translation adapters
                 |
       typed objects, capabilities,
         snapshots and events
                 |
                 v
             BGW / WLDE
                 |
       event and interaction triggers
                 v
             wl_animate
                 |
       sampled property values
                 v
       WLDE / native presenter
\`\`\`

The libraries should remain independently reusable. A third-party app may use \`wl_animate\` without \`wl_translation\`; a diagnostic or appearance tool may use \`wl_translation\` without starting an animation.

## 3. Windows target identity and versioned namespaces

### 3.1 Target identity

A runtime target identity must contain enough information to avoid treating all Windows 10 installations as identical:

- Product family and major version.
- Release identifier, such as Windows 10 22H2.
- OS build and update revision (UBR).
- Edition and installation type where relevant, including LTSC/IoT distinctions.
- Process and OS architecture.
- Interactive session and desktop context.
- Relevant Shell host identity/restart generation.
- Detected API, renderer and policy capabilities.

The first fixed reference target is Windows 10 22H2, build 19045. The actual UBR must still be read at runtime and included in diagnostics/compatibility matching. A target alias such as \`windows.10\` is a query that resolves to the installed release; it is not permission to run an adapter whose tested build range does not match.

### 3.2 Namespace shape

Use a stable, dotted namespace:

\`\`\`text
windows.<release-or-target>.<subsystem>.<object>[.<property-or-event>]
\`\`\`

Candidate canonical paths:

\`\`\`text
windows.10.22h2.shell
windows.10.22h2.shell.explorer.desktop
windows.10.22h2.shell.taskbar
windows.10.22h2.shell.taskbar.start-button
windows.10.22h2.shell.taskbar.pinned-shortcuts
windows.10.22h2.shell.taskbar.running-window-buttons
windows.10.22h2.shell.taskbar.notification-area
windows.10.22h2.shell.start-menu
windows.10.22h2.win32.window
windows.10.22h2.dwm.window
windows.10.22h2.display.monitor
\`\`\`

A resolved object instance is scoped to the current session and adapter generation. Runtime handles such as HWNDs, PIDs, COM pointers and Shell item pointers are private implementation details; they must not be persisted in XML/WLT profiles or treated as stable object IDs.

A generic release alias may resolve to a canonical release namespace, but a quirk or native operation must also match the actual build/UBR and required capabilities. Unknown builds must not silently inherit the nearest known quirk.

### 3.3 Typed object registry

Each registered object kind should expose:

- Stable object path and object-kind identifier.
- Runtime identity and lifetime/generation token.
- Readable properties and their data types.
- Writable properties and their permitted value ranges.
- Events it can publish.
- Capabilities required to read or modify it.
- Native provider and fallback information.
- Whether the object is native Windows-owned or WinLook-owned.
- Snapshot, verification and restoration support, where applicable.

The registry is not a generic raw Win32 command executor. Callers use typed property names and typed operations, not arbitrary function pointers, registry paths, messages or numeric DWM attribute IDs.

## 4. Shell and taskbar object model

### 4.1 Shell targets

The initial Shell object catalogue should cover, where the installed Shell exposes them:

- Explorer desktop host and desktop view.
- Desktop icons and Shell item references.
- Taskbar surface and its monitor/edge relationship.
- Start button and Start Menu activation.
- Pinned shortcut entries.
- Running-window taskbar buttons.
- Notification area and notification-area icons.
- Task View/search/system-control affordances where a supported route exists.
- Shell readiness, restart and relevant lifecycle events.

Explorer remains the Windows Shell authority for filesystem operations, namespace enumeration, activation, permissions and security. WinLook may translate and customize presentation; it must not take ownership of those Windows responsibilities.

### 4.2 Taskbar layout and placement

Taskbar presentation is a set of independent properties, not one monolithic "taskbar style" setting. The abstract model should include:

- Monitor target and display-relative coordinate space.
- Edge/anchor: top, bottom, left, right, or a WinLook-owned free-positioned surface.
- Size, thickness, bounds, alignment and spacing.
- Inward offset from the selected edge. For example, a bottom-anchored WinLook taskbar may be raised by a configured offset rather than remaining flush with the bottom edge.
- Auto-hide/reveal policy and animation.
- Z-order and full-screen interaction policy.
- Work-area reservation and collision handling.
- Per-monitor layout and DPI scaling.
- Visibility policy and event-driven visibility rules for each item category.

**Full arbitrary placement belongs first to a WLDE-owned taskbar surface.** The Windows AppBar contract is a documented candidate for an independent edge-anchored bar and work-area negotiation. \`SHAppBarMessage\` supports registration, edge selection, querying and setting an appbar rectangle; Windows can adjust the requested rectangle to avoid the native taskbar and other appbars. It is not a documented promise that WinLook can freely move the stock Explorer taskbar to any arbitrary coordinate. A free-positioned WLDE surface is therefore a separate presentation path, not a claim about the native taskbar.

When a WLDE-owned taskbar is active, the policy for the stock Explorer taskbar—coexist, auto-hide, minimize overlap or use a verified build-specific integration—must be explicit. If the requested native operation is unavailable, do not fake success: retain the native taskbar or use the independent WLDE surface with a clear status.

### 4.3 Shortcut icons versus running application buttons

The taskbar data model must distinguish at least:

- **Pinned shortcuts:** persistent launch entries, independent of whether the application is running.
- **Running-window buttons:** entries representing currently open windows.
- **System buttons:** Start and other Shell controls.
- **Notification-area icons:** status/notification items, governed by their own Shell contracts.

These categories need independent ordering, icon, animation and visibility policies. A profile must be able to hide pinned shortcut entries while retaining running-window buttons. It must also be able to show or hide running-window buttons for a specific event or rule, without confusing them with pinned shortcuts.

On Windows 7 and later, the old Quick Launch function is integrated into taskbar buttons; the Windows 10 model should call these **pinned shortcuts**, not pretend that the legacy Quick Launch bar is a distinct native object.

WLDE-owned taskbar surfaces can fully control their own pinned shortcut collection and running-window representation. Manipulating the contents/visibility of the stock Explorer taskbar is only supported where a documented interface or an explicitly verified, reversible build adapter provides that exact operation. Do not infer a universal API from \`ITaskbarList\`, which exposes only its documented taskbar-button operations.

### 4.4 Windows Start button identity

The Start affordance in the intended Windows-style taskbar uses the **Windows logo**, not a WinLook logo. It must invoke the actual Windows Start experience and preserve normal keyboard/accessibility semantics. Prefer the installed Shell/system-provided visual resource or a verified system asset route; do not bundle or redistribute proprietary Windows artwork as an unlicensed static asset. This is not a WinLook branding slot.

## 5. Native Win32 windows, borders and DWM translation

The object catalogue should cover application windows and their supported visual properties, including:

- Non-client frame and border appearance.
- Caption/title-bar presentation and supported caption controls.
- DWM attributes available on the detected release.
- Frame extension/custom-frame contracts for applications that own their window implementation.
- Window bounds, visibility, activation state and relevant native style information.
- DPI-aware geometry and monitor association.
- Supported transitions and compositor-related capabilities.

A high-level request such as "change this window's border" is not a guarantee that Windows permits a third-party process to rewrite every target application's frame. Each provider must declare whether it can safely modify a WinLook-owned HWND, a cooperating application's HWND, or a third-party window through a documented cross-process contract. If the capability is not present, return an explicit unsupported/partial result.

DWM attributes are version-specific. The current Microsoft attribute documentation, for example, lists border/caption color support beginning with Windows 11 build 22000 and system backdrop support beginning with Windows 11 build 22621. These must not be advertised as Windows 10 capabilities. \`DwmExtendFrameIntoClientArea\` is a documented custom-frame technique for an application's own window; it is not a universal third-party application skinning interface.

No operation in this library may replace DWM or assume that DWM exposes arbitrary controls over every application's window. Unsupported third-party custom frames require a target-specific quirk or remain unsupported.

## 6. Events, readiness and animation integration

### 6.1 Normalized event stream

\`wl_translation\` should convert native observations into stable semantic event envelopes. Candidate event families include:

- Shell/Explorer process started, restarted, stopped and health changed.
- Desktop view and taskbar discovered, created, ready or unavailable.
- Start Menu opened/closed where observable through a supported event source.
- Window created, destroyed, shown, hidden, activated, deactivated, moved, resized, minimized and restored.
- Display topology, monitor bounds, DPI and orientation changed.
- Shell item changed or desktop inventory invalidated.
- Session unlocked or interactive desktop became available.
- Native integration capability changed or an adapter entered quarantine.

An event envelope should contain a stable event name, target namespace, source adapter, monotonic timestamp, session/adapter generation, optional opaque object reference and a bounded typed payload. Do not place raw HWNDs, process paths, command lines or other unnecessary private details into durable configuration or public event logs.

### 6.2 Readiness is both a query and an event

The proposed callable path is:

\`\`\`text
windows.<version-or-name>.shell.isready(true[])
\`\`\`

Keep this as the user's initial syntax proposal, but do not freeze its parser grammar until the callable's argument/result types are defined. The implementation must distinguish two related contracts:

- A **readiness query** that returns a typed boolean/status for the requested Shell target.
- A **readiness event** that is emitted when the target transitions into a ready state.

A process existing is not the same as the desktop being ready. At minimum, readiness should distinguish Explorer process discovery, desktop host/view availability and taskbar availability. A combined \`shell.ready\` event should only fire when the readiness policy for the requested object set is satisfied. The event payload should include which components were verified and which remain degraded.

### 6.3 Native event sources and callback discipline

Candidate documented sources include:

- \`SetWinEventHook\` for selected USER/accessibility window events.
- \`SHChangeNotifyRegister\` for relevant Shell/filesystem change notifications.
- Window messages and Shell/COM callbacks for the exact objects that own them.
- AppBar notifications for position/state changes affecting a registered WLDE AppBar.

The host thread requirements, message-loop ownership, callback reentrancy, event coalescing and event loss/backpressure must be handled explicitly. Native callbacks should copy the minimum data needed, enqueue a bounded normalized event and return. They must not render, decode media, parse XML, perform network/file-heavy work, block on BGW or run animation timelines inside Explorer.

Use narrowly selected event ranges and process/thread filters. Do not install a global in-context hook by default. Any in-process Explorer adapter must follow the separate build-specific Explorer integration and recovery rules.

### 6.4 wl_animate trigger contract

The generic trigger model belongs in \`wl_animate\`; Windows event production belongs in \`wl_translation\`. The adapter publishes normalized events, while BGW/WLDE forwards matching event envelopes to the animation controller.

The intended declarative vocabulary includes:

- \`@keyframe\` — define typed property samples and their timing/easing.
- \`@event\` — bind a timeline or animation to a semantic event path.
- \`@interact\` — bind to a typed interaction such as hover, click, focus, drag or activation.

The syntax shown here is conceptual; it is not a committed Rust macro grammar. The public API must also support construction through normal Rust types so a parser is not mandatory.

The Windows startup example should be expressible as an event-driven timeline:

1. After the interactive user's Shell/desktop readiness condition is met, the **Windows logo** enters with a fade/slide motion.
2. A WLDE-owned taskbar surface animates into its configured position.
3. The startup-progress representation remains visible while WinLook's configured startup integrations are enumerated/initialized.
4. When the configured WinLook startup set is ready, the Windows logo moves to its resting position and taskbar shortcut icons appear with staggered motion.
5. Normal hover, click, focus, window activation and desktop interaction triggers become active.

Do not claim a universal Windows event that says "every startup application has finished loading." WinLook can observe selected process/window events and its own configured startup set; any quiet-period heuristic must be labelled as a WinLook estimate. The startup sequence begins only on the interactive desktop after Windows logon. Winlogon, authentication UI, secure desktop and the Windows boot screen remain outside the customization surface.

## 7. XML profiles and the shared callable registry

The XML configuration path should be a declarative front end to the same typed object/property/event registry used by Rust callers. This lets profiles describe editable Shell and Windows options while code and the future parser resolve the same canonical names.

An illustrative profile:

\`\`\`xml
<?xml version="1.0" encoding="utf-8"?>
<wl:translation
    xmlns:wl="urn:winlook:translation:v1"
    schemaVersion="1"
    target="windows.10.22h2">

  <wl:object ref="windows.10.22h2.shell.taskbar">
    <wl:set property="surface.anchor" value="bottom" />
    <wl:set property="surface.inward-offset" value="72px" />
    <wl:set property="pinned-shortcuts.visibility" value="hidden" />
    <wl:set property="running-window-buttons.visibility" value="visible" />
  </wl:object>

  <wl:bind event="windows.10.22h2.shell.ready">
    <wl:play timeline="desktop-startup" />
  </wl:bind>
</wl:translation>
\`\`\`

This is a schema sketch, not a finalized XSD or a statement that the parser exists. Exact property names, unit rules, namespace aliases and event grammar remain implementation-contract work.

The parser/compiler must:

- Resolve every \`ref\`, \`property\`, \`event\` and \`timeline\` against registered typed definitions.
- Reject unknown paths, wrong value types, unsupported units, out-of-range values and incompatible schema versions.
- Validate the target build and required capabilities before producing an operation plan.
- Produce a typed AST/translation plan, never directly execute XML nodes as commands.
- Support comments, stable IDs, versioned schemas and useful source-location diagnostics.
- Allow the Manager to expose permitted values as editable controls generated from metadata.
- Preserve user-specified unsupported fields as visible unsupported/degraded settings or reject them according to schema policy; never silently reinterpret them.

XML is **configuration, not a scripting or privilege-escalation language**. It must not support arbitrary native function names, raw Win32 message sends, arbitrary COM activation, DLL loading, shell command execution, script blocks, dynamic code, raw pointer/handle values or arbitrary code injection. Native actions must be allow-listed typed operations implemented by a trusted provider. Any third-party extension/provider is separately installed, permissioned and trusted; an XML file cannot install or authorize one.

WLT may package declarative translation profiles under a dedicated translation/configuration directory, subject to WLT path, size, signature/trust and asset validation rules. Profiles must not package native hook code or executable workarounds as if they were configuration.

## 8. Translation providers, capability detection and result reporting

A translation request should return a structured result rather than a bare success boolean. The result should identify:

- Requested target and resolved target identity.
- Requested properties and applied properties.
- Provider/API selected.
- Exact capability checks used.
- Whether the result is full, partial, unsupported, skipped or failed.
- Any fallback selected and why.
- Snapshot/restore token or transaction ID when a change was applied.
- Diagnostic code and human-readable explanation.

Provider selection order:

1. Documented, supported API that exactly meets the request.
2. A verified, version-specific adapter with explicit compatibility and recovery metadata.
3. A WinLook-owned surface that can provide the requested presentation without modifying native Shell internals.
4. Explicit unsupported/degraded status.

A provider must never report full success when it only changed a nearby but semantically different property. For example, hiding pinned shortcuts is not the same as hiding running-window buttons; changing a WinLook-owned taskbar is not the same as moving the Explorer taskbar.

## 9. Quirks and compatibility rules

The compatibility subsystem is named **Quirks**. It captures target-specific differences that cannot be expressed as one universal Windows API contract.

Each quirk definition should include:

- Stable quirk ID and human-readable reason.
- Exact Windows release/build/UBR range and architecture match.
- Target object/application identity requirements.
- Required and excluded capabilities.
- Affected properties/events.
- Provider/adapter identifier and declared side effects.
- Preconditions, timeout and health checks.
- User-consent requirement and risk classification.
- Expected result and verification method.
- Snapshot and rollback procedure.
- Known conflicts, regression notes and last-tested build.
- Enable/disable status and quarantine reason.

Quirks must be data-driven only where the operation remains within a trusted allow-listed provider. Third-party quirk packs cannot supply executable code through XML. An unknown or stale quirk must be disabled, not guessed at. A quirk that can destabilize Explorer or another application's UI must be isolated, opt-in and independently recoverable.

This system is also the place to record application-specific Win32 frame/DWM differences. The quirk catalogue may select a known provider for a particular app/build; it must not silently patch the app binary or assume all apps share the same non-client implementation.

## 10. Transactions, ownership and restoration

Every write-capable translation provider must declare its reversibility. Where the operation changes native or user-visible state, the transaction flow is:

1. Resolve the target and confirm identity/generation.
2. Query capability and current value.
3. Build a typed plan and detect conflicts.
4. Capture the original value/state and write a bounded restore record.
5. Apply operations in a documented order.
6. Read back and verify the resulting state.
7. Commit the applied-state record only after verification.
8. On failure, roll back completed operations in reverse order.
9. On Explorer restart, BGW crash, adapter quarantine, profile removal or uninstall, reconcile and restore the state owned by WinLook.

Restoration must compare the current value with the value WinLook last applied before replacing it. If the user or Windows changed the value later, do not blindly overwrite the newer state. Report a conflict and preserve user ownership.

The existing [restoration architecture](restoration.md) is the source of truth for crash-safe Explorer/Shell restoration journals. This library must use that contract rather than invent a second journal format. Raw handles are never durable restore records; store stable identity, original state, applied state and adapter generation.

Read-only operations may be available without a write transaction. Irreversible or weakly reversible operations must be marked as such and must not be silently grouped into a "safe" batch.

## 11. Explorer, DWM and security boundaries

- Never patch, replace, binary-modify or redistribute a modified \`explorer.exe\` on disk.
- Never replace or disable DWM.
- Windows keeps filesystem, Shell namespace, file-operation, activation and security authority.
- Do not modify Winlogon, credential UI, authentication controls or secure desktop.
- Do not attach indiscriminately to Explorer folder-window processes; identify the actual desktop-owning Shell host.
- Documented Shell/Win32 interfaces are the baseline. Runtime hooks are exceptional, exact-build adapters and must comply with the approved Explorer Desktop Bridge policy, explicit risk/consent requirements for deep presentation changes, health checks and out-of-band recovery.
- Do not bypass process mitigations, code-integrity protections or access boundaries. If the target rejects an operation, report it as unsupported.
- Keep callbacks short and bounded. Run rendering, animation, persistence, media work and XML parsing outside Explorer.
- No XML/profile is allowed to authorize new code, broad injection or privileged operations.

## 12. Windows 10 baseline and servicing reality

Windows 10 is the chosen initial modern implementation target, with Windows 10 22H2/build 19045 as the first fixed reference. It is a useful fixed engineering baseline, but it is not correct to describe every Windows 10 edition/device as receiving no further updates:

- Microsoft lists Windows 10 22H2 general servicing as ended on **October 14, 2025**.
- The Extended Security Updates (ESU) programme is a separate exception for enrolled devices.
- Windows 10 LTSC/LTSB editions have edition-specific lifecycles; for example, Windows 10 IoT Enterprise LTSC 2021 has a different lifecycle from ordinary 22H2 Home/Pro.
- Therefore the implementation must test and report the actual build/UBR and servicing channel, not assume that "Windows 10" is an immutable ABI or that Shell internals never change.

The design can treat build 19045 as the first compatibility reference while still recording the exact patched UBR and retesting after relevant cumulative updates. An end-of-mainstream-support date is not evidence that unsupported Shell internals have become stable or safe.

See [Windows 10 lifecycle](https://learn.microsoft.com/en-us/lifecycle/announcements/windows-10-end-of-support), [supported Windows client versions](https://learn.microsoft.com/en-in/windows/release-health/supported-versions-windows-client) and [Windows 10 release information/ESU](https://learn.microsoft.com/en-us/windows/release-health/release-information).

## 13. Performance and lifecycle requirements

- One bounded native event observation pipeline per relevant session/host, not one hook or timer per widget/icon.
- No busy polling for Shell readiness; combine event subscriptions with bounded, explicit state reconciliation.
- Coalesce repeated layout/display/Shell notifications and discard stale events from an old Explorer generation.
- Use bounded queues and explicit overflow policy; never let a Shell callback block waiting for a consumer.
- Avoid per-frame Shell queries. Resolve native object state on change and let \`wl_animate\` sample cached properties on the shared frame clock.
- Keep native event observation, presentation, animation sampling and rendering costs independently measurable.
- Pause/resume adapters across session lock/unlock, Explorer restart, monitor changes and resource-pressure states.
- Respect reduced motion, accessibility, keyboard navigation, UI Automation/accessible names and high-contrast/forced-colour policy.
- Restore native presentation if the WinLook-owned surface or adapter stops unexpectedly.

## 14. Implementation backlog for the next agent

The following work belongs to the implementation agent; this planning pass does not perform it.

### Public contract and core model

- [ ] Create the Rust \`wl_translation\` crate in the approved \`rust/library/\` layout and add it to the workspace.
- [ ] Define \`WindowsTarget\`, release/build/UBR identity, edition, architecture, session and adapter-generation models.
- [ ] Define canonical namespace parsing, aliases, typed object paths and lifetime-safe object references.
- [ ] Define the object/property/event/capability registry and structured operation/result/error types.
- [ ] Define the provider interface and capability negotiation/fallback rules.
- [ ] Define a generic normalized event envelope that can be consumed by BGW/WLDE and forwarded to \`wl_animate\` without making \`wl_animate\` depend on Windows.

### Windows 10 native adapters

- [ ] Implement read-only OS/build/UBR/edition/architecture/session discovery and capability reporting.
- [ ] Implement carefully selected WinEvent observation with correct message-loop ownership, unhook lifecycle, callback reentrancy handling and bounded delivery.
- [ ] Implement relevant Shell change notifications with correct registration, lock/unlock and deregistration.
- [ ] Implement taskbar/Start/Shell object discovery and document which properties are truly readable/writable.
- [ ] Implement AppBar registration/position/work-area negotiation for a WLDE-owned taskbar surface; preserve coexistence with the native taskbar.
- [ ] Implement taskbar item-role models for pinned shortcuts, running-window buttons, system buttons and notification-area icons.
- [ ] Implement supported Win32/DWM window-frame inspection and operations with per-attribute version gating.
- [ ] Keep third-party app window operations behind per-target capability/consent and explicit rollback contracts.

### Quirks, XML and transaction support

- [ ] Define the Quirks rule schema, matcher, trust model, test metadata and quarantine mechanism.
- [ ] Define the XML schema/namespace and the exact callable grammar for versioned object/event paths.
- [ ] Resolve the proposed \`shell.isready(true[])\` notation into a typed query/event contract with parser tests.
- [ ] Compile XML into a typed AST and validated operation plan; reject script/native-code escape hatches.
- [ ] Add source-location diagnostics, schema migration, capability-aware validation and Manager-editable metadata.
- [ ] Connect write-capable providers to the existing restoration journal and compare-before-restore conflict logic.

### WLDE, wl_animate and verification

- [ ] Define and test the event handoff from \`wl_translation\` through BGW/WLDE to \`wl_animate\`.
- [ ] Extend \`wl_animate\` with generic \`@keyframe\`, \`@event\` and \`@interact\` concepts, timelines, sequences, parallel groups, staggering and interruption policies.
- [ ] Implement the Windows-logo/Start-button and taskbar startup choreography as a WLDE-owned presentation example, not a Winlogon/secure-desktop modification.
- [ ] Add deterministic tests for parser, event ordering, duplicate/coalesced events, stale generations, transaction rollback, user-change conflicts and unknown builds.
- [ ] Add Windows 10 22H2/build 19045 test coverage and record exact UBR/edition/architecture for every native result.
- [ ] Test Explorer restart, BGW crash, adapter failure/quarantine, multiple monitors, DPI changes, taskbar edge changes, full-screen applications, lock/unlock and uninstall restoration.
- [ ] Benchmark callback cost, event-queue bounds, event-to-animation latency, taskbar frame budget and idle overhead.
- [ ] Document each native capability with API source, exact supported OS range, test evidence and explicit unsupported cases.

## 15. Decisions still open

- Exact canonical release alias spelling and whether UBR ranges are expressed in XML or only in Quirks metadata.
- Whether the Windows 10 initial target means all editions on build 19045 or a narrower tested edition/channel matrix.
- Whether WLDE's custom taskbar should register as an AppBar by default, offer free-positioned overlay mode, or expose both modes.
- The default coexistence/auto-hide/recovery policy for the stock Explorer taskbar while a WLDE surface is active.
- Which native taskbar item categories can be modified safely on stock Windows 10, beyond WLDE-owned item rendering.
- The final object catalogue for Start, Taskbar, Explorer desktop, notification area, system surfaces and third-party Win32 windows.
- Exact event/readiness semantics, especially the distinction between Shell process start, desktop ready, taskbar ready and WinLook startup-app readiness.
- Final XML XSD, schema versioning, Manager editing metadata and WLT packaging path.
- Whether and how a trusted Quirks catalogue can be updated independently from the application.
- The consent, isolation and supported-build policy for deep Explorer runtime presentation adapters.
- Future Windows 11 provider coverage and any separate legacy C counterpart; neither is implied by the Windows 10 Rust baseline.

## 16. Primary references

- [Windows 10 reaching end of support](https://learn.microsoft.com/en-us/lifecycle/announcements/windows-10-end-of-support)
- [Supported versions of Windows client](https://learn.microsoft.com/en-in/windows/release-health/supported-versions-windows-client)
- [Windows 10 release information and ESU](https://learn.microsoft.com/en-us/windows/release-health/release-information)
- [Customize the Windows 10 taskbar](https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/customize-the-taskbar)
- [The Windows taskbar and documented Shell contracts](https://learn.microsoft.com/en-us/windows/win32/shell/taskbar)
- [SHAppBarMessage](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shappbarmessage)
- [Using Application Desktop Toolbars](https://learn.microsoft.com/en-us/windows/win32/shell/application-desktop-toolbars)
- [DWMWINDOWATTRIBUTE](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwmwindowattribute)
- [Custom Window Frame Using DWM](https://learn.microsoft.com/en-us/windows/win32/dwm/customframe)
- [SetWinEventHook](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwineventhook)
- [SHChangeNotifyRegister](https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/nf-shlobj_core-shchangenotifyregister)
- [WLDE architecture](wlde.md)
- [Appearance Engine](appearance-engine.md)
- [Explorer customization](explorer-customization.md)
- [Restoration architecture](restoration.md)
- [WinLook animation suite](animation-suite.md)
- [Windows 10 compatibility notes](../compatibility/win10.md)
