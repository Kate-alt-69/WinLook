use crate::easing::Easing;
use crate::values::Interpolate;
use std::time::Duration;

/// Timestamped value; easing controls interpolation to the next keyframe.
#[derive(Clone, Debug)]
pub struct Keyframe<T> {
    pub at: Duration,
    pub value: T,
    pub easing_to_next: Easing,
}

impl<T> Keyframe<T> {
    pub fn new(at: Duration, value: T, easing_to_next: Easing) -> Self {
        Self { at, value, easing_to_next }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackError {
    Empty,
    KeyframesMustBeStrictlyIncreasing,
}

/// Immutable keyframe track. Sampling performs no heap allocation.
#[derive(Clone, Debug)]
pub struct KeyframeTrack<T: Interpolate> {
    keyframes: Vec<Keyframe<T>>,
}

impl<T: Interpolate> KeyframeTrack<T> {
    pub fn new(keyframes: Vec<Keyframe<T>>) -> Result<Self, TrackError> {
        if keyframes.is_empty() { return Err(TrackError::Empty); }
        if keyframes.windows(2).any(|pair| pair[0].at >= pair[1].at) {
            return Err(TrackError::KeyframesMustBeStrictlyIncreasing);
        }
        Ok(Self { keyframes })
    }

    pub fn keyframes(&self) -> &[Keyframe<T>] { &self.keyframes }
    pub fn start_time(&self) -> Duration { self.keyframes[0].at }
    pub fn end_time(&self) -> Duration { self.keyframes[self.keyframes.len() - 1].at }

    pub fn sample(&self, at: Duration) -> T {
        if at <= self.keyframes[0].at { return self.keyframes[0].value.clone(); }
        let last = self.keyframes.len() - 1;
        if at >= self.keyframes[last].at { return self.keyframes[last].value.clone(); }

        for pair in self.keyframes.windows(2) {
            let left = &pair[0];
            let right = &pair[1];
            if at <= right.at {
                let span = (right.at - left.at).as_secs_f64();
                let offset = (at - left.at).as_secs_f64();
                let progress = if span <= 0.0 { 1.0 } else { offset / span };
                return left.value.interpolate(&right.value, left.easing_to_next.sample(progress));
            }
        }
        self.keyframes[last].value.clone()
    }
}
