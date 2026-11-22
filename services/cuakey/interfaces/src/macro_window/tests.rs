use cuakey_application::PreferencesView;

use super::MacroWindow;

#[test]
fn drawing_frames_asks_for_nothing() {
    let ctx = egui::Context::default();
    let mut window = MacroWindow::new(PreferencesView::default(), Vec::new());
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
fn the_import_question_draws_while_a_file_waits() {
    let ctx = egui::Context::default();
    let mut window = MacroWindow::new(PreferencesView::default(), Vec::new());
    window.form.choose_import_file("macros.txt".into());
    let mut actions = Vec::new();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| actions = window.show(ui));
    output.textures_delta.clear();
    assert!(actions.is_empty(), "{actions:?}");
    assert!(window.form.import_pending());
}
