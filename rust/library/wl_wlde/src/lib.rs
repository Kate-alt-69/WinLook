//! Portable scene and policy core for the WinLook Desktop Engine (WLDE).
//!
//! This crate owns monitor geometry, wallpaper placement calculations,
//! visibility/Game Mode policy and the scene frame clock. It deliberately
//! does not create windows, timers, worker threads, decoders or a renderer.
//! Platform adapters report the presentation capabilities they really have.
//!
//! The host should use CPUv/GPUv for WLDE-owned work and resource leases. Their
//! estimates are admission/accounting units; they are not OS telemetry or a
//! guarantee of actual CPU/GPU execution time.

#![forbid(unsafe_code)]

use std::time::Duration;

pub use wl_animate::{FrameClock, FrameClockError, FrameTick};
pub use wl_cpuv as cpu;
pub use wl_gpuv as gpu;

pub const MAX_MONITORS: usize = 16;
pub const MAX_MONITOR_ID_BYTES: usize = 63;
pub const DEFAULT_FRAME_DELTA_LIMIT: Duration = Duration::from_millis(100);
pub const NORMAL_FRAME_INTERVAL: Duration = FrameClock::SIXTY_FPS_INTERVAL;
pub const OPTIMIZED_FRAME_INTERVAL: Duration = Duration::from_nanos(33_333_330);

fn finite(value: f64) -> bool {
    value.is_finite()
}

/// A position and positive size in desktop-coordinate space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Result<Self, GeometryError> {
        let value = Self {
            x,
            y,
            width,
            height,
        };
        if value.is_valid() {
            Ok(value)
        } else {
            Err(GeometryError::InvalidRectangle)
        }
    }

    pub fn is_valid(self) -> bool {
        finite(self.x)
            && finite(self.y)
            && finite(self.width)
            && finite(self.height)
            && self.width > 0.0
            && self.height > 0.0
    }
}

/// Pixel dimensions of a source image or decoded frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Size {
    pub fn new(width: f64, height: f64) -> Result<Self, GeometryError> {
        if finite(width) && finite(height) && width > 0.0 && height > 0.0 {
            Ok(Self { width, height })
        } else {
            Err(GeometryError::InvalidSize)
        }
    }

    pub fn is_valid(self) -> bool {
        finite(self.width) && finite(self.height) && self.width > 0.0 && self.height > 0.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeometryError {
    InvalidRectangle,
    InvalidSize,
    NonFiniteAspectRatio,
}

impl std::fmt::Display for GeometryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRectangle => {
                f.write_str("rectangle must have finite coordinates and positive dimensions")
            }
            Self::InvalidSize => f.write_str("source size must be finite and positive"),
            Self::NonFiniteAspectRatio => {
                f.write_str("aspect-ratio calculation is outside the finite range")
            }
        }
    }
}

impl std::error::Error for GeometryError {}

/// A monitor snapshot supplied by WLDM or the selected platform adapter.
#[derive(Clone, Debug, PartialEq)]
pub struct Monitor {
    id: String,
    bounds: Rect,
    work_area: Rect,
    primary: bool,
}

impl Monitor {
    pub fn new(
        id: impl Into<String>,
        bounds: Rect,
        work_area: Rect,
        primary: bool,
    ) -> Result<Self, TopologyError> {
        let id = id.into();
        if id.is_empty() {
            return Err(TopologyError::EmptyMonitorId);
        }
        if id.len() > MAX_MONITOR_ID_BYTES {
            return Err(TopologyError::MonitorIdTooLong);
        }
        if !bounds.is_valid() || !work_area.is_valid() {
            return Err(TopologyError::InvalidGeometry);
        }
        Ok(Self {
            id,
            bounds,
            work_area,
            primary,
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn bounds(&self) -> Rect {
        self.bounds
    }

    pub fn work_area(&self) -> Rect {
        self.work_area
    }

    pub fn is_primary(&self) -> bool {
        self.primary
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TopologyError {
    EmptyMonitorId,
    MonitorIdTooLong,
    InvalidGeometry,
    TooManyMonitors,
    DuplicateMonitorId,
    MultiplePrimaryMonitors,
}

impl std::fmt::Display for TopologyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyMonitorId => f.write_str("monitor ID cannot be empty"),
            Self::MonitorIdTooLong => f.write_str("monitor ID exceeds the 63-byte portable limit"),
            Self::InvalidGeometry => {
                f.write_str("monitor bounds and work area must be finite and positive")
            }
            Self::TooManyMonitors => f.write_str("monitor topology exceeds the 16-monitor bound"),
            Self::DuplicateMonitorId => f.write_str("monitor IDs must be unique within a topology"),
            Self::MultiplePrimaryMonitors => {
                f.write_str("a topology may contain at most one primary monitor")
            }
        }
    }
}

impl std::error::Error for TopologyError {}

/// Bounded, replaceable monitor topology. Update it only on display changes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MonitorTopology {
    monitors: Vec<Monitor>,
}

impl MonitorTopology {
    pub fn new(monitors: Vec<Monitor>) -> Result<Self, TopologyError> {
        validate_topology(&monitors)?;
        Ok(Self { monitors })
    }

    /// Validates the entire replacement before changing the active topology.
    pub fn replace(&mut self, monitors: Vec<Monitor>) -> Result<(), TopologyError> {
        validate_topology(&monitors)?;
        self.monitors = monitors;
        Ok(())
    }

    pub fn monitors(&self) -> &[Monitor] {
        &self.monitors
    }

    pub fn primary(&self) -> Option<&Monitor> {
        self.monitors.iter().find(|monitor| monitor.primary)
    }

    pub fn len(&self) -> usize {
        self.monitors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.monitors.is_empty()
    }
}

fn validate_topology(monitors: &[Monitor]) -> Result<(), TopologyError> {
    if monitors.len() > MAX_MONITORS {
        return Err(TopologyError::TooManyMonitors);
    }
    let mut primary_count = 0usize;
    for (index, monitor) in monitors.iter().enumerate() {
        if monitor.id.is_empty() || monitor.id.len() > MAX_MONITOR_ID_BYTES {
            return Err(TopologyError::EmptyMonitorId);
        }
        if !monitor.bounds.is_valid() || !monitor.work_area.is_valid() {
            return Err(TopologyError::InvalidGeometry);
        }
        if monitor.primary {
            primary_count += 1;
        }
        if monitors[..index]
            .iter()
            .any(|previous| previous.id == monitor.id)
        {
            return Err(TopologyError::DuplicateMonitorId);
        }
    }
    if primary_count > 1 {
        return Err(TopologyError::MultiplePrimaryMonitors);
    }
    Ok(())
}

/// How a source image is mapped into one monitor rectangle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScaleMode {
    /// Fill the viewport and crop the source symmetrically.
    Fill,
    /// Fit the entire source and leave unused space letterboxed/pillarboxed.
    Fit,
    /// Stretch the source independently on both axes.
    Stretch,
    /// Keep source pixel dimensions and center it in the viewport.
    Center,
}

/// Placement result. The source crop is normalized to the source image [0, 1].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    pub destination: Rect,
    pub source_crop: Rect,
}

pub fn compute_placement(
    source: Size,
    viewport: Rect,
    mode: ScaleMode,
) -> Result<Placement, GeometryError> {
    if !source.is_valid() {
        return Err(GeometryError::InvalidSize);
    }
    if !viewport.is_valid() {
        return Err(GeometryError::InvalidRectangle);
    }

    let source_ratio = source.width / source.height;
    let viewport_ratio = viewport.width / viewport.height;
    if !finite(source_ratio)
        || !finite(viewport_ratio)
        || source_ratio <= 0.0
        || viewport_ratio <= 0.0
    {
        return Err(GeometryError::NonFiniteAspectRatio);
    }

    let mut destination = viewport;
    let mut source_crop = Rect {
        x: 0.0,
        y: 0.0,
        width: 1.0,
        height: 1.0,
    };

    match mode {
        ScaleMode::Fill => {
            if source_ratio > viewport_ratio {
                source_crop.width = viewport_ratio / source_ratio;
                source_crop.x = (1.0 - source_crop.width) / 2.0;
            } else if source_ratio < viewport_ratio {
                source_crop.height = source_ratio / viewport_ratio;
                source_crop.y = (1.0 - source_crop.height) / 2.0;
            }
        }
        ScaleMode::Fit => {
            let (width, height) = if source_ratio > viewport_ratio {
                (viewport.width, viewport.width / source_ratio)
            } else {
                (viewport.height * source_ratio, viewport.height)
            };
            destination = Rect {
                x: viewport.x + (viewport.width - width) / 2.0,
                y: viewport.y + (viewport.height - height) / 2.0,
                width,
                height,
            };
        }
        ScaleMode::Stretch => {}
        ScaleMode::Center => {
            destination = Rect {
                x: viewport.x + (viewport.width - source.width) / 2.0,
                y: viewport.y + (viewport.height - source.height) / 2.0,
                width: source.width,
                height: source.height,
            };
        }
    }

    if !destination.is_valid() || !source_crop.is_valid() {
        return Err(GeometryError::NonFiniteAspectRatio);
    }
    Ok(Placement {
        destination,
        source_crop,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visibility {
    Visible,
    Contributing,
    FullyOccluded,
    /// Unknown is treated as visible; uncertain occlusion must not blank the desktop.
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameMode {
    Normal,
    Optimize,
    Suspend,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerState {
    Normal,
    PowerSaver,
    Suspended,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameAction {
    /// No animation is active; keep the already-presented static frame.
    StaticOnly,
    /// Advance the shared scene clock and active animation tracks.
    Animate,
    /// The scene is occluded; retain its last frame and stop requesting frames.
    HoldLastFrame,
    /// Game Mode or the power/session state explicitly suspended WLDE.
    Suspended,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SceneConditions {
    pub visibility: Visibility,
    pub game_mode: GameMode,
    pub power_state: PowerState,
    pub active_animations: usize,
}

impl Default for SceneConditions {
    fn default() -> Self {
        Self {
            visibility: Visibility::Unknown,
            game_mode: GameMode::Normal,
            power_state: PowerState::Normal,
            active_animations: 0,
        }
    }
}

/// One wallpaper/widget scene and its single shared host-driven frame clock.
///
/// This type does not create a timer. The host supplies monotonic timestamps
/// from its existing event/composition loop. The tick path does not allocate.
#[derive(Clone, Debug)]
pub struct Scene {
    conditions: SceneConditions,
    clock: FrameClock,
    clock_paused: bool,
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene {
    pub fn new() -> Self {
        Self {
            conditions: SceneConditions::default(),
            clock: FrameClock::sixty_fps(),
            clock_paused: false,
        }
    }

    pub fn conditions(&self) -> SceneConditions {
        self.conditions
    }

    pub fn set_conditions(&mut self, conditions: SceneConditions) {
        self.conditions = conditions;
    }

    pub fn frame_action(&self) -> FrameAction {
        if self.conditions.game_mode == GameMode::Suspend
            || self.conditions.power_state == PowerState::Suspended
        {
            return FrameAction::Suspended;
        }
        if self.conditions.visibility == Visibility::FullyOccluded {
            return FrameAction::HoldLastFrame;
        }
        if self.conditions.active_animations == 0 {
            return FrameAction::StaticOnly;
        }
        FrameAction::Animate
    }

    pub fn recommended_fps(&self) -> u32 {
        if self.frame_action() != FrameAction::Animate {
            return 0;
        }
        if self.conditions.game_mode == GameMode::Optimize
            || self.conditions.power_state == PowerState::PowerSaver
        {
            30
        } else {
            60
        }
    }

    /// Returns an emitted frame tick, or None when rendering should sleep.
    ///
    /// When the scene becomes hidden/suspended, the clock is paused. Resuming
    /// establishes a fresh baseline so hidden time is never replayed.
    pub fn tick(&mut self, now: Duration) -> Option<FrameTick> {
        let fps = self.recommended_fps();
        if fps == 0 {
            if !self.clock_paused {
                self.clock.pause();
            }
            self.clock_paused = true;
            return None;
        }

        let target = if fps == 30 {
            OPTIMIZED_FRAME_INTERVAL
        } else {
            NORMAL_FRAME_INTERVAL
        };
        if self.clock.target_interval() != target {
            self.clock = FrameClock::new(target, DEFAULT_FRAME_DELTA_LIMIT)
                .expect("WLDE frame intervals and delta limits are non-zero");
            self.clock_paused = true;
        }

        if self.clock_paused {
            self.clock.resume(now);
            self.clock_paused = false;
            return None;
        }
        self.clock.tick(now)
    }
}

/// Content class that a platform presentation adapter may support.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentKind {
    StaticImage,
    AnimatedMedia,
    Interactive,
}

/// Capabilities reported by the actual OS renderer/adapter, not inferred from
/// the presence of a portable WLDE core.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PresentationCapabilities {
    pub static_wallpaper: bool,
    pub animated_wallpaper: bool,
    pub interactive_content: bool,
    pub visibility_feedback: bool,
    pub virtual_groups: bool,
    pub crash_safe_group_restore: bool,
}

impl PresentationCapabilities {
    pub fn supports(self, kind: ContentKind) -> bool {
        match kind {
            ContentKind::StaticImage => self.static_wallpaper,
            ContentKind::AnimatedMedia => self.animated_wallpaper,
            ContentKind::Interactive => self.interactive_content,
        }
    }

    /// Groups stay unavailable unless both presentation and crash recovery exist.
    pub fn groups_available(self) -> bool {
        self.virtual_groups && self.crash_safe_group_restore
    }
}

/// Pair of the existing shared admission governors owned by this WLDE host.
///
/// These governors account for WinLook-owned work only. Callers supply
/// estimates and completion signals; they do not measure or control unrelated
/// processes. GPU resource leases are released only after the renderer has
/// released the actual resource, and submissions complete only after a fence.
#[derive(Debug)]
pub struct ResourceGovernors {
    pub cpu: wl_cpuv::Governor,
    pub gpu: wl_gpuv::Governor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceError {
    Cpu(wl_cpuv::Error),
    Gpu(wl_gpuv::Error),
}

impl std::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cpu(error) => write!(f, "CPU governor rejected its policy: {error:?}"),
            Self::Gpu(error) => write!(f, "GPU governor rejected its policy: {error:?}"),
        }
    }
}

impl std::error::Error for ResourceError {}

impl ResourceGovernors {
    pub fn new(
        cpu_policy: wl_cpuv::Policy,
        gpu_policy: wl_gpuv::Policy,
    ) -> Result<Self, ResourceError> {
        let cpu = wl_cpuv::Governor::new(cpu_policy).map_err(ResourceError::Cpu)?;
        let gpu = wl_gpuv::Governor::new(gpu_policy).map_err(ResourceError::Gpu)?;
        Ok(Self { cpu, gpu })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn monitor(id: &str, x: f64, primary: bool) -> Monitor {
        let bounds = Rect::new(x, 0.0, 1920.0, 1080.0).unwrap();
        Monitor::new(id, bounds, bounds, primary).unwrap()
    }

    #[test]
    fn topology_replacement_is_validated_and_bounded() {
        let mut topology = MonitorTopology::new(vec![
            monitor("DISPLAY-A", 0.0, true),
            monitor("DISPLAY-B", 1920.0, false),
        ])
        .unwrap();
        assert_eq!(topology.len(), 2);
        assert_eq!(topology.primary().unwrap().id(), "DISPLAY-A");

        let duplicate = vec![
            monitor("DISPLAY-A", 0.0, true),
            monitor("DISPLAY-A", 1920.0, false),
        ];
        assert_eq!(
            topology.replace(duplicate),
            Err(TopologyError::DuplicateMonitorId)
        );
        assert_eq!(topology.len(), 2);

        let multiple_primary = vec![
            monitor("DISPLAY-A", 0.0, true),
            monitor("DISPLAY-B", 1920.0, true),
        ];
        assert_eq!(
            topology.replace(multiple_primary),
            Err(TopologyError::MultiplePrimaryMonitors)
        );
    }

    #[test]
    fn placement_modes_keep_source_and_viewport_contracts() {
        let viewport = Rect::new(-1920.0, 0.0, 1920.0, 1080.0).unwrap();
        let source = Size::new(4000.0, 1000.0).unwrap();

        let fill = compute_placement(source, viewport, ScaleMode::Fill).unwrap();
        assert_eq!(fill.destination, viewport);
        assert!(fill.source_crop.width < 1.0);
        assert_eq!(fill.source_crop.height, 1.0);

        let fit = compute_placement(source, viewport, ScaleMode::Fit).unwrap();
        assert_eq!(fit.destination.width, 1920.0);
        assert_eq!(fit.destination.height, 480.0);
        assert_eq!(fit.source_crop, Rect::new(0.0, 0.0, 1.0, 1.0).unwrap());

        let stretch = compute_placement(source, viewport, ScaleMode::Stretch).unwrap();
        assert_eq!(stretch.destination, viewport);

        let centered = compute_placement(source, viewport, ScaleMode::Center).unwrap();
        assert_eq!(centered.destination.width, source.width);
        assert_eq!(centered.destination.height, source.height);
    }

    #[test]
    fn invalid_geometry_is_rejected() {
        assert_eq!(
            Rect::new(0.0, 0.0, f64::INFINITY, 1.0),
            Err(GeometryError::InvalidRectangle)
        );
        assert_eq!(Size::new(0.0, 1.0), Err(GeometryError::InvalidSize));
    }

    #[test]
    fn unknown_visibility_is_conservative_and_occlusion_stops_frame_requests() {
        let mut scene = Scene::new();
        scene.set_conditions(SceneConditions {
            visibility: Visibility::Unknown,
            game_mode: GameMode::Normal,
            power_state: PowerState::Normal,
            active_animations: 1,
        });
        assert_eq!(scene.frame_action(), FrameAction::Animate);
        assert_eq!(scene.tick(Duration::ZERO).unwrap().delta, Duration::ZERO);
        assert_eq!(
            scene.tick(Duration::from_millis(17)).unwrap().delta,
            Duration::from_millis(17)
        );

        scene.set_conditions(SceneConditions {
            visibility: Visibility::FullyOccluded,
            ..scene.conditions()
        });
        assert_eq!(scene.frame_action(), FrameAction::HoldLastFrame);
        assert!(scene.tick(Duration::from_millis(34)).is_none());

        scene.set_conditions(SceneConditions {
            visibility: Visibility::Visible,
            game_mode: GameMode::Optimize,
            ..scene.conditions()
        });
        assert_eq!(scene.recommended_fps(), 30);
        assert!(scene.tick(Duration::from_millis(50)).is_none());
        assert!(scene.tick(Duration::from_millis(84)).is_some());
    }

    #[test]
    fn static_scenes_and_suspended_scenes_do_not_request_animation_frames() {
        let mut scene = Scene::new();
        assert_eq!(scene.frame_action(), FrameAction::StaticOnly);
        assert_eq!(scene.recommended_fps(), 0);
        assert!(scene.tick(Duration::ZERO).is_none());

        scene.set_conditions(SceneConditions {
            visibility: Visibility::Visible,
            game_mode: GameMode::Suspend,
            power_state: PowerState::Normal,
            active_animations: 4,
        });
        assert_eq!(scene.frame_action(), FrameAction::Suspended);
        assert!(scene.tick(Duration::from_secs(10)).is_none());
    }

    #[test]
    fn power_saver_uses_lower_cadence_without_changing_source_quality() {
        let mut scene = Scene::new();
        scene.set_conditions(SceneConditions {
            visibility: Visibility::Contributing,
            game_mode: GameMode::Normal,
            power_state: PowerState::PowerSaver,
            active_animations: 1,
        });
        assert_eq!(scene.recommended_fps(), 30);
    }

    #[test]
    fn capabilities_gate_interactive_content_and_virtual_groups() {
        let mut capabilities = PresentationCapabilities::default();
        capabilities.static_wallpaper = true;
        capabilities.virtual_groups = true;
        assert!(capabilities.supports(ContentKind::StaticImage));
        assert!(!capabilities.supports(ContentKind::AnimatedMedia));
        assert!(!capabilities.supports(ContentKind::Interactive));
        assert!(!capabilities.groups_available());
        capabilities.crash_safe_group_restore = true;
        assert!(capabilities.groups_available());
    }

    #[test]
    fn resource_governors_use_the_existing_cpuv_and_gpuv_contracts() {
        let mut resources = ResourceGovernors::new(
            wl_cpuv::Policy {
                window_budget: 100,
                critical_reserve: 20,
                max_in_flight: 2,
            },
            wl_gpuv::Policy {
                memory_budget_bytes: 1024,
                critical_reserve_bytes: 128,
                max_in_flight_submissions: 2,
            },
        )
        .unwrap();

        let cpu_ticket = resources
            .cpu
            .try_acquire(wl_cpuv::WorkClass::Background, 80)
            .unwrap();
        assert!(!resources.cpu.complete(cpu_ticket, 70).unwrap().overrun);

        let gpu_ticket = resources
            .gpu
            .try_reserve(wl_gpuv::WorkClass::Background, 896)
            .unwrap();
        assert_eq!(resources.gpu.allocated_bytes(), 896);
        resources.gpu.release(gpu_ticket).unwrap();

        let submission = resources.gpu.begin_submission().unwrap();
        resources.gpu.complete_submission(submission).unwrap();
    }
}
