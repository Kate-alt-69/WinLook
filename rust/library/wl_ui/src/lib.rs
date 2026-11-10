#![forbid(unsafe_code)]

//! Shared native WinUI 3 shell and page composition for the WinLook Manager.

#[cfg(windows)]
mod shell;

#[cfg(windows)]
pub use shell::run;
