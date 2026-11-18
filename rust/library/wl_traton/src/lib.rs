//! Traton is WinLook's version-aware Windows object-addressing layer.
//!
//! This first core slice provides a deterministic, platform-neutral parser for
//! Traton namespace paths. Native Windows discovery, providers, capabilities,
//! and event adapters are added behind this contract in later slices.
//!
//! A path identifies a target, subsystem/object/member path, and optionally a
//! typed opaque argument. It never embeds raw Win32 calls, handles, or values.

#![forbid(unsafe_code)]

mod namespace;

pub use namespace::{NamespacePath, NamespacePathError};
