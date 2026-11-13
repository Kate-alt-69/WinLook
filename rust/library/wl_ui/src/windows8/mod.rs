//! Windows 8 alias for the baseline Windows 7-compatible Win32 renderer.
//!
//! Enable the `windows8` Cargo feature. The same backend is used because it
//! intentionally limits itself to the classic Win32 API surface.

pub use crate::windows7::*;
