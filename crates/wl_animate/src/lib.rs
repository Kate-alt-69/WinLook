//! Portable animation primitives for WinLook.
//!
//! This crate contains no Win32, DWM, DirectComposition, GPU, window-hooking,
//! scheduler, or rendering backend. A host owns the frame loop and supplies
//! elapsed time. Platform adapters and presentation belong in separate modules
//! so this crate can be tested independently.

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
    fn delayed_tween_waits_before_interpolating() {
        let mut tween = Tween::new_delayed(
            0.0_f64,
            10.0,
            Duration::from_millis(100),
            Duration::from_millis(50),
            Easing::Linear,
        );
        assert_eq!(tween.advance(Duration::from_millis(25)), 0.0);
        assert_eq!(tween.state(), PlaybackState::Pending);
        assert_eq!(tween.advance(Duration::from_millis(25)), 0.0);
        assert_eq!(tween.state(), PlaybackState::Running);
        assert_eq!(tween.advance(Duration::from_millis(50)), 5.0);
        assert_eq!(tween.advance(Duration::from_millis(50)), 10.0);
        assert_eq!(tween.state(), PlaybackState::Finished);
    }

    #[test]
    fn pause_during_delay_preserves_remaining_delay() {
        let mut tween = Tween::new_delayed(
            0.0_f64,
            1.0,
            Duration::from_secs(1),
            Duration::from_millis(100),
            Easing::Linear,
        );
        tween.advance(Duration::from_millis(40));
        tween.pause();
        assert_eq!(tween.state(), PlaybackState::Paused);
        assert_eq!(tween.advance(Duration::from_millis(500)), 0.0);
        tween.resume();
        assert_eq!(tween.state(), PlaybackState::Pending);
        assert_eq!(tween.advance(Duration::from_millis(60)), 0.0);
        assert_eq!(tween.state(), PlaybackState::Running);
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
    fn zero_duration_finishes_after_delay() {
        let mut tween = Tween::new_delayed(
            1.0_f64,
            4.0,
            Duration::ZERO,
            Duration::from_millis(25),
            Easing::Linear,
        );
        assert_eq!(tween.value(), 1.0);
        assert_eq!(tween.advance(Duration::from_millis(24)), 1.0);
        assert_eq!(tween.state(), PlaybackState::Pending);
        assert_eq!(tween.advance(Duration::from_millis(1)), 4.0);
        assert_eq!(tween.state(), PlaybackState::Finished);
    }

    #[test]
    fn cancel_freezes_current_value_and_restart_resets() {
        let mut tween = Tween::new(0.0_f64, 1.0, Duration::from_secs(1), Easing::Linear);
        assert_eq!(tween.advance(Duration::from_millis(250)), 0.25);
        tween.cancel();
        assert_eq!(tween.advance(Duration::from_millis(500)), 0.25);
        assert_eq!(tween.state(), PlaybackState::Cancelled);
        tween.restart();
        assert_eq!(tween.state(), PlaybackState::Pending);
        assert_eq!(tween.advance(Duration::from_millis(500)), 0.5);
    }

    #[test]
    fn easing_is_bounded_and_nan_is_safe() {
        assert_eq!(Easing::EaseInOutCubic.sample(-1.0), 0.0);
        assert_eq!(Easing::EaseInOutCubic.sample(2.0), 1.0);
        assert_eq!(Easing::Linear.sample(f64::NAN), 0.0);
        assert!((Easing::EaseInSine.sample(1.0) - 1.0).abs() < 1e-12);
        assert!((Easing::EaseOutQuart.sample(0.5) - 0.9375).abs() < 1e-12);
    }

    #[test]
    fn point_and_colour_interpolate_all_channels() {
        let point = Point2::new(0.0, 10.0).interpolate(&Point2::new(100.0, 30.0), 0.5);
        assert_eq!(point, Point2::new(50.0, 20.0));

        let color = Rgba::new(0.0, 10.0, 20.0, 30.0)
            .interpolate(&Rgba::new(100.0, 110.0, 120.0, 130.0), 0.5);
        assert_eq!(color, Rgba::new(50.0, 60.0, 70.0, 80.0));
    }

    #[test]
    fn keyframe_track_samples_endpoints_and_middle() {
        let track = KeyframeTrack::new(vec![
            Keyframe::new(Duration::ZERO, 0.0_f64, Easing::Linear),
            Keyframe::new(Duration::from_secs(1), 10.0_f64, Easing::Linear),
        ])
        .expect("valid track");
        assert_eq!(track.sample(Duration::ZERO), 0.0);
        assert_eq!(track.sample(Duration::from_millis(250)), 2.5);
        assert_eq!(track.sample(Duration::from_secs(2)), 10.0);
    }

    #[test]
    fn keyframe_track_rejects_empty_and_duplicate_times() {
        assert_eq!(
            KeyframeTrack::<f64>::new(Vec::new()).err(),
            Some(TrackError::Empty)
        );
        let result = KeyframeTrack::new(vec![
            Keyframe::new(Duration::ZERO, 0.0_f64, Easing::Linear),
            Keyframe::new(Duration::ZERO, 1.0_f64, Easing::Linear),
        ]);
        assert_eq!(
            result.err(),
            Some(TrackError::KeyframesMustBeStrictlyIncreasing)
        );
    }
}
