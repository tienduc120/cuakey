//! The hotkey editor: four modifier checkboxes and a field that takes one key. The mapping between egui's keys and
//! the keys a hotkey may end on is pure, so a test reads it without a window.

use cuakey_application::{HotkeyView, KeyName, Toggle};
use egui::Key;

use crate::labels;

/// The keys the field accepts: A-Z, 0-9, F1-F12 and Space.
const KEYS: [(Key, KeyName); 49] = [
    (Key::A, KeyName::A),
    (Key::B, KeyName::B),
    (Key::C, KeyName::C),
    (Key::D, KeyName::D),
    (Key::E, KeyName::E),
    (Key::F, KeyName::F),
    (Key::G, KeyName::G),
    (Key::H, KeyName::H),
    (Key::I, KeyName::I),
    (Key::J, KeyName::J),
    (Key::K, KeyName::K),
    (Key::L, KeyName::L),
    (Key::M, KeyName::M),
    (Key::N, KeyName::N),
    (Key::O, KeyName::O),
    (Key::P, KeyName::P),
    (Key::Q, KeyName::Q),
    (Key::R, KeyName::R),
    (Key::S, KeyName::S),
    (Key::T, KeyName::T),
    (Key::U, KeyName::U),
    (Key::V, KeyName::V),
    (Key::W, KeyName::W),
    (Key::X, KeyName::X),
    (Key::Y, KeyName::Y),
    (Key::Z, KeyName::Z),
    (Key::Num0, KeyName::Digit0),
    (Key::Num1, KeyName::Digit1),
    (Key::Num2, KeyName::Digit2),
    (Key::Num3, KeyName::Digit3),
    (Key::Num4, KeyName::Digit4),
    (Key::Num5, KeyName::Digit5),
    (Key::Num6, KeyName::Digit6),
    (Key::Num7, KeyName::Digit7),
    (Key::Num8, KeyName::Digit8),
    (Key::Num9, KeyName::Digit9),
    (Key::F1, KeyName::F1),
    (Key::F2, KeyName::F2),
    (Key::F3, KeyName::F3),
    (Key::F4, KeyName::F4),
    (Key::F5, KeyName::F5),
    (Key::F6, KeyName::F6),
    (Key::F7, KeyName::F7),
    (Key::F8, KeyName::F8),
    (Key::F9, KeyName::F9),
    (Key::F10, KeyName::F10),
    (Key::F11, KeyName::F11),
    (Key::F12, KeyName::F12),
    (Key::Space, KeyName::Space),
];

/// The hotkey key an egui key stands for, or `None` for a key the field does not accept.
#[must_use]
pub fn key_from_egui(key: Key) -> Option<KeyName> {
    KEYS.iter()
        .find(|(egui_key, _)| *egui_key == key)
        .map(|(_, name)| *name)
}

/// The egui key a hotkey key is typed with, or `None` for a key the field cannot produce.
#[must_use]
pub fn key_to_egui(name: KeyName) -> Option<Key> {
    KEYS.iter().find(|(_, key_name)| *key_name == name).map(|(key, _)| *key)
}

/// What the field shows for a key.
#[must_use]
pub fn key_label(name: KeyName) -> &'static str {
    key_to_egui(name).map_or(labels::HOTKEY_OTHER_KEY, Key::name)
}

/// The hotkey after `key` is pressed in the field: Backspace clears the key, an accepted key replaces it, and any
/// other key changes nothing.
#[must_use]
pub fn press_key(hotkey: HotkeyView, key: Key) -> HotkeyView {
    if key == Key::Backspace {
        return HotkeyView { key: None, ..hotkey };
    }
    match key_from_egui(key) {
        Some(name) => HotkeyView {
            key: Some(name),
            ..hotkey
        },
        None => hotkey,
    }
}

fn modifier_checkbox(ui: &mut egui::Ui, held: &mut Toggle, label: &str) {
    let mut on = *held == Toggle::On;
    if ui.checkbox(&mut on, label).changed() {
        *held = if on { Toggle::On } else { Toggle::Off };
    }
}

/// Draws the editor for `hotkey` and returns the edited hotkey when the user changed it this frame.
pub(crate) fn show_hotkey(ui: &mut egui::Ui, id_salt: &str, hotkey: HotkeyView) -> Option<HotkeyView> {
    let mut edited = hotkey;
    ui.horizontal(|ui| {
        modifier_checkbox(ui, &mut edited.ctrl, labels::KEY_CTRL);
        modifier_checkbox(ui, &mut edited.alt, labels::KEY_ALT);
        modifier_checkbox(ui, &mut edited.win, labels::KEY_WIN);
        modifier_checkbox(ui, &mut edited.shift, labels::KEY_SHIFT);
        ui.push_id(id_salt, |ui| edited = key_field(ui, edited));
    });
    (edited != hotkey).then_some(edited)
}

/// The one-key field: click it, then press the key. While it has focus it reads the keys pressed.
fn key_field(ui: &mut egui::Ui, hotkey: HotkeyView) -> HotkeyView {
    let idle_text = hotkey.key.map_or(labels::HOTKEY_NO_KEY, key_label);
    let response = ui
        .add(egui::Button::new(idle_text).min_size(egui::vec2(96.0, 0.0)))
        .on_hover_text(labels::HOTKEY_FIELD_HINT);
    if response.clicked() {
        response.request_focus();
    }
    if !response.has_focus() {
        return hotkey;
    }
    let pressed: Vec<Key> = ui.input(|input| {
        input
            .events
            .iter()
            .filter_map(|event| {
                if let egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    ..
                } = event
                {
                    Some(*key)
                } else {
                    None
                }
            })
            .collect()
    });
    let edited = pressed.into_iter().fold(hotkey, press_key);
    if edited != hotkey {
        // One key is the whole input: leave the field so the next key press is not read as another edit.
        response.surrender_focus();
    }
    edited
}

#[cfg(test)]
mod tests;
