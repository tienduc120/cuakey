use cuakey_application::{PreferencesView, Toggle};

use super::{COMPACT_SIZE, EXPANDED_SIZE, SettingsWindow, checkbox};
use crate::{Platform, UiAction, labels};

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

const MESSAGE: &str = "Hai phím tắt không được trùng nhau";

/// Room for everything, so a test sees all that is drawn whatever the test font measures.
const BIG_SCREEN: egui::Vec2 = egui::vec2(2000.0, 2000.0);

fn key(key: egui::Key) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    }
}

fn primary(pos: egui::Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
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

/// What one pass of the window asked for, the text it painted and where, and what it told its viewport.
struct Shown {
    actions: Vec<UiAction>,
    texts: Vec<(String, egui::Rect)>,
    commands: Vec<egui::ViewportCommand>,
}

fn collect_texts(shape: &egui::Shape, texts: &mut Vec<(String, egui::Rect)>) {
    if let egui::Shape::Vec(inner) = shape {
        for shape in inner {
            collect_texts(shape, texts);
        }
    } else if let egui::Shape::Text(text) = shape {
        let place = egui::Rect::from_min_size(text.pos, text.galley.size());
        texts.push((text.galley.text().to_owned(), place));
    }
}

/// Draws one pass of `window` in a screen of `size`, fed `events`.
fn show(ctx: &egui::Context, window: &mut SettingsWindow, size: egui::Vec2, events: Vec<egui::Event>) -> Shown {
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
        events,
        ..egui::RawInput::default()
    };
    let mut actions = Vec::new();
    // eframe hands the app the root ui of each pass, as `run_ui` does.
    let mut output = ctx.run_ui(input, |ui| actions = window.show(ui));
    output.textures_delta.clear();
    let mut texts = Vec::new();
    for clipped in &output.shapes {
        collect_texts(&clipped.shape, &mut texts);
    }
    let commands = output
        .viewport_output
        .get(&egui::ViewportId::ROOT)
        .map(|viewport| viewport.commands.clone())
        .unwrap_or_default();
    Shown {
        actions,
        texts,
        commands,
    }
}

fn place_of(shown: &Shown, text: &str) -> Option<egui::Rect> {
    shown
        .texts
        .iter()
        .find(|(drawn, _)| drawn == text)
        .map(|(_, place)| *place)
}

/// Clicks the button labelled `label` and returns the pass that saw the release.
fn click(ctx: &egui::Context, window: &mut SettingsWindow, label: &str) -> Shown {
    let before = show(ctx, window, BIG_SCREEN, Vec::new());
    let at = place_of(&before, label).unwrap().center();
    show(
        ctx,
        window,
        BIG_SCREEN,
        vec![egui::Event::PointerMoved(at), primary(at, true)],
    );
    show(ctx, window, BIG_SCREEN, vec![primary(at, false)])
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

#[test]
fn error_shows_in_compact_mode() {
    let ctx = egui::Context::default();
    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows);
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, COMPACT_SIZE);
    window.show_error(Some(MESSAGE));
    assert!(!window.form.is_expanded());

    let shown = show(&ctx, &mut window, COMPACT_SIZE, Vec::new());
    let place = place_of(&shown, MESSAGE).expect("the message is drawn");
    assert!(
        screen.contains_rect(place),
        "the message fits in the compact window: {place:?}"
    );

    window.form.toggle_expanded();
    show(&ctx, &mut window, EXPANDED_SIZE, Vec::new());
    window.form.toggle_expanded();
    let shown = show(&ctx, &mut window, COMPACT_SIZE, Vec::new());
    let place = place_of(&shown, MESSAGE).expect("the message stays after collapsing");
    assert!(screen.contains_rect(place), "{place:?}");
}

#[test]
fn the_expand_button_resizes_the_window_and_asks_nothing() {
    let ctx = egui::Context::default();
    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows);

    let expanded = click(&ctx, &mut window, labels::BUTTON_EXPAND);
    assert!(window.form.is_expanded());
    assert!(expanded.actions.is_empty(), "{:?}", expanded.actions);
    assert!(
        expanded
            .commands
            .contains(&egui::ViewportCommand::InnerSize(EXPANDED_SIZE)),
        "{:?}",
        expanded.commands
    );

    let collapsed = click(&ctx, &mut window, labels::BUTTON_COLLAPSE);
    assert!(!window.form.is_expanded());
    assert!(collapsed.actions.is_empty(), "{:?}", collapsed.actions);
    assert!(
        collapsed
            .commands
            .contains(&egui::ViewportCommand::InnerSize(COMPACT_SIZE)),
        "{:?}",
        collapsed.commands
    );
}

#[test]
fn the_reset_is_offered_last_and_only_when_expanded() {
    let ctx = egui::Context::default();
    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows);

    let compact = show(&ctx, &mut window, BIG_SCREEN, Vec::new());
    assert_eq!(place_of(&compact, labels::BUTTON_RESET), None);
    assert!(
        place_of(&compact, labels::SECTION_TYPING).is_none(),
        "the sections wait for the expand button"
    );

    window.form.toggle_expanded();
    let expanded = show(&ctx, &mut window, BIG_SCREEN, Vec::new());
    let reset = place_of(&expanded, labels::BUTTON_RESET).expect("the reset is drawn");
    let lowest_other = expanded
        .texts
        .iter()
        .filter(|(text, _)| text != labels::BUTTON_RESET)
        .map(|(_, place)| place.top())
        .fold(f32::MIN, f32::max);
    assert!(reset.top() > lowest_other, "the reset sits under everything else");
}

#[test]
fn clicking_the_reset_asks_for_it() {
    let ctx = egui::Context::default();
    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows);
    window.form.toggle_expanded();
    let shown = click(&ctx, &mut window, labels::BUTTON_RESET);
    assert_eq!(shown.actions, [UiAction::Reset]);
}
