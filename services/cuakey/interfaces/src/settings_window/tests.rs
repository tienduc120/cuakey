use cuakey_application::{KeyName, PreferencesView, Toggle};
use egui::epaint::RectShape;

use super::{BAR_LABELS, COMPACT_SIZE, EXPANDED_SIZE, SettingsWindow, bar_button_width, checkbox};
use crate::theme::{self, BUTTON_SIZE};
use crate::{Platform, UiAction, labels, widgets};

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

/// A context with the look the app gives every window.
fn themed() -> egui::Context {
    let ctx = egui::Context::default();
    theme::apply(&ctx);
    ctx
}

/// The room the theme leaves between the window's edge and what it shows.
fn margin() -> f32 {
    f32::from(themed().global_style().spacing.window_margin.left)
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

/// What one pass of the window asked for, the text and rectangles it painted, and what it told its viewport.
struct Shown {
    actions: Vec<UiAction>,
    texts: Vec<(String, egui::Rect)>,
    /// The font size and colour each text was laid out with.
    formats: Vec<(String, f32, egui::Color32)>,
    /// The texts painted whole inside the area they are clipped to: on screen, and not scrolled out of view.
    visible: Vec<String>,
    rects: Vec<RectShape>,
    commands: Vec<egui::ViewportCommand>,
}

fn collect(shape: &egui::Shape, clip: egui::Rect, shown: &mut Shown) {
    if let egui::Shape::Vec(inner) = shape {
        for shape in inner {
            collect(shape, clip, shown);
        }
    } else if let egui::Shape::Text(text) = shape {
        let drawn = text.galley.text().to_owned();
        let place = egui::Rect::from_min_size(text.pos, text.galley.size());
        if clip.contains_rect(place) {
            shown.visible.push(drawn.clone());
        }
        if let Some(section) = text.galley.job.sections.first() {
            let format = &section.format;
            shown.formats.push((drawn.clone(), format.font_id.size, format.color));
        }
        shown.texts.push((drawn, place));
    } else if let egui::Shape::Rect(rect) = shape {
        shown.rects.push(rect.clone());
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
    let commands = output
        .viewport_output
        .get(&egui::ViewportId::ROOT)
        .map(|viewport| viewport.commands.clone())
        .unwrap_or_default();
    let mut shown = Shown {
        actions,
        texts: Vec::new(),
        formats: Vec::new(),
        visible: Vec::new(),
        rects: Vec::new(),
        commands,
    };
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, size);
    for clipped in &output.shapes {
        collect(&clipped.shape, clipped.clip_rect.intersect(screen), &mut shown);
    }
    shown
}

fn place_of(shown: &Shown, text: &str) -> Option<egui::Rect> {
    shown
        .texts
        .iter()
        .find(|(drawn, _)| drawn == text)
        .map(|(_, place)| *place)
}

fn places_of(shown: &Shown, text: &str) -> Vec<egui::Rect> {
    shown
        .texts
        .iter()
        .filter(|(drawn, _)| drawn == text)
        .map(|(_, place)| *place)
        .collect()
}

/// The frame of the button labelled `label`: the smallest rectangle painted around its text.
fn button_of(shown: &Shown, label: &str) -> egui::Rect {
    let text = place_of(shown, label).unwrap_or_else(|| panic!("{label} is drawn"));
    shown
        .rects
        .iter()
        .map(|shape| shape.rect)
        .filter(|rect| rect.contains(text.center()))
        .min_by(|a, b| a.area().total_cmp(&b.area()))
        .unwrap_or_else(|| panic!("{label} has a frame"))
}

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.5
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
    let ctx = themed();
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
    let bar = button_of(&shown, labels::BUTTON_CLOSE);
    assert!(place.bottom() < bar.top(), "the message sits above the bar");

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
fn the_reset_is_offered_only_when_expanded() {
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
    assert!(place_of(&expanded, labels::BUTTON_RESET).is_some());
}

#[test]
fn clicking_the_reset_asks_for_it() {
    let ctx = egui::Context::default();
    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows);
    window.form.toggle_expanded();
    let shown = click(&ctx, &mut window, labels::BUTTON_RESET);
    assert_eq!(shown.actions, [UiAction::Reset]);
}

#[test]
fn the_window_reports_the_size_of_what_it_shows() {
    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows);
    assert_eq!(window.size(), COMPACT_SIZE, "a new window is compact");

    window.form.toggle_expanded();
    assert_eq!(window.size(), EXPANDED_SIZE);

    window.form.toggle_expanded();
    assert_eq!(window.size(), COMPACT_SIZE);
}

#[test]
fn a_window_opened_expanded_shows_every_section() {
    let ctx = egui::Context::default();
    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows).expanded();
    assert!(window.form.is_expanded());
    assert_eq!(window.size(), EXPANDED_SIZE);

    let shown = show(&ctx, &mut window, BIG_SCREEN, Vec::new());
    assert!(
        place_of(&shown, labels::SECTION_ABOUT).is_some(),
        "the about section is drawn"
    );
    assert!(place_of(&shown, labels::BUTTON_COLLAPSE).is_some());
}

#[test]
fn about_expands_an_open_window() {
    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows);
    window.expand();
    assert_eq!(window.size(), EXPANDED_SIZE);

    window.expand();
    assert_eq!(window.size(), EXPANDED_SIZE, "an expanded window stays expanded");
}

#[test]
fn buttons_sit_in_a_bottom_bar() {
    let ctx = themed();
    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows);
    let right = [labels::BUTTON_EXPAND, labels::BUTTON_EXIT, labels::BUTTON_CLOSE];
    let size = egui::vec2(
        pass(&ctx, Vec::new(), |ui| bar_button_width(ui, &BAR_LABELS)),
        BUTTON_SIZE.y,
    );

    let compact = show(&ctx, &mut window, COMPACT_SIZE, Vec::new());
    let buttons = right.map(|label| button_of(&compact, label));
    let lowest_row = compact
        .texts
        .iter()
        .filter(|(text, _)| !right.contains(&text.as_str()))
        .map(|(_, place)| place.bottom())
        .fold(f32::MIN, f32::max);
    for (label, button) in right.iter().zip(buttons) {
        assert_eq!(button.size(), size, "{label}");
        assert!(button.top() > lowest_row, "{label} sits below every row");
        assert!(
            near(button.bottom(), COMPACT_SIZE.y - margin()),
            "{label} sits on the bottom edge: {button:?}"
        );
    }
    let [expand, exit, close] = buttons;
    assert!(expand.right() < exit.left() && exit.right() < close.left(), "in order");
    assert!(
        near(close.right(), COMPACT_SIZE.x - margin()),
        "right-aligned: {close:?}"
    );

    window.form.toggle_expanded();
    let expanded = show(&ctx, &mut window, EXPANDED_SIZE, Vec::new());
    let reset = button_of(&expanded, labels::BUTTON_RESET);
    let close = button_of(&expanded, labels::BUTTON_CLOSE);
    assert_eq!(reset.size(), size);
    assert!(near(reset.left(), margin()), "Cài đặt gốc at the left end: {reset:?}");
    assert!(near(reset.top(), close.top()), "on the same bar");
    assert!(
        near(close.bottom(), EXPANDED_SIZE.y - margin()),
        "the bar stays on the bottom edge, past the scrolling sections: {close:?}"
    );
}

#[test]
fn control_rows_have_no_frame() {
    let ctx = themed();
    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows);
    let shown = show(&ctx, &mut window, COMPACT_SIZE, Vec::new());
    let rows = place_of(&shown, labels::INPUT_METHOD)
        .unwrap()
        .union(place_of(&shown, labels::LANGUAGE).unwrap());
    let framing: Vec<_> = shown
        .rects
        .iter()
        .filter(|shape| shape.stroke.width > 0.0 && shape.rect.contains_rect(rows))
        .collect();
    assert!(framing.is_empty(), "no outline around the rows: {framing:?}");
}

#[test]
fn hotkey_rows_line_up() {
    let mut view = PreferencesView::default();
    view.switching.hotkey.key = Some(KeyName::F5);
    view.behaviours.settings_hotkey.key = Some(KeyName::F5);
    let ctx = themed();
    let mut window = SettingsWindow::new(view, Platform::Windows);
    let shown = show(&ctx, &mut window, BIG_SCREEN, Vec::new());

    for text in [labels::KEY_CTRL, "F5"] {
        let places = places_of(&shown, text);
        assert_eq!(places.len(), 2, "{text} in each hotkey row");
        assert!(
            places.iter().all(|place| near(place.left(), places[0].left())),
            "the two rows' {text} start at the same x: {places:?}"
        );
    }
}

#[test]
fn sections_use_two_columns() {
    let ctx = themed();
    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows).expanded();
    let shown = show(&ctx, &mut window, BIG_SCREEN, Vec::new());
    let first = place_of(&shown, labels::NEW_TONE_STYLE).unwrap();
    let second = place_of(&shown, labels::FIX_AUTOCOMPLETE).unwrap();
    assert!(
        near(first.center().y, second.center().y),
        "one row: {first:?} {second:?}"
    );
    assert!(second.left() > first.right(), "side by side: {first:?} {second:?}");
}

#[test]
fn bar_buttons_take_the_widest_label() {
    let long_label = "Reset to defaults and start over";
    let (short, long, needed) = pass(&themed(), Vec::new(), |ui| {
        (
            bar_button_width(ui, &[labels::BUTTON_CLOSE]),
            bar_button_width(ui, &[labels::BUTTON_CLOSE, long_label]),
            widgets::button_text_width(ui, long_label) + 2.0 * ui.spacing().button_padding.x,
        )
    });
    assert!(
        near(short, BUTTON_SIZE.x),
        "short labels keep the smallest width: {short}"
    );
    assert!(long > BUTTON_SIZE.x, "a long label widens the buttons: {long}");
    assert!(near(long, needed), "{long} holds the long label, which needs {needed}");
}

#[test]
fn bar_labels_are_never_cut() {
    let ctx = themed();
    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows).expanded();
    let shown = show(&ctx, &mut window, EXPANDED_SIZE, Vec::new());
    let bar = [
        labels::BUTTON_RESET,
        labels::BUTTON_COLLAPSE,
        labels::BUTTON_EXIT,
        labels::BUTTON_CLOSE,
    ];
    let needs = bar.map(|label| {
        pass(&ctx, Vec::new(), |ui| {
            widgets::button_text_width(ui, label) + 2.0 * ui.spacing().button_padding.x
        })
    });
    assert!(
        needs.iter().any(|need| *need > BUTTON_SIZE.x),
        "the test font's wide glyphs give the bar a label wider than the smallest button: {needs:?}"
    );
    let buttons = bar.map(|label| button_of(&shown, label));
    for (label, button) in bar.iter().zip(buttons) {
        assert_eq!(button.size(), buttons[0].size(), "{label} is as large as the others");
        assert!(button.width() > BUTTON_SIZE.x, "{label} grew with the widest label");
        let drawn = place_of(&shown, label).unwrap().width();
        let full = pass(&ctx, Vec::new(), |ui| widgets::button_text_width(ui, label));
        assert!(near(drawn, full), "{label} is drawn whole: {drawn} of {full}");
    }
}

#[test]
fn section_titles_are_body_size_and_muted() {
    let ctx = themed();
    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows).expanded();
    let shown = show(&ctx, &mut window, BIG_SCREEN, Vec::new());
    let style = ctx.global_style();
    let body = egui::TextStyle::Body.resolve(&style).size;
    let small = egui::TextStyle::Small.resolve(&style).size;
    for title in [
        labels::SECTION_TYPING,
        labels::SECTION_MACROS,
        labels::SECTION_SYSTEM,
        labels::SECTION_ABOUT,
    ] {
        let (_, size, color) = shown
            .formats
            .iter()
            .find(|(text, _, _)| text == title)
            .unwrap_or_else(|| panic!("{title} is drawn"));
        assert!(near(*size, body), "{title} in the body size, not {small}: {size}");
        assert_eq!(*color, style.visuals.weak_text_color(), "{title} is muted");
    }
}

/// Where the widget holding the keyboard focus was drawn.
fn focused_place(ctx: &egui::Context) -> Option<egui::Rect> {
    let id = ctx.memory(egui::Memory::focused)?;
    ctx.read_response(id).map(|response| response.rect)
}

fn is_visible(shown: &Shown, text: &str) -> bool {
    shown.visible.iter().any(|drawn| drawn == text)
}

#[test]
fn the_first_tab_lands_on_a_control() {
    for expanded in [false, true] {
        let ctx = themed();
        let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows);
        if expanded {
            window.expand();
        }
        let size = window.size();
        let shown = show(&ctx, &mut window, size, Vec::new());
        show(&ctx, &mut window, size, vec![key(egui::Key::Tab)]);
        let focused = focused_place(&ctx).expect("Tab focuses a widget");
        let first_row = place_of(&shown, labels::INPUT_METHOD).unwrap();
        assert!(
            near(focused.center().y, first_row.center().y),
            "expanded {expanded}: the first stop is Kiểu gõ's choice, not a bar button: {focused:?}"
        );

        let pressed = show(&ctx, &mut window, size, vec![key(egui::Key::Space)]);
        assert!(
            !pressed.actions.contains(&UiAction::Reset),
            "expanded {expanded}: Tab then Space never resets: {:?}",
            pressed.actions
        );
        assert!(pressed.actions.is_empty(), "{:?}", pressed.actions);
        assert!(pressed.commands.is_empty(), "{:?}", pressed.commands);
    }
}

#[test]
fn tab_walks_the_bar_in_reading_order() {
    let ctx = themed();
    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows).expanded();
    let shown = show(&ctx, &mut window, EXPANDED_SIZE, Vec::new());
    let bar = [
        labels::BUTTON_RESET,
        labels::BUTTON_COLLAPSE,
        labels::BUTTON_EXIT,
        labels::BUTTON_CLOSE,
    ]
    .map(|label| button_of(&shown, label));
    // Shift+Tab from nothing focused lands on the last stop; walk back through the bar from there. egui moves the
    // focus back one pass after the key, so a quiet pass follows each press.
    let mut stops = Vec::new();
    for _ in 0..bar.len() {
        let back = egui::Event::Key {
            key: egui::Key::Tab,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::SHIFT,
        };
        show(&ctx, &mut window, EXPANDED_SIZE, vec![back]);
        show(&ctx, &mut window, EXPANDED_SIZE, Vec::new());
        stops.push(focused_place(&ctx).expect("Shift+Tab focuses a widget"));
    }
    stops.reverse();
    assert_eq!(
        stops, bar,
        "the bar's stops run left to right, the reset first and Đóng last"
    );
}

#[test]
fn about_comes_into_view_when_revealed() {
    let ctx = themed();
    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows);
    window.reveal_about();
    assert_eq!(window.size(), EXPANDED_SIZE, "the about text is in the expanded part");
    let mut shown = show(&ctx, &mut window, EXPANDED_SIZE, Vec::new());
    for _ in 0..2 {
        shown = show(&ctx, &mut window, EXPANDED_SIZE, Vec::new());
    }
    for text in [labels::SECTION_ABOUT, labels::ABOUT_CREDITS] {
        assert!(is_visible(&shown, text), "{text} is scrolled into view");
    }
    assert!(
        !window.about_to_reveal,
        "the request is used up, so the user may scroll away"
    );
}

#[test]
fn expanding_does_not_scroll() {
    let ctx = themed();
    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows).expanded();
    let mut shown = show(&ctx, &mut window, EXPANDED_SIZE, Vec::new());
    for _ in 0..2 {
        shown = show(&ctx, &mut window, EXPANDED_SIZE, Vec::new());
    }
    assert!(
        is_visible(&shown, labels::SECTION_TYPING),
        "the sections start at the top"
    );
    assert!(!is_visible(&shown, labels::SECTION_ABOUT), "the about text waits below");

    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows);
    let compact = show(&ctx, &mut window, COMPACT_SIZE, Vec::new());
    let at = place_of(&compact, labels::BUTTON_EXPAND).unwrap().center();
    show(
        &ctx,
        &mut window,
        COMPACT_SIZE,
        vec![egui::Event::PointerMoved(at), primary(at, true)],
    );
    show(&ctx, &mut window, COMPACT_SIZE, vec![primary(at, false)]);
    assert!(window.form.is_expanded(), "the Mở rộng click expanded the window");
    let mut shown = show(&ctx, &mut window, EXPANDED_SIZE, vec![egui::Event::PointerGone]);
    for _ in 0..2 {
        shown = show(&ctx, &mut window, EXPANDED_SIZE, Vec::new());
    }
    assert!(is_visible(&shown, labels::SECTION_TYPING));
    assert!(!is_visible(&shown, labels::SECTION_ABOUT), "Mở rộng does not scroll");
}
