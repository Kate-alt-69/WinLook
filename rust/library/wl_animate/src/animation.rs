use crate::track::TrackCursor;
use crate::{KeyframeTrack, PlaybackState};
use std::fmt;
use std::time::Duration;

pub const MAX_ANIMATION_TIME: Duration = Duration::from_secs(60 * 60);
pub const MAX_ANIMATION_KEYFRAMES: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IterationCount { Once, Count(u16), Infinite }
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction { Normal, Reverse, Alternate, AlternateReverse }
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FillMode { None, Forwards, Backwards, Both }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnimationError {
    InvalidDuration,
    InvalidDelay,
    InvalidIterationCount,
    InvalidKeyframeCount,
    KeyframesNeedPositiveSpan,
}

impl fmt::Display for AnimationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDuration => f.write_str("animation duration must be positive and at most one hour"),
            Self::InvalidDelay => f.write_str("animation delay must not exceed one hour"),
            Self::InvalidIterationCount => f.write_str("iteration count must be 1 to 1000 or infinite"),
            Self::InvalidKeyframeCount => f.write_str("animation must contain 2 to 64 keyframes"),
            Self::KeyframesNeedPositiveSpan => f.write_str("keyframes must cover a positive time span"),
        }
    }
}
impl std::error::Error for AnimationError {}

/// Host-driven playback. It creates no timers, threads, callbacks or renderers.
#[derive(Clone, Debug)]
pub struct Animation<T: crate::Interpolate> {
    track: KeyframeTrack<T>,
    cursor: TrackCursor,
    duration: Duration,
    delay: Duration,
    iterations: IterationCount,
    direction: Direction,
    fill_mode: FillMode,
    elapsed: Duration,
    state: PlaybackState,
}

impl<T: crate::Interpolate> Animation<T> {
    pub fn new(
        track: KeyframeTrack<T>,
        duration: Duration,
        delay: Duration,
        iterations: IterationCount,
        direction: Direction,
        fill_mode: FillMode,
    ) -> Result<Self, AnimationError> {
        if duration.is_zero() || duration > MAX_ANIMATION_TIME {
            return Err(AnimationError::InvalidDuration);
        }
        if delay > MAX_ANIMATION_TIME {
            return Err(AnimationError::InvalidDelay);
        }
        if let IterationCount::Count(count) = iterations {
            if count == 0 || count > 1000 {
                return Err(AnimationError::InvalidIterationCount);
            }
        }
        if !(2..=MAX_ANIMATION_KEYFRAMES).contains(&track.len()) {
            return Err(AnimationError::InvalidKeyframeCount);
        }
        if track.end_time() <= track.start_time() {
            return Err(AnimationError::KeyframesNeedPositiveSpan);
        }
        Ok(Self {
            track,
            cursor: TrackCursor::default(),
            duration,
            delay,
            iterations,
            direction,
            fill_mode,
            elapsed: Duration::ZERO,
            state: PlaybackState::Pending,
        })
    }

    pub fn state(&self) -> PlaybackState {
        self.state
    }

    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn delay(&self) -> Duration {
        self.delay
    }

    /// Called once by the shared host frame loop. No per-animation wakeup.
    #[inline]
    pub fn advance(&mut self, delta: Duration) -> Option<T> {
        if matches!(self.state, PlaybackState::Pending | PlaybackState::Running) {
            self.elapsed = self.elapsed.saturating_add(delta);
            self.refresh_state();
        }

        let sample = AnimationSample {
            track: &self.track,
            elapsed: self.elapsed,
            delay: self.delay,
            duration: self.duration,
            iterations: self.iterations,
            direction: self.direction,
            fill_mode: self.fill_mode,
            state: self.state,
        };
        sample_animation(sample, Some(&mut self.cursor))
    }

    pub fn sample(&self) -> Option<T> {
        let sample = AnimationSample {
            track: &self.track,
            elapsed: self.elapsed,
            delay: self.delay,
            duration: self.duration,
            iterations: self.iterations,
            direction: self.direction,
            fill_mode: self.fill_mode,
            state: self.state,
        };
        sample_animation(sample, None)
    }

    pub fn pause(&mut self) {
        if matches!(self.state, PlaybackState::Pending | PlaybackState::Running) {
            self.state = PlaybackState::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.state == PlaybackState::Paused {
            self.refresh_state();
        }
    }

    pub fn cancel(&mut self) {
        if self.state != PlaybackState::Finished {
            self.state = PlaybackState::Cancelled;
        }
    }

    pub fn restart(&mut self) {
        self.elapsed = Duration::ZERO;
        self.cursor.reset();
        self.state = PlaybackState::Pending;
    }

    fn refresh_state(&mut self) {
        if self.elapsed < self.delay {
            self.state = PlaybackState::Pending;
            return;
        }
        let active = self.elapsed.saturating_sub(self.delay);
        if let IterationCount::Count(count) = self.iterations {
            let total = self.duration.as_nanos().saturating_mul(u128::from(count));
            if active.as_nanos() >= total {
                self.state = PlaybackState::Finished;
                return;
            }
        }
        self.state = PlaybackState::Running;
    }
}

struct AnimationSample<'a, T: crate::Interpolate> {
    track: &'a KeyframeTrack<T>,
    elapsed: Duration,
    delay: Duration,
    duration: Duration,
    iterations: IterationCount,
    direction: Direction,
    fill_mode: FillMode,
    state: PlaybackState,
}

#[inline]
fn sample_animation<T: crate::Interpolate>(
    sample: AnimationSample<'_, T>,
    cursor: Option<&mut TrackCursor>,
) -> Option<T> {
    let track = sample.track;
    if sample.elapsed < sample.delay {
        if !matches!(sample.fill_mode, FillMode::Backwards | FillMode::Both) {
            return None;
        }
        let reverse = matches!(
            sample.direction,
            Direction::Reverse | Direction::AlternateReverse
        );
        return Some(if reverse {
            track.keyframes()[track.len() - 1].value.clone()
        } else {
            track.keyframes()[0].value.clone()
        });
    }

    let active = sample.elapsed.saturating_sub(sample.delay);
    let duration_nanos = sample.duration.as_nanos();
    let active_nanos = active.as_nanos();
    let finished = sample.state == PlaybackState::Finished
        || matches!(
            sample.iterations,
            IterationCount::Count(count)
                if active_nanos >= duration_nanos.saturating_mul(u128::from(count))
        );
    if finished && !matches!(sample.fill_mode, FillMode::Forwards | FillMode::Both) {
        return None;
    }

    let (iteration, within) = if finished {
        let count = match sample.iterations {
            IterationCount::Once => 1_u128,
            IterationCount::Count(count) => u128::from(count),
            IterationCount::Infinite => 1_u128,
        };
        (count.saturating_sub(1), duration_nanos)
    } else {
        (active_nanos / duration_nanos, active_nanos % duration_nanos)
    };
    let reverse = match sample.direction {
        Direction::Normal => false,
        Direction::Reverse => true,
        Direction::Alternate => iteration % 2 == 1,
        Direction::AlternateReverse => iteration % 2 == 0,
    };
    let progress = if finished {
        1.0
    } else {
        within as f64 / duration_nanos as f64
    };
    let progress = if reverse { 1.0 - progress } else { progress };
    let first = track.start_time();
    let span = track.end_time().saturating_sub(first).as_nanos() as f64;
    let offset_nanos = (span * progress.clamp(0.0, 1.0)).round() as u128;
    let offset = Duration::new(
        (offset_nanos / 1_000_000_000).min(u128::from(u64::MAX)) as u64,
        (offset_nanos % 1_000_000_000) as u32,
    );
    let sample_at = first.saturating_add(offset);
    Some(match cursor {
        Some(cursor) => track.sample_with_cursor(sample_at, cursor),
        None => track.sample(sample_at),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Easing, Keyframe};

    fn sample_track() -> KeyframeTrack<f64> {
        KeyframeTrack::new(vec![
            Keyframe::new(Duration::ZERO, 0.0, Easing::Linear),
            Keyframe::new(Duration::from_millis(100), 1.0, Easing::Linear),
        ])
        .unwrap()
    }

    #[test]
    fn repeats_and_alternates_using_cached_cursor() {
        let mut animation = Animation::new(
            sample_track(),
            Duration::from_millis(100),
            Duration::ZERO,
            IterationCount::Count(2),
            Direction::Alternate,
            FillMode::Both,
        )
        .unwrap();
        assert_eq!(animation.advance(Duration::from_millis(50)), Some(0.5));
        assert_eq!(animation.advance(Duration::from_millis(50)), Some(1.0));
        assert_eq!(animation.advance(Duration::from_millis(50)), Some(0.5));
        assert_eq!(animation.advance(Duration::from_millis(50)), Some(0.0));
        assert_eq!(animation.state(), PlaybackState::Finished);
    }

    #[test]
    fn reverse_direction_and_backwards_fill_are_applied() {
        let mut animation = Animation::new(
            sample_track(),
            Duration::from_millis(100),
            Duration::from_millis(30),
            IterationCount::Once,
            Direction::Reverse,
            FillMode::Both,
        )
        .unwrap();
        assert_eq!(animation.sample(), Some(1.0));
        assert_eq!(animation.advance(Duration::from_millis(80)), Some(0.5));
        assert_eq!(animation.advance(Duration::from_millis(50)), Some(0.0));
    }

    #[test]
    fn pause_cancel_restart_and_limits_are_explicit() {
        let mut animation = Animation::new(
            sample_track(),
            Duration::from_millis(100),
            Duration::ZERO,
            IterationCount::Once,
            Direction::Normal,
            FillMode::Both,
        )
        .unwrap();
        animation.advance(Duration::from_millis(20));
        animation.pause();
        assert_eq!(animation.advance(Duration::from_millis(50)), Some(0.2));
        animation.resume();
        animation.cancel();
        assert_eq!(animation.state(), PlaybackState::Cancelled);
        animation.restart();
        assert_eq!(animation.sample(), Some(0.0));
        assert_eq!(
            Animation::new(
                sample_track(),
                Duration::ZERO,
                Duration::ZERO,
                IterationCount::Once,
                Direction::Normal,
                FillMode::Both,
            )
            .err(),
            Some(AnimationError::InvalidDuration)
        );
    }
}
