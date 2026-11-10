use crate::easing::{clamp_progress, Easing};
use crate::values::Interpolate;
use std::time::Duration;

/// Playback state for a tween or track controller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackState {
    Pending,
    Running,
    Paused,
    Finished,
    Cancelled,
}

/// Single-property animation. The host frame loop supplies elapsed time.
///
/// `elapsed()` includes the configured delay. No timer or worker is created.
#[derive(Clone, Debug)]
pub struct Tween<T: Interpolate> {
    from: T,
    to: T,
    duration: Duration,
    delay: Duration,
    elapsed: Duration,
    easing: Easing,
    state: PlaybackState,
}

impl<T: Interpolate> Tween<T> {
    /// Creates a tween that begins on the first host-driven advance.
    pub fn new(from: T, to: T, duration: Duration, easing: Easing) -> Self {
        Self::new_delayed(from, to, duration, Duration::ZERO, easing)
    }

    /// Creates a tween with a delay before interpolation begins.
    pub fn new_delayed(
        from: T,
        to: T,
        duration: Duration,
        delay: Duration,
        easing: Easing,
    ) -> Self {
        let state = if duration.is_zero() && delay.is_zero() {
            PlaybackState::Finished
        } else {
            PlaybackState::Pending
        };

        Self {
            from,
            to,
            duration,
            delay,
            elapsed: Duration::ZERO,
            easing,
            state,
        }
    }

    /// Current playback state.
    pub fn state(&self) -> PlaybackState {
        self.state
    }

    /// Active interpolation duration, excluding delay.
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Start delay before interpolation begins.
    pub fn delay(&self) -> Duration {
        self.delay
    }

    /// Total elapsed host time, including the delay.
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// Whether the tween has reached its final value.
    pub fn is_finished(&self) -> bool {
        self.state == PlaybackState::Finished
    }

    /// Whether the tween has been cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.state == PlaybackState::Cancelled
    }

    /// Normalized interpolation progress, excluding the start delay.
    pub fn progress(&self) -> f64 {
        if self.elapsed < self.delay {
            return 0.0;
        }
        if self.duration.is_zero() {
            return 1.0;
        }

        let active_elapsed = self.elapsed.saturating_sub(self.delay);
        clamp_progress(active_elapsed.as_secs_f64() / self.duration.as_secs_f64())
    }

    /// Samples the current value without changing playback state.
    pub fn value(&self) -> T {
        if self.state == PlaybackState::Finished {
            return self.to.clone();
        }
        if self.elapsed < self.delay || self.duration.is_zero() {
            return self.from.clone();
        }

        self.from
            .interpolate(&self.to, self.easing.sample(self.progress()))
    }

    /// Advances by a non-negative host-provided time delta and returns the value.
    pub fn advance(&mut self, delta: Duration) -> T {
        if self.state == PlaybackState::Pending || self.state == PlaybackState::Running {
            self.elapsed = self.elapsed.saturating_add(delta);
            self.refresh_state();
        }

        self.value()
    }

    /// Pauses a pending or running tween, including one waiting on its delay.
    pub fn pause(&mut self) {
        if self.state == PlaybackState::Pending || self.state == PlaybackState::Running {
            self.state = PlaybackState::Paused;
        }
    }

    /// Resumes a paused tween without discarding its elapsed time.
    pub fn resume(&mut self) {
        if self.state == PlaybackState::Paused {
            self.refresh_state();
        }
    }

    /// Cancels a tween while preserving its current sampled value.
    pub fn cancel(&mut self) {
        if self.state != PlaybackState::Finished {
            self.state = PlaybackState::Cancelled;
        }
    }

    /// Restarts the tween and its delay from the beginning.
    pub fn restart(&mut self) {
        self.elapsed = Duration::ZERO;
        self.state = if self.duration.is_zero() && self.delay.is_zero() {
            PlaybackState::Finished
        } else {
            PlaybackState::Pending
        };
    }

    fn refresh_state(&mut self) {
        if self.elapsed < self.delay {
            self.state = PlaybackState::Pending;
        } else if self.duration.is_zero()
            || self.elapsed.saturating_sub(self.delay) >= self.duration
        {
            self.state = PlaybackState::Finished;
        } else {
            self.state = PlaybackState::Running;
        }
    }
}
