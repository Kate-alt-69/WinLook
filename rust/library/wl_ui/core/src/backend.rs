use crate::{UiApplication, UiError};

/// Capabilities reported by a renderer. A shared UI model does not imply that
/// every backend can render every style or animation feature.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiBackendCapabilities {
    pub native_window: bool,
    pub css: bool,
    pub animation: bool,
    pub reduced_motion: bool,
    pub accessibility: bool,
}

/// Host-renderer extension point for non-Windows platforms and custom shells.
///
/// Implementations own their event loop, native resources, frame scheduling,
/// and teardown. The shared UI model remains platform-neutral.
pub trait UiBackend {
    /// Stable backend identifier suitable for diagnostics.
    fn name(&self) -> &'static str;

    /// Features implemented by this backend.
    fn capabilities(&self) -> UiBackendCapabilities;

    /// Runs the validated application model using this backend.
    fn run(&mut self, application: UiApplication) -> Result<(), UiError>;
}
