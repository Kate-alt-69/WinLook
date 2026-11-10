# WinLook CPUv and GPUv Resource Governors

> Status: design proposal, 2026-11-09. This is the architecture contract for the planned CPUv/GPUv libraries, not an implementation or a claim of system-wide resource control.
>
> Initial target baseline: Windows 10.

## 1. Goal

Create two shared WinLook libraries:

| Modern library | Legacy library | Responsibility |
|---|---|---|
| `wl_cpuv` | `wl_Lcpuv` | Admission, budgeting, concurrency and scheduling of WinLook-owned CPU work |
| `wl_gpuv` | `wl_Lgpuv` | Admission and accounting for WinLook-owned GPU allocations/submissions, plus memory-pressure response |

The “v” means a **virtual software budget/envelope** inside WinLook. It is not a virtual processor, virtual GPU, hardware partition, driver, hypervisor or guaranteed reservation.

WinLook should route all of its own managed CPU jobs and GPU allocations/submissions through these governors before scheduling or allocating them. A code path that bypasses the broker must be an explicit, reviewed exception. These libraries cannot intercept every API call made by Windows, unrelated applications or arbitrary third-party drivers, and cannot force a game or another process to surrender resources.

## 2. What they help WinLook do

- Prevent background WinLook work from consuming an uncontrolled share of CPU/GPU capacity.
- Keep UI, window-management and restore/recovery tasks responsive.
- Yield WinLook wallpaper, widget, blur, animation and preview work when a game is active.
- Use available idle time for bounded background work without continuously saturating the machine.
- Observe OS-provided GPU memory budgets and respond before WinLook causes avoidable residency pressure.
- Set one policy for several WinLook engines instead of each engine inventing its own thread, memory or frame quotas.
- Record estimates versus actual work so future policies use measurements rather than guesses.
- Provide CPU-only fallback when graphics capability, budget or device health makes GPU work unavailable.

## 3. System boundary and enforcement

The enforceable boundary is **WinLook-owned work that participates in the API**.

| Resource/work | Can WinLook govern it? | Enforcement |
|---|---|---|
| WinLook Rust/C tasks submitted through CPUv | Yes | queue admission, concurrency, priorities, cancellation and bounded workers |
| WinLook GPU allocations/submissions through GPUv-aware adapters | Yes, within estimates and renderer cooperation | memory/work admission, queue backpressure, resource trimming and deferred work |
| Other processes, including a running game | No, not by these libraries | Windows/driver owns their scheduling and memory decisions |
| Arbitrary third-party native code that bypasses the brokers | No | integrate through a documented adapter or report it as unmanaged |
| Windows/DWM/compositor/kernel/driver workloads | No | Windows and the graphics driver remain authoritative |
| Whole-machine memory, disk and network I/O | Not in the CPUv/GPUv contract | separate subsystem budgets would be required |

“Every WinLook resource operation goes through CPUv/GPUv” is a codebase integration rule for managed CPU/GPU work, not a global operating-system interception promise.

CI should audit new direct thread creation, worker-queue submissions, graphics resource creation and command submission sites. An explicit bypass needs a documented reason, owner and test. No engine should implement its own competing global CPU/GPU budget.

## 4. Shared architecture

Each WinLook process/session has one authoritative resource-governor owner per language/runtime. All engine modules request work through it.

Proposed flow:

1. A subsystem describes a work request: owner/module, class, priority, estimated cost, memory estimate, deadline and cancellation behavior.
2. The coordinator checks current activity mode, OS/adapter pressure, configured policy and queue limits.
3. CPUv admits or rejects the CPU portion and returns an opaque ticket/permit.
4. GPUv admits or rejects the GPU allocation/submission portion and returns a GPU ticket where needed.
5. The coordinator commits both permits, or cancels the first if the second cannot be admitted. This prevents half-reserved jobs.
6. The scheduler dispatches admitted CPU work. The renderer allocates/submits admitted GPU work.
7. Completion reports measured CPU cost and, where supported, GPU timestamp/fence evidence. GPU allocation estimates are reconciled with the next OS budget snapshot.
8. Cancellation, timeout, device loss or target disposal releases all outstanding permits and owned resources.
9. Snapshots expose aggregated counters and pressure states to the Manager/BGW and profiles.

The shared coordinator may live inside the BGW runtime; it is not automatically a third public library. Keep CPUv and GPUv independently usable and independently testable.

## 5. CPUv contract

### Responsibilities

- Bounded queues and maximum in-flight work.
- Work classes and priority/fairness policy.
- Admission based on per-window CPU budget and reserved headroom.
- A worker-pool adapter with an explicit maximum concurrency.
- Deadline/expiry handling and cooperative cancellation.
- Measurement of CPU time where the host can supply it.
- Estimates-versus-actual accounting and overrun reporting.
- Activity-mode policy selection without pretending that a mode is an OS scheduler setting.

### Work classes

| Class | Examples | Policy |
|---|---|---|
| Critical | resource cleanup, restoration journal consistency, device-loss recovery | reserved headroom; bounded and auditable |
| Interactive | UI response, window events, visible animation updates | low queue latency and small work units |
| Foreground | visible wallpaper/widget/render preparation | prioritized but budgeted |
| Background | thumbnail generation, cache preparation, indexing owned by WinLook | deferred under pressure |
| Idle | optional maintenance/precomputation | runs only when its policy permits; still capped |

Critical must not become a generic “always bypass quotas” switch. Define a narrow list of critical operations, maximum sizes and abuse diagnostics.

### Scheduler behavior

The preferred modern route is a dedicated/bounded WinLook worker pool or a carefully configured Windows thread pool, with CPUv controlling what is posted and how much concurrency is allowed. CPUv must not create one thread per request, busy-wait, or block the UI thread while waiting for a permit. Queueing is bounded; rejection/defer is a normal result.

Windows thread-pool APIs expose explicit minimum/maximum thread controls, but Windows still owns actual scheduling. See [CreateThreadpool](https://learn.microsoft.com/en-us/windows/win32/api/threadpoolapiset/nf-threadpoolapiset-createthreadpool).

Job Object CPU rate controls are an optional adapter only for a helper process or job WinLook owns and is authorized to manage. They are not the default throttle for the interactive desktop/Manager process. A hard cap pauses all threads associated with the job after it uses its interval allocation; that can cause visible stalls. Weight-based and hard-cap controls are distinct modes. See [JOBOBJECT_CPU_RATE_CONTROL_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_cpu_rate_control_information).

### Measurements

Distinguish:
- wall-clock duration (how long a task took);
- CPU time consumed (user + kernel execution time where measurable);
- queue wait time;
- worker concurrency;
- estimated cost and measured cost.

Do not call elapsed wall time “CPU usage.” A sleeping or I/O-bound task can have high wall time and low CPU time. Define how each OS adapter obtains measurements and report “unknown” when it cannot.

## 6. GPUv contract

### Responsibilities

- Identify adapter, node and memory segment (local/non-local).
- Consume OS-provided process memory budget/usage snapshots.
- Track pending estimated allocation bytes and avoid admitting work beyond configured WinLook headroom.
- Bound per-frame submissions/work units and background GPU work.
- Surface Normal/Constrained/Critical/Unknown pressure.
- Ask renderers to defer optional work, freeze the last frame where appropriate, release recoverable caches or use a supported CPU fallback without silently degrading configured visual quality.
- Handle adapter changes and device removal/reset by invalidating device-owned resources and rebuilding from logical scene data.

### OS authority and DXGI integration

On Windows, an adapter can report `Budget`, `CurrentUsage`, `AvailableForReservation` and `CurrentReservation` through DXGI video-memory information. Query separately by node and local/non-local memory segment. The values are process-oriented and OS-provided; do not substitute physical VRAM capacity or a marketing specification for the assigned budget.

DXGI can signal budget changes through a CPU event so GPUv does not need to poll at a high frequency. `SetVideoMemoryReservation` communicates a minimum memory need to the OS; it is not a hard VRAM partition and does not grant more budget. Reservations are hints used by the OS to help manage a process's working set. Sources: [DXGI memory info](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_4/ns-dxgi1_4-dxgi_query_video_memory_info), [QueryVideoMemoryInfo](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_4/nf-dxgi1_4-idxgiadapter3-queryvideomemoryinfo), [SetVideoMemoryReservation](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_4/nf-dxgi1_4-idxgiadapter3-setvideomemoryreservation), [budget-change notification](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_4/nf-dxgi1_4-idxgiadapter3-registervideomemorybudgetchangenotificationevent), and [D3D12 residency guidance](https://learn.microsoft.com/en-us/windows/win32/direct3d12/residency).

### GPU cost model

Do not claim an arbitrary “work unit” is a real percentage of GPU execution. Separate:
- estimated bytes for a resource admission;
- number/size of queued render or compute submissions;
- actual GPU execution time, only when the backend exposes valid timestamp/fence measurements;
- CPU driver-submission time;
- cache/residency/eviction observations.

Where execution-time measurement is unavailable, GPUv reports cost as unknown and uses conservative queue/memory limits. It must not fabricate precision.

GPUv does not replace D3D/DXGI/Direct2D allocation rules and cannot guarantee that a driver will keep resources resident. It asks renderers to reduce WinLook's own footprint and pressure, and treats every API allocation/device-loss error as authoritative.

## 7. Gaming, idle and interactive modes

Modes are policy presets and signals, not hardware switches.

| Mode | CPUv | GPUv |
|---|---|---|
| Interactive | favor UI/window events and visible state updates; defer maintenance | preserve visible composition; cap background uploads/effects |
| Gaming | sharply reduce or pause nonessential WinLook background tasks; keep only small necessary integration work | reduce WinLook wallpaper/widget/effect/animation work and release optional caches; never interfere with the game process |
| Idle | allow bounded cache/preparation tasks in low-priority lanes | allow bounded pre-rasterization/uploads while maintaining headroom |
| Power saver | smaller worker pool and stricter task admission | defer optional submissions and release recoverable caches while preserving configured visual quality |
| Critical pressure | reject/defer nonessential work and run cleanup/recovery only | release optional resources, freeze the last frame where appropriate and suspend optional submissions; do not silently degrade visual quality |

Do not assume “idle” means consume 100% of unused resources. Use measured load, user settings, deadlines and policy ceilings. Foreground-game detection is supplied by a WinLook policy adapter; CPUv/GPUv should not scan arbitrary game memory or inject into games to infer state.

## 7A. Quality-preserving pressure response

**Automatic Dynamic Quality Scaling is not part of WinLook's resource-governance strategy.** CPUv/GPUv and BGW must not silently lower wallpaper resolution, reduce image/video source quality, degrade visual fidelity or simplify the user's scene as an automatic response to resource pressure.

Prefer scheduling and lifecycle controls: defer optional work, pause playback, freeze the last successfully displayed frame, stop decoding frames that will not be presented, suspend unnecessary widget/animation updates, bound queues/concurrency/submissions, and release unused or recoverable resources. Resume safely when pressure subsides. A user may explicitly choose a lower-resource visual preset, but automatic BGW policy must preserve the configured visual quality.

The policy governs WinLook-owned work only. It cannot suspend, throttle or reclaim resources from unrelated processes. BGW must preserve a minimal control/recovery path during critical pressure.

## 8. C and Rust parity

Proposed source mappings, subject to the approved repository library migration:

| Rust library | C library | Intended home |
|---|---|---|
| `wl_animate` | `wl_Lanimate` | `rust/library/wl_animate`, `C/library/wl_Lanimate` |
| `wl_cpuv` | `wl_Lcpuv` | `rust/library/wl_cpuv`, `C/library/wl_Lcpuv` |
| `wl_gpuv` | `wl_Lgpuv` | `rust/library/wl_gpuv`, `C/library/wl_Lgpuv` |

Rust and C must share concepts, units, result states and reference test vectors, not internal structure layouts. The C API uses an explicit ABI/version, fixed-width integer types where the chosen compiler provides them, caller-visible ownership, capacity limits, stale-ticket detection and well-defined synchronization.

Because C89 has no standard threading primitive, the library must either receive host-supplied lock/unlock operations or require and verify serialized access. It must not imply thread safety without an implementation. No lock may be held while invoking caller callbacks or executing work.

If different language processes share a single resource-governor authority, they must request through BGW's versioned IPC contract. Separate in-process governors must not pretend to share one global budget.

## 9. Public API shape (proposal)

### CPUv

- policy creation/validation and activity mode;
- bounded scheduler/queue configuration;
- request/acquire with module, class, priority, estimate and deadline;
- permit/ticket lifecycle: queued, started, completed, cancelled, expired, rejected;
- cooperative cancellation token;
- CPU/wall-time completion report;
- snapshots, pressure and diagnostic counters;
- policy update and shutdown/drain.

### GPUv

- adapter/node/segment identity and OS budget snapshot;
- memory/work request with owner, class, estimated bytes and queue cost;
- permit/ticket lifecycle tied to renderer resource creation/submission and completion fence;
- begin-frame/accounting boundary;
- cancellation/refund prior to submission;
- dynamic budget update and pressure state;
- release/trim/degrade recommendations;
- device lost, adapter changed and unsupported capability results.

Both APIs need explicit errors such as InvalidRequest, NoBudget, BudgetExceeded, QueueFull, ConcurrencyLimit, DeadlineExpired, Cancelled, Unknown/StaleTicket, DeviceLost, Unsupported and ShuttingDown. Return status plus a stable reason code; do not silently drop accepted work.

## 10. API stress and invariant tests

| Test | Expected invariant |
|---|---|
| Acquire exactly to budget | accepted; next over-budget request rejected |
| Critical reserve | ordinary work cannot consume reserved headroom |
| Complete with actual cost above estimate | overrun reported; later admissions reflect actual use |
| Cancel before dispatch | all reservations and queue slots refunded once |
| Duplicate/stale ticket | rejected; accounting never changes twice |
| Begin accounting window with outstanding jobs | spent work resets per documented window; live reservations remain |
| Budget shrinks during active work | no new over-budget admission; pressure escalates; in-flight work is not corrupted |
| Concurrency storm | active count never exceeds configured limit |
| Ticket ID wrap/reuse | stale handle cannot complete a new job |
| Queue full/deadline expired | explicit rejection/expiry; no hidden unbounded queue |
| Counter overflow/invalid values | saturating or rejected according to contract; never wraps into extra capacity |
| GPU local vs non-local budget | accounting is separate for each adapter/node/segment |
| GPU device lost | old permits/resources invalidated; logical work can be rebuilt or CPU-fallback |
| Multi-thread requests | no data races, double refunds or lock-held callback deadlocks |
| Fairness under load | interactive/critical work has defined bounded latency; background work cannot starve forever unless policy intentionally pauses it |
| Broker bypass audit | all WinLook-owned managed task/graphics sites use the broker or have a reviewed exception |

Use deterministic fake clocks/budget providers, property-based state-machine tests, concurrent stress tests, C/Rust differential contract vectors and hardware-in-the-loop Windows 10 runs. Record P50/P95 queue wait, admission/rejection rate, accounting error, lock contention and total governor overhead.

## 11. Rollout and beta gates

1. Freeze units, ticket lifecycle, thread-safety and ownership semantics before expanding implementation.
2. Build CPUv/GPUv pure policy cores and C/Rust contract tests.
3. Add Windows adapters: bounded thread-pool integration, CPU-time telemetry, DXGI budget snapshots/events and device-loss reporting.
4. Integrate one subsystem first (animation/vector rendering), then WLDE wallpaper/widgets, then WLOSDE and Manager previews.
5. Add CI auditing for broker bypasses and resource-lifetime leaks.
6. Run gaming/idle/interactive scenarios on Windows 10 VMs and physical systems with integrated and discrete GPUs.
7. Publish only after stress tests demonstrate bounded queues, correct accounting, safe pressure response and no regressions to foreground responsiveness.

## 12. Non-goals

- No virtual CPU instructions or GPU machine-code rewriting.
- No system-wide scheduler replacement, hypervisor, virtual GPU partition or global memory reservation.
- No throttling of unrelated apps or games by default.
- No driver/kernel hooks, undocumented GPU-control interface or process injection to obtain hardware telemetry.
- No promise that estimated GPU work cost is equal to real GPU execution cost.
- No general CPU/GPU management of arbitrary third-party workloads.
