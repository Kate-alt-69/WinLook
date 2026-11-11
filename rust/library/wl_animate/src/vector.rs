use crate::values::{Interpolate, Rgba};

/// Two-dimensional vector. Coordinates are expressed in the caller's units.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

impl Vector2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    pub fn dot(self, rhs: Self) -> f64 {
        self.x * rhs.x + self.y * rhs.y
    }
    pub fn length(self) -> f64 {
        self.dot(self).sqrt()
    }
    pub fn normalized(self) -> Option<Self> {
        let length = self.length();
        if !length.is_finite() || length <= f64::EPSILON {
            None
        } else {
            Some(Self::new(self.x / length, self.y / length))
        }
    }
    pub fn lerp(self, rhs: Self, progress: f64) -> Self {
        self.interpolate(&rhs, progress)
    }
}

impl Interpolate for Vector2 {
    fn interpolate(&self, end: &Self, progress: f64) -> Self {
        Self::new(
            self.x.interpolate(&end.x, progress),
            self.y.interpolate(&end.y, progress),
        )
    }
}

/// Three-dimensional vector.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
    pub fn dot(self, rhs: Self) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
    pub fn cross(self, rhs: Self) -> Self {
        Self::new(
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        )
    }
    pub fn length(self) -> f64 {
        self.dot(self).sqrt()
    }
}

impl Interpolate for Vector3 {
    fn interpolate(&self, end: &Self, progress: f64) -> Self {
        Self::new(
            self.x.interpolate(&end.x, progress),
            self.y.interpolate(&end.y, progress),
            self.z.interpolate(&end.z, progress),
        )
    }
}

/// Four-dimensional vector (also useful for homogeneous coordinates).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector4 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Vector4 {
    pub const fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self { x, y, z, w }
    }
}

impl Interpolate for Vector4 {
    fn interpolate(&self, end: &Self, progress: f64) -> Self {
        Self::new(
            self.x.interpolate(&end.x, progress),
            self.y.interpolate(&end.y, progress),
            self.z.interpolate(&end.z, progress),
            self.w.interpolate(&end.w, progress),
        )
    }
}

/// Axis-aligned rectangle. Width and height are not implicitly clamped.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
    pub fn contains(self, point: Vector2) -> bool {
        point.x >= self.x
            && point.y >= self.y
            && point.x <= self.x + self.width
            && point.y <= self.y + self.height
    }
}

impl Interpolate for Rect {
    fn interpolate(&self, end: &Self, progress: f64) -> Self {
        Self::new(
            self.x.interpolate(&end.x, progress),
            self.y.interpolate(&end.y, progress),
            self.width.interpolate(&end.width, progress),
            self.height.interpolate(&end.height, progress),
        )
    }
}

/// A 2D affine matrix in SVG/Direct2D coefficient order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix3x2 {
    pub m11: f64,
    pub m12: f64,
    pub m21: f64,
    pub m22: f64,
    pub dx: f64,
    pub dy: f64,
}

impl Default for Matrix3x2 {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Matrix3x2 {
    pub const IDENTITY: Self = Self {
        m11: 1.0,
        m12: 0.0,
        m21: 0.0,
        m22: 1.0,
        dx: 0.0,
        dy: 0.0,
    };
    pub const fn new(m11: f64, m12: f64, m21: f64, m22: f64, dx: f64, dy: f64) -> Self {
        Self {
            m11,
            m12,
            m21,
            m22,
            dx,
            dy,
        }
    }
    pub fn translation(x: f64, y: f64) -> Self {
        Self::new(1.0, 0.0, 0.0, 1.0, x, y)
    }
    pub fn scale(x: f64, y: f64) -> Self {
        Self::new(x, 0.0, 0.0, y, 0.0, 0.0)
    }
    pub fn rotation(radians: f64) -> Self {
        let (sin, cos) = radians.sin_cos();
        Self::new(cos, sin, -sin, cos, 0.0, 0.0)
    }
    pub fn transform_point(self, point: Vector2) -> Vector2 {
        Vector2::new(
            self.m11 * point.x + self.m21 * point.y + self.dx,
            self.m12 * point.x + self.m22 * point.y + self.dy,
        )
    }
    /// Matrix composition: the returned matrix applies rhs and then self.
    pub fn compose(self, rhs: Self) -> Self {
        Self::new(
            self.m11 * rhs.m11 + self.m21 * rhs.m12,
            self.m12 * rhs.m11 + self.m22 * rhs.m12,
            self.m11 * rhs.m21 + self.m21 * rhs.m22,
            self.m12 * rhs.m21 + self.m22 * rhs.m22,
            self.m11 * rhs.dx + self.m21 * rhs.dy + self.dx,
            self.m12 * rhs.dx + self.m22 * rhs.dy + self.dy,
        )
    }
}

impl Interpolate for Matrix3x2 {
    fn interpolate(&self, end: &Self, progress: f64) -> Self {
        Self::new(
            self.m11.interpolate(&end.m11, progress),
            self.m12.interpolate(&end.m12, progress),
            self.m21.interpolate(&end.m21, progress),
            self.m22.interpolate(&end.m22, progress),
            self.dx.interpolate(&end.dx, progress),
            self.dy.interpolate(&end.dy, progress),
        )
    }
}

/// Decomposed 2D transform for animation without matrix-shear interpolation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform2D {
    pub translation: Vector2,
    pub scale: Vector2,
    pub rotation_radians: f64,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            translation: Vector2::ZERO,
            scale: Vector2::new(1.0, 1.0),
            rotation_radians: 0.0,
        }
    }
}

impl Transform2D {
    pub const fn new(translation: Vector2, scale: Vector2, rotation_radians: f64) -> Self {
        Self {
            translation,
            scale,
            rotation_radians,
        }
    }
    pub fn matrix(self) -> Matrix3x2 {
        Matrix3x2::translation(self.translation.x, self.translation.y).compose(
            Matrix3x2::rotation(self.rotation_radians)
                .compose(Matrix3x2::scale(self.scale.x, self.scale.y)),
        )
    }
}

impl Interpolate for Transform2D {
    fn interpolate(&self, end: &Self, progress: f64) -> Self {
        let mut delta =
            (end.rotation_radians - self.rotation_radians).rem_euclid(std::f64::consts::TAU);
        if delta > std::f64::consts::PI {
            delta -= std::f64::consts::TAU;
        }
        let t = if progress.is_nan() {
            0.0
        } else {
            progress.clamp(0.0, 1.0)
        };
        Self {
            translation: self.translation.interpolate(&end.translation, t),
            scale: self.scale.interpolate(&end.scale, t),
            rotation_radians: self.rotation_radians + delta * t,
        }
    }
}

/// How RGBA colors are blended during interpolation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlphaMode {
    Straight,
    Premultiplied,
}

/// Opacity normalized to the closed interval 0..=1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Opacity(f64);

impl Opacity {
    pub fn new(value: f64) -> Self {
        Self(if value.is_nan() {
            0.0
        } else {
            value.clamp(0.0, 1.0)
        })
    }
    pub const fn opaque() -> Self {
        Self(1.0)
    }
    pub const fn transparent() -> Self {
        Self(0.0)
    }
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl Default for Opacity {
    fn default() -> Self {
        Self::opaque()
    }
}
impl Interpolate for Opacity {
    fn interpolate(&self, end: &Self, progress: f64) -> Self {
        Self::new(self.0.interpolate(&end.0, progress))
    }
}

/// Paint accepted by the initial render-independent vector scene API.
#[derive(Clone, Debug, PartialEq)]
pub enum Paint {
    Solid(Rgba),
    LinearGradient(LinearGradient),
    RadialGradient(RadialGradient),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientStop {
    pub offset: f64,
    pub color: Rgba,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LinearGradient {
    pub start: Vector2,
    pub end: Vector2,
    pub stops: Vec<GradientStop>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RadialGradient {
    pub center: Vector2,
    pub radius: f64,
    pub stops: Vec<GradientStop>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FillRule {
    NonZero,
    EvenOdd,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrokeCap {
    Butt,
    Round,
    Square,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrokeJoin {
    Miter,
    Round,
    Bevel,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VectorStyle {
    pub fill: Option<Paint>,
    pub stroke: Option<Paint>,
    pub stroke_width: f64,
    pub opacity: Opacity,
    pub fill_opacity: Opacity,
    pub stroke_opacity: Opacity,
    pub fill_rule: FillRule,
    pub stroke_cap: StrokeCap,
    pub stroke_join: StrokeJoin,
}

impl Default for VectorStyle {
    fn default() -> Self {
        Self {
            fill: Some(Paint::Solid(Rgba::new(0.0, 0.0, 0.0, 1.0))),
            stroke: None,
            stroke_width: 1.0,
            opacity: Opacity::opaque(),
            fill_opacity: Opacity::opaque(),
            stroke_opacity: Opacity::opaque(),
            fill_rule: FillRule::NonZero,
            stroke_cap: StrokeCap::Butt,
            stroke_join: StrokeJoin::Miter,
        }
    }
}

/// A render-independent item in an SVG-style vector scene.
#[derive(Clone, Debug, PartialEq)]
pub struct VectorItem {
    pub name: Option<String>,
    pub path: crate::svg::VectorPath,
    pub transform: Transform2D,
    pub style: VectorStyle,
}

impl VectorItem {
    pub fn from_svg_path(path_data: &str) -> Result<Self, crate::svg::SvgPathError> {
        Ok(Self {
            name: None,
            path: crate::svg::VectorPath::parse(path_data)?,
            transform: Transform2D::default(),
            style: VectorStyle::default(),
        })
    }
}

/// A vector document owns a view box and ordered vector items.
#[derive(Clone, Debug, PartialEq)]
pub struct VectorDocument {
    pub view_box: Rect,
    items: Vec<VectorItem>,
}

impl VectorDocument {
    pub fn new(view_box: Rect) -> Self {
        Self {
            view_box,
            items: Vec::new(),
        }
    }
    pub fn add(&mut self, item: VectorItem) {
        self.items.push(item);
    }
    pub fn items(&self) -> &[VectorItem] {
        &self.items
    }
    pub fn items_mut(&mut self) -> &mut [VectorItem] {
        &mut self.items
    }
    pub fn len(&self) -> usize {
        self.items.len()
    }
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_math_and_matrix_composition_are_consistent() {
        let v = Vector2::new(3.0, 4.0);
        assert_eq!(v.length(), 5.0);
        assert_eq!(v.normalized(), Some(Vector2::new(0.6, 0.8)));
        let matrix = Matrix3x2::translation(10.0, 20.0).compose(Matrix3x2::scale(2.0, 3.0));
        assert_eq!(matrix.transform_point(v), Vector2::new(16.0, 32.0));
        assert_eq!(
            Vector3::new(1.0, 0.0, 0.0).cross(Vector3::new(0.0, 1.0, 0.0)),
            Vector3::new(0.0, 0.0, 1.0)
        );
    }

    #[test]
    fn transform_rotation_uses_shortest_arc() {
        let start = Transform2D::new(Vector2::ZERO, Vector2::new(1.0, 1.0), 350_f64.to_radians());
        let end = Transform2D::new(Vector2::ZERO, Vector2::new(1.0, 1.0), 10_f64.to_radians());
        let mid = start.interpolate(&end, 0.5);
        assert!((mid.rotation_radians.to_degrees() - 360.0).abs() < 1e-9);
    }

    #[test]
    fn opacity_clamps_and_interpolates() {
        assert_eq!(Opacity::new(2.0).get(), 1.0);
        assert_eq!(Opacity::new(-1.0).get(), 0.0);
        assert_eq!(Opacity::new(f64::NAN).get(), 0.0);
        assert_eq!(
            Opacity::transparent()
                .interpolate(&Opacity::opaque(), 0.5)
                .get(),
            0.5
        );
    }

    #[test]
    fn vector_document_stores_svg_path_and_alpha_style() {
        let mut item = VectorItem::from_svg_path("M0 0 L10 10 Z").expect("valid SVG path");
        item.style.fill = Some(Paint::Solid(Rgba::new(1.0, 0.0, 0.0, 0.5)));
        item.style.opacity = Opacity::new(0.75);
        let mut document = VectorDocument::new(Rect::new(0.0, 0.0, 100.0, 100.0));
        document.add(item);
        assert_eq!(document.len(), 1);
        assert_eq!(document.items()[0].style.opacity.get(), 0.75);
        assert_eq!(
            document.items()[0].style.fill,
            Some(Paint::Solid(Rgba::new(1.0, 0.0, 0.0, 0.5)))
        );
    }
}
