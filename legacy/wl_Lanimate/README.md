# wl_Lanimate

`wl_Lanimate` is the legacy C animation library for WinLook builds targeting older Windows generations and toolchains.

## Initial API

- Caller-owned scalar tweens with pending/running/paused/finished/cancelled state.
- Linear, quadratic and cubic easing functions.
- Point2 and RGBA interpolation helpers.
- Caller-owned keyframe arrays and timestamp sampling.

## Legacy optimization rules

- C89-compatible source/header surface.
- No heap allocation in the per-frame path.
- No internal worker thread, timer or message pump; the host owns scheduling.
- Caller retains storage for tween and keyframe structures.
- No DWM, DirectComposition, D3D, modern CRT or modern Win32 imports in this portable core.
- Platform drawing, GDI resources, timers and window lifecycle belong in separate OS adapters.

## Host-side test

Run `make test` with a C89-capable compiler. This is a host compiler test, not proof of Open Watcom or Windows 95 runtime compatibility. The exact legacy compiler, linker, runtime, imported DLLs and target VM/hardware still require verification.
