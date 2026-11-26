use super::{ScreenRect, is_on_a_monitor};

#[test]
fn a_rect_at_the_primary_monitors_top_left_is_on_a_monitor() {
    // The primary monitor's top-left corner is where the virtual screen's coordinates start.
    let corner = ScreenRect {
        left: 0,
        top: 0,
        right: 100,
        bottom: 30,
    };

    assert!(is_on_a_monitor(corner));
}

#[test]
fn a_rect_far_from_every_monitor_is_on_none() {
    let far = ScreenRect {
        left: -100_000,
        top: -100_000,
        right: -99_900,
        bottom: -99_970,
    };

    assert!(!is_on_a_monitor(far));
}
