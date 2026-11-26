//! Where a window opens and where it is. A place is kept in the physical pixels of the virtual screen, which mean the
//! same on every monitor, with the scale of the monitor the window was on. egui's points depend on the monitor: a
//! window takes a place in points and turns it into pixels at the scale of the monitor it is created on, so a window
//! saved on a monitor of another scale lands elsewhere and is moved onto its place once it says where it is.

use cuakey_application::{PlacedWindow, WindowPlace};
use cuakey_interfaces::WindowKind;
use eframe::egui::{InputState, Pos2, Vec2, pos2};
use platform_windows::ScreenRect;

/// The height of the title bar, in points: the strip of a window that must be on a monitor for it to open at its place,
/// since that is where the user takes hold of it.
const TITLE_BAR_HEIGHT: f32 = 30.0;

/// The window whose place `kind` takes: the about text is in the settings window.
pub(super) const fn placed_window(kind: WindowKind) -> PlacedWindow {
    match kind {
        WindowKind::Settings | WindowKind::About => PlacedWindow::Settings,
        WindowKind::Macros => PlacedWindow::Macros,
        WindowKind::Convert => PlacedWindow::Convert,
        WindowKind::AlwaysOn => PlacedWindow::AlwaysOn,
    }
}

/// Where a window `size` points big opens so its frame lands on `saved`, in egui's points at `zoom`. `None` when its
/// title bar there is on no monitor, as `on_a_monitor` says, or the place holds no scale: the window then opens where it
/// opens without one.
pub(super) fn opening_point(
    saved: WindowPlace,
    size: Vec2,
    zoom: f32,
    on_a_monitor: impl FnOnce(ScreenRect) -> bool,
) -> Option<Pos2> {
    let pixels_per_point = zoom * saved.scale;
    if !(pixels_per_point.is_finite() && pixels_per_point > 0.0) {
        return None;
    }
    let title_bar = ScreenRect {
        left: whole_pixels(saved.x),
        top: whole_pixels(saved.y),
        right: whole_pixels(saved.x + size.x * pixels_per_point),
        bottom: whole_pixels(saved.y + TITLE_BAR_HEIGHT * pixels_per_point),
    };
    on_a_monitor(title_bar).then(|| pos2(saved.x, saved.y) / pixels_per_point)
}

/// Where the native window of `input`'s viewport is, when it says: egui gives its frame in points, which are its pixels
/// divided by the viewport's pixels per point.
pub(super) fn reported_place(input: &InputState) -> Option<WindowPlace> {
    let viewport = input.viewport();
    let pixels = viewport.outer_rect?.min * input.pixels_per_point;
    Some(WindowPlace {
        x: pixels.x.round(),
        y: pixels.y.round(),
        scale: viewport.native_pixels_per_point?,
    })
}

/// The point in the viewport's points to move a window to, `pixels_per_point` giving their size, when it is `now` and
/// not on `target`.
pub(super) fn correction(target: WindowPlace, now: WindowPlace, pixels_per_point: f32) -> Option<Pos2> {
    // Whole pixels apart by less than half a pixel are the same pixel.
    let off = (target.x - now.x).abs() >= 0.5 || (target.y - now.y).abs() >= 0.5;
    off.then(|| pos2(target.x, target.y) / pixels_per_point)
}

/// `value` rounded to whole pixels, as Windows takes them.
#[expect(
    clippy::cast_possible_truncation,
    reason = "no conversion from a float to an integer is lossless; `as` saturates past the range, which no screen \
              reaches"
)]
fn whole_pixels(value: f32) -> i32 {
    value.round() as i32
}
