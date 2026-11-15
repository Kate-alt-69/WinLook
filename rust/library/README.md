# WinLook shared library catalog

This page is the entry point for reusing WinLook's shared libraries. Each crate or
C library should keep its own README as the detailed contract; this catalog
shows how the pieces fit together and where to start.

The WLDE-specific implementation in this branch is split deliberately into a
portable policy/model core, a Windows adapter, an out-of-process BGW runtime,
and a matching portable C89 core. These layers can be reused independently;
a consumer should depend only on the layer it actually needs.

## WLDE libraries

| Library | Language / target | Responsibility | Start here |
| --- | --- | --- | --- |
| `wl_wlde` | Rust, portable | Scene policy, monitor geometry, wallpaper placement, resource/Game Mode policy, virtual Groups/layout, adaptive icon composition, provider metadata and the typed write-ahead restoration record codec/state machine | [Rust core README](wl_wlde/README.md) |
| `wl_wlde_win32` | Rust, Windows adapter | Monitor snapshots, read-only User/Public Desktop inventory and change watcher, Explorer desktop-host identity/monitoring, owned Shell icon extraction and bounded RGBA rasterization | [Win32 adapter README](wl_wlde_win32/README.md) |
| `wl_bgw_runtime` | Rust, Windows runtime | BGW host loop, validated local Groups/layout store and private memory-only Shell icon worker/cache | [BGW runtime README](wl_bgw_runtime/README.md) |
| `wl_Lwlde` | C89, portable | Caller-owned fixed-storage companion to the Rust WLDE policy/model contracts | [C89 core README](../../C/library/wl_Lwlde/README.md) |

## Dependency map

```text
wl_animate ─┐
wl_cpuv ────┼──> wl_wlde ───> wl_wlde_win32 ───> wl_bgw_runtime
wl_gpuv ────┘

wl_Lanimate ─┐
wl_Lcpuv ────┼──> wl_Lwlde (C89 static library)
wl_Lgpuv ────┘
```

- `wl_wlde` has no operating-system dependency and forbids unsafe Rust. It
  does not create windows, worker threads, timers, Shell objects or decoders.
- `wl_wlde_win32` contains the platform calls. It returns owned snapshots and
  handles; it does not install Shell hooks, replace icon registrations, hide
  native desktop items or own a host message loop.
- `wl_bgw_runtime` consumes the portable model and Win32 adapter from the
  separate BGW process. Its internal icon worker is private implementation,
  not a public worker API.
- `wl_Lwlde` mirrors the portable contracts in strict C90 and uses caller-owned
  storage in its data and rendering paths.

## Existing shared building blocks

These workspace libraries are dependencies of WLDE or other WinLook modules;
they are not replacements for the WLDE-specific layers above.

| Library | Role | Documentation |
| --- | --- | --- |
| `wl_animate` | Shared frame clock and animation primitives | [README](wl_animate/README.md) |
| `wl_cpuv` | CPU work admission/accounting governor | [README](wl_cpuv/README.md) |
| `wl_gpuv` | GPU work admission/accounting governor | [README](wl_gpuv/README.md) |
| `wl_ui` | Shared UI facade and backend integration | [README](wl_ui/README.md), [UI docs](../../docs/wl_ui/README.md) |
| `wl_ui` core and platform backend packages | Shared UI core plus Win32 / Windows 7 / Windows 10 / Windows 11 implementations | [UI architecture](../../docs/wl_ui/backends.md) |

The C companion libraries are under `C/library/`. Their own README files are
the function-level reference:

| Library | Role | Documentation |
| --- | --- | --- |
| `wl_Lanimate` | C89 shared frame clock and animation primitives | [README](../../C/library/wl_Lanimate/README.md) |
| `wl_Lcpuv` | C89 CPU work admission/accounting governor | [README](../../C/library/wl_Lcpuv/README.md) |
| `wl_Lgpuv` | C89 GPU work admission/accounting governor | [README](../../C/library/wl_Lgpuv/README.md) |
| `wl_Lwlde` | C89 portable WLDE scene and desktop policy core | [README](../../C/library/wl_Lwlde/README.md) |
| `wl_Lui` | C89 shared UI library | [README](../../C/library/wl_Lui/README.md) |

The root CMake aggregator is `C/library/CMakeLists.txt`; each library also
has its own CMake target.

## Reuse from another Rust crate

The repository currently defines these packages as workspace/path dependencies.
Do not assume that an alpha version string means the package has been published
to crates.io. For a local checkout, add only the layer you need:

```toml
[dependencies]
# Portable scene, geometry and policy API.
wl_wlde = { path = "../WinLook/rust/library/wl_wlde" }

# Optional modern Windows adapter.
wl_wlde_win32 = { path = "../WinLook/rust/library/wl_wlde_win32" }

# Optional BGW runtime and local configuration store.
wl_bgw_runtime = { path = "../WinLook/rust/library/wl_bgw_runtime" }
```

Paths are examples and must be adjusted relative to the consuming project's
manifest. The current package metadata declares Rust 1.77 and version
`0.1.0-alpha.1` for these Rust crates; these are pre-1.0 APIs and can change.
Check each crate's manifest before pinning a toolchain or copying a dependency
list. The Win32 adapter's `sha2 = 0.10.9` dependency is Windows-target-only.

### Portable core

Use `wl_wlde` when the application needs the shared scene model or policy but
owns its own platform discovery and presenter.

Its public surface is grouped around:

- Geometry and monitor topology: `Rect`, `Size`, `Monitor`,
  `MonitorTopology`, placement calculations and scale modes.
- Scheduling and visibility: `FrameClock`, `FrameTick`, scene state,
  visibility and Normal/Optimize/Suspend policy.
- Resource pressure and Game Mode: `ResourceGovernor`, `ResourcePolicy`,
  caller-fed `ResourceSample`, `GameModePolicy`, profiles and resolver.
- Desktop model: `DesktopItemRef`, `DesktopGroups`, `DesktopLayout`,
  validated transactional group/layout changes and typed context actions.
- Restoration record core: `DesktopRestoreOperation`, `DesktopRestoreLedger`,
  versioned record encoding/decoding, evidence-checked state transitions and
  pure recovery planning (no file I/O or native adapter execution).
- Icon composition: `AdaptiveAppIcon`, `AppIconLayers`, masks and the
  bounded `render_adaptive_app_icon*_into` / monochrome compositor functions.
- Store and Wallpaper Engine hand-off metadata: provider directory records,
  local installation discovery and target validation.

No telemetry is collected by the core. CPUv/GPUv values are admission and
accounting estimates, not OS usage measurements. Icon rendering accepts
already-decoded RGBA; image decoding and Shell asset extraction belong to
adapters.

Minimal topology construction:

```rust
use wl_wlde::{Monitor, MonitorTopology, Rect};

fn make_topology() -> Result<MonitorTopology, Box<dyn std::error::Error>> {
    let bounds = Rect::new(0.0, 0.0, 1920.0, 1080.0)?;
    let work_area = Rect::new(0.0, 0.0, 1920.0, 1040.0)?;
    let primary = Monitor::new("DISPLAY1", bounds, work_area, true)?;
    Ok(MonitorTopology::new(vec![primary])?)
}
```

### Win32 adapter

Use `wl_wlde_win32` only in the Windows process that is allowed to query the
interactive desktop. A monitor snapshot is finite and owned by the caller:

```rust
use std::error::Error;
use wl_wlde_win32::{enumerate_monitors, refresh_topology, should_refresh_topology};

fn on_display_message(message: u32, w_param: usize) -> Result<(), Box<dyn Error>> {
    let mut topology = enumerate_monitors()?;
    if should_refresh_topology(message, w_param) {
        refresh_topology(&mut topology)?;
    }
    Ok(())
}
```

The host message loop should call `should_refresh_topology(message, w_param)`
for display/device notifications. The filesystem inventory is deliberately
one-level and read-only. Its `filesystem_path()` is ephemeral and must not be
persisted or sent as a durable identity. Filesystem-backed IDs and path
fallback IDs have different rename guarantees; preserve the identity-kind
field when building a local Group store.

Shell icon calls can involve third-party handlers. Run
`extract_desktop_shell_icon()` and `rasterize_desktop_shell_icon_rgba()`
on a dedicated COM STA worker with a message pump, not on a UI or Explorer
callback thread. The current Shell path API rejects paths beyond its documented
MAX_PATH argument bound. RGBA output is limited to 512×512 pixels.

### BGW runtime and local state store

Use `wl_bgw_runtime::run()` only as the Windows BGW process entry point. It
monitors the Explorer desktop host, keeps a read-only inventory for that host,
loads validated local configuration and queues Shell artwork work. On
non-Windows targets, `run()` returns `UnsupportedPlatform`.

The reusable public state API consists of:

- `DesktopItemIdentityRecord` — path-free item reference plus identity kind.
- `DesktopStateStore` — per-user store or caller-selected path.
- `DesktopStateSnapshot` — validated Groups and layout after load.
- `DesktopStateStoreError` — typed persistence/validation errors.

The store is single-writer. Its default Windows file is under
`%LOCALAPPDATA%/WinLook/WLDE/desktop-state.v1.bin`; its versioned format is
bounded to 4 MiB and uses CRC32 to detect accidental corruption. CRC32 is not
authentication or tamper protection. The file stores opaque IDs, identity
kinds, Group labels/members and normalized layout only—not filesystem paths,
file contents, executable paths or icon pixels. The `at_path()` constructor
is provided for tests/controlled embedding; the caller must choose a
user-private directory and serialize writers.

Example load:

```rust
use std::error::Error;
use wl_bgw_runtime::{DesktopItemIdentityRecord, DesktopStateStore};
use wl_wlde_win32::{enumerate_desktop_items, enumerate_monitors};

fn load_state() -> Result<(), Box<dyn Error>> {
    let inventory = enumerate_desktop_items()?;
    let topology = enumerate_monitors()?;
    let identities = DesktopItemIdentityRecord::from_inventory(&inventory);
    let store = DesktopStateStore::for_current_user()?;
    let _restored = store.load(&identities, &topology)?;
    Ok(())
}
```

The BGW icon pipeline is intentionally not a public library interface yet.
Its worker, queue and LRU cache are private; reuse should go through the
public Win32 extraction/RGBA APIs or the BGW host rather than coupling another
application to internal module details.

## Reuse from C89

Add the C library aggregator and link the exported static target:

```cmake
add_subdirectory(path/to/WinLook/C/library)
target_link_libraries(your_target PRIVATE WinLook::Lwlde)
```

Then include `wl_Lwlde.h`. CMake propagates the public include directory and
the dependencies on `wl_Lanimate`, `wl_Lcpuv` and `wl_Lgpuv`. The library
builds as required C90 with extensions disabled. Caller-provided arrays and
output buffers remain owned by the application; do not free or retain them
beyond their documented lifetime. See the [C89 README](../../C/library/wl_Lwlde/README.md)
for function-level details.

## Build and verification

From the repository root:

- `python build.py verify libraries --sanitizers` — shared Rust/C library gate,
  including sanitizer verification for the C libraries.
- `python build.py verify c --sanitizers` — C-only checks.
- `python build.py build bgw` — BGW executable package build.
- `cargo test -p wl_wlde` — portable WLDE unit tests.
- `cargo test -p wl_wlde_win32` — adapter tests; Windows-only API smoke coverage
  requires a Windows runner.
- `cargo test -p wl_bgw_runtime` — BGW store and runtime unit tests; Windows
  Shell/STA behaviour requires Windows CI/device validation.

Passing portable tests does not prove live Explorer presentation or restoration.
This branch does not yet ship native icon suppression, a live Group presenter,
Shell registration changes, the crash-safe native restoration journal,
wallpaper rendering or Manager-to-BGW IPC.

## Library documentation standard

For every new reusable library added to WinLook, keep a README next to its
manifest/header and add it to this catalog. Document the purpose, supported
platforms and compiler/runtime minimums, dependency graph, public API map,
small usage example, ownership/lifetime rules, threading model, bounds,
non-goals, and exact verification commands. When a public API changes, update
both the crate README and this catalog in the same change.


The portable `wl_wlde` core also contains a bounded restoration journal
**codec/state-machine model**. It is not a storage implementation: only the
future BGW writer may append/flush records, after validating its local journal
ACL and adapter allowlist. See the crate README and
[restoration architecture](../../plan/architecture/restoration.md).
