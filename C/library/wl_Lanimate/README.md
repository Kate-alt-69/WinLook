# wl_Lanimate

`wl_Lanimate` is the C animation library paired with the modern Rust `wl_animate` crate. Both libraries share the same initial Windows 10 implementation baseline and matching animation concepts; they do not share source code or require the same application runtime.

## Initial API

- Caller-owned scalar tweens with pending/running/paused/finished/cancelled states.
- Start delay, elapsed-time advancement, progress queries, cancellation and restart.
- Linear, quadratic, cubic, sine and quartic easing functions.
- `Point2`, RGBA, Vector3/4 and rectangle interpolation helpers.
- Affine `Matrix3x2` composition/point transformation, normalized opacity and shortest-arc `Transform2D` interpolation.
- The C implementation does not yet include the Rust SVG path parser or full vector scene model.
- Caller-owned scalar keyframe arrays and timestamp sampling with input validation.

## C and resource contract

- C89-compatible public header and implementation.
- Static library output; no third-party runtime dependency beyond the platform C math library.
- No heap allocation in the per-frame path.
- No internal worker thread, timer or message pump; the host owns scheduling.
- Caller retains storage for tween and keyframe structures.
- No DWM, DirectComposition, D3D, modern CRT or modern Win32 imports in this portable core.
- Platform drawing, GDI resources, timers and window lifecycle belong in separate OS adapters.

C89 compatibility is an implementation constraint, **not** a claim of Windows 95–8.1 support. The current library build and test baseline is Windows 10; older OS/compiler/runtime combinations require separate evidence and are not claimed by this alpha.

## Build and test on a host compiler

With a C89-capable compiler, Make and an archiver:

```sh
make -C C/library/wl_Lanimate clean
make -C C/library/wl_Lanimate all
make -C C/library/wl_Lanimate test
```

This creates `build/libwl_Lanimate.a` and runs the test executable.

## Build and test with Windows MSVC

From a Visual Studio Developer PowerShell/Command Prompt with CMake available:

```powershell
cmake -S C/library/wl_Lanimate -B build/wl_Lanimate-msvc -A x64
cmake --build build/wl_Lanimate-msvc --config Release
ctest --test-dir build/wl_Lanimate-msvc -C Release --output-on-failure
```

CMake produces the platform-native static library (`wl_Lanimate.lib` with MSVC) and installs the public header/library using `cmake --install`. This verifies compilation with the selected Windows toolchain; it does not replace an actual Windows 10 client runtime test.
