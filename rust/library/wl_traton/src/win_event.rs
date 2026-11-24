//! Portable normalization for the selected Windows WinEvent notifications.
//!
//! Translates Win32 event identifiers and callback object coordinates into
//! Traton categories. It does not install hooks or retain native handles.

use crate::events::TratonEventKind;

/// Exact event ranges BGW registers on its dedicated out-of-context pump thread.
pub const WIN_EVENT_HOOK_RANGES: &[(u32, u32)] = &[
    (0x0003, 0x0003), // EVENT_SYSTEM_FOREGROUND
    (0x0016, 0x0017), // EVENT_SYSTEM_MINIMIZESTART..EVENT_SYSTEM_MINIMIZEEND
    (0x8000, 0x8002), // EVENT_OBJECT_CREATE..EVENT_OBJECT_SHOW
    (0x800B, 0x800B), // EVENT_OBJECT_LOCATIONCHANGE
];

/// Coalesced category, not an identified window. OBJID_WINDOW / CHILDID_SELF
/// describes the window object itself; it does not prove root/top-level HWND.
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

    pub const fn bit(self) -> u32 {
        1_u32 << self as u32
    }

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

/// A non-null HWND is required. Foreground and object-level events are filtered
/// to OBJID_WINDOW / CHILDID_SELF. Minimize/restore use their system IDs and HWND.
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
        0x800B if object_id == 0 && child_id == 0 => Some(WinEventCategory::WindowGeometryChanged),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{classify_win_event, WinEventCategory, WIN_EVENT_HOOK_RANGES};
    use crate::TratonEventKind;

    #[test]
    fn classifies_selected_categories() {
        let cases = [
            (0x0003, WinEventCategory::ForegroundChanged),
            (0x8000, WinEventCategory::WindowCreated),
            (0x8001, WinEventCategory::WindowDestroyed),
            (0x8002, WinEventCategory::WindowShown),
            (0x800B, WinEventCategory::WindowGeometryChanged),
            (0x0016, WinEventCategory::WindowMinimized),
            (0x0017, WinEventCategory::WindowRestored),
        ];
        for (event, expected) in cases {
            assert_eq!(classify_win_event(event, true, 0, 0), Some(expected));
        }
    }

    #[test]
    fn object_events_require_window_object_and_self_child() {
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
    fn minimize_restore_require_window_but_ignore_unspecified_object_coordinates() {
        assert_eq!(
            classify_win_event(0x0016, true, 7, 12),
            Some(WinEventCategory::WindowMinimized)
        );
        assert_eq!(
            classify_win_event(0x0017, true, -1, 4),
            Some(WinEventCategory::WindowRestored)
        );
        assert_eq!(classify_win_event(0x0016, false, 0, 0), None);
        assert_eq!(classify_win_event(0x0017, false, 0, 0), None);
    }

    #[test]
    fn category_bits_and_normalized_kinds_are_stable() {
        let kinds = [
            TratonEventKind::WindowActivated,
            TratonEventKind::WindowCreated,
            TratonEventKind::WindowDestroyed,
            TratonEventKind::WindowShown,
            TratonEventKind::WindowGeometryChanged,
            TratonEventKind::WindowMinimized,
            TratonEventKind::WindowRestored,
        ];
        let mut seen = 0;
        for (index, category) in WinEventCategory::ALL.into_iter().enumerate() {
            assert_eq!(category.bit(), 1_u32 << index);
            assert_eq!(category.event_kind(), kinds[index]);
            assert_eq!(seen & category.bit(), 0);
            seen |= category.bit();
        }
    }

    #[test]
    fn hook_ranges_cover_selected_native_ids() {
        assert_eq!(
            WIN_EVENT_HOOK_RANGES,
            &[
                (0x0003, 0x0003),
                (0x0016, 0x0017),
                (0x8000, 0x8002),
                (0x800B, 0x800B)
            ]
        );
    }
}
