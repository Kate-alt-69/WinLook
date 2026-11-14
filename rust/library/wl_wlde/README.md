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
- Explicit presentation capability checks. Virtual Groups require both a
  presenter capability and a crash-safe restoration capability.

## Deliberately not claimed

This crate does not draw pixels or create a desktop surface. It does not
decode images/video, parse .wl.wp/.wl.i.wp packages, execute widgets,
host Rainmeter plugins, run an in-app catalogue search/download client, or
manipulate desktop icons. It does now expose the integrated Store provider
directory and a safe hand-off adapter for an already-installed Wallpaper
Engine. Interactive content is unavailable unless a future restricted host
reports that capability. Groups remain disabled until the OS adapter proves
crash/disable/uninstall restoration.

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
