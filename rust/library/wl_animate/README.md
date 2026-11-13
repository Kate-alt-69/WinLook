# wl_animate

`wl_animate` is the modern Rust animation library for WinLook. It is also intended as a reusable public API for third-party applications and integrations.

## Initial API

- `Interpolate` for values that can be blended, with built-in `f32`, `f64`, `Point2` and `Rgba` implementations.
- Linear, quadratic, cubic, sine and quartic easing curves.
- `Tween<T>` with pending/running/paused/finished/cancelled states, delay, elapsed time, progress, cancellation and restart.
- `Keyframe<T>` and immutable `KeyframeTrack<T>` with ordered-time validation and timestamp sampling.
- `Vector2/3/4`, rectangles, affine matrices, decomposed transforms, opacity and vector-scene/style value types.
- An SVG **path-data parser** for M/L/H/V/C/S/Q/T/A/Z, absolute/relative commands, repeated groups and compact numeric syntax. The parser rejects path data above 1 MiB and more than 100,000 normalized commands. This is not a complete SVG XML/document importer or renderer.
- C/Rust APIs are not yet feature-identical: the C library currently exposes vector/transform math but not the Rust path parser or scene model.

The core is caller-driven: a host frame loop supplies elapsed `Duration` values to `Tween::advance`, or samples a keyframe track at a timestamp. The crate creates no threads, timers, windows, graphics contexts, hooks or scheduler. Sampling does not allocate internally for built-in fixed-size values; a custom value type's own `Clone`/`Interpolate` implementation may allocate.

## Example

```rust
use std::time::Duration;
use wl_animate::{Easing, Tween};

let mut fade = Tween::new_delayed(
    0.0_f32,
    1.0,
    Duration::from_millis(180),
    Duration::from_millis(40),
    Easing::EaseOutCubic,
);
let opacity = fade.advance(Duration::from_millis(16));
```

`elapsed()` includes the delay. `progress()` reports interpolation progress only. While pending or cancelled, sampling returns the current value; a finished tween returns its final value.

## Platform contract

The animation core has no Windows-specific API, target, windowing dependency or
OS baseline. It can be built wherever the selected Rust toolchain supports the
crate. The host owns the frame loop and supplies elapsed time; platform
renderers and resource management remain separate adapters. A successful
portable build does not certify a platform-specific renderer or application
binary for a particular OS release.

## Shared frame-clock contract

`FrameClock` is a small, host-driven cadence helper. The default targets
60 FPS, clamps a long pause to 100 ms, suppresses early duplicate calls and
resets its baseline on resume. It does not sleep, spin, create a timer or own
an event loop. Each host should keep one clock per scene/application and pass
the same `FrameTick::delta` to every active animation. WinLook's wl_ui and
WLDE should use that same clock rather than creating per-widget timers.

The host should stop requesting frames while the scene is hidden, fully
occluded or has no active animated content, and resume with a fresh timestamp.
Use the platform compositor for simple transforms/opacity where supported;
the clock only schedules sampling and never renders to a GPU.

## Low-overhead 60 FPS usage

Use one host frame clock and pass the same frame delta to every active
animation. Do not create one OS timer, worker or thread per animated property.
Each Animation owns a cached keyframe cursor, so consecutive samples within
the same segment avoid another binary search. Tracks remain immutable and can
be shared by multiple playback objects.

Sine easing uses a degree-11 polynomial approximation rather than sin/cos in
the frame path (maximum absolute error approximately 6.1e-12 on the supported
interval). Tween progress uses a nanosecond ratio. Animation advancement and
sampling do not allocate memory.

Run the local 60 FPS workload microbenchmark:

```sh
cargo run --release -p wl_animate --example frame_budget
```

The example advances 1,024 animations for 600 frames and reports CPU timing
on the current machine. It is not a GPU benchmark or a universal utilization
guarantee. wlDE and wl_ui should share one frame clock and reuse the same
animation objects; GPU cost depends on the host renderer and compositor path.

## Scope boundary

This is the renderer-independent animation core, not the complete OS animation suite. Win32 window observation, DWM, D3D11, DirectComposition, overlay presentation, frame scheduling, target-FPS pacing and platform resource management belong in separate adapters. A backend must be selected through capability detection and retain a native fallback. The core does not inject into processes or manipulate Windows UI by itself.

## Build and test

```sh
cargo fmt --manifest-path rust/library/wl_animate/Cargo.toml -- --check
cargo test --manifest-path rust/library/wl_animate/Cargo.toml
cargo clippy --manifest-path rust/library/wl_animate/Cargo.toml --all-targets --all-features -- -D warnings
```

The Windows CI workflow uses the MSVC Rust target. These portable tests validate the library logic; a Windows 10 client runtime test remains a separate environment-level check.

## Status

Preview API `0.2.0-beta.1`. Public signatures may change before a stable release. No Windows rendering or scheduler backend is included yet.
