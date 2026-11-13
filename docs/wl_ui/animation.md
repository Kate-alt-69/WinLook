# Shared animation runtime

WinLook uses one small portable animation runtime for native controls, desktop
transitions, wallpaper and widget properties. The Rust implementation is
`wl_animate`; the C89 implementation is `wl_Lanimate`. Both are host-driven:
they calculate values but do not create timers, threads, windows, renderers or
GPU resources.

The declarative CSS keyframe API is documented in
[styling.md — Motion and transforms](styling.md#motion-and-transforms). This
page covers runtime sampling and shared frame scheduling.

## One clock per visible scene

Create one frame clock for the whole application or scene, not one timer per
widget/property. Pass the same emitted delta to every active animation:

- Rust: `wl_ui::FrameClock` or `wl_ui::animate::FrameClock`.
- C89: `wl_Lanimate_frame_clock`.
- Both default to a roughly 60 FPS target and cap a long frame delta at 100 ms.
- Fractional cadence is preserved when the host polls faster at 90, 120 or
  144 Hz; it does not simply emit every second/third host poll.
- A long stall emits at most one capped tick. It is never replayed as a burst
  of missed frames.
- Pause the clock while the scene is hidden, fully occluded, suspended by
  Game Mode or has no active motion. Resume it with a fresh monotonic baseline.

The host owns the event/composition callback and decides when to request the
next frame. The clock only filters host timestamps; it never sleeps, spins or
starts a worker.

## Rust API

The `wl_ui` facade re-exports the clock and exposes the complete runtime under
`wl_ui::animate`. Applications using `wl_animate` directly can import the
same types from that crate.

```rust
use std::time::Duration;
use wl_ui::{
    animate::{Animation, Direction, Easing, FillMode, IterationCount, Keyframe, KeyframeTrack},
    FrameClock,
};

fn create_animation() -> Result<Animation<f64>, Box<dyn std::error::Error>> {
    let track = KeyframeTrack::new(vec![
        Keyframe::new(Duration::ZERO, 0.0, Easing::EaseOutCubic),
        Keyframe::new(Duration::from_millis(180), 1.0, Easing::EaseInOutCubic),
    ])?;

    Ok(Animation::new(
        track,
        Duration::from_millis(180),
        Duration::ZERO,
        IterationCount::Once,
        Direction::Normal,
        FillMode::Forwards,
    )?)
}

fn advance_from_host(
    clock: &mut FrameClock,
    animation: &mut Animation<f64>,
    monotonic_now: Duration,
) -> Option<f64> {
    clock
        .tick(monotonic_now)
        .and_then(|frame| animation.advance(frame.delta))
}
```

Call `clock.tick(Duration::ZERO)` once when starting a scene to establish its
baseline. On subsequent host frames pass monotonic elapsed time since the
scene's start. Apply a sampled value only when it has changed. Keep the clock
and each `Animation` alive across frames; do not reconstruct them in the
render callback.

### Tracks, values and lifecycle

`KeyframeTrack<T>` is immutable. Keyframe timestamps must be strictly
increasing. Keep one `TrackCursor` per playback when sampling a track
directly; `Animation` already owns its cursor. Sequential samples in the same
segment use constant-time lookup, and reverse seeks or large jumps use binary
search. Sampling does not allocate internally.

Built-in interpolation types are `f32`, `f64`, `Point2` and `Rgba`.
Their interpolation and cloning paths are allocation-free. Third-party
`Interpolate` implementations define their own clone/interpolation behavior
and may allocate.

`Animation` supports:

| Setting | Values / limit |
|---|---|
| Duration | Positive; at most 1 hour |
| Delay | 0 to 1 hour |
| Iterations | Once, 1–1000, or infinite |
| Direction | Normal, reverse, alternate, alternate-reverse |
| Fill | None, forwards, backwards, both |
| Keyframes | 2–64, strictly increasing timestamps |
| Playback | Pending, running, pause, resume, cancel, restart |

Durations and delays are host-time values. The animation does not determine
whether a renderer can apply a property; native backends must implement and
report their actual animation capability.

## C89 API

`wl_Lanimate` provides the same host-owned timing contract without a Rust
runtime or C++ dependency. Its public header and implementation are strict
C89/C90. Tracks, cursors, animations and clocks are caller-owned; the library
does not allocate per frame.

```c
#include <wl_Lanimate.h>

int run_scene(void)
{
    wl_Lanimate_keyframe frames[2];
    wl_Lanimate_track track;
    wl_Lanimate_animation animation;
    wl_Lanimate_frame_clock clock;
    wl_Lanimate_frame_tick tick;
    double value;
    double now_ms;

    frames[0].at_ms = 0.0;
    frames[0].value = 0.0;
    frames[0].easing_to_next = WL_LANIMATE_EASE_OUT_CUBIC;
    frames[1].at_ms = 180.0;
    frames[1].value = 1.0;
    frames[1].easing_to_next = WL_LANIMATE_LINEAR;

    if (wl_Lanimate_track_init(&track, frames, 2U) != WL_LANIMATE_OK) {
        return 1;
    }
    if (wl_Lanimate_animation_init(
        &animation, &track, 180.0, 0.0, 1U,
        WL_LANIMATE_DIRECTION_NORMAL, WL_LANIMATE_FILL_FORWARDS
    ) != WL_LANIMATE_OK) {
        return 2;
    }
    if (wl_Lanimate_frame_clock_init(
        &clock,
        WL_LANIMATE_FRAME_60_FPS_INTERVAL_MS,
        WL_LANIMATE_FRAME_DEFAULT_MAXIMUM_DELTA_MS
    ) != WL_LANIMATE_OK) {
        return 3;
    }

    now_ms = 0.0;
    (void)wl_Lanimate_frame_clock_tick(&clock, now_ms, &tick);
    /* Call this block from the host's existing frame callback. */
    now_ms += 16.6666667;
    if (wl_Lanimate_frame_clock_tick(&clock, now_ms, &tick) == WL_LANIMATE_OK) {
        if (wl_Lanimate_animation_advance(&animation, tick.delta_ms, &value)
            == WL_LANIMATE_OK) {
            /* Apply value to a native property only when it changes. */
        }
    }
    return 0;
}
```

For multi-axis values, sample independent scalar tracks or use the provided
point/vector/rect/transform interpolation helpers. A C application should keep
the keyframe arrays alive until the borrowed tracks and animations are no
longer used. `WL_LANIMATE_NO_FRAME` means the host poll did not reach the next
frame cadence; `WL_LANIMATE_NO_VALUE` means the animation fill mode has no
value to apply at that time.

## Renderer and WLDE integration

- **Tauri/WebKit:** use the validated CSS keyframes and browser/compositor
  animation path. Do not run a second native timer for the same CSS property.
- **Win32/WinUI:** use one scene clock and apply sampled values only for
  properties actually supported by the renderer. The current Windows 10
  Win32 shell does not animate every CSS style property.
- **WLDE:** wallpaper, widget and desktop-transition tracks share the same
  clock as the visible scene. Pause when fully occluded or suspended by
  resource policy/Game Mode. Keep media decode, upload, composition and
  property interpolation as separately profiled costs.

The animation library computes scalar/vector values; it does not draw, upload
textures, submit GPU commands or guarantee compositor promotion. The host
renderer must use native compositor-backed transforms/opacity where supported.
No universal CPU/GPU utilization promise is meaningful across machines,
drivers, renderers, frame rates and scene sizes.

## Benchmark

Run the Rust microbenchmark from the repository root:

```sh
cargo run --release -p wl_animate --example frame_budget
```

It advances 1,024 built-in scalar animations for 600 frames through the shared
60 FPS clock using the polynomial sine easing curves, then reports CPU time
on the current machine. The matching C89 workload is available with:

```sh
make -C C/library/wl_Lanimate benchmark
```

CMake/CTest also runs `wl_Lanimate.frame_budget` and prints its timings in
the library verification logs. Both are repeatable CPU workloads, not GPU
benchmarks or universal performance targets.
