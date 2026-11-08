/// Built-in easing curves. Custom curves can be added without changing the
/// value interpolation contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Easing {
    Linear,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
}

impl Default for Easing {
    fn default() -> Self { Self::Linear }
}

impl Easing {
    /// Maps normalized progress to eased progress. Input is clamped to 0..=1.
    pub fn sample(self, progress: f64) -> f64 {
        let t = clamp_progress(progress);
        match self {
            Self::Linear => t,
            Self::EaseInQuad => t * t,
            Self::EaseOutQuad => 1.0 - (1.0 - t) * (1.0 - t),
            Self::EaseInOutQuad => {
                if t < 0.5 { 2.0 * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(2) / 2.0 }
            }
            Self::EaseInCubic => t * t * t,
            Self::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
            Self::EaseInOutCubic => {
                if t < 0.5 { 4.0 * t * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0 }
            }
        }
    }
}

pub(crate) fn clamp_progress(progress: f64) -> f64 {
    if progress.is_nan() { 0.0 } else { progress.clamp(0.0, 1.0) }
}
