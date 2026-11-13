use crate::easing::Easing;
use crate::values::Interpolate;
use std::fmt;
use std::time::Duration;

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

/// Mutable lookup hint owned by one playback instance, never shared by tracks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TrackCursor {
    segment_index: usize,
}

impl TrackCursor {
    pub fn reset(&mut self) {
        self.segment_index = 0;
    }

    pub fn segment_index(&self) -> usize {
        self.segment_index
    }
}

/// Immutable keyframe track; sampling never allocates.
#[derive(Clone, Debug)]
pub struct KeyframeTrack<T: Interpolate> {
    keyframes: Vec<Keyframe<T>>,
}

impl<T: Interpolate> KeyframeTrack<T> {
    pub fn new(keyframes: Vec<Keyframe<T>>) -> Result<Self, TrackError> {
        if keyframes.is_empty() {
            return Err(TrackError::Empty);
        }
        if keyframes.windows(2).any(|pair| pair[0].at >= pair[1].at) {
            return Err(TrackError::KeyframesMustBeStrictlyIncreasing);
        }
        Ok(Self { keyframes })
    }

    pub fn keyframes(&self) -> &[Keyframe<T>] { &self.keyframes }
    pub fn start_time(&self) -> Duration { self.keyframes[0].at }
    pub fn end_time(&self) -> Duration { self.keyframes[self.keyframes.len() - 1].at }
    pub fn len(&self) -> usize { self.keyframes.len() }
    pub fn is_empty(&self) -> bool { self.keyframes.is_empty() }

    /// Convenience sampler. Retain a cursor for repeated animation playback.
    #[inline]
    pub fn sample(&self, at: Duration) -> T {
        self.sample_with_cursor(at, &mut TrackCursor::default())
    }

    /// Sequential samples in the active segment use O(1) lookup. Reverse seeks
    /// or large jumps use binary search; neither path allocates.
    #[inline]
    pub fn sample_with_cursor(&self, at: Duration, cursor: &mut TrackCursor) -> T {
        let last = self.keyframes.len() - 1;
        if at <= self.keyframes[0].at {
            cursor.segment_index = 0;
            return self.keyframes[0].value.clone();
        }
        if at >= self.keyframes[last].at {
            cursor.segment_index = last.saturating_sub(1);
            return self.keyframes[last].value.clone();
        }

        let mut left_index = cursor.segment_index.min(last - 1);
        let left_at = self.keyframes[left_index].at;
        let right_at = self.keyframes[left_index + 1].at;
        if at < left_at || at > right_at {
            left_index = match self.keyframes.binary_search_by_key(&at, |frame| frame.at) {
                Ok(index) => {
                    cursor.segment_index = index.min(last - 1);
                    return self.keyframes[index].value.clone();
                }
                Err(right_index) => right_index - 1,
            };
        } else if at == right_at {
            left_index += 1;
            if left_index == last {
                cursor.segment_index = last - 1;
                return self.keyframes[last].value.clone();
            }
        }

        cursor.segment_index = left_index;
        let left = &self.keyframes[left_index];
        let right = &self.keyframes[left_index + 1];
        let span = (right.at - left.at).as_nanos() as f64;
        let offset = (at - left.at).as_nanos() as f64;
        let progress = if span <= 0.0 { 1.0 } else { offset / span };
        left.value.interpolate(&right.value, left.easing_to_next.sample(progress))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_track() -> KeyframeTrack<f64> {
        KeyframeTrack::new(vec![
            Keyframe::new(Duration::ZERO, 0.0, Easing::Linear),
            Keyframe::new(Duration::from_millis(100), 10.0, Easing::Linear),
            Keyframe::new(Duration::from_millis(200), 20.0, Easing::Linear),
            Keyframe::new(Duration::from_millis(300), 30.0, Easing::Linear),
        ]).unwrap()
    }

    #[test]
    fn cursor_reuses_forward_segment_and_advances_at_boundaries() {
        let track = sample_track();
        let mut cursor = TrackCursor::default();
        assert_eq!(track.sample_with_cursor(Duration::from_millis(25), &mut cursor), 2.5);
        assert_eq!(track.sample_with_cursor(Duration::from_millis(75), &mut cursor), 7.5);
        assert_eq!(cursor.segment_index(), 0);
        assert_eq!(track.sample_with_cursor(Duration::from_millis(125), &mut cursor), 12.5);
        assert_eq!(cursor.segment_index(), 1);
    }

    #[test]
    fn cursor_binary_searches_reverse_seeks_and_clamps() {
        let track = sample_track();
        let mut cursor = TrackCursor::default();
        assert_eq!(track.sample_with_cursor(Duration::from_millis(275), &mut cursor), 27.5);
        assert_eq!(track.sample_with_cursor(Duration::from_millis(50), &mut cursor), 5.0);
        assert_eq!(cursor.segment_index(), 0);
        assert_eq!(track.sample_with_cursor(Duration::from_secs(2), &mut cursor), 30.0);
    }
}
