//! Exclusive wallpaper-presenter ownership for each current monitor.
//!
//! This is a portable configuration contract only. It does not launch an
//! external engine, inspect its runtime state, reserve a display surface, or
//! prove that a wallpaper has been applied. BGW/Manager must perform the
//! explicit hand-off and observe the result before treating the new presenter
//! as active.

use crate::{MonitorTopology, MAX_MONITORS, MAX_MONITOR_ID_BYTES};

/// A wallpaper presenter that may own one monitor at a time.
///
/// Rainmeter is deliberately absent: it is a widget/skin overlay and does not
/// compete for exclusive wallpaper ownership.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WallpaperPresenter {
    /// WinLook's native WLDE decoder and desktop presenter.
    NativeWlde,
    /// An already-installed Wallpaper Engine instance controlled by Steam.
    WallpaperEngine,
    /// An already-installed Lively Wallpaper instance.
    Lively,
}

/// One current monitor-to-wallpaper-presenter assignment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonitorPresentationAssignment {
    monitor_id: String,
    presenter: WallpaperPresenter,
}

impl MonitorPresentationAssignment {
    /// Creates an assignment for a current monitor snapshot ID.
    pub fn new(
        monitor_id: impl Into<String>,
        presenter: WallpaperPresenter,
    ) -> Result<Self, PresentationMapError> {
        let monitor_id = monitor_id.into();
        if monitor_id.is_empty() {
            return Err(PresentationMapError::EmptyMonitorId);
        }
        if monitor_id.len() > MAX_MONITOR_ID_BYTES {
            return Err(PresentationMapError::MonitorIdTooLong);
        }
        Ok(Self {
            monitor_id,
            presenter,
        })
    }

    pub fn monitor_id(&self) -> &str {
        &self.monitor_id
    }

    pub fn presenter(&self) -> WallpaperPresenter {
        self.presenter
    }
}

/// A complete, bounded mapping with exactly one wallpaper presenter per
/// monitor in the supplied current topology.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MonitorPresentationMap {
    assignments: Vec<MonitorPresentationAssignment>,
}

/// Errors returned when validating a monitor presentation map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentationMapError {
    TooManyAssignments,
    EmptyMonitorId,
    MonitorIdTooLong,
    DuplicateMonitorAssignment,
    UnknownMonitor,
    MissingMonitorAssignment,
}

impl std::fmt::Display for PresentationMapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooManyAssignments => {
                f.write_str("wallpaper presenter map exceeds the 16-monitor limit")
            }
            Self::EmptyMonitorId => f.write_str("monitor ID cannot be empty"),
            Self::MonitorIdTooLong => {
                f.write_str("monitor ID exceeds the 63-byte portable limit")
            }
            Self::DuplicateMonitorAssignment => {
                f.write_str("each monitor must have exactly one wallpaper presenter")
            }
            Self::UnknownMonitor => {
                f.write_str("wallpaper presenter map refers to a monitor outside the current topology")
            }
            Self::MissingMonitorAssignment => {
                f.write_str("every current monitor must have a wallpaper presenter assignment")
            }
        }
    }
}

impl std::error::Error for PresentationMapError {}

impl MonitorPresentationMap {
    /// Builds a complete map and validates it against the current topology.
    pub fn new(
        topology: &MonitorTopology,
        assignments: Vec<MonitorPresentationAssignment>,
    ) -> Result<Self, PresentationMapError> {
        validate_presentation_assignments(topology, &assignments)?;
        Ok(Self { assignments })
    }

    /// Creates a native-WLDE assignment for every current monitor.
    pub fn native_for(topology: &MonitorTopology) -> Self {
        let assignments = topology
            .monitors()
            .iter()
            .map(|monitor| MonitorPresentationAssignment {
                monitor_id: monitor.id().to_owned(),
                presenter: WallpaperPresenter::NativeWlde,
            })
            .collect();
        Self { assignments }
    }

    /// Returns the selected presenter for a current monitor ID.
    pub fn presenter_for(&self, monitor_id: &str) -> Option<WallpaperPresenter> {
        self.assignments
            .iter()
            .find(|assignment| assignment.monitor_id == monitor_id)
            .map(MonitorPresentationAssignment::presenter)
    }

    pub fn assignments(&self) -> &[MonitorPresentationAssignment] {
        &self.assignments
    }

    /// Replaces the full mapping only after validating it against the supplied
    /// topology. An incomplete/stale candidate never destroys the last valid map.
    pub fn replace(
        &mut self,
        topology: &MonitorTopology,
        assignments: Vec<MonitorPresentationAssignment>,
    ) -> Result<(), PresentationMapError> {
        validate_presentation_assignments(topology, &assignments)?;
        self.assignments = assignments;
        Ok(())
    }

    /// Changes one monitor's selected presenter transactionally.
    pub fn set_presenter(
        &mut self,
        topology: &MonitorTopology,
        monitor_id: &str,
        presenter: WallpaperPresenter,
    ) -> Result<(), PresentationMapError> {
        if !topology
            .monitors()
            .iter()
            .any(|monitor| monitor.id() == monitor_id)
        {
            return Err(PresentationMapError::UnknownMonitor);
        }

        let mut candidate = self.assignments.clone();
        let Some(assignment) = candidate
            .iter_mut()
            .find(|assignment| assignment.monitor_id == monitor_id)
        else {
            return Err(PresentationMapError::MissingMonitorAssignment);
        };
        assignment.presenter = presenter;
        self.replace(topology, candidate)
    }
}

fn validate_presentation_assignments(
    topology: &MonitorTopology,
    assignments: &[MonitorPresentationAssignment],
) -> Result<(), PresentationMapError> {
    if assignments.len() > MAX_MONITORS {
        return Err(PresentationMapError::TooManyAssignments);
    }
    if assignments.len() != topology.len() {
        return Err(PresentationMapError::MissingMonitorAssignment);
    }

    for (index, assignment) in assignments.iter().enumerate() {
        if assignment.monitor_id.is_empty() {
            return Err(PresentationMapError::EmptyMonitorId);
        }
        if assignment.monitor_id.len() > MAX_MONITOR_ID_BYTES {
            return Err(PresentationMapError::MonitorIdTooLong);
        }
        if !topology
            .monitors()
            .iter()
            .any(|monitor| monitor.id() == assignment.monitor_id)
        {
            return Err(PresentationMapError::UnknownMonitor);
        }
        if assignments[..index]
            .iter()
            .any(|previous| previous.monitor_id == assignment.monitor_id)
        {
            return Err(PresentationMapError::DuplicateMonitorAssignment);
        }
    }

    for monitor in topology.monitors() {
        if !assignments
            .iter()
            .any(|assignment| assignment.monitor_id == monitor.id())
        {
            return Err(PresentationMapError::MissingMonitorAssignment);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Monitor, Rect};

    fn topology() -> MonitorTopology {
        let first = Rect::new(0.0, 0.0, 1920.0, 1080.0).unwrap();
        let second = Rect::new(1920.0, 0.0, 2560.0, 1440.0).unwrap();
        MonitorTopology::new(vec![
            Monitor::new("DISPLAY-A", first, first, true).unwrap(),
            Monitor::new("DISPLAY-B", second, second, false).unwrap(),
        ])
        .unwrap()
    }

    fn assignment(id: &str, presenter: WallpaperPresenter) -> MonitorPresentationAssignment {
        MonitorPresentationAssignment::new(id, presenter).unwrap()
    }

    #[test]
    fn native_default_assigns_exactly_one_presenter_per_monitor() {
        let topology = topology();
        let map = MonitorPresentationMap::native_for(&topology);
        assert_eq!(map.assignments().len(), 2);
        assert_eq!(
            map.presenter_for("DISPLAY-A"),
            Some(WallpaperPresenter::NativeWlde)
        );
        assert_eq!(
            map.presenter_for("DISPLAY-B"),
            Some(WallpaperPresenter::NativeWlde)
        );
    }

    #[test]
    fn each_monitor_can_select_a_different_external_wallpaper_engine() {
        let topology = topology();
        let map = MonitorPresentationMap::new(
            &topology,
            vec![
                assignment("DISPLAY-A", WallpaperPresenter::WallpaperEngine),
                assignment("DISPLAY-B", WallpaperPresenter::Lively),
            ],
        )
        .unwrap();

        assert_eq!(
            map.presenter_for("DISPLAY-A"),
            Some(WallpaperPresenter::WallpaperEngine)
        );
        assert_eq!(
            map.presenter_for("DISPLAY-B"),
            Some(WallpaperPresenter::Lively)
        );
    }

    #[test]
    fn stale_unknown_or_duplicate_monitor_assignments_are_rejected() {
        let topology = topology();
        assert_eq!(
            MonitorPresentationMap::new(
                &topology,
                vec![
                    assignment("DISPLAY-A", WallpaperPresenter::NativeWlde),
                    assignment("DISPLAY-C", WallpaperPresenter::Lively),
                ],
            )
            .err(),
            Some(PresentationMapError::UnknownMonitor)
        );
        assert_eq!(
            MonitorPresentationMap::new(
                &topology,
                vec![
                    assignment("DISPLAY-A", WallpaperPresenter::NativeWlde),
                    assignment("DISPLAY-A", WallpaperPresenter::Lively),
                ],
            )
            .err(),
            Some(PresentationMapError::DuplicateMonitorAssignment)
        );
    }

    #[test]
    fn failed_replacement_preserves_the_last_valid_map() {
        let topology = topology();
        let mut map = MonitorPresentationMap::native_for(&topology);
        let before = map.clone();

        assert_eq!(
            map.replace(
                &topology,
                vec![assignment("DISPLAY-A", WallpaperPresenter::Lively)]
            ),
            Err(PresentationMapError::MissingMonitorAssignment)
        );
        assert_eq!(map, before);
    }

    #[test]
    fn changing_one_monitor_does_not_change_the_other_monitor() {
        let topology = topology();
        let mut map = MonitorPresentationMap::native_for(&topology);
        map.set_presenter(
            &topology,
            "DISPLAY-B",
            WallpaperPresenter::Lively,
        )
        .unwrap();

        assert_eq!(
            map.presenter_for("DISPLAY-A"),
            Some(WallpaperPresenter::NativeWlde)
        );
        assert_eq!(
            map.presenter_for("DISPLAY-B"),
            Some(WallpaperPresenter::Lively)
        );
        assert_eq!(
            map.set_presenter(
                &topology,
                "DISPLAY-C",
                WallpaperPresenter::WallpaperEngine
            ),
            Err(PresentationMapError::UnknownMonitor)
        );
    }
}
