# wl_Lanimate

`wl_Lanimate` is the C animation library paired with the modern Rust `wl_animate` crate. Both libraries are platform-neutral caller-driven animation cores with matching animation concepts; they do not share source code or require the same application runtime. Neither core contains Windows APIs or creates an OS event loop.

## Initial API

- Caller-owned scalar tweens with pending/running/paused/finished/cancelled states.
- Start delay, elapsed-time advancement, progress queries, cancellation and restart.
- Linear, quadratic, cubic, sine and quartic easing functions.
- `Point2`, RGBA, Vector3/4 and rectangle interpolation helpers.
- Affine `Matrix3x2` composition/point transformation, normalized opacity and shortest-arc `Transform2D` interpolation.
- The C implementation does not yet include the Rust SVG path parser or full vector scene model.
- Caller-owned scalar keyframe arrays with binary-search sampling and a cached per-playback segment cursor.
- Caller-owned keyframe playback with delay, repeat/alternate direction, fill modes, pause/resume/cancel/restart and host-driven advancement.

## C and resource contract

- C89-compatible public header and implementation.
- Static library output; no third-party runtime dependency beyond the platform C math library.
- No heap allocation in the per-frame path.
- No internal worker thread, timer, message pump or per-animation wake-up; one host frame loop advances all active animations.
- The C cached per-frame path does not allocate and normally needs only a segment-bound check, easing and scalar interpolation.
- Sine easing uses a degree-15 polynomial approximation (about 6.1e-12 maximum absolute error) rather than trigonometric library calls.
- Caller retains storage for tween and keyframe structures.
- No DWM, DirectComposition, D3D, modern CRT or modern Win32 imports in this portable core.
- Platform drawing, GDI resources, timers and window lifecycle belong in separate OS adapters.

C89 compatibility is an implementation constraint, not an OS compatibility claim. The core can be compiled on any host with a conforming C compiler and the platform math library; integrating it with a UI still requires a renderer and host-owned frame scheduler.

## Shared frame-clock contract

The C library provides `wl_Lanimate_frame_clock` for the single host-owned
frame loop. Pass monotonic milliseconds to `wl_Lanimate_frame_clock_tick`;
only emitted ticks should advance the active animations. The default 60 FPS
interval is `WL_LANIMATE_FRAME_60_FPS_INTERVAL_MS`, and the default maximum
delta is 100 ms. Fractional cadence is retained so faster 90/120/144 Hz host
poll rates can average 60 emitted frames per second instead of collapsing to
a fixed divisor. Pause/resume discards time spent hidden or suspended.

One clock is shared by the entire application/scene (wl_ui or WLDE). The host
decides whether to request another frame based on visibility and active
animations. The library does not create a timer, thread, message pump or GPU
resource. This avoids waking the machine once per widget/property.

A track may contain one to 64 keyframes; larger tracks are rejected at
initialization so the direct sampler also has a bounded lookup workload.

## Efficient host scheduling

Keep one frame clock per application or scene (for example wl_ui or wlDE),
compute the frame delta once, and advance active animations from that loop.
Store one wl_Lanimate_animation per animated track. Do not create one timer
or thread per view/property. The animation controller and cursor are caller-
owned; the library performs no heap allocation during sampling. Update native
properties only when sampled values change and let a compositor retain simple
transforms/opacity when supported.

The C library calculates property values; it does not render to a GPU. GPU
utilization depends on the host renderer and whether transforms or opacity
remain on a compositor path without repainting full surfaces. No universal
sub-1% CPU/GPU guarantee is meaningful across different machines, drivers,
backends and scene sizes.

## Build and test on a host compiler

With a C89-capable compiler, Make and an archiver:

```sh
make -C C/library/wl_Lanimate clean
make -C C/library/wl_Lanimate all
make -C C/library/wl_Lanimate test
make -C C/library/wl_Lanimate benchmark
```

This creates `build/libwl_Lanimate.a`, runs the test executable, and reports
the C89 1,024-animation / 600-frame CPU benchmark. When
linking the static library into a non-MSVC application, also link the platform
math library (for example, `-lm` on GCC/Clang Unix-like toolchains).

## Build and test with Windows MSVC

From a Visual Studio Developer PowerShell/Command Prompt with CMake available:

```powershell
cmake -S C/library/wl_Lanimate -B build/wl_Lanimate-msvc -A x64
cmake --build build/wl_Lanimate-msvc --config Release
ctest --test-dir build/wl_Lanimate-msvc -C Release --output-on-failure
```

CMake produces the platform-native static library (`wl_Lanimate.lib` with MSVC) and installs the public header/library using `cmake --install`. Non-MSVC consumers inherit the `m` link requirement from the exported CMake target. This verifies compilation with the selected Windows toolchain; it does not certify any platform-specific UI renderer.
