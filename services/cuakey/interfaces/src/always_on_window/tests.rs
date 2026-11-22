use cuakey_application::PreferencesView;

use super::AlwaysOnWindow;

#[test]
fn drawing_frames_asks_for_nothing() {
    let ctx = egui::Context::default();
    let mut window = AlwaysOnWindow::new(PreferencesView::default());
    for _ in 0..2 {
        ctx.begin_pass(egui::RawInput::default());
        let actions = window.show(&ctx);
        let mut output = ctx.end_pass();
        // No renderer takes the font texture here.
        output.textures_delta.clear();
        assert!(actions.is_empty(), "{actions:?}");
    }
}
