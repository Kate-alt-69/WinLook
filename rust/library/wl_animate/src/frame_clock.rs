use std::fmt;
use std::time::Duration;

/// Timestamp and delta emitted by the one host-owned application frame clock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameTick {
    /// Monotonic time supplied by the host.
    pub timestamp: Duration,
    /// Time since the previous emitted frame, capped by the clock's maximum delta.
    pub delta: Duration,
    /// Monotonically increasing frame number for this clock instance.
    pub frame_index: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameClockError {
    ZeroMaximumDelta,
}

impl fmt::Display for FrameClockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroMaximumDelta => f.write_str("maximum frame delta must be greater than zero"),
        }
    }
}

impl std::error::Error for FrameClockError {}

/// Lightweight frame clock driven by the host's existing event/composition loop.
///
/// It never sleeps, spins, creates a timer or starts a thread. The host supplies
/// monotonic timestamps and should share each returned delta across every
/// animation in its scene. The default interval targets 60 FPS and clamps long
/// pauses to 100 ms to avoid a burst of catch-up work after the app is hidden.
#[derive(Clone, Debug)]
pub struct FrameClock {
    target_interval: Duration,
    maximum_delta: Duration,
    last_timestamp: Option<Duration>,
    frame_index: u64,
    paused: bool,
}

impl FrameClock {
    pub const SIXTY_FPS_INTERVAL: Duration = Duration::from_nanos(16_666_667);
    pub const DEFAULT_MAXIMUM_DELTA: Duration = Duration::from_millis(100);

    pub fn new(
        target_interval: Duration,
        maximum_delta: Duration,
    ) -> Result<Self, FrameClockError> {
        if maximum_delta.is_zero() {
            return Err(FrameClockError::ZeroMaximumDelta);
        }
        Ok(Self {
            target_interval,
            maximum_delta,
            last_timestamp: None,
            frame_index: 0,
            paused: false,
        })
    }

    pub fn sixty_fps() -> Self {
        Self::new(Self::SIXTY_FPS_INTERVAL, Self::DEFAULT_MAXIMUM_DELTA)
            .expect("the default frame delta is non-zero")
    }

    pub fn target_interval(&self) -> Duration {
        self.target_interval
    }

    pub fn maximum_delta(&self) -> Duration {
        self.maximum_delta
    }

    /// Emits a frame only when the configured interval has elapsed.
    ///
    /// The first call establishes the monotonic baseline and emits a zero-delta
    /// initial tick. A backwards timestamp resets the baseline without emitting
    /// a frame. Long gaps are capped rather than replayed as many missed frames.
    #[inline]
    pub fn tick(&mut self, now: Duration) -> Option<FrameTick> {
        if self.paused {
            return None;
        }

        let Some(last) = self.last_timestamp else {
            self.last_timestamp = Some(now);
            return Some(self.emit(now, Duration::ZERO));
        };

        let Some(elapsed) = now.checked_sub(last) else {
            self.last_timestamp = Some(now);
            return None;
        };
        if elapsed < self.target_interval {
            return None;
        }

        self.last_timestamp = Some(now);
        Some(self.emit(now, elapsed.min(self.maximum_delta)))
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resumes from a fresh baseline, discarding hidden/suspended time.
    pub fn resume(&mut self, now: Duration) {
        self.paused = false;
        self.last_timestamp = Some(now);
    }

    /// Resets time and frame numbering without changing the configured cadence.
    pub fn reset(&mut self) {
        self.last_timestamp = None;
        self.frame_index = 0;
        self.paused = false;
    }

    fn emit(&mut self, timestamp: Duration, delta: Duration) -> FrameTick {
        let tick = FrameTick {
            timestamp,
            delta,
            frame_index: self.frame_index,
        };
        self.frame_index = self.frame_index.saturating_add(1);
        tick
    }
}

impl Default for FrameClock {
    fn default() -> Self {
        Self::sixty_fps()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sixty_fps_clock_throttles_and_emits_monotonic_ticks() {
        let mut clock = FrameClock::sixty_fps();
        let first = clock.tick(Duration::from_secs(1)).unwrap();
        assert_eq!(first.delta, Duration::ZERO);
        assert_eq!(first.frame_index, 0);
        assert_eq!(clock.tick(Duration::from_millis(1010)), None);

        let second = clock.tick(Duration::from_millis(1017)).unwrap();
        assert_eq!(second.delta, Duration::from_millis(17));
        assert_eq!(second.frame_index, 1);
    }

    #[test]
    fn long_gaps_are_clamped_and_resume_discards_hidden_time() {
        let mut clock = FrameClock::new(Duration::from_millis(10), Duration::from_millis(50))
            .unwrap();
        clock.tick(Duration::from_secs(1)).unwrap();
        let delayed = clock.tick(Duration::from_secs(2)).unwrap();
        assert_eq!(delayed.delta, Duration::from_millis(50));

        clock.pause();
        assert_eq!(clock.tick(Duration::from_secs(3)), None);
        clock.resume(Duration::from_secs(10));
        assert_eq!(clock.tick(Duration::from_millis(10_005)), None);
        let resumed = clock.tick(Duration::from_millis(10_010)).unwrap();
        assert_eq!(resumed.delta, Duration::from_millis(10));
    }

    #[test]
    fn backwards_timestamps_reset_the_baseline_without_a_spike() {
        let mut clock = FrameClock::new(Duration::from_millis(10), Duration::from_millis(100))
            .unwrap();
        clock.tick(Duration::from_millis(100)).unwrap();
        assert_eq!(clock.tick(Duration::from_millis(90)), None);
        let next = clock.tick(Duration::from_millis(100)).unwrap();
        assert_eq!(next.delta, Duration::from_millis(10));
    }

    #[test]
    fn zero_maximum_delta_is_rejected() {
        assert_eq!(
            FrameClock::new(Duration::ZERO, Duration::ZERO).err(),
            Some(FrameClockError::ZeroMaximumDelta)
        );
    }
}
