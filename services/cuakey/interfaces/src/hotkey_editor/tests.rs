use cuakey_application::{HotkeyView, KeyName, Toggle};

use super::{key_from_egui, key_label, key_to_egui, press_key};

fn ctrl_shift(key: Option<KeyName>) -> HotkeyView {
    HotkeyView {
        ctrl: Toggle::On,
        alt: Toggle::Off,
        win: Toggle::Off,
        shift: Toggle::On,
        key,
    }
}

#[test]
fn hotkey_key_names() {
    assert_eq!(key_from_egui(egui::Key::F5), Some(KeyName::F5));
    assert_eq!(key_to_egui(KeyName::F5), Some(egui::Key::F5));
    assert_eq!(key_from_egui(egui::Key::A), Some(KeyName::A));
    assert_eq!(key_from_egui(egui::Key::Num7), Some(KeyName::Digit7));
    assert_eq!(key_from_egui(egui::Key::Space), Some(KeyName::Space));
    assert_eq!(key_from_egui(egui::Key::Enter), None);
    assert_eq!(key_to_egui(KeyName::Enter), None);
    assert_eq!(key_label(KeyName::F5), "F5");
    assert_eq!(key_label(KeyName::Digit0), "0");
}

#[test]
fn every_accepted_key_maps_back() {
    for key in [
        egui::Key::A,
        egui::Key::Z,
        egui::Key::Num0,
        egui::Key::Num9,
        egui::Key::F1,
        egui::Key::F12,
    ] {
        let name = key_from_egui(key).unwrap();
        assert_eq!(key_to_egui(name), Some(key));
    }
}

#[test]
fn backspace_clears_the_key() {
    let held = ctrl_shift(Some(KeyName::F5));
    assert_eq!(press_key(held, egui::Key::Backspace), ctrl_shift(None));
}

#[test]
fn an_accepted_key_replaces_the_key_and_keeps_modifiers() {
    assert_eq!(
        press_key(ctrl_shift(None), egui::Key::F6),
        ctrl_shift(Some(KeyName::F6))
    );
}

#[test]
fn an_unaccepted_key_is_ignored() {
    let held = ctrl_shift(Some(KeyName::F5));
    assert_eq!(press_key(held, egui::Key::Enter), held);
}
