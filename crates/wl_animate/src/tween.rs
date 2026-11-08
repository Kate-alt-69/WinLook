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
#[derive(Clone, Debug)]
pub struct Tween<T: Interpolate> {
    from: T,
    to: T,
    duration: Duration,
    elapsed: Duration,
    easing: Easing,
    state: PlaybackState,
}

impl<T: Interpolate> Tween<T> {
    pub fn new(from: T, to: T, duration: Duration, easing: Easing) -> Self {
        let state = if duration.is_zero() { PlaybackState::Finished } else { PlaybackState::Pending };
        Self { from, to, duration, elapsed: Duration::ZERO, easing, state }
    }

    pub fn state(&self) -> PlaybackState { self.state }
    pub fn duration(&self) -> Duration { self.duration }
    pub fn elapsed(&self) -> Duration { self.elapsed }
    pub fn is_finished(&self) -> bool { self.state == PlaybackState::Finished }
    pub fn is_cancelled(&self) -> bool { self.state == PlaybackState::Cancelled }

    pub fn progress(&self) -> f64 {
        if self.duration.is_zero() { return 1.0; }
        clamp_progress(self.elapsed.as_secs_f64() / self.duration.as_secs_f64())
    }

    pub fn value(&self) -> T {
        if self.duration.is_zero() || self.state == PlaybackState::Finished {
            return self.to.clone();
        }
        self.from.interpolate(&self.to, self.easing.sample(self.progress()))
    }

    pub fn advance(&mut self, delta: Duration) -> T {
        if self.state == PlaybackState::Pending { self.state = PlaybackState::Running; }
        if self.state == PlaybackState::Running {
            let remaining = self.duration.saturating_sub(self.elapsed);
            if delta >= remaining {
                self.elapsed = self.duration;
                self.state = PlaybackState::Finished;
            } else {
                self.elapsed += delta;
            }
        }
        self.value()
    }

    pub fn pause(&mut self) {
        if self.state == PlaybackState::Running { self.state = PlaybackState::Paused; }
    }

    pub fn resume(&mut self) {
        if self.state == PlaybackState::Paused { self.state = PlaybackState::Running; }
    }

    pub fn cancel(&mut self) {
        if self.state != PlaybackState::Finished { self.state = PlaybackState::Cancelled; }
    }

    pub fn restart(&mut self) {
        self.elapsed = Duration::ZERO;
        self.state = if self.duration.is_zero() { PlaybackState::Finished } else { PlaybackState::Pending };
    }
}
