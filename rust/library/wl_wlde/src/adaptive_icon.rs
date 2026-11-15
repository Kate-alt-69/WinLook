//! Bounded adaptive application-icon composition.
//!
//! The platform adapter decodes Shell icon resources into RGBA views. WLDE
//! composes the layers once into a mask-ready bitmap; callers cache the result
//! and invalidate it when the source asset, profile or output size changes.

use crate::{AdaptiveAppIcon, AppIconMask};
use std::fmt;

pub const MAX_ADAPTIVE_ICON_SOURCE_DIMENSION: u32 = 4096;
pub const MIN_ADAPTIVE_ICON_OUTPUT_SIZE: u32 = 16;
pub const MAX_ADAPTIVE_ICON_OUTPUT_SIZE: u32 = 512;

/// Borrowed, tightly packed, straight-alpha RGBA pixels in row-major order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdaptiveIconImageView<'a> {
    width: u32,
    height: u32,
    pixels: &'a [u8],
}

impl<'a> AdaptiveIconImageView<'a> {
    pub fn new(width: u32, height: u32, pixels: &'a [u8]) -> Result<Self, AdaptiveIconRenderError> {
        if width == 0
            || height == 0
            || width > MAX_ADAPTIVE_ICON_SOURCE_DIMENSION
            || height > MAX_ADAPTIVE_ICON_SOURCE_DIMENSION
        {
            return Err(AdaptiveIconRenderError::InvalidImageDimensions);
        }
        let expected = (width as usize)
            .checked_mul(height as usize)
            .and_then(|value| value.checked_mul(4))
            .ok_or(AdaptiveIconRenderError::SizeOverflow)?;
        if pixels.len() != expected {
            return Err(AdaptiveIconRenderError::PixelBufferLengthMismatch);
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    pub fn width(self) -> u32 {
        self.width
    }

    pub fn height(self) -> u32 {
        self.height
    }

    pub fn pixels(self) -> &'a [u8] {
        self.pixels
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdaptiveIconRenderError {
    InvalidImageDimensions,
    PixelBufferLengthMismatch,
    OutputSizeOutOfRange,
    OutputBufferTooSmall,
    SizeOverflow,
    AllocationFailed,
}

impl fmt::Display for AdaptiveIconRenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidImageDimensions => {
                f.write_str("adaptive icon source dimensions must be between 1 and 4096")
            }
            Self::PixelBufferLengthMismatch => {
                f.write_str("adaptive icon RGBA buffer length does not match its dimensions")
            }
            Self::OutputSizeOutOfRange => {
                f.write_str("adaptive icon output size must be between 16 and 512 pixels")
            }
            Self::OutputBufferTooSmall => {
                f.write_str("adaptive icon output buffer is smaller than the requested bitmap")
            }
            Self::SizeOverflow => f.write_str("adaptive icon bitmap size overflowed"),
            Self::AllocationFailed => f.write_str("adaptive icon output allocation failed"),
        }
    }
}

impl std::error::Error for AdaptiveIconRenderError {}

/// Composes a background and foreground layer into one masked RGBA app icon.
///
/// Background imagery uses a centered cover crop. Foreground artwork is
/// contained inside the icon's configured safe zone so important artwork is
/// not clipped by the selected mask. Each source is bilinearly sampled in
/// premultiplied-alpha space to avoid coloured fringes around transparent
/// pixels. A 2x2 coverage test antialiases the final mask.
///
/// The result is intended to be cached by the platform adapter. This function
/// allocates only the final output buffer and performs no file or Shell I/O.
pub fn render_adaptive_app_icon(
    icon: &AdaptiveAppIcon,
    background: AdaptiveIconImageView<'_>,
    foreground: AdaptiveIconImageView<'_>,
    output_size: u32,
) -> Result<Vec<u8>, AdaptiveIconRenderError> {
    let output_bytes = output_buffer_length(output_size)?;
    let mut output = Vec::new();
    output
        .try_reserve_exact(output_bytes)
        .map_err(|_| AdaptiveIconRenderError::AllocationFailed)?;
    output.resize(output_bytes, 0);
    render_adaptive_app_icon_into(icon, background, foreground, output_size, &mut output)?;
    Ok(output)
}

/// Composes an adaptive app icon into storage supplied by the caller.
///
/// This is the allocation-free path intended for BGW's reusable render buffers.
/// The output slice may be larger than the required tile size; bytes after the
/// square RGBA bitmap are left untouched.
pub fn render_adaptive_app_icon_into(
    icon: &AdaptiveAppIcon,
    background: AdaptiveIconImageView<'_>,
    foreground: AdaptiveIconImageView<'_>,
    output_size: u32,
    output_rgba: &mut [u8],
) -> Result<(), AdaptiveIconRenderError> {
    render_adaptive_app_icon_into_with_tint(
        icon,
        background,
        foreground,
        None,
        output_size,
        output_rgba,
    )
}

/// Renders a themed monochrome adaptive icon into a newly allocated RGBA tile.
///
/// The source image's alpha channel is used as the glyph mask; its RGB channels
/// are ignored and replaced with the caller's tint. The background still uses
/// the normal centered cover crop, and the glyph is contained inside the
/// configured safe zone and clipped by the selected mask.
pub fn render_adaptive_app_icon_monochrome(
    icon: &AdaptiveAppIcon,
    background: AdaptiveIconImageView<'_>,
    monochrome: AdaptiveIconImageView<'_>,
    tint_rgb: [u8; 3],
    output_size: u32,
) -> Result<Vec<u8>, AdaptiveIconRenderError> {
    let output_bytes = output_buffer_length(output_size)?;
    let mut output = Vec::new();
    output
        .try_reserve_exact(output_bytes)
        .map_err(|_| AdaptiveIconRenderError::AllocationFailed)?;
    output.resize(output_bytes, 0);
    render_adaptive_app_icon_monochrome_into(
        icon,
        background,
        monochrome,
        tint_rgb,
        output_size,
        &mut output,
    )?;
    Ok(output)
}

/// Allocation-free themed monochrome adaptive-icon renderer.
///
/// The monochrome source alpha is the glyph mask. Its source RGB values are
/// ignored so the caller can apply the active user/theme tint without
/// modifying the source asset.
pub fn render_adaptive_app_icon_monochrome_into(
    icon: &AdaptiveAppIcon,
    background: AdaptiveIconImageView<'_>,
    monochrome: AdaptiveIconImageView<'_>,
    tint_rgb: [u8; 3],
    output_size: u32,
    output_rgba: &mut [u8],
) -> Result<(), AdaptiveIconRenderError> {
    render_adaptive_app_icon_into_with_tint(
        icon,
        background,
        monochrome,
        Some(tint_rgb),
        output_size,
        output_rgba,
    )
}

fn render_adaptive_app_icon_into_with_tint(
    icon: &AdaptiveAppIcon,
    background: AdaptiveIconImageView<'_>,
    foreground: AdaptiveIconImageView<'_>,
    tint_rgb: Option<[u8; 3]>,
    output_size: u32,
    output_rgba: &mut [u8],
) -> Result<(), AdaptiveIconRenderError> {
    let output_bytes = output_buffer_length(output_size)?;
    if output_rgba.len() < output_bytes {
        return Err(AdaptiveIconRenderError::OutputBufferTooSmall);
    }

    let safe_zone = output_size as f64 * f64::from(icon.safe_zone_percent()) / 100.0;
    let foreground_scale =
        (safe_zone / f64::from(foreground.width)).min(safe_zone / f64::from(foreground.height));
    let foreground_width = f64::from(foreground.width) * foreground_scale;
    let foreground_height = f64::from(foreground.height) * foreground_scale;
    let foreground_left = (f64::from(output_size) - foreground_width) * 0.5;
    let foreground_top = (f64::from(output_size) - foreground_height) * 0.5;

    for y in 0..output_size {
        for x in 0..output_size {
            let (background_x, background_y) =
                cover_source_coordinates(background, x, y, output_size);
            let mut pixel = sample_bilinear(background, background_x, background_y);

            let destination_x = f64::from(x) + 0.5;
            let destination_y = f64::from(y) + 0.5;
            if destination_x >= foreground_left
                && destination_x < foreground_left + foreground_width
                && destination_y >= foreground_top
                && destination_y < foreground_top + foreground_height
            {
                let foreground_x = (destination_x - foreground_left) / foreground_scale - 0.5;
                let foreground_y = (destination_y - foreground_top) / foreground_scale - 0.5;
                let mut foreground_pixel = sample_bilinear(foreground, foreground_x, foreground_y);
                if let Some(tint) = tint_rgb {
                    foreground_pixel[0] = tint[0];
                    foreground_pixel[1] = tint[1];
                    foreground_pixel[2] = tint[2];
                }
                pixel = alpha_over(pixel, foreground_pixel);
            }

            let coverage = mask_coverage(icon.mask(), x, y, output_size);
            pixel[3] = ((u16::from(pixel[3]) * u16::from(coverage) + 2) / 4) as u8;

            let offset = ((y as usize * output_size as usize) + x as usize) * 4;
            output_rgba[offset..offset + 4].copy_from_slice(&pixel);
        }
    }

    Ok(())
}

fn output_buffer_length(output_size: u32) -> Result<usize, AdaptiveIconRenderError> {
    if !(MIN_ADAPTIVE_ICON_OUTPUT_SIZE..=MAX_ADAPTIVE_ICON_OUTPUT_SIZE).contains(&output_size) {
        return Err(AdaptiveIconRenderError::OutputSizeOutOfRange);
    }
    (output_size as usize)
        .checked_mul(output_size as usize)
        .and_then(|value| value.checked_mul(4))
        .ok_or(AdaptiveIconRenderError::SizeOverflow)
}

fn cover_source_coordinates(
    image: AdaptiveIconImageView<'_>,
    x: u32,
    y: u32,
    output_size: u32,
) -> (f64, f64) {
    let output = f64::from(output_size);
    if image.width >= image.height {
        let side = f64::from(image.height);
        let crop_x = (f64::from(image.width) - side) * 0.5;
        (
            crop_x + (f64::from(x) + 0.5) * side / output - 0.5,
            (f64::from(y) + 0.5) * side / output - 0.5,
        )
    } else {
        let side = f64::from(image.width);
        let crop_y = (f64::from(image.height) - side) * 0.5;
        (
            (f64::from(x) + 0.5) * side / output - 0.5,
            crop_y + (f64::from(y) + 0.5) * side / output - 0.5,
        )
    }
}

fn sample_bilinear(image: AdaptiveIconImageView<'_>, x: f64, y: f64) -> [u8; 4] {
    let source_x = x.clamp(0.0, f64::from(image.width - 1));
    let source_y = y.clamp(0.0, f64::from(image.height - 1));
    let x0 = source_x.floor() as u32;
    let y0 = source_y.floor() as u32;
    let x1 = (x0 + 1).min(image.width - 1);
    let y1 = (y0 + 1).min(image.height - 1);
    let fraction_x = source_x - f64::from(x0);
    let fraction_y = source_y - f64::from(y0);

    let weights = [
        (1.0 - fraction_x) * (1.0 - fraction_y),
        fraction_x * (1.0 - fraction_y),
        (1.0 - fraction_x) * fraction_y,
        fraction_x * fraction_y,
    ];
    let points = [(x0, y0), (x1, y0), (x0, y1), (x1, y1)];
    let mut alpha = 0.0;
    let mut premultiplied = [0.0; 3];

    for (index, (pixel_x, pixel_y)) in points.iter().enumerate() {
        let offset = ((*pixel_y as usize * image.width as usize) + *pixel_x as usize) * 4;
        let source = &image.pixels[offset..offset + 4];
        let weighted_alpha = f64::from(source[3]) * weights[index];
        alpha += weighted_alpha;
        for channel in 0..3 {
            premultiplied[channel] += f64::from(source[channel]) * weighted_alpha;
        }
    }

    if alpha <= f64::EPSILON {
        return [0, 0, 0, 0];
    }

    [
        to_u8(premultiplied[0] / alpha),
        to_u8(premultiplied[1] / alpha),
        to_u8(premultiplied[2] / alpha),
        to_u8(alpha),
    ]
}

fn alpha_over(destination: [u8; 4], source: [u8; 4]) -> [u8; 4] {
    let source_alpha = f64::from(source[3]) / 255.0;
    if source_alpha <= 0.0 {
        return destination;
    }
    if source_alpha >= 1.0 {
        return source;
    }

    let destination_alpha = f64::from(destination[3]) / 255.0;
    let output_alpha = source_alpha + destination_alpha * (1.0 - source_alpha);
    if output_alpha <= f64::EPSILON {
        return [0, 0, 0, 0];
    }

    let mut output = [0; 4];
    for channel in 0..3 {
        let blended = (f64::from(source[channel]) * source_alpha
            + f64::from(destination[channel]) * destination_alpha * (1.0 - source_alpha))
            / output_alpha;
        output[channel] = to_u8(blended);
    }
    output[3] = to_u8(output_alpha * 255.0);
    output
}

fn mask_coverage(mask: AppIconMask, x: u32, y: u32, size: u32) -> u8 {
    const SAMPLES: [f64; 2] = [0.25, 0.75];
    let mut covered = 0;
    for sample_y in SAMPLES {
        for sample_x in SAMPLES {
            let normalized_x = ((f64::from(x) + sample_x) / f64::from(size)) * 2.0 - 1.0;
            let normalized_y = ((f64::from(y) + sample_y) / f64::from(size)) * 2.0 - 1.0;
            if inside_mask(mask, normalized_x, normalized_y) {
                covered += 1;
            }
        }
    }
    covered
}

fn inside_mask(mask: AppIconMask, x: f64, y: f64) -> bool {
    match mask {
        AppIconMask::Circle => x * x + y * y <= 1.0,
        AppIconMask::RoundedSquare => {
            let radius = 0.24;
            let qx = x.abs() - (1.0 - radius);
            let qy = y.abs() - (1.0 - radius);
            let outside_x = qx.max(0.0);
            let outside_y = qy.max(0.0);
            if qx.max(qy) <= 0.0 {
                true
            } else {
                outside_x * outside_x + outside_y * outside_y <= radius * radius
            }
        }
        AppIconMask::Squircle => {
            let x2 = x * x;
            let y2 = y * y;
            x2 * x2 + y2 * y2 <= 1.0
        }
    }
}

fn to_u8(value: f64) -> u8 {
    value.round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AppIconLayers, DEFAULT_ADAPTIVE_ICON_SAFE_ZONE_PERCENT};

    fn icon(mask: AppIconMask) -> AdaptiveAppIcon {
        AdaptiveAppIcon::new(
            "app:test",
            "Test",
            AppIconLayers::new("foreground", "background", None).unwrap(),
            mask,
            DEFAULT_ADAPTIVE_ICON_SAFE_ZONE_PERCENT,
        )
        .unwrap()
    }

    #[test]
    fn compositor_combines_layers_and_applies_the_squircle_mask() {
        let background = AdaptiveIconImageView::new(1, 1, &[255, 0, 0, 255]).unwrap();
        let foreground = AdaptiveIconImageView::new(1, 1, &[0, 0, 255, 255]).unwrap();
        let rendered =
            render_adaptive_app_icon(&icon(AppIconMask::Squircle), background, foreground, 32)
                .unwrap();

        let center = (16 * 32 + 16) * 4;
        assert_eq!(&rendered[center..center + 4], &[0, 0, 255, 255]);
        assert_eq!(rendered[3], 0);
        assert_eq!(background.pixels(), &[255, 0, 0, 255]);
        assert_eq!(foreground.pixels(), &[0, 0, 255, 255]);
    }

    #[test]
    fn compositor_keeps_the_foreground_inside_its_safe_zone() {
        let background = AdaptiveIconImageView::new(1, 1, &[12, 24, 36, 255]).unwrap();
        let foreground = AdaptiveIconImageView::new(1, 1, &[240, 200, 160, 255]).unwrap();
        let rendered = render_adaptive_app_icon(
            &icon(AppIconMask::RoundedSquare),
            background,
            foreground,
            64,
        )
        .unwrap();

        let center = (32 * 64 + 32) * 4;
        let outside_safe_zone = (8 * 64 + 32) * 4;
        assert_eq!(&rendered[center..center + 3], &[240, 200, 160]);
        assert_eq!(
            &rendered[outside_safe_zone..outside_safe_zone + 3],
            &[12, 24, 36]
        );
    }

    #[test]
    fn caller_buffer_renderer_matches_allocating_helper_and_preserves_tail() {
        let background = AdaptiveIconImageView::new(1, 1, &[20, 40, 60, 255]).unwrap();
        let foreground = AdaptiveIconImageView::new(1, 1, &[200, 160, 120, 128]).unwrap();
        let model = icon(AppIconMask::Circle);
        let expected = render_adaptive_app_icon(&model, background, foreground, 32).unwrap();
        let mut caller_buffer = vec![0xCC; expected.len() + 12];
        render_adaptive_app_icon_into(&model, background, foreground, 32, &mut caller_buffer)
            .unwrap();
        assert_eq!(&caller_buffer[..expected.len()], expected.as_slice());
        assert!(caller_buffer[expected.len()..]
            .iter()
            .all(|byte| *byte == 0xCC));
    }

    #[test]
    fn caller_buffer_renderer_rejects_small_storage_without_writing() {
        let background = AdaptiveIconImageView::new(1, 1, &[0, 0, 0, 255]).unwrap();
        let foreground = AdaptiveIconImageView::new(1, 1, &[255, 255, 255, 255]).unwrap();
        let mut output = vec![0xA5; 16 * 16 * 4 - 1];
        assert_eq!(
            render_adaptive_app_icon_into(
                &icon(AppIconMask::Circle),
                background,
                foreground,
                16,
                &mut output,
            ),
            Err(AdaptiveIconRenderError::OutputBufferTooSmall)
        );
        assert!(output.iter().all(|byte| *byte == 0xA5));
    }

    #[test]
    fn monochrome_mode_uses_alpha_as_mask_and_applies_theme_tint() {
        let background = AdaptiveIconImageView::new(1, 1, &[12, 24, 36, 255]).unwrap();
        let monochrome_pixels = [
            240, 10, 20, 0, 240, 10, 20, 0, 240, 10, 20, 0, 240, 10, 20, 0, 240, 10, 20, 255,
            240, 10, 20, 255, 240, 10, 20, 0, 240, 10, 20, 255, 240, 10, 20, 255,
        ];
        let monochrome = AdaptiveIconImageView::new(3, 3, &monochrome_pixels).unwrap();
        let rendered = render_adaptive_app_icon_monochrome(
            &icon(AppIconMask::Squircle),
            background,
            monochrome,
            [32, 160, 224],
            32,
        )
        .unwrap();

        let center = (16 * 32 + 16) * 4;
        let outside_glyph = (5 * 32 + 16) * 4;
        assert_eq!(&rendered[center..center + 4], &[32, 160, 224, 255]);
        assert_eq!(&rendered[outside_glyph..outside_glyph + 3], &[12, 24, 36]);
        assert_eq!(monochrome.pixels(), &monochrome_pixels);
    }

    #[test]
    fn monochrome_caller_buffer_matches_allocating_renderer_and_preserves_tail() {
        let background = AdaptiveIconImageView::new(1, 1, &[0, 0, 0, 255]).unwrap();
        let monochrome = AdaptiveIconImageView::new(1, 1, &[10, 20, 30, 128]).unwrap();
        let model = icon(AppIconMask::Circle);
        let tint = [180, 120, 60];
        let expected =
            render_adaptive_app_icon_monochrome(&model, background, monochrome, tint, 32).unwrap();
        let mut output = vec![0xD7; expected.len() + 8];
        render_adaptive_app_icon_monochrome_into(
            &model,
            background,
            monochrome,
            tint,
            32,
            &mut output,
        )
        .unwrap();
        assert_eq!(&output[..expected.len()], expected.as_slice());
        assert!(output[expected.len()..].iter().all(|byte| *byte == 0xD7));
    }

    #[test]
    fn compositor_rejects_bad_buffers_and_unbounded_output_sizes() {
        assert_eq!(
            AdaptiveIconImageView::new(2, 2, &[0; 15]),
            Err(AdaptiveIconRenderError::PixelBufferLengthMismatch)
        );
        assert_eq!(
            AdaptiveIconImageView::new(4097, 1, &[]),
            Err(AdaptiveIconRenderError::InvalidImageDimensions)
        );
        let background = AdaptiveIconImageView::new(1, 1, &[0, 0, 0, 255]).unwrap();
        let foreground = AdaptiveIconImageView::new(1, 1, &[255, 255, 255, 255]).unwrap();
        assert_eq!(
            render_adaptive_app_icon(&icon(AppIconMask::Circle), background, foreground, 513),
            Err(AdaptiveIconRenderError::OutputSizeOutOfRange)
        );
    }
}
