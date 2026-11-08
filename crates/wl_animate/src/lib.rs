//! Portable animation primitives for WinLook.
//!
//! This crate intentionally contains no Win32, DWM, DirectComposition, GPU,
//! window-hooking, scheduler, or rendering backend. Those belong in separate
//! platform adapters. The core can therefore be used by WinLook modules and
//! third-party applications without forcing a particular renderer or runtime.

#![forbid(unsafe_code)]

mod easing;
mod track;
mod tween;
mod values;

pub use easing::Easing;
pub use track::{Keyframe, KeyframeTrack, TrackError};
pub use tween::{PlaybackState, Tween};
pub use values::{Interpolate, Point2, Rgba};

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn tween_advances_and_finishes() {
        let mut tween = Tween::new(0.0_f64, 10.0, Duration::from_millis(100), Easing::Linear);
        assert_eq!(tween.state(), PlaybackState::Pending);
        assert_eq!(tween.advance(Duration::from_millis(25)), 2.5);
        assert_eq!(tween.advance(Duration::from_millis(100)), 10.0);
        assert_eq!(tween.state(), PlaybackState::Finished);
    }

    #[test]
    fn pause_and_resume_preserve_elapsed_time() {
        let mut tween = Tween::new(0.0_f64, 1.0, Duration::from_secs(1), Easing::Linear);
        tween.advance(Duration::from_millis(200));
        tween.pause();
        assert_eq!(tween.advance(Duration::from_millis(500)), 0.2);
        tween.resume();
        assert_eq!(tween.advance(Duration::from_millis(300)), 0.5);
    }

    #[test]
    fn zero_duration_returns_end_value() {
        let tween = Tween::new(1.0_f64, 4.0, Duration::ZERO, Easing::Linear);
        assert_eq!(tween.state(), PlaybackState::Finished);
        assert_eq!(tween.value(), 4.0);
    }

    #[test]
    fn easing_is_bounded() {
        assert_eq!(Easing::EaseInOutCubic.sample(-1.0), 0.0);
        assert_eq!(Easing::EaseInOutCubic.sample(2.0), 1.0);
    }

    #[test]
    fn keyframe_track_samples_between_frames() {
        let track = KeyframeTrack::new(vec![
            Keyframe::new(Duration::ZERO, 0.0_f64, Easing::Linear),
            Keyframe::new(Duration::from_secs(1), 10.0_f64, Easing::Linear),
        ]).expect("valid track");
        assert_eq!(track.sample(Duration::from_millis(250)), 2.5);
    }

    #[test]
    fn keyframe_track_rejects_duplicate_times() {
        let result = KeyframeTrack::new(vec![
            Keyframe::new(Duration::ZERO, 0.0_f64, Easing::Linear),
            Keyframe::new(Duration::ZERO, 1.0_f64, Easing::Linear),
        ]);
        assert_eq!(result.err(), Some(TrackError::KeyframesMustBeStrictlyIncreasing));
    }
}
