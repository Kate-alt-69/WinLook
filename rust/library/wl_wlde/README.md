# wl_wlde

**WLDE (pronounced WALDE)** is WinLook's integrated desktop engine. This
portable Rust crate is the first implementation layer: it owns scene policy,
monitor topology, wallpaper placement math and the shared frame schedule.

## Implemented in this core

- A bounded 16-monitor topology with current-snapshot adapter IDs, primary
  monitor validation, separate monitor bounds/work areas and transactional
  topology replacement. GDI device names are not durable hardware/profile IDs.
- Fill, Fit, Stretch and Center placement calculations. Fill returns a
  normalized source crop; Fit returns a centered destination rectangle.
- Conservative visibility handling: Unknown and partially contributing
  scenes stay eligible; only known full occlusion suppresses animation.
- Normal (60 FPS), Optimize/Power Saver (30 FPS) and Suspend policies using
  one host-driven wl_animate::FrameClock for the scene. No timer or worker
  is created; paused time is discarded on resume and frame deltas remain
  capped at 100 ms.
- Direct ownership of the existing CPUv and GPUv admission governors through
  ResourceGovernors. They account only for caller-estimated WinLook work;
  they do not poll system use, schedule threads or control other processes.
- An allocation-free ResourceGovernor that consumes caller-provided CPU, GPU,
  RAM and VRAM utilization samples independently, applies configurable
  consecutive-window hysteresis, and feeds Normal/Optimize/Suspend pressure
  into the Scene frame policy. It does not collect OS telemetry.
- A bounded Game Mode resolver keyed by host-supplied executable SHA-256
  fingerprints. Explicit per-application profiles override optional OS
  Game Mode and fullscreen fallbacks. The resolver allocates nothing and
  does not enumerate processes, inspect files or hash executables itself.
- Explicit presentation capability checks. Virtual Groups require both a
  presenter capability and a crash-safe restoration capability.
- A bounded desktop model for file, folder and application references;
  virtual Groups validate membership and can be replaced transactionally.
- Adaptive application-icon metadata plus a bounded RGBA compositor. It combines
  caller-decoded foreground/background artwork, scales the foreground into its
  safe zone, applies Circle/RoundedSquare/Squircle masks and returns a cacheable
  16–512 px RGBA bitmap. Source files are never edited.
- Type-aware, non-destructive context-menu action contracts for applications,
  files, folders and Groups, including app-only icon customization.

## Deliberately not claimed

The adaptive-icon compositor can render an icon bitmap from RGBA layers supplied
by a platform decoder, but it does not extract/decode Shell icon assets or create
a desktop surface. This crate does not enumerate live Shell items, persist the
desktop inventory, hide native icon representations or render Group popups.
The Group and context-menu APIs remain portable data/validation contracts;
Groups remain unavailable until a platform adapter proves crash-safe restoration.
The crate also does not decode wallpaper images/video, parse .wl.wp/.wl.i.wp
packages, execute widgets, host Rainmeter plugins or run an in-app catalogue
search/download client. It does expose the integrated Store provider
directory and a safe hand-off adapter for an already-installed Wallpaper
Engine. Interactive content is unavailable unless a future restricted host
reports that capability. Groups remain disabled until the OS adapter proves
crash/disable/uninstall restoration.

## Game Mode profile resolver

`resolve_game_mode(signals, policy, profiles)` accepts a host-supplied
foreground executable SHA-256 fingerprint, foreground/fullscreen state and an
optional operating-system Game Mode signal. It checks at most 64 unique
profiles without allocating. Exact user-selected profiles win first (including
an explicit Normal override), then the enabled OS Game Mode fallback, then the
enabled fullscreen fallback. Non-foreground applications always resolve to
Normal.

The host computes and supplies the executable digest; the portable library
does not scan processes, read executable files, hash binaries, install hooks
or poll the system. The resulting `GameMode` can be applied through
`Scene::set_game_mode`. OS-specific foreground/fullscreen detection,
profile persistence and Manager editing controls remain platform work.
The matching C89 resolver uses the same fingerprint/precedence contract.

## Resource pressure policy

ResourceGovernor accepts a ResourceSample with separate CPU, GPU, RAM and VRAM
percentages. It uses the maximum domain pressure rather than an average, so a
saturated GPU or memory domain cannot be hidden by idle CPU. ResourcePolicy
requires explicit, ordered thresholds and non-zero sample window counts. One
noisy sample cannot change playback state.

Feed the resulting ResourcePressure to Scene::set_resource_pressure:
Optimize caps the scene at 30 FPS and Suspend stops animation requests while
retaining the last presented frame. The controller allocates no memory and
creates no sampler or thread. A platform adapter still needs to provide
trustworthy system telemetry, define sampling cadence, and separately account
for WLDE-owned resource use. Percentages are not inferred from CPUv/GPUv
admission tickets.

The matching C89 API exposes wl_Lwlde_resource_controller with the same
threshold/window semantics and an explicit scene pressure setter.


## Desktop groups, adaptive app icons and menus

`DesktopGroups` stores virtual groups over opaque Shell item IDs. A group
can contain application shortcuts, files and folders; it never creates a
filesystem directory or moves/copies/renames the source item. Replacements
validate all group IDs and item references before changing the active model.
The Rust model also exposes transactional `add_group`, `rename_group`,
`add_members`, `remove_members` and `ungroup` operations. Each operation
validates the candidate set before commit; removing the last member removes the
empty Group record without touching the Shell item. The context-action contract
includes Create Group and Desktop Settings on an empty desktop background.

`AdaptiveAppIcon` describes app-tile artwork as foreground and background
asset references, an optional monochrome layer, a Circle/RoundedSquare/Squircle
mask and a 48–72% safe-zone percentage (61% default). `render_adaptive_app_icon()`
combines caller-decoded RGBA layers, center-crops the background, contains the
foreground inside the safe zone, composites in premultiplied-alpha space and
applies a 2×2 antialiased shape mask. Output is bounded to 16–512 px and should
be cached by the adapter rather than rendered on every desktop frame.
`render_adaptive_app_icon_into()` writes the same bitmap into caller-owned
reusable storage without allocating, leaving any extra trailing capacity
untouched. BGW should use this version for repeated rendering/cache refreshes.
Native assets are referenced, not rewritten; WinLook does not change
executable, shortcut or original icon resources.

`desktop_context_actions()` returns type-aware actions for Apps, Files,
Folders and Groups. Apps include Customize Icon; grouped members include
Remove from Group; Group tiles include Rename and Ungroup. Native Open and
Properties execution remains the Shell adapter's responsibility.

The compositor is a portable bitmap operation, not a live desktop presenter.
The Explorer integration is a required BGW/WLDE runtime subsystem on Windows
sessions where Explorer owns the desktop. The platform adapter must identify
the correct desktop host, bridge Shell events to BGW, and restore original icon
presentation after failure. Documented Shell interfaces are used where
sufficient; a version-specific runtime hook adapter is used where desktop
behaviour is not exposed through supported interfaces. This is not a
per-feature user opt-in. Unsupported builds must fail safely and preserve
native desktop behaviour. See the
[WLDE architecture](../../../plan/architecture/wlde.md) and
[Explorer customization](../../../plan/architecture/explorer-customization.md).

## Win32 monitor adapter

The separate `wl_wlde_win32` package implements modern Win32 monitor snapshot
collection without adding unsafe code or OS calls to this portable crate.
It filters to desktop-attached, non-mirroring display devices, then reads GDI
monitor/work-area rectangles and primary flags. The shared Win32 backend
re-exports the API for both `wl_ui::windows10` and `wl_ui::windows11`.

Call `enumerate_monitors()` for a new snapshot, or
`refresh_topology(&mut current)` to replace one transactionally. The adapter
does not poll or own a message loop; hosts use
`should_refresh_topology(message, w_param)` to decide when to refresh after
display/device notifications. DPI, orientation and persistent monitor identity
remain separate follow-up work. See
[wl_wlde_win32](../wl_wlde_win32/README.md).

## Wallpaper Store provider bridge

The first provider hub is available through the public Rust API:

- Wallpaper Engine: opens the official Steam Workshop and can hand a local
  project.json or supported media file to an existing
  Wallpaper Engine installation.
- Free/open catalogues: Openverse, Wikimedia Commons and NASA Image and Video
  Library. The Manager opens each provider's official catalogue in the
  default browser; an item's actual license and attribution still must be
  checked individually.
- Provider metadata includes stable IDs, provider description, browse URL,
  access class, free/mixed price model and a rights note.
- WallpaperEngineInstall::discover() checks common Steam locations and
  libraryfolders.vdf without recursively scanning disks. Use from_executable()
  when Steam is installed elsewhere.
- The file picker starts in the detected Steam Workshop directory
  steamapps/workshop/content/431960 when that directory exists. Steam keeps
  ownership of subscription/download/update behaviour.
- open_wallpaper() uses the documented -control openWallpaper -file command
  with separate process arguments. Wallpaper Engine must already be running.
  WLDE does not unpack, copy, move or delete Workshop files, initiate
  subscriptions, or redistribute creator content.

The matching C89 wl_Lwlde API exposes the same immutable provider metadata
through wl_Lwlde_store_provider_get(). Process discovery, file pickers and
process launch remain platform-adapter responsibilities, not portable C
hot-path behaviour.

This is a provider hub, not yet a full in-app Store catalogue. Searching,
previews, item-detail pages, attribution sidecars, direct downloads,
resumable transfers, package signing/moderation and local library installation
remain future gates.

The Rust core is intended for the modern Win10+ host. The matching
C/library/wl_Lwlde contract is C89 and keeps caller-owned fixed storage.
C89 syntax alone is not a statement of Windows 95/legacy OS support.

## Usage

Use the same scene tick for every animated wallpaper/widget property. Do not
create a timer per item. CPU/GPU admission is for bounded decode/upload work,
not a charge on every interpolated property.

## Next platform gates

1. Wire Win10/11 host event loops to the shared monitor adapter and test
   display/device notifications, DPI changes, docking and remote sessions.
2. Implement and test a documented static-wallpaper adapter with original
   wallpaper capture and restoration.
3. Decide and test a supported animated desktop-surface integration that keeps
   Explorer and DWM intact; do not assume an undocumented desktop parent.
4. Add bounded media decode and GPU presentation with device-loss recovery.
5. Add isolated interactive/widget hosting and catalogue/package verification.
6. Keep Groups disabled until crash-safe native icon restoration is proven.


### Low-allocation rendering

`render_adaptive_app_icon_into()` is the allocation-free Rust entry point.
It checks the output capacity before writing, renders only the requested
square bitmap, and leaves any bytes after that bitmap unchanged. The existing
`render_adaptive_app_icon()` convenience helper allocates one output vector
and delegates to the same implementation. The matching C89 function always
uses caller-owned output storage. Neither renderer performs image decoding,
file I/O, Shell calls or source-asset modification.


### Themed monochrome mode

The optional monochrome source can now be rendered with
`render_adaptive_app_icon_monochrome()` or the allocation-free
`render_adaptive_app_icon_monochrome_into()`. The monochrome source alpha is
the glyph mask; its RGB values are ignored and replaced by a caller-supplied
theme tint. Background cover, safe-zone contain and the selected Circle,
RoundedSquare or Squircle mask are identical to color mode. The source asset
is never rewritten, and the caller supplies the theme tint at render time.


## Normalized desktop layout model

The desktop model now includes a transactional top-level placement map:

- Each ungrouped inventory item and each Group has exactly one top-level entry.
- Group members are presented within their Group and cannot also have a separate desktop-icon position.
- Each entry identifies a current monitor snapshot ID, normalized x/y anchors in
  basis points (0–10,000) relative to that monitor's work area, and a stable
  z-order hint. A native presenter resolves those anchors against current
  monitor dimensions and its selected icon/grid metrics.
- Layout replacement validates the full current inventory, Group set and
  monitor topology before committing. Unknown/stale items, unknown monitors,
  duplicate targets, incomplete layouts and out-of-range coordinates are
  rejected without losing the last valid layout.
- The layout is capped at 4,160 entries (4,096 desktop inventory items plus
  64 Groups). It stores opaque item IDs and Group IDs, not filesystem paths,
  file contents, absolute pixels or Shell pointers.

This remains a portable data contract. It does not move or rename source files,
draw/hide desktop icons, render Group tiles or register a Shell view. Those
presentation operations stay behind the required BGW Explorer Desktop Bridge
and the crash-safe native restoration gate.


## Reuse as a Rust library

Package: `wl_wlde` `0.1.0-alpha.1` (Rust 2021; declared MSRV 1.77).
The crate is portable and forbids unsafe Rust. It depends on the shared
`wl_animate`, `wl_cpuv` and `wl_gpuv` workspace libraries.

Add a path dependency while developing against this repository:

```toml
[dependencies]
wl_wlde = { path = "../WinLook/rust/library/wl_wlde" }
```

The main API families are geometry/topology (`Rect`, `Size`, `Monitor`,
`MonitorTopology`), scene timing and visibility (`FrameClock`,
`FrameTick`), caller-fed resource pressure (`ResourceGovernor`,
`ResourcePolicy`, `ResourceSample`), Game Mode resolution
(`resolve_game_mode` and its policy/profile/signal types), virtual desktop
Groups and normalized layouts, adaptive icon metadata/composition, and
immutable Store provider / Wallpaper Engine hand-off metadata. The public
re-exports are declared in `src/lib.rs`; start there when selecting a smaller
API subset.

Minimal topology construction:

```rust
use wl_wlde::{Monitor, MonitorTopology, Rect};

fn topology() -> Result<MonitorTopology, Box<dyn std::error::Error>> {
    let bounds = Rect::new(0.0, 0.0, 1920.0, 1080.0)?;
    let work_area = Rect::new(0.0, 0.0, 1920.0, 1040.0)?;
    let primary = Monitor::new("DISPLAY1", bounds, work_area, true)?;
    Ok(MonitorTopology::new(vec![primary])?)
}
```

The core owns policy and validation, not platform side effects. It does not
sample OS telemetry, enumerate processes, hash executables, decode image files,
call the Shell, create threads/timers/windows or render a desktop surface.
Callers supply the relevant measurements, identities, capabilities and
decoded pixel views. Use the matching `wl_Lwlde` C89 library when the target
needs fixed caller-owned storage and strict C90 compatibility.

Run `cargo test -p wl_wlde` or the repository-wide
`python build.py verify libraries --sanitizers` gate. The API is pre-1.0;
treat `0.1.0-alpha.1` as an evolving workspace contract, not a stable
external semver commitment.

For a cross-library dependency map, Win32/BGW examples and the C89 build
recipe, see the [shared library catalog](../README.md).


## Portable restoration journal record core

The portable core now provides a **record codec and state-machine validator** for
the future Explorer Desktop Bridge write-ahead journal. It is reusable by any
platform adapter that needs the same conservative transaction model:

- `DesktopRestoreLedger::begin()` validates and records the immutable original
  value, desired value, adapter key/version, resource key and ownership proof.
- `transition()` appends only legal phases: Prepared → Applying → Committed →
  Restoring → Restored, with explicit Aborted and Conflict terminal outcomes.
- `encode_desktop_restore_record()` / `encode_desktop_restore_ledger()` produce
  bounded versioned frames protected by CRC32.
- `decode_desktop_restore_ledger()` verifies the complete prefix and exposes
  the byte offset for a partial final append. A full frame with invalid magic,
  schema or checksum is corruption; it is never silently discarded.
- `plan_desktop_restore_recovery()` requires current adapter readback and a
  separate ownership proof before recommending a restore or retry.

The format is capped at 4 MiB and 8,192 records. Each resource key is a bounded
path-free token; before/desired values are bounded opaque adapter payloads,
and ownership proof bytes are bounded separately. The adapter registry must
still enforce an explicit allowlist and validate its own payload schema.

**This is not yet a persistent journal.** The portable crate performs no file
I/O, ACL/owner validation, durable flush, adapter registration or native
mutation. A future BGW storage backend must provide a user-private supported
local file, serialize the single writer, durably append and flush each record
before invoking an adapter, and test the complete crash/fault-injection matrix
in [the restoration architecture](../../../plan/architecture/restoration.md).
Do not use this codec alone to hide, suppress or replace native desktop items.
