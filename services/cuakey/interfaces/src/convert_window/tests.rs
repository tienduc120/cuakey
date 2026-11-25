use cuakey_application::{ConvertCaseValue, PreferencesView};

use super::ConvertWindow;
use crate::UiAction;

#[test]
fn drawing_frames_asks_for_nothing() {
    let ctx = egui::Context::default();
    let mut window = ConvertWindow::new(PreferencesView::default());
    for _ in 0..2 {
        let mut actions = Vec::new();
        // eframe hands the app the root ui of each pass, as `run_ui` does.
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| actions = window.show(ui));
        // No renderer takes the font texture here.
        output.textures_delta.clear();
        assert!(actions.is_empty(), "{actions:?}");
    }
}

#[test]
fn space_on_the_first_case_box_ticks_that_case() {
    let ctx = egui::Context::default();
    let mut window = ConvertWindow::new(PreferencesView::default());
    let mut expected = PreferencesView::default();
    expected.convert.case = ConvertCaseValue::Upper;
    assert_ne!(expected, PreferencesView::default(), "the default ticks no case");

    let mut actions = Vec::new();
    for key in [egui::Key::Tab, egui::Key::Space] {
        let input = egui::RawInput {
            events: vec![egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            ..egui::RawInput::default()
        };
        let mut output = ctx.run_ui(input, |ui| actions = window.show(ui));
        output.textures_delta.clear();
    }
    assert_eq!(actions, [UiAction::Save(expected)]);
}
