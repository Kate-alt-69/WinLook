/// Built-in easing curves shared conceptually with `wl_Lanimate`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Easing {
    /// Constant-speed interpolation.
    Linear,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
}

impl Default for Easing {
    fn default() -> Self {
        Self::Linear
    }
}

impl Easing {
    /// Maps normalized progress to eased progress.
    ///
    /// The input is clamped to `0..=1`. NaN is treated as zero progress.
    pub fn sample(self, progress: f64) -> f64 {
        let t = clamp_progress(progress);
        let inverse = 1.0 - t;

        match self {
            Self::Linear => t,
            Self::EaseInQuad => t * t,
            Self::EaseOutQuad => 1.0 - inverse * inverse,
            Self::EaseInOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Self::EaseInCubic => t * t * t,
            Self::EaseOutCubic => 1.0 - inverse.powi(3),
            Self::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            // Polynomial sine avoids transcendental calls in the 60 FPS hot path.
            Self::EaseInSine => {
                let value = fast_sin(t * std::f64::consts::FRAC_PI_4);
                2.0 * value * value
            }
            Self::EaseOutSine => fast_sin(t * std::f64::consts::FRAC_PI_2),
            Self::EaseInOutSine => {
                let value = fast_sin(t * std::f64::consts::FRAC_PI_2);
                value * value
            }
            Self::EaseInQuart => t.powi(4),
            Self::EaseOutQuart => 1.0 - inverse.powi(4),
            Self::EaseInOutQuart => {
                if t < 0.5 {
                    8.0 * t.powi(4)
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(4) / 2.0
                }
            }
        }
    }
}

pub(crate) fn clamp_progress(progress: f64) -> f64 {
    if progress.is_nan() {
        0.0
    } else {
        progress.clamp(0.0, 1.0)
    }
}

/// Degree-15 sine approximation on [0, pi/2], using only multiply/add.
#[inline]
fn fast_sin(value: f64) -> f64 {
    let square = value * value;
    value
        * (1.0
            + square
                * (-1.0 / 6.0
                    + square
                        * (1.0 / 120.0
                            + square
                                * (-1.0 / 5040.0
                                    + square
                                        * (1.0 / 362880.0
                                            + square
                                                * (-1.0 / 39916800.0
                                                    + square
                                                        * (1.0 / 6227020800.0
                                                            - square / 1307674368000.0)))))))
}
