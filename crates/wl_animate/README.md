# wl_animate

`wl_animate` is the modern Rust animation crate for WinLook. It is also intended as a public reusable API for third-party applications and integrations.

## Initial API surface

The first preview contains renderer-independent primitives:

- `Interpolate` for values that can be blended;
- `Easing` curves;
- `Tween<T>` with pending/running/paused/finished/cancelled states;
- `Keyframe<T>` and immutable `KeyframeTrack<T>`;
- common `Point2` and `Rgba` value types.

The core is caller-driven: a host frame loop passes elapsed `Duration` values to `advance` or samples a keyframe track at a timestamp. The crate creates no threads, timers, windows, graphics contexts, hooks or scheduler. Its algorithms do not allocate during sampling for the built-in fixed-size values. A custom value type's own `Clone` implementation may allocate.

## Example

```rust
use std::time::Duration;
use wl_animate::{Easing, Tween};

let mut fade = Tween::new(0.0_f32, 1.0, Duration::from_millis(180), Easing::EaseOutCubic);
let opacity = fade.advance(Duration::from_millis(16));
```

## Windows target boundary

The crate's `rust-version = 1.56` manifest field describes its minimum Rust compiler version, not its minimum Windows version. The current official Rust/MSVC Windows target documentation requires Windows 10 or later for client installs. A portable crate that avoids Win32 APIs does not by itself make a current Rust/MSVC application binary compatible with Vista, Windows 7, 8 or 8.1. The planned older-Windows implementation uses `wl_Lanimate` unless a historical Rust toolchain is independently verified.

See [Rust MSVC target support](https://doc.rust-lang.org/rustc/platform-support/windows-msvc.html) and [WinLook's toolchain compatibility policy](../../plan/compatibility/README.md).

## Scope boundary

This is an API foundation, not the complete OS animation suite. Win32 window observation, injection/hooking, DWM, D3D11, DirectComposition, overlay-window presentation, frame scheduling and resource management belong in separate backend crates/modules. A backend must be selected through OS/driver capability detection and retain a native fallback.

The `wl_Lanimate` C library is the legacy counterpart. It shares concepts, not source code or a requirement to use modern Rust runtimes.

## Status

Preview API `0.1.0-alpha.1`. Public signatures may change before a stable release. No runtime Windows backend is included yet.
