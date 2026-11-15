//! Read-only extraction of the Windows Shell icon for a desktop filesystem item.
//!
//! This adapter returns an owned HICON handle and its native dimensions. It
//! does not replace Explorer's icon registration, mutate the source item,
//! suppress the native tile or perform RGBA conversion. Rasterization and
//! adaptive composition are separate BGW work.

use std::fmt;
use std::ptr::NonNull;

use crate::DesktopInventoryItem;

/// The system icon size requested from the Shell image handler.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DesktopShellIconSize {
    Small,
    Large,
}

/// Errors returned while extracting a filesystem item's Shell icon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DesktopShellIconError {
    UnsupportedPlatform,
    ShellIconUnavailable,
    IconMetadataUnavailable,
    InvalidIconDimensions,
}

impl fmt::Display for DesktopShellIconError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => {
                f.write_str("desktop Shell icon extraction is available only on Windows")
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
        unsafe {
            DestroyIcon(self.0.as_ptr());
        }
    }
}

struct OwnedGdiObject(*mut std::ffi::c_void);

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

#[cfg(windows)]
mod platform {
    use super::*;
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;

    const SHGFI_ICON: u32 = 0x0000_0100;
    const SHGFI_SHELLICONSIZE: u32 = 0x0000_0004;
    const SHGFI_SMALLICON: u32 = 0x0000_0001;

    #[repr(C)]
    struct ShellFileInfoW {
        icon: *mut std::ffi::c_void,
        icon_index: i32,
        attributes: u32,
        display_name: [u16; 260],
        type_name: [u16; 80],
    }

    impl Default for ShellFileInfoW {
        fn default() -> Self {
            Self {
                icon: std::ptr::null_mut(),
                icon_index: 0,
                attributes: 0,
                display_name: [0; 260],
                type_name: [0; 80],
            }
        }
    }

    #[repr(C)]
    struct IconInfo {
        is_icon: i32,
        x_hotspot: u32,
        y_hotspot: u32,
        mask_bitmap: *mut std::ffi::c_void,
        color_bitmap: *mut std::ffi::c_void,
    }

    impl Default for IconInfo {
        fn default() -> Self {
            Self {
                is_icon: 0,
                x_hotspot: 0,
                y_hotspot: 0,
                mask_bitmap: std::ptr::null_mut(),
                color_bitmap: std::ptr::null_mut(),
            }
        }
    }

    #[repr(C)]
    #[derive(Default)]
    struct Bitmap {
        bitmap_type: i32,
        width: i32,
        height: i32,
        width_bytes: i32,
        planes: u16,
        bits_pixel: u16,
        bits: *mut std::ffi::c_void,
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
        fn DestroyIcon(icon: *mut std::ffi::c_void) -> i32;
        fn GetIconInfo(icon: *mut std::ffi::c_void, info: *mut IconInfo) -> i32;
    }

    #[link(name = "gdi32")]
    extern "system" {
        fn GetObjectW(
            object: *mut std::ffi::c_void,
            object_size: i32,
            object_data: *mut std::ffi::c_void,
        ) -> i32;
        fn DeleteObject(object: *mut std::ffi::c_void) -> i32;
    }

    pub(super) fn extract(
        path: &Path,
        size: DesktopShellIconSize,
    ) -> Result<DesktopShellIcon, DesktopShellIconError> {
        let mut wide_path = path.as_os_str().encode_wide().collect::<Vec<_>>();
        if wide_path.iter().any(|unit| *unit == 0) {
            return Err(DesktopShellIconError::ShellIconUnavailable);
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
        if unsafe { GetIconInfo(icon.0.as_ptr(), &mut icon_info) } == 0 {
            return Err(DesktopShellIconError::IconMetadataUnavailable);
        }
        let _mask = OwnedGdiObject(icon_info.mask_bitmap);
        let color = OwnedGdiObject(icon_info.color_bitmap);

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
        assert!(icon.width() <= 512);
        assert!(icon.height() <= 512);
    }
}
