# WLDE — WinLook Desktop Engine (WALDE)

WLDE is the integrated subsystem for wallpapers, widgets, desktop presentation,
the local wallpaper library and community catalogue. It is not a separate
Wallpaper Engine product and it does not replace Explorer or DWM.

## Current implementation

The new portable core is split into paired language libraries:

- Rust: [wl_wlde](../../rust/library/wl_wlde/README.md)
- C89: [wl_Lwlde](../../C/library/wl_Lwlde/README.md)

Both currently implement bounded monitor topology and placement math, a single
scene clock built on the existing animation libraries, visibility-aware frame
scheduling, Normal/Optimize/Suspend policies and explicit presentation
capability gates. CPUv/GPUv remain the resource admission/accounting layer.

The C core uses caller-supplied monitor storage (up to 16 records), does not
allocate during placement or scene ticks, and keeps non-owning pointers to
already-initialized CPUv/GPUv governors. The Rust core validates topology
replacement before applying it and owns its CPUv/GPUv governor instances.
Neither API creates timers or worker threads.

## Renderer boundary

**This is not yet a usable desktop wallpaper renderer.** The current branch
does not enumerate real Windows monitors, create desktop-owned surfaces, draw
pixels, decode image/video assets, host widgets, parse wallpaper packages,
access a Store/catalogue, run Rainmeter skins or manage desktop icons.

On Windows 8 and later, the documented IDesktopWallpaper interface sets and
queries static wallpaper paths by monitor. It does not expose an animated
frame-surface API. Windows.UI.Composition supplies application visual-layer
primitives, but that alone is not a documented way to install an animated
surface behind Explorer's desktop icons. See Microsoft's
[SetWallpaper API](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-idesktopwallpaper-setwallpaper),
[IDesktopWallpaper interface](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-idesktopwallpaper)
and [visual-layer documentation](https://learn.microsoft.com/en-us/windows/apps/develop/ui/visual-layer/).

Do not substitute undocumented WorkerW/Progman parent-window assumptions,
modify Explorer.exe, disable DWM or move real desktop shortcuts. Any future
presentation adapter must report its actual capabilities and restore the
native desktop after failure, shutdown and uninstall.

## Runtime policy

- Visibility Unknown and partially contributing regions are treated as visible.
  Only known full occlusion suppresses animation frame requests.
- One frame clock is shared across the full scene. Normal uses 60 FPS;
  Optimize and Power Saver use 30 FPS; Suspend, static-only and full-occlusion
  states request no animation frames. Resume discards hidden elapsed time.
- Automatic pressure handling preserves source image/video fidelity. It defers
  nonessential work, pauses decode/animation, holds a last-known-good frame,
  releases recoverable resources and suspends when required. It does not
  silently downsample or re-encode user assets.
- CPUv/GPUv estimates and tickets are caller-provided accounting/admission
  values. They are not system CPU/GPU telemetry and do not schedule unrelated
  processes.
- Interactive content requires an isolated runtime and explicit capabilities.
  The current cores do not execute scripts.
- Desktop Groups remain unavailable unless the adapter proves both virtual
  presentation and crash-safe restoration of original Shell items.

## Next implementation gates

1. Implement modern Windows monitor enumeration and topology-change handling.
2. Implement a documented static-wallpaper fallback and record/restore the
   user's original wallpaper state.
3. Resolve and test an animated desktop surface that preserves Explorer/DWM
   ownership; do not ship undocumented parenting as a supported contract.
4. Add bounded image/video decoding, GPU allocation/submission, device-loss
   recovery and visibility/power/session integration.
5. Add signed/hash-verified .wl.wp/.wl.i.wp import, local library and offline
   catalogue metadata.
6. Add restricted widget/interactive hosting and a reviewed Rainmeter subset.
7. Keep Groups disabled until crash-safe restore is demonstrated on each OS.
8. Add real Windows 10/11 visual, multi-monitor, lock/unlock, remote-session,
   fullscreen/game, low-end hardware and uninstall restoration testing.

Run the shared Rust/C gate with:

    python build.py verify libraries --sanitizers

A green portable library gate verifies only these shared contracts; it does not
prove Windows desktop presentation or overall WLDE acceptance.
