use cuakey_application::{PreferencesView, Toggle};

use super::{SettingsWindow, checkbox};
use crate::{Platform, labels};

#[test]
fn drawing_frames_asks_for_nothing() {
    let ctx = egui::Context::default();
    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows);
    for _ in 0..2 {
        let mut actions = Vec::new();
        // eframe hands the app the root ui of each pass, as `run_ui` does.
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| actions = window.show(ui));
        // No renderer takes the font texture here.
        output.textures_delta.clear();
        assert!(actions.is_empty(), "{actions:?}");
    }
}

fn key(key: egui::Key) -> egui::Event {
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
    output.textures_delta.clear();
    drawn.unwrap()
}

#[test]
fn a_checkbox_reports_the_flipped_value() {
    for (current, flipped) in [(Toggle::Off, true), (Toggle::On, false)] {
        let ctx = egui::Context::default();
        pass(&ctx, vec![key(egui::Key::Tab)], |ui| {
            checkbox(ui, labels::BEEP, current)
        });
        let changed = pass(&ctx, vec![key(egui::Key::Space)], |ui| {
            checkbox(ui, labels::BEEP, current)
        });
        assert_eq!(changed, Some(flipped), "Space on a box showing {current:?}");
    }
}
