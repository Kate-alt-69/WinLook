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
/// monotonic timestamps and shares each returned delta across every animation
/// in its scene. A fractional cadence accumulator keeps 60 FPS average pacing
/// when the host polls faster at rates such as 90 or 144 Hz. The default clock
/// clamps long stalls to 100 ms and never emits catch-up bursts.
#[derive(Clone, Debug)]
pub struct FrameClock {
    target_interval: Duration,
    maximum_delta: Duration,
    last_timestamp: Option<Duration>,
    last_frame_timestamp: Option<Duration>,
    phase_elapsed: Duration,
    frame_index: u64,
    paused: bool,
}

impl FrameClock {
    // Slightly below the mathematical period avoids dropping a frame because
    // integer nanosecond host timestamps round 1/60 second down on some frames.
    pub const SIXTY_FPS_INTERVAL: Duration = Duration::from_nanos(16_666_665);
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
            last_frame_timestamp: None,
            phase_elapsed: Duration::ZERO,
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

    /// Emits a frame when accumulated host time reaches the target cadence.
    ///
    /// The first call establishes the time baseline and emits a zero-delta
    /// initial tick. Fractional remainder is retained, so a 90 Hz host can
    /// alternate between one- and two-poll intervals while averaging 60 FPS.
    /// Backwards timestamps reset the baseline. Long stalls are capped once,
    /// then their remainder is discarded instead of replayed as catch-up work.
    #[inline]
    pub fn tick(&mut self, now: Duration) -> Option<FrameTick> {
        if self.paused {
            return None;
        }

        let last_poll = match self.last_timestamp {
            Some(last) => last,
            None => {
                self.last_timestamp = Some(now);
                self.last_frame_timestamp = Some(now);
                self.phase_elapsed = Duration::ZERO;
                return Some(self.emit(now, Duration::ZERO));
            }
        };

        let poll_delta = match now.checked_sub(last_poll) {
            Some(elapsed) => elapsed,
            None => {
                self.reset_baseline(now);
                return None;
            }
        };
        self.last_timestamp = Some(now);

        let last_frame = self.last_frame_timestamp.unwrap_or(now);
        let frame_delta = match now.checked_sub(last_frame) {
            Some(elapsed) => elapsed,
            None => {
                self.reset_baseline(now);
                return None;
            }
        };
        let stalled = poll_delta > self.maximum_delta && poll_delta >= self.target_interval;

        let accumulated = if stalled {
            poll_delta
        } else {
            self.phase_elapsed.saturating_add(poll_delta)
        };

        if self.target_interval.is_zero() {
            self.phase_elapsed = Duration::ZERO;
        } else {
            let interval_nanos = self.target_interval.as_nanos();
            let accumulated_nanos = accumulated.as_nanos();
            if accumulated_nanos < interval_nanos {
                self.phase_elapsed = accumulated;
                return None;
            }

            let remainder = if stalled {
                0
            } else {
                let remainder = accumulated_nanos - interval_nanos;
                if remainder >= interval_nanos {
                    0
                } else {
                    remainder
                }
            };
            self.phase_elapsed = duration_from_nanos(remainder);
        }

        self.last_frame_timestamp = Some(now);
        Some(self.emit(now, frame_delta.min(self.maximum_delta)))
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resumes from a fresh baseline, discarding hidden/suspended time.
    pub fn resume(&mut self, now: Duration) {
        self.paused = false;
        self.reset_baseline(now);
    }

    /// Resets time and frame numbering without changing the configured cadence.
    pub fn reset(&mut self) {
        self.last_timestamp = None;
        self.last_frame_timestamp = None;
        self.phase_elapsed = Duration::ZERO;
        self.frame_index = 0;
        self.paused = false;
    }

    fn reset_baseline(&mut self, now: Duration) {
        self.last_timestamp = Some(now);
        self.last_frame_timestamp = Some(now);
        self.phase_elapsed = Duration::ZERO;
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

fn duration_from_nanos(value: u128) -> Duration {
    Duration::new(
        (value / 1_000_000_000).min(u128::from(u64::MAX)) as u64,
        (value % 1_000_000_000) as u32,
    )
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
    fn fractional_cadence_averages_sixty_fps_on_ninety_and_144_hz_hosts() {
        let mut ninety_hz = FrameClock::sixty_fps();
        let mut now = Duration::ZERO;
        ninety_hz.tick(now).unwrap();
        let mut emitted = 0;
        for _ in 0..90 {
            now = now.saturating_add(Duration::from_nanos(11_111_111));
            if ninety_hz.tick(now).is_some() {
                emitted += 1;
            }
        }
        assert_eq!(emitted, 60);

        let mut one_forty_four_hz = FrameClock::sixty_fps();
        now = Duration::ZERO;
        one_forty_four_hz.tick(now).unwrap();
        emitted = 0;
        for _ in 0..144 {
            now = now.saturating_add(Duration::from_nanos(6_944_444));
            if one_forty_four_hz.tick(now).is_some() {
                emitted += 1;
            }
        }
        assert_eq!(emitted, 60);
    }

    #[test]
    fn long_gaps_are_clamped_and_resume_discards_hidden_time() {
        let mut clock =
            FrameClock::new(Duration::from_millis(10), Duration::from_millis(50)).unwrap();
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
    fn small_delta_cap_does_not_starve_a_slower_frame_cadence() {
        let mut clock =
            FrameClock::new(Duration::from_millis(16), Duration::from_millis(5)).unwrap();
        clock.tick(Duration::ZERO).unwrap();
        assert_eq!(clock.tick(Duration::from_millis(10)), None);
        let first = clock.tick(Duration::from_millis(20)).unwrap();
        assert_eq!(first.delta, Duration::from_millis(5));
        assert_eq!(clock.tick(Duration::from_millis(30)), None);
        let second = clock.tick(Duration::from_millis(40)).unwrap();
        assert_eq!(second.delta, Duration::from_millis(5));
    }

    #[test]
    fn backwards_timestamps_reset_the_baseline_without_a_spike() {
        let mut clock =
            FrameClock::new(Duration::from_millis(10), Duration::from_millis(100)).unwrap();
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
