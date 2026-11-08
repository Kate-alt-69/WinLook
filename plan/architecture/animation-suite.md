# WinLook Animation Suite

## Two implementations

| Name | Language | Contract |
|---|---|---|
| wl_animate | Rust crate | Modern reusable animation engine and public developer API |
| wl_Lanimate | C library | Legacy animation engine/library |

wl_animate is intended for WinLook modules and third-party developers creating apps or integrations. wl_Lanimate provides the corresponding C library surface. The two implementations share concepts and observable behavior, not source code or a requirement to use identical runtimes.

**Initial target baseline for both libraries: Windows 10.** The Rust Windows build uses the current official Rust/MSVC target. The C implementation is kept C89-compatible, but that language constraint does not establish support on Windows 95 through 8.1. Older Windows compatibility is a separate future validation effort and is not part of this alpha baseline.

## Whole-OS suite scope
Window open/minimize/restore/close, move/resize/snap/focus, workspace changes, desktop icon/Group interactions, wallpaper/widget transitions, WLOSDE notification/OSD transitions, WinLook Manager controls and third-party UI.

## Shared conceptual features
The suite is intended to cover duration/delay, easing, interpolation, keyframes, timelines, sequences, parallel groups, pause/resume/cancel, interruption handling, progress/state events, frame pacing, target FPS, reduced-motion and deterministic disposal. Its vector geometry, SVG, color/alpha and render contracts are specified separately in [vector-animation.md](vector-animation.md). CPU and GPU admission/budgeting is centralized in the planned [CPUv/GPUv resource governors](resource-governors.md), not reimplemented inside animation.

## Modern wl_animate
For Windows 10+ Rust/MSVC builds, candidate presentation backends include Direct2D geometry and Windows.UI.Composition, with a capability-selected CPU raster fallback. Do not select an API solely from its minimum OS version: Windows 10 guidance recommends Windows.UI.Composition instead of DirectComposition for app composition. Device loss, pressure and resource accounting must be handled by the separate renderer adapter and GPUv. **No presentation backend is included in either library core yet.**

Sources: [DwmIsCompositionEnabled](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmiscompositionenabled), [IDCompositionDesktopDevice](https://learn.microsoft.com/en-us/windows/win32/api/dcomp/nn-dcomp-idcompositiondesktopdevice).

## C `wl_Lanimate`
The current C implementation is a renderer-independent static library with a C89-compatible public header and implementation. Its first build/test baseline is Windows 10/MSVC plus portable host compilers. C89 compatibility is not an older-Windows support statement. Any future older-OS target must separately validate its compiler, linker, runtime, imports, graphics adapter and recovery behavior.

Platform drawing, GDI resources, timers and window lifecycle are not included in the core. They belong in separately tested OS adapters. The portable core uses caller-owned tween and keyframe storage and does not allocate in its per-frame path.

Open Watcom documents Win95 and NT target variants: [Linker Guide](https://open-watcom.github.io/open-watcom-v2-wikidocs/lguide.html). That is toolchain evidence, not a tested support statement.

## API minimums
Microsoft documents AnimateWindow with Windows 2000 minimum client; it must not be assumed on Windows 95/98/ME. SetWinEventHook has a Windows 2000 minimum, with a separate Active Accessibility 1.3 RDK route documented for Windows 95 and NT4 SP6. Legacy adapters must detect actual installed APIs/runtime.

Sources: [AnimateWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-animatewindow), [SetWinEventHook](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwineventhook).

## Third-party developer contract
Publish Rust API docs/examples and semver once stable. Define a C ABI for legacy consumers. Provide capability queries, bounded sequences, cancellation and errors. Third-party code must not be able to deadlock the BGW or Manager. WLT may include animation configuration/assets, not arbitrary native code disguised as an animation profile.


## Verified implementation checkpoint — alpha.2 (historical)

The first buildable library slice now provides a paired Rust crate and C static library. Both target Windows 10 first. They deliberately remain renderer-independent: neither library owns a scheduler, timer, message loop, window, graphics context, hook or process-injection path.

### Rust: `wl_animate`

- Crate: [`crates/wl_animate`](../../crates/wl_animate/README.md), version `0.1.0-alpha.2`.
- Public API: `Interpolate`, `Easing`, `Tween<T>`, `PlaybackState`, `Keyframe<T>`, `KeyframeTrack<T>`, `TrackError`, `Point2` and `Rgba`.
- Built-in interpolation: `f32`, `f64`, 2D points and four-channel RGBA values.
- Easing: linear, quadratic, cubic, sine and quartic in/out/in-out curves.
- Tweens support delay, host-driven elapsed-time advancement, pause/resume, cancellation, restart, progress and state inspection.
- Keyframe tracks validate strictly increasing timestamps and use binary search for sampling.
- The crate forbids unsafe Rust. No external dependencies are required.

### C: `wl_Lanimate`

- Sources: [`legacy/wl_Lanimate`](../../legacy/wl_Lanimate/README.md), version `0.1.0-alpha.2`.
- C89-compatible public header and implementation; matching easing family and scalar tween/keyframe concepts.
- Point2 and RGBA interpolation helpers. The alpha-channel interpolation defect from the previous prototype is fixed and covered by a dedicated test.
- Caller-owned tween/keyframe storage; no allocation in the per-frame core.
- Make and CMake build paths produce a static library. MSVC uses the native `.lib` output; host Make builds use the platform archiver.
- CMake exposes the public include directory and builds a focused CTest target.

### Verification evidence

- **GCC C89 Make build and tests: passed** with GCC 14.2.0, `-std=c89 -Wall -Wextra -Werror -pedantic -O2`; this built `libwl_Lanimate.a` and ran the full C test program.
- **GCC CMake build and CTest: passed** with CMake 3.31.6; 1/1 CTest target passed.
- **Clang C89 + AddressSanitizer + UndefinedBehaviorSanitizer: passed** for the C test executable.
- **Windows MSVC CI: passed** on GitHub Actions run [42513514864](https://github.com/Kate-alt-69/WinLook/actions/runs/42513514864): Rust formatting, 10 tests, Clippy and API docs; CMake/MSVC static-library build and CTest (1/1). The local host did not have a Rust toolchain, so no local Rust command was run.
- **Windows 10 client runtime: not run.** The CI host was Windows Server 2022, not a Windows 10 client VM/hardware test.
- No build artifacts are committed.

### Next implementation gates

1. Obtain a completed green Windows Rust/MSVC and C/MSVC workflow run.
2. Test the resulting libraries on an actual Windows 10 client VM and, later, representative physical hardware.
3. Review and expand the public API for timelines, sequences, parallel groups, interruption policies, callbacks, reduced motion and resource budgets.
4. Add platform schedulers/presentation adapters separately, with capability detection and native fallbacks.
5. Benchmark frame-time overhead, allocations, CPU/GPU use and platform resource cleanup before tuning.

## Planned repository location

The long-term layout proposal places reusable WinLook-owned libraries under the root `lib` directory:
- `wl_animate` → `lib/wl_animate` (Rust).
- `wl_Lanimate` → `lib/wl_Lanimate` (C).

The current source paths remain unchanged until a dedicated layout migration updates manifests, imports, tests, build workflows and documentation links together.


## Current beta planning status — 2026-11-09

The crate manifest currently carries `0.2.0-beta.1` and the branch includes initial `vector.rs`/`svg.rs` source files. That version string is not a release-quality assertion: these modules are not yet wired into the crate's public exports, and the vector/SVG parser still needs the grammar, alpha, document-import and stress-test gates in [vector-animation.md](vector-animation.md). Do not publish or advertise a completed beta until those gates pass.

The user has paused further source implementation to continue planning. The next source pass should be driven by [vector-animation.md](vector-animation.md) and [resource-governors.md](resource-governors.md). Those docs define the intended contract; planned API entries are not available features yet.

