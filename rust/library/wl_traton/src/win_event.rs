//! Portable normalization for the selected Windows WinEvent notifications.
//!
//! This module translates Win32 event identifiers and callback object coordinates
//! into Traton categories. It does not install hooks or retain native handles.

use crate::events::TratonEventKind;

/// Exact event ranges BGW registers on its dedicated out-of-context pump thread.
pub const WIN_EVENT_HOOK_RANGES: &[(u32, u32)] = &[
    (0x0003, 0x0003), // EVENT_SYSTEM_FOREGROUND
    (0x0016, 0x0017), // EVENT_SYSTEM_MINIMIZESTART..EVENT_SYSTEM_MINIMIZEEND
    (0x8000, 0x8002), // EVENT_OBJECT_CREATE..EVENT_OBJECT_SHOW
    (0x800B, 0x800B), // EVENT_OBJECT_LOCATIONCHANGE
];

/// Coalescible categories emitted by Traton's selected WinEvent classifier.
///
/// The category describes an event type, not an identified window. In particular,
/// the object filter does not establish that an HWND is a root/top-level window.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum WinEventCategory {
    ForegroundChanged = 0,
    WindowCreated = 1,
    WindowDestroyed = 2,
    WindowShown = 3,
    WindowGeometryChanged = 4,
    WindowMinimized = 5,
    WindowRestored = 6,
}

impl WinEventCategory {
    pub const ALL: [Self; 7] = [
        Self::ForegroundChanged,
        Self::WindowCreated,
        Self::WindowDestroyed,
        Self::WindowShown,
        Self::WindowGeometryChanged,
        Self::WindowMinimized,
        Self::WindowRestored,
    ];

    /// Bit used by BGW's atomic pending-category mask.
    pub const fn bit(self) -> u32 {
        1_u32 << self as u32
    }

    /// Converts a native category into a platform-neutral Traton event kind.
    pub const fn event_kind(self) -> TratonEventKind {
        match self {
            Self::ForegroundChanged => TratonEventKind::WindowActivated,
            Self::WindowCreated => TratonEventKind::WindowCreated,
            Self::WindowDestroyed => TratonEventKind::WindowDestroyed,
            Self::WindowShown => TratonEventKind::WindowShown,
            Self::WindowGeometryChanged => TratonEventKind::WindowGeometryChanged,
            Self::WindowMinimized => TratonEventKind::WindowMinimized,
            Self::WindowRestored => TratonEventKind::WindowRestored,
        }
    }
}

/// Classifies only event categories subscribed by BGW.
///
/// A non-null HWND is required for every category. Foreground and object-level
/// events are restricted to OBJID_WINDOW and CHILDID_SELF. That identifies the
/// window object itself; it does not prove that the HWND has no parent. System
/// minimize/restore events are classified by event ID and non-null HWND because
/// their object coordinates are not used as a separate filter by this adapter.
pub fn classify_win_event(
    event: u32,
    has_window: bool,
    object_id: i32,
    child_id: i32,
) -> Option<WinEventCategory> {
    if !has_window {
        return None;
    }

    match event {
        0x0003 if object_id == 0 && child_id == 0 => Some(WinEventCategory::ForegroundChanged),
        0x0016 => Some(WinEventCategory::WindowMinimized),
        0x0017 => Some(WinEventCategory::WindowRestored),
        0x8000 if object_id == 0 && child_id == 0 => Some(WinEventCategory::WindowCreated),
        0x8001 if object_id == 0 && child_id == 0 => Some(WinEventCategory::WindowDestroyed),
        0x8002 if object_id == 0 && child_id == 0 => Some(WinEventCategory::WindowShown),
        0x800B if object_id == 0 && child_id == 0 => {
            Some(WinEventCategory::WindowGeometryChanged)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{classify_win_event, WinEventCategory, WIN_EVENT_HOOK_RANGES};
    use crate::TratonEventKind;

    #[test]
    fn classifies_each_selected_window_event_category() {
        assert_eq!(classify_win_event(0x0003, true, 0, 0), Some(WinEventCategory::ForegroundChanged));
        assert_eq!(classify_win_event(0x8000, true, 0, 0), Some(WinEventCategory::WindowCreated));
        assert_eq!(classify_win_event(0x8001, true, 0, 0), Some(WinEventCategory::WindowDestroyed));
        assert_eq!(classify_win_event(0x8002, true, 0, 0), Some(WinEventCategory::WindowShown));
        assert_eq!(classify_win_event(0x800B, true, 0, 0), Some(WinEventCategory::WindowGeometryChanged));
        assert_eq!(classify_win_event(0x0016, true, 0, 0), Some(WinEventCategory::WindowMinimized));
        assert_eq!(classify_win_event(0x0017, true, 0, 0), Some(WinEventCategory::WindowRestored));
    }

    #[test]
    fn object_events_require_the_window_object_and_self_child() {
        for event in [0x8000, 0x8001, 0x8002, 0x800B] {
            assert_eq!(classify_win_event(event, true, 1, 0), None);
            assert_eq!(classify_win_event(event, true, 0, 1), None);
            assert_eq!(classify_win_event(event, false, 0, 0), None);
        }
        assert_eq!(classify_win_event(0x0003, true, 1, 0), None);
        assert_eq!(classify_win_event(0x0003, true, 0, 1), None);
        assert_eq!(classify_win_event(0x0003, false, 0, 0), None);
    }

    #[test]
    fn minimize_restore_categories_require_a_window_but_do_not_assume_object_ids() {
        assert_eq!(classify_win_event(0x0016, true, 7, 12), Some(WinEventCategory::WindowMinimized));
        assert_eq!(classify_win_event(0x0017, true, -1, 4), Some(WinEventCategory::WindowRestored));
        assert_eq!(classify_win_event(0x0016, false, 0, 0), None);
        assert_eq!(classify_win_event(0x0017, false, 0, 0), None);
    }

    #[test]
    fn category_bits_are_unique_and_map_to_normalized_event_kinds() {
        let expected = [
            TratonEventKind::WindowActivated,
            TratonEventKind::WindowCreated,
            TratonEventKind::WindowDestroyed,
            TratonEventKind::WindowShown,
            TratonEventKind::WindowGeometryChanged,
            TratonEventKind::WindowMinimized,
            TratonEventKind::WindowRestored,
        ];
        let mut seen = 0_u32;
        for (index, category) in WinEventCategory::ALL.into_iter().enumerate() {
            assert_eq!(category.bit(), 1_u32 << index);
            assert_eq!(category.event_kind(), expected[index]);
            assert_eq!(seen & category.bit(), 0);
            seen |= category.bit();
        }
    }

    #[test]
    fn hook_ranges_cover_the_selected_native_event_ids() {
        assert_eq!(WIN_EVENT_HOOK_RANGES, &[(0x0003, 0x0003), (0x0016, 0x0017), (0x8000, 0x8002), (0x800B, 0x800B)]);
    }
}
