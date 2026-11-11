use crate::easing::Easing;
use crate::values::Interpolate;
use std::fmt;
use std::time::Duration;

/// Timestamped value; easing controls interpolation from this frame to the next.
#[derive(Clone, Debug)]
pub struct Keyframe<T> {
    /// Timestamp relative to the start of the track.
    pub at: Duration,
    /// Value stored at this timestamp.
    pub value: T,
    /// Curve used to reach the next keyframe.
    pub easing_to_next: Easing,
}

impl<T> Keyframe<T> {
    /// Creates a keyframe.
    pub fn new(at: Duration, value: T, easing_to_next: Easing) -> Self {
        Self {
            at,
            value,
            easing_to_next,
        }
    }
}

/// Errors returned when constructing an immutable keyframe track.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackError {
    /// A track needs at least one keyframe.
    Empty,
    /// Keyframe timestamps must be strictly increasing.
    KeyframesMustBeStrictlyIncreasing,
}

impl fmt::Display for TrackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("a keyframe track cannot be empty"),
            Self::KeyframesMustBeStrictlyIncreasing => {
                f.write_str("keyframe timestamps must be strictly increasing")
            }
        }
    }
}

impl std::error::Error for TrackError {}

/// Immutable keyframe track. Sampling does not allocate internally.
#[derive(Clone, Debug)]
pub struct KeyframeTrack<T: Interpolate> {
    keyframes: Vec<Keyframe<T>>,
}

impl<T: Interpolate> KeyframeTrack<T> {
    /// Creates a track after validating that it contains ordered keyframes.
    pub fn new(keyframes: Vec<Keyframe<T>>) -> Result<Self, TrackError> {
        if keyframes.is_empty() {
            return Err(TrackError::Empty);
        }
        if keyframes.windows(2).any(|pair| pair[0].at >= pair[1].at) {
            return Err(TrackError::KeyframesMustBeStrictlyIncreasing);
        }

        Ok(Self { keyframes })
    }

    /// Returns the borrowed keyframe slice.
    pub fn keyframes(&self) -> &[Keyframe<T>] {
        &self.keyframes
    }

    /// First keyframe timestamp.
    pub fn start_time(&self) -> Duration {
        self.keyframes[0].at
    }

    /// Last keyframe timestamp.
    pub fn end_time(&self) -> Duration {
        self.keyframes[self.keyframes.len() - 1].at
    }

    /// Number of keyframes in the track.
    pub fn len(&self) -> usize {
        self.keyframes.len()
    }

    /// Whether the track has no keyframes (always false for a valid track).
    pub fn is_empty(&self) -> bool {
        self.keyframes.is_empty()
    }

    /// Samples the track. Times outside its range clamp to the nearest endpoint.
    pub fn sample(&self, at: Duration) -> T {
        if at <= self.keyframes[0].at {
            return self.keyframes[0].value.clone();
        }

        let last = self.keyframes.len() - 1;
        if at >= self.keyframes[last].at {
            return self.keyframes[last].value.clone();
        }

        let right_index = match self.keyframes.binary_search_by_key(&at, |frame| frame.at) {
            Ok(index) => return self.keyframes[index].value.clone(),
            Err(index) => index,
        };
        let left = &self.keyframes[right_index - 1];
        let right = &self.keyframes[right_index];
        let span = (right.at - left.at).as_secs_f64();
        let offset = (at - left.at).as_secs_f64();
        let progress = if span <= 0.0 { 1.0 } else { offset / span };

        left.value
            .interpolate(&right.value, left.easing_to_next.sample(progress))
    }
}
