//! Caller-fed system pressure policy for WLDE playback decisions.
//!
//! This module consumes telemetry supplied by a platform adapter. It does not
//! sample the operating system, create a polling thread or infer unavailable
//! utilization metrics.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ResourcePressure {
    #[default]
    Normal,
    Optimize,
    Suspend,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResourceSample {
    pub cpu_percent: u8,
    pub gpu_percent: u8,
    pub ram_percent: u8,
    pub vram_percent: u8,
}

impl ResourceSample {
    fn peak_percent(self) -> Result<u8, ResourcePolicyError> {
        if self.cpu_percent > 100
            || self.gpu_percent > 100
            || self.ram_percent > 100
            || self.vram_percent > 100
        {
            return Err(ResourcePolicyError::InvalidSample);
        }
        Ok(self
            .cpu_percent
            .max(self.gpu_percent)
            .max(self.ram_percent)
            .max(self.vram_percent))
    }
}

/// Explicit thresholds and consecutive sample windows used by the controller.
/// The maximum of CPU, GPU, RAM and VRAM is used; no averaging hides a saturated domain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourcePolicy {
    optimize_at_percent: u8,
    suspend_at_percent: u8,
    recovery_below_percent: u8,
    enter_windows: u16,
    recovery_windows: u16,
}

impl ResourcePolicy {
    pub fn new(
        optimize_at_percent: u8,
        suspend_at_percent: u8,
        recovery_below_percent: u8,
        enter_windows: u16,
        recovery_windows: u16,
    ) -> Result<Self, ResourcePolicyError> {
        if recovery_below_percent >= optimize_at_percent
            || optimize_at_percent >= suspend_at_percent
            || suspend_at_percent > 100
        {
            return Err(ResourcePolicyError::InvalidThresholds);
        }
        if enter_windows == 0 || recovery_windows == 0 {
            return Err(ResourcePolicyError::InvalidWindowCount);
        }
        Ok(Self {
            optimize_at_percent,
            suspend_at_percent,
            recovery_below_percent,
            enter_windows,
            recovery_windows,
        })
    }

    pub const fn optimize_at_percent(self) -> u8 {
        self.optimize_at_percent
    }
    pub const fn suspend_at_percent(self) -> u8 {
        self.suspend_at_percent
    }
    pub const fn recovery_below_percent(self) -> u8 {
        self.recovery_below_percent
    }
    pub const fn enter_windows(self) -> u16 {
        self.enter_windows
    }
    pub const fn recovery_windows(self) -> u16 {
        self.recovery_windows
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourcePolicyError {
    InvalidThresholds,
    InvalidWindowCount,
    InvalidSample,
}

impl std::fmt::Display for ResourcePolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidThresholds => {
                f.write_str("resource thresholds must satisfy recovery < optimize < suspend <= 100")
            }
            Self::InvalidWindowCount => {
                f.write_str("resource enter and recovery windows must be non-zero")
            }
            Self::InvalidSample => {
                f.write_str("resource utilization percentages must be between 0 and 100")
            }
        }
    }
}

impl std::error::Error for ResourcePolicyError {}

/// Allocation-free hysteresis controller for caller-supplied telemetry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceGovernor {
    policy: ResourcePolicy,
    pressure: ResourcePressure,
    optimize_streak: u16,
    suspend_streak: u16,
    recovery_streak: u16,
}

impl ResourceGovernor {
    pub const fn new(policy: ResourcePolicy) -> Self {
        Self {
            policy,
            pressure: ResourcePressure::Normal,
            optimize_streak: 0,
            suspend_streak: 0,
            recovery_streak: 0,
        }
    }

    pub const fn pressure(&self) -> ResourcePressure {
        self.pressure
    }
    pub const fn policy(&self) -> ResourcePolicy {
        self.policy
    }

    /// Apply one telemetry window and return the resulting pressure state.
    pub fn update(
        &mut self,
        sample: ResourceSample,
    ) -> Result<ResourcePressure, ResourcePolicyError> {
        let peak = sample.peak_percent()?;

        if self.pressure == ResourcePressure::Suspend {
            if peak < self.policy.recovery_below_percent {
                if advance_streak(&mut self.recovery_streak, self.policy.recovery_windows) {
                    self.pressure = ResourcePressure::Normal;
                    self.reset_streaks();
                }
            } else {
                self.recovery_streak = 0;
            }
            return Ok(self.pressure);
        }

        if self.pressure == ResourcePressure::Optimize {
            if peak < self.policy.recovery_below_percent {
                if advance_streak(&mut self.recovery_streak, self.policy.recovery_windows) {
                    self.pressure = ResourcePressure::Normal;
                    self.reset_streaks();
                    return Ok(self.pressure);
                }
            } else {
                self.recovery_streak = 0;
            }
        } else {
            self.recovery_streak = 0;
        }

        if peak >= self.policy.suspend_at_percent {
            self.suspend_streak =
                advance_streak_value(self.suspend_streak, self.policy.enter_windows);
            self.optimize_streak = 0;
            if self.suspend_streak >= self.policy.enter_windows {
                self.pressure = ResourcePressure::Suspend;
                self.reset_streaks();
            }
        } else if peak >= self.policy.optimize_at_percent {
            self.optimize_streak =
                advance_streak_value(self.optimize_streak, self.policy.enter_windows);
            self.suspend_streak = 0;
            if self.optimize_streak >= self.policy.enter_windows {
                self.pressure = ResourcePressure::Optimize;
                self.optimize_streak = 0;
            }
        } else {
            self.optimize_streak = 0;
            self.suspend_streak = 0;
        }

        Ok(self.pressure)
    }

    fn reset_streaks(&mut self) {
        self.optimize_streak = 0;
        self.suspend_streak = 0;
        self.recovery_streak = 0;
    }
}

fn advance_streak(counter: &mut u16, required: u16) -> bool {
    *counter = advance_streak_value(*counter, required);
    *counter >= required
}

fn advance_streak_value(counter: u16, required: u16) -> u16 {
    if counter >= required {
        required
    } else {
        counter + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> ResourcePolicy {
        ResourcePolicy::new(75, 90, 65, 3, 4).unwrap()
    }

    fn sample(cpu: u8, gpu: u8, ram: u8, vram: u8) -> ResourceSample {
        ResourceSample {
            cpu_percent: cpu,
            gpu_percent: gpu,
            ram_percent: ram,
            vram_percent: vram,
        }
    }

    #[test]
    fn policy_rejects_ambiguous_thresholds_and_empty_windows() {
        assert_eq!(
            ResourcePolicy::new(90, 80, 70, 3, 4),
            Err(ResourcePolicyError::InvalidThresholds)
        );
        assert_eq!(
            ResourcePolicy::new(75, 90, 65, 0, 4),
            Err(ResourcePolicyError::InvalidWindowCount)
        );
        assert_eq!(
            ResourcePolicy::new(75, 90, 65, 3, 0),
            Err(ResourcePolicyError::InvalidWindowCount)
        );
    }

    #[test]
    fn each_resource_domain_is_checked_without_averaging() {
        let mut governor = ResourceGovernor::new(policy());
        let high_ram = sample(12, 20, 91, 10);
        assert_eq!(governor.update(high_ram), Ok(ResourcePressure::Normal));
        assert_eq!(governor.update(high_ram), Ok(ResourcePressure::Normal));
        assert_eq!(governor.update(high_ram), Ok(ResourcePressure::Suspend));
    }

    #[test]
    fn optimize_and_suspend_require_consecutive_windows() {
        let mut governor = ResourceGovernor::new(policy());
        let optimize = sample(80, 20, 30, 10);
        assert_eq!(governor.update(optimize), Ok(ResourcePressure::Normal));
        assert_eq!(
            governor.update(sample(10, 10, 10, 10)),
            Ok(ResourcePressure::Normal)
        );
        assert_eq!(governor.update(optimize), Ok(ResourcePressure::Normal));
        assert_eq!(governor.update(optimize), Ok(ResourcePressure::Normal));
        assert_eq!(governor.update(optimize), Ok(ResourcePressure::Optimize));

        let critical = sample(30, 20, 20, 95);
        assert_eq!(governor.update(critical), Ok(ResourcePressure::Optimize));
        assert_eq!(governor.update(critical), Ok(ResourcePressure::Optimize));
        assert_eq!(governor.update(critical), Ok(ResourcePressure::Suspend));
    }

    #[test]
    fn recovery_requires_all_domains_below_recovery_threshold() {
        let mut governor = ResourceGovernor::new(policy());
        let critical = sample(20, 95, 20, 20);
        for _ in 0..3 {
            governor.update(critical).unwrap();
        }
        assert_eq!(governor.pressure(), ResourcePressure::Suspend);

        let recovered = sample(40, 50, 60, 64);
        for _ in 0..3 {
            assert_eq!(governor.update(recovered), Ok(ResourcePressure::Suspend));
        }
        assert_eq!(governor.update(recovered), Ok(ResourcePressure::Normal));

        for _ in 0..3 {
            governor.update(sample(80, 10, 10, 10)).unwrap();
        }
        assert_eq!(governor.pressure(), ResourcePressure::Optimize);
        for _ in 0..3 {
            governor.update(sample(60, 60, 60, 60)).unwrap();
        }
        assert_eq!(governor.pressure(), ResourcePressure::Optimize);
        assert_eq!(
            governor.update(sample(60, 60, 60, 60)),
            Ok(ResourcePressure::Normal)
        );
    }

    #[test]
    fn invalid_sample_does_not_mutate_controller_state() {
        let mut governor = ResourceGovernor::new(policy());
        governor.update(sample(80, 0, 0, 0)).unwrap();
        let before = governor;
        assert_eq!(
            governor.update(sample(10, 101, 10, 10)),
            Err(ResourcePolicyError::InvalidSample)
        );
        assert_eq!(governor, before);
    }
}
