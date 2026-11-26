use egui::{Rect, Vec2, pos2, vec2};

use super::{Placement, fit_on_screen};
use crate::{COMPACT_SIZE, EXPANDED_SIZE};

/// The height of a window's title bar, in points.
const TITLE_BAR: f32 = 31.0;

/// The width of the resize border Windows keeps around a window's sides and bottom, inside its outer rectangle.
const BORDER: f32 = 8.0;

/// The room the taskbar takes along the bottom of the screen.
const TASKBAR: f32 = 48.0;

/// A window of the inner size `size` with its outer top-left corner at `corner`: its outer and inner rectangles.
fn window_at(corner: egui::Pos2, size: Vec2) -> (Rect, Rect) {
    let outer = Rect::from_min_size(corner, size + vec2(2.0 * BORDER, TITLE_BAR + BORDER));
    let inner = Rect::from_min_size(corner + vec2(BORDER, TITLE_BAR), size);
    (outer, inner)
}

/// The compact settings window where it first opens on `monitor`: centred for its inner size, as eframe places it.
fn compact_centred_on(monitor: Vec2) -> (Rect, Rect) {
    window_at(((monitor - COMPACT_SIZE) * 0.5).to_pos2(), COMPACT_SIZE)
}

/// Where the inner bottom of a window `placed` from `outer` and `inner` ends up.
fn inner_bottom(outer: Rect, inner: Rect, placed: Placement) -> f32 {
    let top = placed.move_to.map_or_else(|| outer.top(), |corner| corner.y);
    top + (inner.top() - outer.top()) + placed.inner_size.y
}

#[test]
fn the_expanded_window_moves_up_to_stay_above_the_taskbar() {
    for (screen, monitor) in [
        ("1920x1080 at 125%", vec2(1536.0, 864.0)),
        ("1920x1080 at 150%", vec2(1280.0, 720.0)),
        ("1366x768 at 100%", vec2(1366.0, 768.0)),
    ] {
        let (outer, inner) = compact_centred_on(monitor);
        assert!(
            outer.top() + EXPANDED_SIZE.y + TITLE_BAR + BORDER > monitor.y - TASKBAR,
            "{screen}: growing in place would hide the bar"
        );

        let placed = fit_on_screen(outer, inner, monitor, EXPANDED_SIZE);

        assert_eq!(placed.inner_size, EXPANDED_SIZE, "{screen}: room for the whole size");
        let corner = placed.move_to.unwrap_or_else(|| panic!("{screen}: the window moves"));
        assert_eq!(corner.x, outer.left(), "{screen}: only up");
        assert!(
            corner.y >= 0.0 && corner.y < outer.top(),
            "{screen}: up, on screen: {corner:?}"
        );
        let bottom = inner_bottom(outer, inner, placed);
        assert!(
            bottom <= monitor.y - TASKBAR,
            "{screen}: the bar ends above the taskbar: {bottom}"
        );
    }
}

#[test]
fn a_tall_screen_leaves_the_window_where_it_is() {
    let monitor = vec2(2560.0, 1440.0);
    let (outer, inner) = compact_centred_on(monitor);

    let placed = fit_on_screen(outer, inner, monitor, EXPANDED_SIZE);

    assert_eq!(
        placed,
        Placement {
            move_to: None,
            inner_size: EXPANDED_SIZE,
        }
    );
}

#[test]
fn a_screen_shorter_than_the_window_caps_its_height() {
    let monitor = vec2(1024.0, 600.0);
    let (outer, inner) = compact_centred_on(monitor);

    let placed = fit_on_screen(outer, inner, monitor, EXPANDED_SIZE);

    assert_eq!(placed.move_to, Some(pos2(outer.left(), 0.0)), "the top of the screen");
    assert_eq!(
        placed.inner_size,
        vec2(EXPANDED_SIZE.x, placed.inner_size.y),
        "the width never changes"
    );
    assert!(placed.inner_size.y < EXPANDED_SIZE.y, "{:?}", placed.inner_size);
    let bottom = inner_bottom(outer, inner, placed) + BORDER;
    assert!(
        (bottom - (monitor.y - TASKBAR)).abs() < 0.5,
        "as tall as the screen allows, border and all: {bottom}"
    );
}

#[test]
fn a_window_at_the_top_of_a_short_screen_stays_there() {
    let monitor = vec2(1024.0, 600.0);
    let (outer, inner) = window_at(pos2(100.0, 0.0), COMPACT_SIZE);

    let placed = fit_on_screen(outer, inner, monitor, EXPANDED_SIZE);

    assert_eq!(placed.move_to, None, "it is as high as it goes");
    assert!(placed.inner_size.y < EXPANDED_SIZE.y, "{:?}", placed.inner_size);
}

#[test]
fn a_screen_shorter_than_the_open_window_never_shrinks_it() {
    let monitor = vec2(533.0, 400.0);
    let (outer, inner) = compact_centred_on(monitor);

    let placed = fit_on_screen(outer, inner, monitor, EXPANDED_SIZE);

    assert_eq!(
        placed.inner_size, COMPACT_SIZE,
        "growing keeps at least the size it had"
    );
    assert!(placed.move_to.is_some_and(|corner| corner.y >= 0.0));
}

#[test]
fn shrinking_on_screen_keeps_the_place() {
    let monitor = vec2(1280.0, 720.0);
    let (outer, inner) = window_at(pos2(360.0, 13.0), EXPANDED_SIZE);

    let placed = fit_on_screen(outer, inner, monitor, COMPACT_SIZE);

    assert_eq!(
        placed,
        Placement {
            move_to: None,
            inner_size: COMPACT_SIZE,
        }
    );
}
