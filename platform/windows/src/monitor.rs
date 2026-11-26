//! The monitors attached now.

use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Gdi::{MONITOR_DEFAULTTONULL, MonitorFromRect};

/// A rectangle of the virtual screen in physical pixels, which mean the same on every monitor. The right and bottom
/// edges are just outside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

/// Whether some part of `rect` is on a monitor attached now.
pub fn is_on_a_monitor(rect: ScreenRect) -> bool {
    let rect = RECT {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    };
    // SAFETY: the call only reads the rectangle, which lives on this frame for the whole call, and the monitor handle
    // it returns is the system's, never freed by the caller.
    let monitor = unsafe { MonitorFromRect(&raw const rect, MONITOR_DEFAULTTONULL) };
    !monitor.is_invalid()
}

#[cfg(test)]
mod tests;
