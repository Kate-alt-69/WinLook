//! Read-only extraction of the Windows Shell icon for a desktop filesystem item.
//!
//! This adapter returns an owned HICON handle and its native dimensions,
//! with a bounded straight-alpha RGBA rasterizer for BGW. It does not replace
//! Explorer's icon registration, mutate the source item or suppress the
//! native tile. Adaptive composition and live presentation are separate work.

use std::fmt;
use std::ptr::NonNull;

use wl_wlde::{AdaptiveIconImageView, AdaptiveIconRenderError};

use crate::DesktopInventoryItem;

#[cfg(windows)]
#[link(name = "user32")]
extern "system" {
    fn DestroyIcon(icon: *mut std::ffi::c_void) -> i32;
}

#[cfg(windows)]
#[link(name = "gdi32")]
extern "system" {
    fn DeleteObject(object: *mut std::ffi::c_void) -> i32;
}

/// The system icon size requested from the Shell image handler.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DesktopShellIconSize {
    Small,
    Large,
}

/// Errors returned while extracting a filesystem item's Shell icon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DesktopShellIconError {
    UnsupportedPlatform,
    PathTooLong,
    ShellIconUnavailable,
    IconMetadataUnavailable,
    InvalidIconDimensions,
    PixelBufferLengthMismatch,
    IconTooLarge,
    RasterizationUnavailable,
    UnsupportedRasterOperation,
    AllocationFailed,
}

impl fmt::Display for DesktopShellIconError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => {
                f.write_str("desktop Shell icon extraction is available only on Windows")
            }
            Self::PathTooLong => {
                f.write_str(
                    "Shell icon extraction path exceeds the SHGetFileInfoW MAX_PATH limit",
                )
            }
            Self::ShellIconUnavailable => {
                f.write_str("Windows Shell did not provide an icon for this desktop item")
            }
            Self::IconMetadataUnavailable => {
                f.write_str("Windows could not read the extracted Shell icon metadata")
            }
            Self::InvalidIconDimensions => {
                f.write_str("Windows returned invalid dimensions for the Shell icon")
            }
            Self::PixelBufferLengthMismatch => {
                f.write_str("Shell icon RGBA buffer length does not match its dimensions")
            }
            Self::IconTooLarge => {
                f.write_str("Shell icon dimensions exceed the 512-pixel rasterization bound")
            }
            Self::RasterizationUnavailable => {
                f.write_str("Windows could not rasterize the Shell icon into a DIB")
            }
            Self::UnsupportedRasterOperation => {
                f.write_str(
                    "the Shell icon uses screen-dependent inversion and cannot be represented as static RGBA",
                )
            }
            Self::AllocationFailed => {
                f.write_str("not enough memory to rasterize the Shell icon")
            }
        }
    }
}

impl std::error::Error for DesktopShellIconError {}

/// An owned Shell icon handle. Dropping it releases the HICON with DestroyIcon.
///
/// The raw handle is borrowed and remains valid only while this value is alive.
/// Callers must not destroy it or retain it beyond the owner.
pub struct DesktopShellIcon {
    icon: OwnedIcon,
    width: u32,
    height: u32,
}

impl DesktopShellIcon {
    /// Borrow the raw HICON for immediate platform drawing/conversion.
    pub fn as_raw_handle(&self) -> *mut std::ffi::c_void {
        self.icon.0.as_ptr()
    }

    /// Native icon bitmap width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Native icon bitmap height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }
}

struct OwnedIcon(NonNull<std::ffi::c_void>);

impl Drop for OwnedIcon {
    fn drop(&mut self) {
        #[cfg(windows)]
        unsafe {
            DestroyIcon(self.0.as_ptr());
        }
    }
}

#[cfg(windows)]
struct OwnedGdiObject(*mut std::ffi::c_void);

#[cfg(windows)]
impl Drop for OwnedGdiObject {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                DeleteObject(self.0);
            }
        }
    }
}

/// Extract the icon currently associated with a User/Public Desktop item.
///
/// The call is synchronous and may cause the Shell to inspect disk or invoke
/// an icon handler. Run it on a BGW worker, never on the Manager UI thread.
#[cfg(windows)]
pub fn extract_desktop_shell_icon(
    item: &DesktopInventoryItem,
    size: DesktopShellIconSize,
) -> Result<DesktopShellIcon, DesktopShellIconError> {
    platform::extract(item.filesystem_path(), size)
}

/// Non-Windows builds expose an explicit unsupported-platform error.
#[cfg(not(windows))]
pub fn extract_desktop_shell_icon(
    _item: &DesktopInventoryItem,
    _size: DesktopShellIconSize,
) -> Result<DesktopShellIcon, DesktopShellIconError> {
    Err(DesktopShellIconError::UnsupportedPlatform)
}

/// Maximum width or height accepted by the BGW Shell icon rasterizer.
pub const MAX_DESKTOP_SHELL_ICON_DIMENSION: u32 = 512;

/// Owned, tightly packed, straight-alpha RGBA Shell icon pixels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DesktopShellIconRgba {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl DesktopShellIconRgba {
    /// Validates an owned straight-alpha RGBA buffer for the BGW icon cache.
    pub fn from_pixels(
        width: u32,
        height: u32,
        pixels: Vec<u8>,
    ) -> Result<Self, DesktopShellIconError> {
        if width == 0 || height == 0 {
            return Err(DesktopShellIconError::InvalidIconDimensions);
        }
        if width > MAX_DESKTOP_SHELL_ICON_DIMENSION
            || height > MAX_DESKTOP_SHELL_ICON_DIMENSION
        {
            return Err(DesktopShellIconError::IconTooLarge);
        }
        let expected = (width as usize)
            .checked_mul(height as usize)
            .and_then(|length| length.checked_mul(4))
            .ok_or(DesktopShellIconError::InvalidIconDimensions)?;
        if pixels.len() != expected {
            return Err(DesktopShellIconError::PixelBufferLengthMismatch);
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Straight-alpha pixels in row-major RGBA byte order.
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// Borrows this image directly as WLDE compositor input without copying.
    pub fn as_adaptive_image_view(
        &self,
    ) -> Result<AdaptiveIconImageView<'_>, AdaptiveIconRenderError> {
        AdaptiveIconImageView::new(self.width, self.height, &self.pixels)
    }
}

/// Rasterizes an owned HICON into bounded straight-alpha RGBA pixels.
///
/// The renderer probes the icon against opaque black and white DIB backgrounds.
/// This reconstructs source alpha for normal alpha-blended icons and classic
/// AND/XOR masks. Screen-dependent inversion pixels are rejected because a
/// static RGBA image cannot represent an operation on the eventual desktop.
#[cfg(windows)]
pub fn rasterize_desktop_shell_icon_rgba(
    icon: &DesktopShellIcon,
) -> Result<DesktopShellIconRgba, DesktopShellIconError> {
    platform::rasterize(icon)
}

/// Non-Windows builds expose an explicit unsupported-platform error.
#[cfg(not(windows))]
pub fn rasterize_desktop_shell_icon_rgba(
    _icon: &DesktopShellIcon,
) -> Result<DesktopShellIconRgba, DesktopShellIconError> {
    Err(DesktopShellIconError::UnsupportedPlatform)
}

fn reconstruct_rgba_pixel(
    black: [u8; 3],
    white: [u8; 3],
) -> Result<[u8; 4], DesktopShellIconError> {
    let differences = [
        i16::from(white[0]) - i16::from(black[0]),
        i16::from(white[1]) - i16::from(black[1]),
        i16::from(white[2]) - i16::from(black[2]),
    ];
    let minimum = *differences.iter().min().unwrap_or(&0);
    let maximum = *differences.iter().max().unwrap_or(&0);
    if minimum < 0 || maximum > 255 || maximum - minimum > 4 {
        return Err(DesktopShellIconError::UnsupportedRasterOperation);
    }

    let average_difference = (u16::from(differences[0] as u8)
        + u16::from(differences[1] as u8)
        + u16::from(differences[2] as u8)
        + 1)
        / 3;
    let alpha = 255u16.saturating_sub(average_difference);
    if alpha == 0 {
        return Ok([0, 0, 0, 0]);
    }

    let mut rgba = [0u8; 4];
    for channel in 0..3 {
        let premultiplied = u16::from(black[channel]);
        let straight = (premultiplied * 255 + (alpha / 2)) / alpha;
        rgba[channel] = straight.min(255) as u8;
    }
    rgba[3] = alpha as u8;
    Ok(rgba)
}

#[cfg(windows)]
mod platform {
    use super::*;
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;

    const MAX_SHELL_PATH_UNITS: usize = 260;
    const SHGFI_ICON: u32 = 0x0000_0100;
    const SHGFI_SHELLICONSIZE: u32 = 0x0000_0004;
    const SHGFI_SMALLICON: u32 = 0x0000_0001;
    const DIB_RGB_COLORS: u32 = 0;
    const BI_RGB: u32 = 0;
    const DI_NORMAL: u32 = 0x0003;
    const HGDI_ERROR: isize = -1;

    #[repr(C)]
    struct ShellFileInfoW {
        icon: *mut std::ffi::c_void,
        _icon_index: i32,
        _attributes: u32,
        _display_name: [u16; 260],
        _type_name: [u16; 80],
    }

    impl Default for ShellFileInfoW {
        fn default() -> Self {
            Self {
                icon: std::ptr::null_mut(),
                _icon_index: 0,
                _attributes: 0,
                _display_name: [0; 260],
                _type_name: [0; 80],
            }
        }
    }

    #[repr(C)]
    struct IconInfo {
        _is_icon: i32,
        _x_hotspot: u32,
        _y_hotspot: u32,
        mask_bitmap: *mut std::ffi::c_void,
        color_bitmap: *mut std::ffi::c_void,
    }

    impl Default for IconInfo {
        fn default() -> Self {
            Self {
                _is_icon: 0,
                _x_hotspot: 0,
                _y_hotspot: 0,
                mask_bitmap: std::ptr::null_mut(),
                color_bitmap: std::ptr::null_mut(),
            }
        }
    }

    #[repr(C)]
    struct Bitmap {
        _bitmap_type: i32,
        width: i32,
        height: i32,
        _width_bytes: i32,
        _planes: u16,
        _bits_pixel: u16,
        _bits: *mut std::ffi::c_void,
    }

    impl Default for Bitmap {
        fn default() -> Self {
            Self {
                _bitmap_type: 0,
                width: 0,
                height: 0,
                _width_bytes: 0,
                _planes: 0,
                _bits_pixel: 0,
                _bits: std::ptr::null_mut(),
            }
        }
    }

    #[link(name = "shell32")]
    extern "system" {
        fn SHGetFileInfoW(
            path: *const u16,
            file_attributes: u32,
            file_info: *mut ShellFileInfoW,
            file_info_size: u32,
            flags: u32,
        ) -> usize;
    }

    #[link(name = "user32")]
    extern "system" {
        fn GetIconInfo(icon: *mut std::ffi::c_void, info: *mut IconInfo) -> i32;
        fn DrawIconEx(
            dc: *mut std::ffi::c_void,
            x: i32,
            y: i32,
            icon: *mut std::ffi::c_void,
            width: i32,
            height: i32,
            step: u32,
            flicker_free_brush: *mut std::ffi::c_void,
            flags: u32,
        ) -> i32;
    }

    #[allow(dead_code)]
    #[repr(C)]
    struct BitmapInfoHeader {
        size: u32,
        width: i32,
        height: i32,
        planes: u16,
        bit_count: u16,
        compression: u32,
        size_image: u32,
        x_pels_per_meter: i32,
        y_pels_per_meter: i32,
        colors_used: u32,
        colors_important: u32,
    }

    #[allow(dead_code)]
    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct RgbQuad {
        blue: u8,
        green: u8,
        red: u8,
        reserved: u8,
    }

    #[allow(dead_code)]
    #[repr(C)]
    struct BitmapInfo {
        header: BitmapInfoHeader,
        colors: [RgbQuad; 1],
    }

    struct OwnedDeviceContext(*mut std::ffi::c_void);

    impl Drop for OwnedDeviceContext {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    DeleteDC(self.0);
                }
            }
        }
    }

    struct SelectedBitmap {
        dc: *mut std::ffi::c_void,
        previous: *mut std::ffi::c_void,
    }

    impl Drop for SelectedBitmap {
        fn drop(&mut self) {
            if !self.previous.is_null() {
                unsafe {
                    SelectObject(self.dc, self.previous);
                }
            }
        }
    }

    #[link(name = "gdi32")]
    extern "system" {
        fn GetObjectW(
            object: *mut std::ffi::c_void,
            object_size: i32,
            object_data: *mut std::ffi::c_void,
        ) -> i32;
        fn CreateCompatibleDC(dc: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
        fn DeleteDC(dc: *mut std::ffi::c_void) -> i32;
        fn CreateDIBSection(
            dc: *mut std::ffi::c_void,
            bitmap_info: *const BitmapInfo,
            usage: u32,
            bits: *mut *mut std::ffi::c_void,
            section: *mut std::ffi::c_void,
            offset: u32,
        ) -> *mut std::ffi::c_void;
        fn SelectObject(
            dc: *mut std::ffi::c_void,
            object: *mut std::ffi::c_void,
        ) -> *mut std::ffi::c_void;
        fn GdiFlush() -> i32;
    }

    pub(super) fn rasterize(
        icon: &DesktopShellIcon,
    ) -> Result<DesktopShellIconRgba, DesktopShellIconError> {
        let width = icon.width;
        let height = icon.height;
        if width == 0 || height == 0 {
            return Err(DesktopShellIconError::InvalidIconDimensions);
        }
        if width > MAX_DESKTOP_SHELL_ICON_DIMENSION || height > MAX_DESKTOP_SHELL_ICON_DIMENSION {
            return Err(DesktopShellIconError::IconTooLarge);
        }

        let canvas_width = width
            .checked_mul(2)
            .ok_or(DesktopShellIconError::InvalidIconDimensions)?;
        let byte_length = (canvas_width as usize)
            .checked_mul(height as usize)
            .and_then(|value| value.checked_mul(4))
            .ok_or(DesktopShellIconError::InvalidIconDimensions)?;
        let pixel_length = (width as usize)
            .checked_mul(height as usize)
            .and_then(|value| value.checked_mul(4))
            .ok_or(DesktopShellIconError::InvalidIconDimensions)?;

        let dc = NonNull::new(unsafe { CreateCompatibleDC(std::ptr::null_mut()) })
            .ok_or(DesktopShellIconError::RasterizationUnavailable)?;
        let dc = OwnedDeviceContext(dc.as_ptr());
        let bitmap_info = BitmapInfo {
            header: BitmapInfoHeader {
                size: std::mem::size_of::<BitmapInfoHeader>() as u32,
                width: canvas_width as i32,
                height: -(height as i32),
                planes: 1,
                bit_count: 32,
                compression: BI_RGB,
                size_image: byte_length as u32,
                x_pels_per_meter: 0,
                y_pels_per_meter: 0,
                colors_used: 0,
                colors_important: 0,
            },
            colors: [RgbQuad::default()],
        };
        let mut bits = std::ptr::null_mut();
        let raw_bitmap = unsafe {
            CreateDIBSection(
                dc.0,
                &bitmap_info,
                DIB_RGB_COLORS,
                &mut bits,
                std::ptr::null_mut(),
                0,
            )
        };
        if raw_bitmap.is_null() {
            return Err(DesktopShellIconError::RasterizationUnavailable);
        }
        let bitmap = OwnedGdiObject(raw_bitmap);
        if bits.is_null() {
            return Err(DesktopShellIconError::RasterizationUnavailable);
        }
        let previous = unsafe { SelectObject(dc.0, bitmap.0) };
        if previous.is_null() || previous as isize == HGDI_ERROR {
            return Err(DesktopShellIconError::RasterizationUnavailable);
        }
        let _selected = SelectedBitmap { dc: dc.0, previous };

        // Seed opaque black and white backgrounds. Their output difference
        // reveals coverage/alpha for source-over icons without assuming whether
        // the native icon RGB bytes are straight or premultiplied.
        unsafe {
            let canvas = std::slice::from_raw_parts_mut(bits.cast::<u8>(), byte_length);
            for y in 0..height as usize {
                for x in 0..canvas_width as usize {
                    let offset = (y * canvas_width as usize + x) * 4;
                    let value = if x < width as usize { 0 } else { 255 };
                    canvas[offset..offset + 4].copy_from_slice(&[value, value, value, 255]);
                }
            }
        }

        let black_drawn = unsafe {
            DrawIconEx(
                dc.0,
                0,
                0,
                icon.as_raw_handle(),
                width as i32,
                height as i32,
                0,
                std::ptr::null_mut(),
                DI_NORMAL,
            )
        };
        let white_drawn = unsafe {
            DrawIconEx(
                dc.0,
                width as i32,
                0,
                icon.as_raw_handle(),
                width as i32,
                height as i32,
                0,
                std::ptr::null_mut(),
                DI_NORMAL,
            )
        };
        if black_drawn == 0 || white_drawn == 0 {
            return Err(DesktopShellIconError::RasterizationUnavailable);
        }
        // CreateDIBSection requires a GDI flush before directly reading its
        // memory after drawing commands have been issued to the selected DIB.
        if unsafe { GdiFlush() } == 0 {
            return Err(DesktopShellIconError::RasterizationUnavailable);
        }

        let source = unsafe { std::slice::from_raw_parts(bits.cast::<u8>(), byte_length) };
        let mut pixels = Vec::new();
        pixels
            .try_reserve_exact(pixel_length)
            .map_err(|_| DesktopShellIconError::AllocationFailed)?;
        pixels.resize(pixel_length, 0);

        for y in 0..height as usize {
            for x in 0..width as usize {
                let black_offset = (y * canvas_width as usize + x) * 4;
                let white_offset = (y * canvas_width as usize + x + width as usize) * 4;
                let black = [
                    source[black_offset + 2],
                    source[black_offset + 1],
                    source[black_offset],
                ];
                let white = [
                    source[white_offset + 2],
                    source[white_offset + 1],
                    source[white_offset],
                ];
                let rgba = reconstruct_rgba_pixel(black, white)?;
                let output_offset = (y * width as usize + x) * 4;
                pixels[output_offset..output_offset + 4].copy_from_slice(&rgba);
            }
        }

        Ok(DesktopShellIconRgba {
            width,
            height,
            pixels,
        })
    }

    pub(super) fn extract(
        path: &Path,
        size: DesktopShellIconSize,
    ) -> Result<DesktopShellIcon, DesktopShellIconError> {
        let mut wide_path = path.as_os_str().encode_wide().collect::<Vec<_>>();
        if wide_path.contains(&0) {
            return Err(DesktopShellIconError::ShellIconUnavailable);
        }
        // SHGetFileInfoW documents a MAX_PATH-bounded path input; do not pass
        // extended-length paths through an API that does not promise support.
        if wide_path.len() + 1 > MAX_SHELL_PATH_UNITS {
            return Err(DesktopShellIconError::PathTooLong);
        }
        wide_path.push(0);

        let size_flag = match size {
            DesktopShellIconSize::Small => SHGFI_SMALLICON,
            DesktopShellIconSize::Large => 0,
        };
        let mut file_info = ShellFileInfoW::default();
        let result = unsafe {
            SHGetFileInfoW(
                wide_path.as_ptr(),
                0,
                &mut file_info,
                std::mem::size_of::<ShellFileInfoW>() as u32,
                SHGFI_ICON | SHGFI_SHELLICONSIZE | size_flag,
            )
        };
        if result == 0 {
            return Err(DesktopShellIconError::ShellIconUnavailable);
        }

        let icon = NonNull::new(file_info.icon)
            .map(OwnedIcon)
            .ok_or(DesktopShellIconError::ShellIconUnavailable)?;

        let mut icon_info = IconInfo::default();
        let icon_info_succeeded = unsafe { GetIconInfo(icon.0.as_ptr(), &mut icon_info) } != 0;
        // Own any handles the API populated even if it reports failure, so all
        // returned GDI objects are released on every error path.
        let _mask = OwnedGdiObject(icon_info.mask_bitmap);
        let color = OwnedGdiObject(icon_info.color_bitmap);
        if !icon_info_succeeded {
            return Err(DesktopShellIconError::IconMetadataUnavailable);
        }

        let (width, raw_height) = if !color.0.is_null() {
            bitmap_dimensions(color.0)?
        } else if !_mask.0.is_null() {
            // A monochrome HICON stores the AND and XOR masks stacked vertically.
            let (mask_width, mask_height) = bitmap_dimensions(_mask.0)?;
            if mask_height % 2 != 0 {
                return Err(DesktopShellIconError::InvalidIconDimensions);
            }
            (mask_width, mask_height / 2)
        } else {
            return Err(DesktopShellIconError::IconMetadataUnavailable);
        };

        if width == 0 || raw_height == 0 {
            return Err(DesktopShellIconError::InvalidIconDimensions);
        }

        Ok(DesktopShellIcon {
            icon,
            width,
            height: raw_height,
        })
    }

    fn bitmap_dimensions(
        handle: *mut std::ffi::c_void,
    ) -> Result<(u32, u32), DesktopShellIconError> {
        let mut bitmap = Bitmap::default();
        let bytes = unsafe {
            GetObjectW(
                handle,
                std::mem::size_of::<Bitmap>() as i32,
                (&mut bitmap as *mut Bitmap).cast(),
            )
        };
        if bytes == 0 || bitmap.width <= 0 || bitmap.height <= 0 {
            return Err(DesktopShellIconError::InvalidIconDimensions);
        }
        Ok((bitmap.width as u32, bitmap.height as u32))
    }
}

#[cfg(test)]
mod pixel_tests {
    use super::*;

    #[test]
    fn rgba_cache_images_reject_invalid_dimensions_and_buffer_lengths() {
        assert_eq!(
            DesktopShellIconRgba::from_pixels(1, 1, vec![0; 3]),
            Err(DesktopShellIconError::PixelBufferLengthMismatch)
        );
        assert_eq!(
            DesktopShellIconRgba::from_pixels(0, 1, Vec::new()),
            Err(DesktopShellIconError::InvalidIconDimensions)
        );
        assert_eq!(
            DesktopShellIconRgba::from_pixels(
                MAX_DESKTOP_SHELL_ICON_DIMENSION + 1,
                1,
                Vec::new()
            ),
            Err(DesktopShellIconError::IconTooLarge)
        );
        let image = DesktopShellIconRgba::from_pixels(1, 1, vec![10, 20, 30, 255]).unwrap();
        assert_eq!(image.pixels(), &[10, 20, 30, 255]);
        let view = image.as_adaptive_image_view().unwrap();
        assert_eq!(view.width(), 1);
        assert_eq!(view.height(), 1);
        assert_eq!(view.pixels(), &[10, 20, 30, 255]);
    }

    #[test]
    fn reconstructs_opaque_and_transparent_pixels() {
        assert_eq!(
            reconstruct_rgba_pixel([25, 80, 190], [25, 80, 190]).unwrap(),
            [25, 80, 190, 255]
        );
        assert_eq!(
            reconstruct_rgba_pixel([0, 0, 0], [255, 255, 255]).unwrap(),
            [0, 0, 0, 0]
        );
    }

    #[test]
    fn reconstructs_straight_color_from_premultiplied_source_over_probes() {
        // Original straight RGB [200, 100, 50] at approximately 50% coverage.
        let recovered = reconstruct_rgba_pixel([100, 50, 25], [227, 177, 152]).unwrap();
        assert!((i16::from(recovered[0]) - 200).abs() <= 2);
        assert!((i16::from(recovered[1]) - 100).abs() <= 2);
        assert!((i16::from(recovered[2]) - 50).abs() <= 2);
        assert!((i16::from(recovered[3]) - 128).abs() <= 2);
    }

    #[test]
    fn rejects_screen_dependent_inversion_pixels() {
        assert_eq!(
            reconstruct_rgba_pixel([255, 255, 255], [0, 0, 0]),
            Err(DesktopShellIconError::UnsupportedRasterOperation)
        );
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn extracts_an_owned_shell_icon_for_the_current_executable() {
        let path = std::env::current_exe().expect("resolve current test executable");
        let icon = platform::extract(&path, DesktopShellIconSize::Large)
            .expect("extract the executable's Shell icon");

        assert!(!icon.as_raw_handle().is_null());
        assert!(icon.width() > 0);
        assert!(icon.height() > 0);
        assert!(icon.width() <= MAX_DESKTOP_SHELL_ICON_DIMENSION);
        assert!(icon.height() <= MAX_DESKTOP_SHELL_ICON_DIMENSION);

        let rgba = rasterize_desktop_shell_icon_rgba(&icon)
            .expect("rasterize the current executable's Shell icon");
        assert_eq!(rgba.width(), icon.width());
        assert_eq!(rgba.height(), icon.height());
        assert_eq!(
            rgba.pixels().len(),
            rgba.width() as usize * rgba.height() as usize * 4
        );
        assert!(rgba.pixels().chunks_exact(4).any(|pixel| pixel[3] != 0));
    }
}
