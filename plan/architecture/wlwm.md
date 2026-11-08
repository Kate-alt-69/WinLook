# WLWM — WinLook Window Manager (WELMO)

## Purpose
WLWM brings selected Linux desktop/window-manager capabilities to Windows while retaining the Windows windowing system and DWM. Hyprland is a design reference for layouts, window rules, workspaces, animation and visual effects; WLWM is not a port of its Wayland compositor, binaries or config language.

References: [Hyprland](https://hypr.land/), [window rules](https://wiki.hypr.land/Configuring/Basics/Window-Rules/), [workspace rules](https://wiki.hypr.land/Configuring/Workspace-Rules/).

## Coexistence
- DWM remains intact.
- Windows continues to create/activate applications and owns native input/focus/z-order fundamentals.
- WLWM adds policy and optional visual integrations over eligible windows.
- If WLWM stops or a target is incompatible, native Windows behaviour remains the fallback.
- Do not patch/replace Explorer.exe on disk. Deep Explorer presentation customization is handled by a separate opt-in, per-build Appearance Engine adapter; it is not a normal WLWM target-injection feature.
- Never inject DWM or credential/authentication/security-critical processes.

## Features
### Layouts
Candidate layouts: dynamic tiling, master/stack, dwindle/split, monocle, floating, hybrid tiling/floating and user-defined saved arrangements. Account for work areas, monitors, DPI, taskbar/docks, owned dialogs, child windows and user overrides.

### Window rules
Settings live in WinLook Manager and are included in WLT where portable. Rules can match verified target identity, process architecture, window class/title, monitor, workspace and user tags. Actions may set floating/tiling, geometry, workspace, animation profile, border appearance and focus behaviour. Define deterministic precedence, conflict reporting and live updates.

### Workspaces
- Windows 10+: use documented IVirtualDesktopManager operations when they fit (query membership and move a window).
- The interface does not provide every create/switch operation WLWM may want. Undocumented interfaces are not stable contracts.
- Legacy systems: WLWM may implement logical workspaces by tracking/hiding/showing eligible windows. This is a workflow/presentation layer, not a security desktop or independent logon session.
- Do not automatically switch the user to another workspace just because an app launches.

Source: [IVirtualDesktopManager](https://learn.microsoft.com/en-us/windows/desktop/api/shobjidl_core/nn-shobjidl_core-ivirtualdesktopmanager).

## BGW compatibility layer
The main BGW owns target detection, compatibility analysis/cache and injection orchestration.

### Target classification
Potential inputs: executable file identity/hash/version, PE architecture/subsystem, process bitness, OS family/version/build, runtime/graphics state where detectable, process mitigations, known exclusions, WLWM module version and active features.

### Binary compatibility cache
Local, binary-encoded records store target identity, analyzed environment, supported feature/module set, result (supported/partial/unsupported/unknown/quarantined), confidence/reason codes, schema version, analysis time and invalidation data.

Invalidate/re-evaluate when binary hash/version/architecture, OS/build, relevant mitigation, WLWM module or compatibility database changes. Even on cache hit re-check target identity, current process state, current mitigations and exclusions. Cache analysis is not a permanent guarantee.

### Injection
Process injection and hooks are accepted techniques for compatible targets and are coordinated by the BGW. Keep in-process modules small, versioned and health monitored. Do not perform expensive analysis/network access under loader lock. If injection fails or is rejected, do not prevent the app from launching.

Windhawk is a reference for early-load rationale, process exclusions and compatibility tradeoffs: [injection targets and critical system processes](https://github.com/ramensoftware/windhawk/wiki/Injection-targets-and-critical-system-processes). Do not copy its default breadth without WinLook-specific risk review.

### Mitigations
Windows 8.1's PROCESS_MITIGATION_DYNAMIC_CODE_POLICY can prohibit dynamic code generation or modification of executable code. Query applicable target policies and skip techniques that conflict; do not try to bypass process policy.

Source: [PROCESS_MITIGATION_DYNAMIC_CODE_POLICY](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-process_mitigation_dynamic_code_policy).

## Settings, animation and fallback
WLWM has flexible settings in WinLook Manager, not a mandatory independent config language. Settings/rules/layouts can be included in WLT. It consumes wl_animate on modern Windows and wl_Lanimate on legacy systems. It must retain native fallback for unsupported effects and ensure BGW shutdown has a defined safe state.

## Detailed target lifecycle and policy contract

### Target lifecycle

1. **Observed:** an OS-specific process/window observer reports a process.
2. **Identity resolved:** BGW identifies the executable and process architecture. Ambiguous identity is not eligible for injection.
3. **Excluded:** protected/system/security targets, explicit deny rules and unsupported architectures stop here.
4. **Compatibility evaluated:** BGW reads a cache record or performs bounded local analysis.
5. **Launch revalidated:** current identity, OS/build, architecture, relevant mitigations, module version and exclusion policy are checked again.
6. **Policy outcome:** observation-only, approved optional integration, or no WLWM integration. These outcomes are distinct.
7. **Integration requested:** only a reviewed target-specific adapter can request an in-process module.
8. **Handshake pending:** module version and expected contract are verified within a bounded time.
9. **Active:** BGW records health and target ownership.
10. **Degraded/failed/quarantined:** stop new work; preserve native app behaviour; recover or unload only where safe.
11. **Detached:** process exit, user disable or policy change ends the integration and releases owned resources.

A failed or rejected path never blocks the target application from launching. Unknown compatibility is not a weak form of allow.

### Compatibility cache record

Proposed local record fields:

| Field | Purpose |
|---|---|
| Schema/record version | Safe decoding and migration |
| Canonical target identity | Distinguishes executable targets |
| File hash and file version | Detects replacement/update |
| OS family/version/build/architecture | Captures platform assumptions |
| Target process architecture/bitness | Selects compatible modules |
| Relevant mitigation snapshot | Captures policies affecting integration |
| WLWM/BGW module version | Invalidates stale analysis |
| Compatibility database version | Invalidates changed rules |
| Result and confidence | Supported, Partial, Unsupported, Unknown or Quarantined |
| Feature/module allowlist | Limits exact integration surface |
| Reason codes/evidence | Explains the decision |
| Analysis timestamp/schema metadata | Supports diagnosis and invalidation |

Encoding, location, maximum size, retention and corruption recovery remain open. Cache storage must be local and not require a network lookup. Use atomic writes where the filesystem supports them. Corrupt or unknown records are discarded safely and re-evaluated conservatively.

A cache hit is only an optimization. Every launch rechecks current process identity, runtime context, relevant mitigations and deny/exclusion policy.

### Result semantics

- **Supported:** requirements for a specific module/feature set passed.
- **Partial:** only an explicit subset may be enabled.
- **Unsupported:** a known requirement is absent.
- **Unknown:** evidence is insufficient; integration remains disabled.
- **Quarantined:** a prior failure or user/policy decision blocks automatic retry.

Supported is scoped to a specific module contract, not all WLWM features.

### Injection policy requirements

- Separate process observation from code injection; observation may be available while injection is not.
- DWM, authentication/security-critical processes and protected targets are excluded.
- General WLWM injection into Explorer remains disallowed. A dedicated Explorer presentation adapter may use a narrowly scoped in-process technique only under its own opt-in, per-build compatibility and recovery policy; Explorer executable files must never be modified or replaced.
- Respect target process mitigations and access-control decisions. Never bypass them.
- Do not perform network operations or expensive analysis under loader locks.
- Target continues natively if integration cannot load, initialize, handshake or remain healthy.
- Define unload semantics per target; do not assume every in-process module can always be safely unloaded.
- User exclusions and disable controls take precedence over cached eligibility.

### Window rules and layout contract

A rule declares match conditions, priority, scope, actions, enabled state and compatibility requirements. Candidate match keys include verified process identity, window class, title pattern, monitor, workspace, user tag and window type. Title alone is not a reliable application identity.

Proposed precedence:
1. hard safety/system exclusions;
2. explicit per-window user action;
3. explicit per-application rule;
4. workspace/profile rule;
5. global rule;
6. native/default behaviour.

Within a tier, explicit priority wins; ties require deterministic behaviour. The final precedence requires approval.

Layout calculations consume monitor work area, taskbar/dock exclusions, DPI/scaling, window min/max constraints, owned dialogs, child windows and user overrides. The layout engine proposes geometry; Windows retains native window creation, activation, focus and composition.

User-initiated move/resize/focus must not be immediately undone by a background rule. Define a temporary override or rule-suspension window and a clear way to resume configured layout.

### Workspace contract

Use native virtual desktop membership/move operations only where the documented interface and target OS support the requested action. Do not assume public API coverage for creating, naming or switching every native desktop operation.

Logical workspaces on older systems are presentation state over eligible windows, not secure desktops or independent logon sessions. They must not hide critical dialogs or security surfaces. App launch does not automatically switch workspaces.

### WLWM acceptance gates

- [ ] Observation-only mode works without injection.
- [ ] Cache invalidation covers every identity/policy field above.
- [ ] Unknown, stale and corrupt cache entries fail closed.
- [ ] Every deny/exclusion rule is checked at launch, including cache hits.
- [ ] Target application still starts when integration fails.
- [ ] Focus, z-order and user manual movement remain stable.
- [ ] Shutdown, crash and disable return the session to native behaviour.
- [ ] Any injection pilot has a separate security review and target allowlist.

