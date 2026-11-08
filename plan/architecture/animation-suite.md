# WinLook Animation Suite

## Two implementations

| Name | Language | Contract |
|---|---|---|
| wl_animate | Rust crate | Modern reusable animation engine and public developer API |
| wl_Lanimate | C library | Legacy animation engine/library |

wl_animate is meant for WinLook modules and third-party developers creating apps or integrations for WinLook. wl_Lanimate provides corresponding conceptual functionality for legacy Windows. They need not share source code or implementation details.

For WinLook Windows binaries built with the current official Rust/MSVC target, the documented client OS floor is Windows 10. The portable Rust timing/value crate itself has no Windows API dependency, but that does not establish that a current Rust/MSVC binary can run on Vista, Windows 7, 8 or 8.1. The C library is the planned older-Windows path; each compiler/runtime combination still requires testing.

## Whole-OS suite scope
Window open/minimize/restore/close, move/resize/snap/focus, workspace changes, desktop icon/Group interactions, wallpaper/widget transitions, WLOSDE notification/OSD transitions, WinLook Manager controls and third-party UI.

## Shared conceptual features
The suite is intended to cover duration/delay, easing, interpolation, keyframes, timelines, sequences, parallel groups, pause/resume/cancel, interruption handling, progress/state callbacks, frame pacing, target FPS, reduced-motion, resource budgets and deterministic disposal. The first alpha prototypes establish primitive signatures; the stable cross-language contract, C ABI, scheduler, sequencing and callback model still require design review.

## Modern wl_animate
Candidate backends: Win32 window/message observation, DWM APIs (Vista+), D3D11, DirectComposition (selected interfaces have Windows 8.1+ minimums) and independent layered/overlay surfaces where appropriate. Capability detection must select backend at runtime.

Sources: [DwmIsCompositionEnabled](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmiscompositionenabled), [IDCompositionDesktopDevice](https://learn.microsoft.com/en-us/windows/win32/api/dcomp/nn-dcomp-idcompositiondesktopdevice).

## Legacy wl_Lanimate
Optimize specifically for selected compiler/linker/runtime, CPU baseline, GDI/USER APIs and old hardware. Research:
- calling convention and C ABI;
- minimal imports/CRT assumptions;
- bounded allocation and no per-frame heap churn;
- GDI off-screen bitmaps/dirty rectangles;
- timer/message-loop scheduling;
- resource cleanup for DCs, bitmaps, regions and windows;
- fixed-point arithmetic only when measured beneficial;
- static/no-animation fallback.

Open Watcom documents Win95 and NT target variants: [Linker Guide](https://open-watcom.github.io/open-watcom-v2-wikidocs/lguide.html). That is toolchain evidence, not a tested support statement.

## API minimums
Microsoft documents AnimateWindow with Windows 2000 minimum client; it must not be assumed on Windows 95/98/ME. SetWinEventHook has a Windows 2000 minimum, with a separate Active Accessibility 1.3 RDK route documented for Windows 95 and NT4 SP6. Legacy adapters must detect actual installed APIs/runtime.

Sources: [AnimateWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-animatewindow), [SetWinEventHook](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwineventhook).

## Third-party developer contract
Publish Rust API docs/examples and semver once stable. Define a C ABI for legacy consumers. Provide capability queries, bounded sequences, cancellation and errors. Third-party code must not be able to deadlock the BGW or Manager. WLT may include animation configuration/assets, not arbitrary native code disguised as an animation profile.


## Initial implementation slice — portable animation primitives (alpha.1)

The first code slice establishes the renderer-independent timing/value layer only. It is deliberately **not** the complete OS animation suite: no Win32 window hooks, process injection, BGW integration, DWM/D3D/DirectComposition, GDI drawing, overlay presentation, timer loop, or Windows backend is included yet.

### Rust: `wl_animate`

- Crate: [`crates/wl_animate`](../../crates/wl_animate/README.md)
- Public concepts: `Interpolate`, `Easing`, `Tween<T>`, `PlaybackState`, `Keyframe<T>`, `KeyframeTrack<T>`, `Point2` and `Rgba`.
- Host-driven timing: consumers provide elapsed `std::time::Duration`; the crate creates no scheduler, worker thread, timer, window, graphics context or hook.
- Sampling does not allocate internally for the built-in fixed-size value types. A third-party `Interpolate` implementation may allocate inside its own `Clone` implementation; the crate cannot guarantee otherwise.
- Initial version: `0.1.0-alpha.1`. Public API is provisional.

### Legacy C: `wl_Lanimate`

- Library sources: [`legacy/wl_Lanimate`](../../legacy/wl_Lanimate/README.md)
- C89-compatible header and implementation; caller-owned tween/keyframe storage; no heap allocation in the per-frame path.
- Includes scalar tween state, linear/quadratic/cubic easing, Point2/RGBA interpolation and timestamped keyframe sampling.
- The library core has no internal worker thread, timer, message pump, DWM, DirectComposition, D3D or modern Win32 imports. OS-specific scheduling and drawing belong in separate adapters.

### Verification evidence

- **C host compile/test: passed** with GCC 14.2.0 using `make clean && make test` and `-std=c89 -Wall -Wextra -Werror -pedantic -O2`. This validates the portable C source against a C89 compiler mode; it does not prove Open Watcom, Windows 95, or any other Windows runtime compatibility.
- **Rust compile/test: not run in the original prototype checkpoint.** That execution environment did not have `rustc` or `cargo` installed. Install/use a Rust toolchain and run `cargo fmt --check`, `cargo test` and `cargo clippy --all-targets --all-features` before treating the crate as build-verified.
- **Windows runtime/backend tests: not run.** No Win32/DWM/GDI backend or injection integration exists in this slice.
- No compiled test executable or other build output is committed.

### Next implementation gates

1. Build and test `wl_animate` with the selected Rust toolchain; separately verify the Windows 10+ MSVC runtime baseline and do not infer pre-Windows-10 runtime support.
2. Verify `wl_Lanimate` with the selected Open Watcom compiler/linker and actual legacy target imports.
3. Define the stable shared conceptual API and the C ABI before promising third-party compatibility.
4. Add platform schedulers/renderers behind capability-detected adapters, retaining native fallbacks.
5. Add frame-time, allocation, CPU/GPU and GDI-resource benchmarks before optimizing compiler-specific paths.

## Verification
Profile frame times, CPU/GPU usage, allocations, GDI handle leaks, cancellation, target destruction, display changes, reduced-motion, low power, concurrency, legacy VM/hardware and third-party ABI compatibility.

## Planned repository location

The long-term layout proposal places reusable WinLook-owned libraries under the root lib directory:
- wl_animate → lib/wl_animate (Rust).
- wl_Lanimate → lib/wl_Lanimate (C).

The existing prototypes remain in their current repository paths until the layout is approved and a dedicated migration updates manifests, imports, tests, build workflows and documentation links. This is a planned path mapping, not a completed move.

