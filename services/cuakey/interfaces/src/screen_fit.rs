//! Where a window that changes size goes on screen. Windows keeps the top edge of a window it resizes and lets the
//! bottom pass the taskbar or the screen, which would hide the settings window's bottom bar.

use egui::{Pos2, Rect, Vec2, pos2, vec2};

/// The room kept free along the bottom of the monitor for the taskbar, which the monitor's size includes.
const TASKBAR_ALLOWANCE: f32 = 48.0;

/// Where to put a native window, and the inner size to ask of it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Placement {
    /// The new top-left corner of the outer window, when it must move.
    pub(crate) move_to: Option<Pos2>,
    pub(crate) inner_size: Vec2,
}

/// Where a window goes when it takes the inner size `wanted`, given its `outer` and `inner` rectangles now and the
/// size of its `monitor`, all in points. It stays where it is while its bottom ends above the taskbar. Otherwise it
/// moves up, no higher than the top of the screen, and if it still passes the taskbar it is only as tall as the room
/// left, though never shorter than it is now. Its width is always the wanted one.
pub(crate) fn fit_on_screen(outer: Rect, inner: Rect, monitor: Vec2, wanted: Vec2) -> Placement {
    let chrome = outer.height() - inner.height();
    let usable_bottom = monitor.y - TASKBAR_ALLOWANCE;
    let lowest_top = usable_bottom - (wanted.y + chrome);
    if outer.top() <= lowest_top {
        return Placement {
            move_to: None,
            inner_size: wanted,
        };
    }
    if lowest_top >= 0.0 {
        return Placement {
            move_to: Some(pos2(outer.left(), lowest_top)),
            inner_size: wanted,
        };
    }
    let room = usable_bottom - chrome;
    Placement {
        move_to: (outer.top() > 0.0).then(|| pos2(outer.left(), 0.0)),
        inner_size: vec2(wanted.x, room.max(inner.height()).min(wanted.y)),
    }
}

#[cfg(test)]
mod tests;
