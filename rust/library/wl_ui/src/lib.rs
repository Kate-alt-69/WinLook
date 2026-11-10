#![forbid(unsafe_code)]

//! Shared native WinUI 3 shell and page composition for the WinLook Manager.
//!
//! The initial prototype deliberately reports unavailable BGW capabilities instead of
//! pretending that placeholder settings have been applied.

#[cfg(windows)]
mod shell;

#[cfg(windows)]
pub use shell::run;
