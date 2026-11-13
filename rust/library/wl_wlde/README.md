# wl_wlde

**WLDE (pronounced WALDE)** is WinLook's integrated desktop engine. This
portable Rust crate is the first implementation layer: it owns scene policy,
monitor topology, wallpaper placement math and the shared frame schedule.

## Implemented in this core

- A bounded 16-monitor topology with stable adapter-supplied IDs, primary
  monitor validation, separate monitor bounds/work areas and transactional
  topology replacement.
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
- Explicit presentation capability checks. Virtual Groups require both a
  presenter capability and a crash-safe restoration capability.

## Deliberately not claimed

This crate does not draw pixels or create a desktop surface. It does not
decode images/video, parse .wl.wp/.wl.i.wp packages, execute widgets,
host Rainmeter plugins, access a Store, or manipulate desktop icons. No
platform presenter is included yet. Interactive content is unavailable unless
a future restricted host reports that capability. Groups remain disabled
until the OS adapter proves crash/disable/uninstall restoration.

The Rust core is intended for the modern Win10+ host. The matching
C/library/wl_Lwlde contract is C89 and keeps caller-owned fixed storage.
C89 syntax alone is not a statement of Windows 95/legacy OS support.

## Usage

Use the same scene tick for every animated wallpaper/widget property. Do not
create a timer per item. CPU/GPU admission is for bounded decode/upload work,
not a charge on every interpolated property.

## Next platform gates

1. Implement and test a documented static-wallpaper adapter and monitor
   topology watcher for each modern Windows generation.
2. Decide and test a supported animated desktop-surface integration that keeps
   Explorer and DWM intact; do not assume an undocumented desktop parent.
3. Add bounded media decode and GPU presentation with device-loss recovery.
4. Add isolated interactive/widget hosting and catalogue/package verification.
5. Keep Groups disabled until crash-safe native icon restoration is proven.
