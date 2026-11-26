use cuakey_application::{HotkeyView, InterfaceLanguageValue, KeyName, Toggle};

use super::{KEYS, key_field, key_from_egui, key_label, key_to_egui, press_key, show_hotkey};
use crate::{labels, theme};

const VIETNAMESE: InterfaceLanguageValue = InterfaceLanguageValue::Vietnamese;

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
    assert_eq!(key_label(KeyName::F5, VIETNAMESE), "F5");
    assert_eq!(key_label(KeyName::Digit0, VIETNAMESE), "0");
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

fn key_event(key: egui::Key) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    }
}

/// Runs one pass fed `events` and returns what `draw` returned.
fn pass<R>(ctx: &egui::Context, events: Vec<egui::Event>, mut draw: impl FnMut(&mut egui::Ui) -> R) -> R {
    let input = egui::RawInput {
        events,
        ..egui::RawInput::default()
    };
    let mut drawn = None;
    let mut output = ctx.run_ui(input, |ui| drawn = Some(draw(ui)));
    // No renderer takes the font texture here.
    output.textures_delta.clear();
    drawn.unwrap()
}

#[test]
fn space_on_a_modifier_box_flips_that_modifier() {
    let ctx = egui::Context::default();
    let held = ctrl_shift(Some(KeyName::F5));
    pass(&ctx, vec![key_event(egui::Key::Tab)], |ui| {
        show_hotkey(ui, "hotkey", held, VIETNAMESE)
    });
    let edited = pass(&ctx, vec![key_event(egui::Key::Space)], |ui| {
        show_hotkey(ui, "hotkey", held, VIETNAMESE)
    });
    let ctrl_released = HotkeyView {
        ctrl: Toggle::Off,
        ..held
    };
    assert_eq!(
        edited,
        Some(ctrl_released),
        "Space on the focused Ctrl box releases Ctrl"
    );
}

/// A context with the look the app gives every window.
fn themed() -> egui::Context {
    let ctx = egui::Context::default();
    theme::apply(&ctx);
    ctx
}

/// The size of the key field showing `key`.
fn key_field_size(key: Option<KeyName>) -> egui::Vec2 {
    pass(&themed(), Vec::new(), |ui| {
        ui.scope(|ui| key_field(ui, ctrl_shift(key), VIETNAMESE))
            .response
            .rect
            .size()
    })
}

#[test]
fn key_field_takes_the_same_room_in_either_language() {
    let size_in = |language| {
        pass(&themed(), Vec::new(), |ui| {
            ui.scope(|ui| key_field(ui, ctrl_shift(None), language))
                .response
                .rect
                .size()
        })
    };
    let (width, needed) = pass(&themed(), Vec::new(), |ui| {
        let width = ui
            .scope(|ui| key_field(ui, ctrl_shift(None), VIETNAMESE))
            .response
            .rect
            .width();
        let widest = [labels::VIETNAMESE.hotkey_other_key, labels::ENGLISH.hotkey_other_key]
            .map(|text| crate::widgets::text_width(ui, text, egui::TextStyle::Button))
            .into_iter()
            .fold(0.0, f32::max);
        (width, widest + 2.0 * ui.spacing().button_padding.x)
    });
    assert_eq!(
        size_in(VIETNAMESE),
        size_in(InterfaceLanguageValue::English),
        "what follows the field stays in place when the language changes"
    );
    assert!(
        width >= needed,
        "{width} holds either language's text, which needs {needed}"
    );
}

#[test]
fn key_field_width_is_fixed() {
    assert_eq!(
        key_field_size(None),
        key_field_size(Some(KeyName::F5)),
        "a blank field and \"F5\" take the same room, so the rows stay aligned"
    );
}

fn collect_texts(shape: &egui::Shape, texts: &mut Vec<String>) {
    if let egui::Shape::Vec(inner) = shape {
        for shape in inner {
            collect_texts(shape, texts);
        }
    } else if let egui::Shape::Text(text) = shape {
        texts.push(text.galley.text().to_owned());
    }
}

#[test]
fn no_key_shows_blank() {
    let ctx = themed();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        key_field(ui, ctrl_shift(None), VIETNAMESE);
    });
    output.textures_delta.clear();
    let mut texts = Vec::new();
    for clipped in &output.shapes {
        collect_texts(&clipped.shape, &mut texts);
    }
    texts.retain(|text| !text.is_empty());
    assert!(texts.is_empty(), "a field with no key shows nothing: {texts:?}");
}

#[test]
fn key_field_fits_every_key_name() {
    let (width, needed) = pass(&themed(), Vec::new(), |ui| {
        let width = ui
            .scope(|ui| key_field(ui, ctrl_shift(Some(KeyName::F5)), VIETNAMESE))
            .response
            .rect
            .width();
        let widest = KEYS
            .iter()
            .map(|(_, name)| key_label(*name, VIETNAMESE))
            .chain([labels::VIETNAMESE.hotkey_other_key])
            .map(|text| {
                egui::WidgetText::from(text)
                    .into_galley(
                        ui,
                        Some(egui::TextWrapMode::Extend),
                        f32::INFINITY,
                        egui::TextStyle::Button,
                    )
                    .size()
                    .x
            })
            .fold(0.0, f32::max);
        (width, widest + 2.0 * ui.spacing().button_padding.x)
    });
    assert!(
        width >= needed,
        "{width} holds the widest key name, which needs {needed}"
    );
    assert!(width < 108.0, "{width} is narrower than the old field");
}
