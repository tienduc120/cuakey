use cuakey_interfaces::WindowKind;
use platform_windows::{AppMessage, WindowKind as ShownWindow};

use super::{
    BACKGROUND_CLASS, BACKGROUND_MUTEX, UI_CLASS, UI_MUTEX, instance_mutex, second_launch, shown_window, window_kind,
};
use crate::args::Mode;

const ALL_KINDS: [WindowKind; 5] = [
    WindowKind::Settings,
    WindowKind::Macros,
    WindowKind::Convert,
    WindowKind::AlwaysOn,
    WindowKind::About,
];

#[test]
fn the_names_are_fixed() {
    assert_eq!(BACKGROUND_MUTEX, r"Local\CuaKey.Background");
    assert_eq!(BACKGROUND_CLASS, "CuaKey.Messages");
    assert_eq!(UI_MUTEX, r"Local\CuaKey.Ui");
    assert_eq!(UI_CLASS, "CuaKey.UiMessages");
}

#[test]
fn each_mode_keeps_to_one_instance_with_its_own_mutex() {
    assert_eq!(instance_mutex(&Mode::Background { focus_log: None }), BACKGROUND_MUTEX);
    assert_eq!(instance_mutex(&Mode::Ui(WindowKind::Convert)), UI_MUTEX);
}

#[test]
fn a_second_background_asks_the_first_for_the_settings() {
    assert_eq!(
        second_launch(&Mode::Background { focus_log: None }),
        (BACKGROUND_CLASS, AppMessage::ShowWindow(ShownWindow::Settings))
    );
}

#[test]
fn a_second_window_process_asks_the_first_for_its_window() {
    assert_eq!(
        second_launch(&Mode::Ui(WindowKind::Macros)),
        (UI_CLASS, AppMessage::ShowWindow(ShownWindow::Macros))
    );
}

#[test]
fn window_kinds_cross_between_the_processes_unchanged() {
    for kind in ALL_KINDS {
        assert_eq!(window_kind(shown_window(kind)), kind);
    }
}
