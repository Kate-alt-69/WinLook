# WLWM — WinLook Window Manager (WELMO)

## Purpose
WLWM brings selected Linux desktop/window-manager capabilities to Windows while retaining the Windows windowing system and DWM. Hyprland is a design reference for layouts, window rules, workspaces, animation and visual effects; WLWM is not a port of its Wayland compositor, binaries or config language.

References: [Hyprland](https://hypr.land/), [window rules](https://wiki.hypr.land/Configuring/Basics/Window-Rules/), [workspace rules](https://wiki.hypr.land/Configuring/Workspace-Rules/).

## Coexistence
- DWM remains intact.
- Windows continues to create/activate applications and owns native input/focus/z-order fundamentals.
- WLWM adds policy and optional visual integrations over eligible windows.
- If WLWM stops or a target is incompatible, native Windows behaviour remains the fallback.
- Do not patch/replace Explorer.exe on disk. Explorer-targeted injection is excluded by default and needs separate review.
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
