use crate::easing::clamp_progress;

/// Interpolates a value between `self` and `end` for normalized progress 0..=1.
pub trait Interpolate: Clone {
    fn interpolate(&self, end: &Self, progress: f64) -> Self;
}

impl Interpolate for f64 {
    fn interpolate(&self, end: &Self, progress: f64) -> Self {
        self + (end - self) * clamp_progress(progress)
    }
}

impl Interpolate for f32 {
    fn interpolate(&self, end: &Self, progress: f64) -> Self {
        let progress = clamp_progress(progress) as f32;
        self + (end - self) * progress
    }
}

/// Two-dimensional point/value useful for position and translation tracks.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

impl Point2 {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

impl Interpolate for Point2 {
    fn interpolate(&self, end: &Self, progress: f64) -> Self {
        Self {
            x: self.x.interpolate(&end.x, progress),
            y: self.y.interpolate(&end.y, progress),
        }
    }
}

/// Four-channel floating-point colour. Channels are not implicitly clamped.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rgba {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Rgba {
    pub const fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }
}

impl Interpolate for Rgba {
    fn interpolate(&self, end: &Self, progress: f64) -> Self {
        Self {
            r: self.r.interpolate(&end.r, progress),
            g: self.g.interpolate(&end.g, progress),
            b: self.b.interpolate(&end.b, progress),
            a: self.a.interpolate(&end.a, progress),
        }
    }
}
