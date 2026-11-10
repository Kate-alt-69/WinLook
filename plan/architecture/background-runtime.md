# WinLook Background Runtime and Two-Executable Architecture

> Status: user-directed architecture baseline, 2026-11-10. This is a design/documentation contract, not an implementation claim. The process split and repository layout below are the current project direction; transport details, per-OS adapters and several subsystem policies remain implementation decisions.

## 1. The two principal executables

WinLook is organized around two cooperating application executables:

| Executable | Role | Lifetime |
|---|---|---|
| `winlook.exe` | Main Manager UI: settings, configuration, wallpaper/library browsing, themes, profiles, controls, previews and user-facing diagnostics | Runs when the user opens the UI; may exit independently |
| `winlook.bg.exe` | BGW background runtime/worker: hosts and coordinates WinLook's background functionality and applies it to the Windows session | Starts through the user's Windows startup-app mechanism and remains active when the Manager UI closes |

These are the two principal application executables. Do not split the background runtime into separate required Agent, WLDE runtime, WLWM daemon or per-engine executables merely because those are useful logical modules. A separately isolated helper may be justified later for a specific security, compatibility or crash boundary, but it is not a third mandatory core application.

### `winlook.exe` — Manager UI

The Manager owns the interactive control experience:

- Settings and configuration editing.
- Wallpaper library, previews and Store/catalogue interface.
- Themes, icons, profiles and customization controls.
- Window/display and subsystem configuration.
- Diagnostics and user-visible capability/error reporting.
- Sending validated configuration and commands to BGW over IPC.

The Manager is not required to remain open for configured background functionality to continue. Closing it must not stop wallpaper playback, window policy or other active BGW-managed features.

### `winlook.bg.exe` — BGW runtime

BGW is the actual background application runtime, not merely a WLWM process detector or a thin policy broker. It is responsible for initializing, hosting, coordinating, monitoring and shutting down WinLook's background subsystems and applying their work to the user's Windows session.

Its responsibilities include:

- Background lifecycle and startup/shutdown.
- Wallpaper and desktop runtime through WLDE.
- Window management policy and integrations through WLWM.
- Display monitoring and supported display operations through WLDM.
- WinLook notifications, popups and OSD through WLOSDE.
- Appearance, icon, audio/media and other enabled background integrations.
- Subsystem registration, capability discovery, event routing and health monitoring.
- CPUv/GPUv resource policy coordination for WinLook-managed work.
- Configuration validation/application and recovery after Manager disconnects or crashes.
- Structured diagnostics and controlled subsystem restart/quarantine.

BGW runs with ordinary user privileges by default. Any future privileged Windows operation must be narrowly scoped and separately justified; an optional service is not a replacement for the interactive BGW runtime.

## 2. Runtime topology

The following is a logical view inside the two-executable model. The subsystem boxes are modules hosted/coordinated by BGW, not additional required executables.

```text
                 Windows user startup
                         |
                         v
                  winlook.bg.exe
                   BGW runtime
                         |
       +-----------------+------------------+
       |                 |                  |
      WLDE              WLWM               WLDM
 Wallpaper/Desktop   Window policy     Display management
       |                 |                  |
       +-----------------+------------------+
                         |
             +-----------+-----------+
             |           |           |
          WLOSDE     Appearance    Audio & Media
             |        / Icons
             +-----------+-----------+
                         |
                 BGW event/policy loop
                         |
                  CPUv / GPUv
              WinLook resource control
                         |
                         v
                   Windows APIs


                   winlook.exe
                  Manager UI
                       |
                authenticated IPC
                       |
                  winlook.bg.exe
```

BGW may use internal threads, bounded worker pools and in-process modules. Those are implementation details, not extra user-facing applications. A module should have a clear owner and failure boundary even when it shares the BGW process.

## 3. Startup, configuration and lifecycle

1. Windows starts `winlook.bg.exe` through the configured per-user startup mechanism.
2. BGW establishes its per-session identity, loads the last valid committed configuration and initializes only components whose dependencies and OS capabilities pass checks.
3. Each subsystem registers its identity, dependencies, capabilities, health state and supported safe-response actions.
4. BGW begins observing relevant Windows/session/resource events and applies the current policy.
5. When the user opens `winlook.exe`, the Manager authenticates to the matching BGW instance and queries protocol version, capabilities, configuration revision and runtime state.
6. The Manager sends validated configuration changes or commands. BGW validates and applies them, then returns an explicit acknowledgement/result and the applied revision.
7. Closing or crashing the Manager does not stop BGW. BGW retains the last valid committed configuration and continues background work.
8. If BGW exits unexpectedly, the Windows startup/recovery mechanism may relaunch it. On restart, BGW restores the last valid configuration, re-probes capabilities and safely reinitializes owned resources; it must not blindly replay unsafe transient actions.
9. Shutdown is bounded and ordered: stop accepting new optional work, notify modules, cancel/defer queued work, release owned resources, persist committed state and finish cleanup.

Proposed subsystem lifecycle states: `Uninitialized`, `Initializing`, `Ready`, `Running`, `Suspended`, `Degraded`, `Stopping`, `Stopped`, `Recovering` and `Quarantined`. A module must expose its actual state; a healthy BGW process does not imply every subsystem is healthy.

BGW must preserve a minimal control/recovery path during pressure or emergency suspension. It cannot suspend the very worker, event loop or resource needed to resume itself.

## 4. Manager ↔ BGW IPC

The Manager and BGW communicate through a local IPC channel. The exact transport is selected per OS family and must not assume modern Windows IPC APIs exist on Windows 95.

The protocol should provide:

- Explicit major/minor protocol version and capability negotiation.
- Peer authentication bound to the expected user and session; do not trust a caller-supplied role string.
- Bounded, typed messages with strict payload/string/queue limits.
- Request IDs, correlation IDs, deadlines and cancellation semantics.
- Explicit accepted/applied/rejected/partial/failed results.
- Configuration revisions and transactional acknowledgements.
- Idempotency for retryable state-changing operations.
- Reconnect and BGW restart detection.
- Safe handling of stale UI state and concurrent configuration edits.
- No generic command execution, arbitrary library loading or executable payloads over IPC.

BGW is authoritative for **applied runtime state**. The Manager may hold an editable draft, but it must distinguish draft settings from settings BGW has accepted and applied. A successful UI save is not assumed until BGW acknowledges it.

Transport, authentication and serialization must be evaluated separately for Windows 10/11 and each legacy OS target. If a platform cannot provide a safe equivalent, expose a capability limitation rather than pretending the modern transport is portable.

## 5. Subsystem registration and event-driven policy

Subsystems register with BGW once at initialization. A registration describes:

- Stable subsystem ID and version.
- Required/optional dependencies.
- OS capabilities required and detected.
- Resources/work classes it owns.
- Events it can consume.
- Safe actions it can perform in response to policy.
- Current lifecycle/health state.
- How to suspend, resume, release recoverable resources and restore state.

BGW should not need the user to register every external game, window or process to make general resource policy work. It observes supported system/session signals and evaluates policy centrally. Per-window or per-application rules remain WLWM/user-policy features where relevant, not a prerequisite for global background pressure handling.

Example WLDE response capabilities include pausing wallpaper playback, freezing the last displayed frame, stopping unnecessary decoding, releasing discardable render caches, suspending widget refresh and resuming from saved playback position. Other subsystems expose their own safe actions; BGW does not need to know their internal implementation.

## 6. CPUv and GPUv are core runtime libraries

CPUv and GPUv are core WinLook libraries, not optional per-engine helpers. All WinLook-managed CPU work and GPU allocation/submission paths should pass through the appropriate library/adapters, with explicit, reviewed exceptions.

- Rust: `wl_cpuv`, `wl_gpuv`.
- C: `wl_Lcpuv`, `wl_Lgpuv`.

BGW coordinates the combined policy and event response. CPUv and GPUv enforce the resource envelope for WinLook's own managed work. The shared coordinator can live inside BGW; this does not require a third public coordinator library or a third primary executable.

The libraries can observe OS-provided system pressure and adapt WinLook's own work. They cannot seize resources from, suspend, throttle or terminate unrelated processes, including games. Windows and graphics drivers remain authoritative for system scheduling, memory residency and device operation.

### CPUv responsibilities

- Bounded task queues, work classes, priorities and fairness.
- Admission control, concurrency ceilings and reserved recovery capacity.
- Deferral, cooperative cancellation and deadline handling.
- Worker-pool integration and CPU-time/queue-wait accounting where measurable.
- Power/session/activity-aware scheduling.
- Diagnostics, estimates-versus-measured cost and policy enforcement.

### GPUv responsibilities

- Adapter/node discovery and OS-reported memory-budget observation.
- WinLook allocation and submission admission/backpressure.
- Resource lifetime, fence-aware recycling and discardable-cache release.
- Frame pacing and bounded background submissions.
- Device-loss, adapter-change and residency-pressure recovery.
- GPU timing only where the graphics backend provides valid measurements; otherwise report unknown rather than invent precision.

### Resource response — preserve visual quality

WinLook must **not** use Dynamic Quality Scaling as its resource-pressure strategy. Do not automatically lower wallpaper resolution, reduce source quality, degrade image/video fidelity, or silently simplify the user's visual experience to save resources.

Instead, respond by changing *when and whether WinLook performs work*:

- Defer nonessential tasks and maintenance.
- Pause or suspend optional background activity.
- Freeze the last successfully displayed frame when appropriate.
- Stop decoding frames that do not need to be presented.
- Stop unnecessary widget refresh and animation advancement.
- Release unused or recoverable caches/resources.
- Bound queues and reduce concurrency/submission frequency.
- Resume work safely when pressure subsides.

A subsystem may expose an explicit user-selected lower-resource mode, but BGW must not silently change visual quality under an automatic policy. Emergency recovery and essential control work retain bounded reserved capacity.

### Example: a foreground game or system pressure event

1. BGW detects a supported foreground-game/Game Mode signal or elevated system pressure.
2. BGW evaluates the active policy and tells registered subsystems which response actions are appropriate.
3. CPUv defers/rejects nonessential WinLook CPU work and limits WinLook worker concurrency.
4. GPUv blocks unnecessary new WinLook submissions, manages its own outstanding resources safely and releases discardable allocations.
5. WLDE pauses playback or freezes the last frame, stops unneeded decoding and suspends optional updates without reducing image quality.
6. WLWM, WLDM, WLOSDE and other modules take their registered safe actions.
7. BGW retains its control/recovery loop and monitors for the condition to clear.
8. BGW resumes subsystems in a controlled order and reconciles resource state.

The policy affects WinLook only. It must not inject into a game, inspect game memory or attempt to control the game's scheduler.

## 7. FFmpeg integration

The planned FFmpeg integration is a dedicated WinLook wrapper library, not a fork or replacement decoder:

- Rust candidate: `wl_ffmpeg` under `rust/library/wl_ffmpeg/`.
- C candidate: `wl_Lffmpeg` under `C/library/wl_Lffmpeg/`.

The wrapper should provide a consistent WinLook-facing API for media probing, decoding, playback/seek lifecycle, frame delivery, thumbnail/preview extraction, audio decode where needed and hardware-decoder selection where supported. It should keep upstream FFmpeg libraries behind a narrow adapter and integrate decoder work/resource ownership with CPUv/GPUv.

Performance principles:

- Avoid transcoding merely to play source media.
- Bound decoder threads and frame queues through CPUv.
- Do not decode frames that are not needed for presentation.
- Prefer direct/low-copy delivery to a usable render surface where supported.
- Avoid unnecessary GPU→CPU→GPU transfers.
- Reuse decoder and rendering resources safely.
- Preserve original media; caches/previews are derived artifacts, not replacements.
- Hardware decoding is capability- and workload-dependent; it is not guaranteed to be faster in every path.

FFmpeg packaging, codec configuration and licensing need an explicit review. GPL-enabled components can change redistribution obligations; nonfree-enabled builds may not be redistributable. The Windows 10/11 FFmpeg toolchain does not establish Windows 95 compatibility. Legacy media support remains an explicit unresolved platform decision and must not be claimed until a compatible backend/toolchain is researched and tested.

## 8. Failure boundaries and recovery

- Manager failure: BGW continues with last valid applied configuration.
- Individual subsystem failure: quarantine/restart that subsystem where safe; unrelated subsystems continue.
- BGW failure: Windows can relaunch it; native Windows desktop remains usable while WinLook is unavailable.
- GPU device loss: invalidate device-owned resources, rebuild from logical state or use an explicitly supported fallback; do not corrupt source content.
- IPC disconnect: Manager shows disconnected/stale state and reconnects; BGW continues independently.
- Configuration validation failure: reject the affected transaction and retain the last valid applied revision.
- Resource emergency: suspend optional WinLook work while preserving bounded BGW control/recovery capacity.

No WinLook failure may prevent Windows from launching applications, managing windows natively, displaying the secure desktop or operating its filesystem/Shell.

## 9. Relationship to other architecture documents

- [Runtime and module contracts](runtime-contracts.md) — detailed IPC, capability, ownership, trust and lifecycle contracts.
- [CPUv/GPUv resource governors](resource-governors.md) — detailed resource API, accounting and invariant tests.
- [WLDE](wlde.md), [WLWM](wlwm.md), [WLDM/WLOSDE](wlosde-wldm.md) — subsystem-specific behaviour.
- [Repository layout](repository-layout.md) — authoritative source-tree organization.
- [Compatibility plan](../compatibility/README.md) — OS-specific capability evidence and test requirements.

## 10. Open implementation decisions

- IPC transport/authentication and serialization per OS family.
- Exact startup registration and recovery mechanism per supported Windows release.
- BGW internal thread/module isolation and restart strategy.
- Configuration persistence format, transaction and migration rules.
- Exact system-pressure signals and policy thresholds.
- FFmpeg version/build configuration, codec policy and redistribution model.
- Whether Windows 95 receives FFmpeg-backed playback or a separately researched legacy media backend.
- Exact CPU/GPU platform adapters and test hardware matrix.
