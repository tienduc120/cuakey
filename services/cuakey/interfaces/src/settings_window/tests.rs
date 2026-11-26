use cuakey_application::{InterfaceLanguageValue, KeyName, PreferencesView, Toggle};
use egui::epaint::RectShape;

use super::{COMPACT_SIZE, EXPANDED_SIZE, SECTIONS, SettingsWindow, bar_button_width, bar_labels, checkbox};
use crate::labels::{ENGLISH, Labels, VIETNAMESE};
use crate::theme::{self, BUTTON_SIZE};
use crate::{Platform, UiAction, widgets};

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

/// The room the theme leaves between the window's side edges and what it shows.
fn margin() -> f32 {
    f32::from(themed().global_style().spacing.window_margin.left)
}

/// The room the theme leaves between the window's bottom edge and what it shows.
fn bottom_margin() -> f32 {
    f32::from(themed().global_style().spacing.window_margin.bottom)
}

/// The default preferences with the Vietnamese interface. The tests of the bar's place at the window's own size use
/// it: the test font draws about 9 points a letter, which makes the Vietnamese reset label wider than the smallest
/// button, so the bar is at its widest.
fn vietnamese() -> PreferencesView {
    let mut view = PreferencesView::default();
    view.system.interface_language = InterfaceLanguageValue::Vietnamese;
    view
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

/// Draws one pass of `window` in a screen of `size`, fed `events`, with nothing known of its native window.
fn show(ctx: &egui::Context, window: &mut SettingsWindow, size: egui::Vec2, events: Vec<egui::Event>) -> Shown {
    show_in(ctx, window, (size, &egui::ViewportInfo::default()), events)
}

/// Draws one pass of `window` in a screen of the size given, in the native window `native` describes, fed `events`.
fn show_in(
    ctx: &egui::Context,
    window: &mut SettingsWindow,
    (size, native): (egui::Vec2, &egui::ViewportInfo),
    events: Vec<egui::Event>,
) -> Shown {
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
        events,
        ..egui::RawInput::default()
    };
    input.viewports.insert(egui::ViewportId::ROOT, native.clone());
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

/// The width of `label` on one line, as a button draws it.
fn button_text_width(ui: &egui::Ui, label: &str) -> f32 {
    widgets::text_width(ui, label, egui::TextStyle::Button)
}

/// Clicks the button labelled `label` and returns the pass that saw the release.
fn click(ctx: &egui::Context, window: &mut SettingsWindow, label: &str) -> Shown {
    click_in(ctx, window, &egui::ViewportInfo::default(), label)
}

/// Clicks the button labelled `label` in the native window `native` describes, and returns the pass that saw the
/// release.
fn click_in(ctx: &egui::Context, window: &mut SettingsWindow, native: &egui::ViewportInfo, label: &str) -> Shown {
    let screen = (BIG_SCREEN, native);
    let before = show_in(ctx, window, screen, Vec::new());
    let at = place_of(&before, label).unwrap().center();
    show_in(
        ctx,
        window,
        screen,
        vec![egui::Event::PointerMoved(at), primary(at, true)],
    );
    show_in(ctx, window, screen, vec![primary(at, false)])
}

/// The compact window as it first opens on a 1920x1080 screen at 150%: centred, with the title bar and the resize
/// border Windows draws around it. Grown in place, it would end under the taskbar.
fn compact_on_a_short_screen() -> egui::ViewportInfo {
    let corner = egui::pos2(360.0, 190.0);
    egui::ViewportInfo {
        monitor_size: Some(egui::vec2(1280.0, 720.0)),
        outer_rect: Some(egui::Rect::from_min_size(corner, COMPACT_SIZE + egui::vec2(16.0, 39.0))),
        inner_rect: Some(egui::Rect::from_min_size(corner + egui::vec2(8.0, 31.0), COMPACT_SIZE)),
        ..egui::ViewportInfo::default()
    }
}

/// Where `commands` move the native window, if anywhere.
fn moved_to(commands: &[egui::ViewportCommand]) -> Option<egui::Pos2> {
    commands.iter().find_map(|command| {
        if let egui::ViewportCommand::OuterPosition(corner) = command {
            Some(*corner)
        } else {
            None
        }
    })
}

/// Whether `commands` resize the native window.
fn resizes(commands: &[egui::ViewportCommand]) -> bool {
    commands
        .iter()
        .any(|command| matches!(command, egui::ViewportCommand::InnerSize(_)))
}

#[test]
fn a_checkbox_reports_the_flipped_value() {
    for (current, flipped) in [(Toggle::Off, true), (Toggle::On, false)] {
        let ctx = egui::Context::default();
        pass(&ctx, vec![key(egui::Key::Tab)], |ui| {
            checkbox(ui, ENGLISH.beep, current)
        });
        let changed = pass(&ctx, vec![key(egui::Key::Space)], |ui| {
            checkbox(ui, ENGLISH.beep, current)
        });
        assert_eq!(changed, Some(flipped), "Space on a box showing {current:?}");
    }
}

/// The inner size `commands` ask of the native window, if any.
fn asked_size(commands: &[egui::ViewportCommand]) -> Option<egui::Vec2> {
    commands.iter().find_map(|command| {
        if let egui::ViewportCommand::InnerSize(size) = command {
            Some(*size)
        } else {
            None
        }
    })
}

/// The compact window's size once it measured what it shows: the first frame fits it to the test font.
fn fitted(ctx: &egui::Context, window: &mut SettingsWindow) -> egui::Vec2 {
    show(ctx, window, BIG_SCREEN, Vec::new());
    window.size()
}

#[test]
fn error_shows_in_compact_mode() {
    let ctx = themed();
    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows);
    window.show_error(Some(MESSAGE));
    assert!(!window.form.is_expanded());
    let size = fitted(&ctx, &mut window);
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, size);

    let shown = show(&ctx, &mut window, size, Vec::new());
    let place = place_of(&shown, MESSAGE).expect("the message is drawn");
    assert!(
        screen.contains_rect(place),
        "the message fits in the compact window: {place:?}"
    );
    let bar = button_of(&shown, ENGLISH.button_close);
    assert!(place.bottom() < bar.top(), "the message sits above the bar");

    window.form.toggle_expanded();
    show(&ctx, &mut window, EXPANDED_SIZE, Vec::new());
    window.form.toggle_expanded();
    assert_eq!(window.size(), size, "the compact size keeps the message's line");
    let shown = show(&ctx, &mut window, size, Vec::new());
    let place = place_of(&shown, MESSAGE).expect("the message stays after collapsing");
    assert!(screen.contains_rect(place), "{place:?}");
}

#[test]
fn the_rows_keep_their_place_from_frame_to_frame() {
    let mut view = PreferencesView::default();
    view.switching.hotkey.key = Some(KeyName::F5);
    let ctx = themed();
    let mut window = SettingsWindow::new(view, Platform::Windows);
    let first = show(&ctx, &mut window, BIG_SCREEN, Vec::new());
    for frame in 1..4 {
        let next = show(&ctx, &mut window, BIG_SCREEN, Vec::new());
        assert_eq!(
            next.texts, first.texts,
            "frame {frame} draws every text where the first did"
        );
    }
}

#[test]
fn compact_has_no_empty_space_above_the_bar() {
    let ctx = themed();
    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows);
    let size = fitted(&ctx, &mut window);
    assert!(near(size.x, COMPACT_SIZE.x), "only the height follows: {size:?}");

    let shown = show(&ctx, &mut window, size, Vec::new());
    assert!(
        !resizes(&shown.commands),
        "a fitted window is not resized again: {:?}",
        shown.commands
    );
    let bar = [ENGLISH.button_expand, ENGLISH.button_exit, ENGLISH.button_close];
    let last_row = shown
        .texts
        .iter()
        .filter(|(text, _)| !bar.contains(&text.as_str()))
        .map(|(_, place)| place.bottom())
        .fold(f32::MIN, f32::max);
    let gap = button_of(&shown, ENGLISH.button_close).top() - last_row;
    let allowed = ctx.global_style().spacing.item_spacing.y + bottom_margin();
    assert!(
        gap > 0.0 && gap <= allowed,
        "{gap} between the last row and the bar, at most {allowed}"
    );

    window.show_error(Some(MESSAGE));
    let grown = asked_size(&show(&ctx, &mut window, size, Vec::new()).commands).expect("the window grows");
    assert!(grown.y > size.y, "{grown:?} makes room for the message: {size:?}");
    assert!(
        is_visible(&show(&ctx, &mut window, grown, Vec::new()), MESSAGE),
        "the message shows whole in the grown window"
    );

    window.show_error(None);
    let shrunk = asked_size(&show(&ctx, &mut window, grown, Vec::new()).commands);
    assert_eq!(shrunk, Some(size), "the window shrinks back when the message clears");
}

#[test]
fn the_expand_button_resizes_the_window_and_asks_nothing() {
    let ctx = egui::Context::default();
    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows);

    let expanded = click(&ctx, &mut window, ENGLISH.button_expand);
    assert!(window.form.is_expanded());
    assert!(expanded.actions.is_empty(), "{:?}", expanded.actions);
    assert!(
        expanded
            .commands
            .contains(&egui::ViewportCommand::InnerSize(EXPANDED_SIZE)),
        "{:?}",
        expanded.commands
    );

    let collapsed = click(&ctx, &mut window, ENGLISH.button_collapse);
    assert!(!window.form.is_expanded());
    assert!(collapsed.actions.is_empty(), "{:?}", collapsed.actions);
    let compact = window.size();
    assert!(compact.y < EXPANDED_SIZE.y, "back to the compact size: {compact:?}");
    assert!(
        collapsed.commands.contains(&egui::ViewportCommand::InnerSize(compact)),
        "{:?}",
        collapsed.commands
    );
}

#[test]
fn expanding_near_the_taskbar_moves_the_window_up() {
    let ctx = egui::Context::default();
    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows);
    let native = compact_on_a_short_screen();

    let expanded = click_in(&ctx, &mut window, &native, ENGLISH.button_expand);

    let corner = moved_to(&expanded.commands).expect("the window moves");
    let opened_at = native.outer_rect.unwrap().min;
    assert!(
        corner.y < opened_at.y && corner.y >= 0.0,
        "up, and still on screen: {corner:?}"
    );
    assert!(
        expanded
            .commands
            .ends_with(&[egui::ViewportCommand::InnerSize(EXPANDED_SIZE)]),
        "moved first, then grown: {:?}",
        expanded.commands
    );
}

#[test]
fn a_window_revealing_about_takes_its_size_on_the_next_frame() {
    let ctx = egui::Context::default();
    let native = compact_on_a_short_screen();
    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows);
    let places = |shown: &Shown| moved_to(&shown.commands).is_some() || resizes(&shown.commands);
    // The first frame fits the compact window to what it shows in the test font.
    show_in(&ctx, &mut window, (BIG_SCREEN, &native), Vec::new());
    let compact = show_in(&ctx, &mut window, (BIG_SCREEN, &native), Vec::new());
    assert!(!places(&compact), "{:?}", compact.commands);

    window.reveal_about();
    let grown = show_in(&ctx, &mut window, (BIG_SCREEN, &native), Vec::new());
    assert!(moved_to(&grown.commands).is_some(), "{:?}", grown.commands);
    assert!(resizes(&grown.commands), "{:?}", grown.commands);

    let next = show_in(&ctx, &mut window, (BIG_SCREEN, &native), Vec::new());
    assert!(!places(&next), "once: {:?}", next.commands);
    window.reveal_about();
    let again = show_in(&ctx, &mut window, (BIG_SCREEN, &native), Vec::new());
    assert!(
        !places(&again),
        "an expanded window keeps the size the user gave it: {:?}",
        again.commands
    );
}

#[test]
fn the_reset_is_offered_only_when_expanded() {
    let ctx = egui::Context::default();
    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows);

    let compact = show(&ctx, &mut window, BIG_SCREEN, Vec::new());
    assert_eq!(place_of(&compact, ENGLISH.button_reset), None);
    assert!(
        place_of(&compact, ENGLISH.section_typing).is_none(),
        "the sections wait for the expand button"
    );

    window.form.toggle_expanded();
    let expanded = show(&ctx, &mut window, BIG_SCREEN, Vec::new());
    assert!(place_of(&expanded, ENGLISH.button_reset).is_some());
}

#[test]
fn clicking_the_reset_asks_for_it() {
    let ctx = egui::Context::default();
    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows);
    window.form.toggle_expanded();
    let shown = click(&ctx, &mut window, ENGLISH.button_reset);
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
        place_of(&shown, ENGLISH.section_about).is_some(),
        "the about section is drawn"
    );
    assert!(place_of(&shown, ENGLISH.button_collapse).is_some());
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
    let mut window = SettingsWindow::new(vietnamese(), Platform::Windows);
    let right = [
        VIETNAMESE.button_expand,
        VIETNAMESE.button_exit,
        VIETNAMESE.button_close,
    ];
    let size = egui::vec2(pass(&ctx, Vec::new(), |ui| bar_button_width(ui)), BUTTON_SIZE.y);

    let compact_size = fitted(&ctx, &mut window);
    let compact = show(&ctx, &mut window, compact_size, Vec::new());
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
            near(button.bottom(), compact_size.y - bottom_margin()),
            "{label} sits on the bottom edge: {button:?}"
        );
    }
    let [expand, exit, close] = buttons;
    assert!(expand.right() < exit.left() && exit.right() < close.left(), "in order");
    assert!(
        near(close.right(), compact_size.x - margin()),
        "right-aligned: {close:?}"
    );

    window.form.toggle_expanded();
    let expanded = show(&ctx, &mut window, EXPANDED_SIZE, Vec::new());
    let reset = button_of(&expanded, VIETNAMESE.button_reset);
    let close = button_of(&expanded, VIETNAMESE.button_close);
    assert_eq!(reset.size(), size);
    assert!(near(reset.left(), margin()), "the reset at the left end: {reset:?}");
    assert!(near(reset.top(), close.top()), "on the same bar");
    assert!(
        near(close.bottom(), EXPANDED_SIZE.y - bottom_margin()),
        "the bar stays on the bottom edge, past the scrolling sections: {close:?}"
    );
}

#[test]
fn control_rows_have_no_frame() {
    let ctx = themed();
    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows);
    let shown = show(&ctx, &mut window, COMPACT_SIZE, Vec::new());
    let rows = place_of(&shown, ENGLISH.input_method)
        .unwrap()
        .union(place_of(&shown, ENGLISH.interface_language).unwrap());
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

    for text in [ENGLISH.key_ctrl, "F5"] {
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
    let first = place_of(&shown, ENGLISH.new_tone_style).unwrap();
    let second = place_of(&shown, ENGLISH.fix_autocomplete).unwrap();
    assert!(
        near(first.center().y, second.center().y),
        "one row: {first:?} {second:?}"
    );
    assert!(second.left() > first.right(), "side by side: {first:?} {second:?}");
}

#[test]
fn bar_buttons_take_the_widest_label_of_either_language() {
    let (width, needs) = pass(&themed(), Vec::new(), |ui| {
        let padding = 2.0 * ui.spacing().button_padding.x;
        let needs: Vec<(&str, f32)> = [&VIETNAMESE, &ENGLISH]
            .into_iter()
            .flat_map(bar_labels)
            .map(|label| (label, button_text_width(ui, label) + padding))
            .collect();
        (bar_button_width(ui), needs)
    });
    for (label, need) in &needs {
        assert!(width + 0.01 >= *need, "{width} holds {label}, which needs {need}");
    }
    let widest = needs.iter().map(|(_, need)| *need).fold(BUTTON_SIZE.x, f32::max);
    assert!(
        near(width, widest),
        "{width} is the smallest width or the widest label's, {widest}"
    );
}

#[test]
fn bar_labels_are_never_cut() {
    for (view, text) in [(vietnamese(), &VIETNAMESE), (PreferencesView::default(), &ENGLISH)] {
        let ctx = themed();
        let mut window = SettingsWindow::new(view, Platform::Windows).expanded();
        let shown = show(&ctx, &mut window, EXPANDED_SIZE, Vec::new());
        let bar = [
            text.button_reset,
            text.button_collapse,
            text.button_exit,
            text.button_close,
        ];
        let buttons = bar.map(|label| button_of(&shown, label));
        for (label, button) in bar.iter().zip(buttons) {
            assert_eq!(button.size(), buttons[0].size(), "{label} is as large as the others");
            let need = pass(&ctx, Vec::new(), |ui| {
                button_text_width(ui, label) + 2.0 * ui.spacing().button_padding.x
            });
            assert!(button.width() + 0.5 >= need, "{label} needs {need}: {button:?}");
            let drawn = place_of(&shown, label).unwrap().width();
            let full = pass(&ctx, Vec::new(), |ui| button_text_width(ui, label));
            assert!(near(drawn, full), "{label} is drawn whole: {drawn} of {full}");
        }
    }
}

#[test]
fn the_english_bar_fits_the_expanded_window() {
    let ctx = themed();
    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows).expanded();
    let shown = show(&ctx, &mut window, EXPANDED_SIZE, Vec::new());
    let bar = [
        ENGLISH.button_reset,
        ENGLISH.button_collapse,
        ENGLISH.button_exit,
        ENGLISH.button_close,
    ];

    for label in bar {
        assert!(is_visible(&shown, label), "{label} is drawn whole, inside the window");
        let text = place_of(&shown, label).unwrap();
        assert!(button_of(&shown, label).contains_rect(text), "{label} fits its button");
    }
    let [reset, collapse, _, close] = bar.map(|label| button_of(&shown, label));
    assert!(
        close.right() <= EXPANDED_SIZE.x - margin() + 0.5,
        "the last button ends inside the window: {close:?}"
    );
    // The bar takes the width of the wider Vietnamese labels, which the test font's 9 points a letter make wider than
    // the real fonts do; this gap only has to stand out from the room between the other buttons.
    let gap = collapse.left() - reset.right();
    let spacing = ctx.global_style().spacing.item_spacing.x;
    assert!(
        gap >= 2.0 * spacing,
        "the reset stays apart from the buttons used every day: {gap}"
    );
}

#[test]
fn a_window_too_narrow_for_the_bar_keeps_every_button_inside() {
    let ctx = themed();
    let narrow = egui::vec2(400.0, EXPANDED_SIZE.y);
    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows).expanded();
    let shown = show(&ctx, &mut window, narrow, Vec::new());

    let bar = [
        ENGLISH.button_reset,
        ENGLISH.button_collapse,
        ENGLISH.button_exit,
        ENGLISH.button_close,
    ]
    .map(|label| button_of(&shown, label));
    let [reset, collapse, _, close] = bar;
    assert!(
        near(close.right(), narrow.x - margin()),
        "the last button ends at the window's edge: {close:?}"
    );
    let spacing = ctx.global_style().spacing.item_spacing.x;
    assert!(
        near(collapse.left() - reset.right(), spacing),
        "the room before the right-hand buttons went first: {reset:?} {collapse:?}"
    );
    for button in bar {
        assert_eq!(button.size(), reset.size(), "the buttons shrank alike: {bar:?}");
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
        ENGLISH.section_typing,
        ENGLISH.section_macros,
        ENGLISH.section_system,
        ENGLISH.section_about,
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
        let first_row = place_of(&shown, ENGLISH.input_method).unwrap();
        assert!(
            near(focused.center().y, first_row.center().y),
            "expanded {expanded}: the first stop is the input method choice, not a bar button: {focused:?}"
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
    let mut window = SettingsWindow::new(vietnamese(), Platform::Windows).expanded();
    let shown = show(&ctx, &mut window, EXPANDED_SIZE, Vec::new());
    let bar = [
        VIETNAMESE.button_reset,
        VIETNAMESE.button_collapse,
        VIETNAMESE.button_exit,
        VIETNAMESE.button_close,
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
        "the bar's stops run left to right, the reset first and the close button last"
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
    for text in [ENGLISH.section_about, ENGLISH.about_credits] {
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
        is_visible(&shown, ENGLISH.section_typing),
        "the sections start at the top"
    );
    assert!(!is_visible(&shown, ENGLISH.section_about), "the about text waits below");

    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows);
    let compact = show(&ctx, &mut window, COMPACT_SIZE, Vec::new());
    let at = place_of(&compact, ENGLISH.button_expand).unwrap().center();
    show(
        &ctx,
        &mut window,
        COMPACT_SIZE,
        vec![egui::Event::PointerMoved(at), primary(at, true)],
    );
    show(&ctx, &mut window, COMPACT_SIZE, vec![primary(at, false)]);
    assert!(window.form.is_expanded(), "the expand click expanded the window");
    let mut shown = show(&ctx, &mut window, EXPANDED_SIZE, vec![egui::Event::PointerGone]);
    for _ in 0..2 {
        shown = show(&ctx, &mut window, EXPANDED_SIZE, Vec::new());
    }
    assert!(is_visible(&shown, ENGLISH.section_typing));
    assert!(!is_visible(&shown, ENGLISH.section_about), "expanding does not scroll");
}

#[test]
fn english_interface_draws_english() {
    let mut window = SettingsWindow::new(PreferencesView::default(), Platform::Windows);
    assert_eq!(
        window.interface_language(),
        InterfaceLanguageValue::English,
        "English by default"
    );

    let shown = show(&themed(), &mut window, BIG_SCREEN, Vec::new());

    for text in ["Exit", "Interface:"] {
        assert!(place_of(&shown, text).is_some(), "{text} is drawn");
    }
    assert_eq!(place_of(&shown, "Thoát"), None, "no Vietnamese label");
}

#[test]
fn choosing_english_redraws_at_once() {
    let ctx = themed();
    let mut window = SettingsWindow::new(vietnamese(), Platform::Windows);
    let before = show(&ctx, &mut window, BIG_SCREEN, Vec::new());
    assert!(place_of(&before, "Thoát").is_some(), "the window opens in Vietnamese");

    let chosen = click(&ctx, &mut window, "English");
    let next = show(&ctx, &mut window, BIG_SCREEN, Vec::new());

    assert_eq!(
        window.form.view().system.interface_language,
        InterfaceLanguageValue::English
    );
    assert!(
        matches!(
            chosen.actions.as_slice(),
            [UiAction::Save(view)] if view.system.interface_language == InterfaceLanguageValue::English
        ),
        "the choice is saved: {:?}",
        chosen.actions
    );
    assert!(place_of(&next, "Exit").is_some(), "the next frame speaks English");
    assert_eq!(place_of(&next, "Thoát"), None);
}

#[test]
fn interface_radios_name_each_language_in_itself() {
    for (language, row_label) in [
        (InterfaceLanguageValue::Vietnamese, "Giao diện:"),
        (InterfaceLanguageValue::English, "Interface:"),
    ] {
        let mut view = PreferencesView::default();
        view.system.interface_language = language;
        let mut window = SettingsWindow::new(view, Platform::Windows);
        let shown = show(&themed(), &mut window, BIG_SCREEN, Vec::new());

        let row = place_of(&shown, row_label).unwrap_or_else(|| panic!("{row_label} is drawn"));
        for choice in ["Tiếng Việt", "English"] {
            assert!(
                places_of(&shown, choice)
                    .iter()
                    .any(|place| near(place.center().y, row.center().y) && place.left() > row.right()),
                "{row_label} offers {choice}"
            );
        }
    }
}

/// `view` speaking `language`.
fn speaking(mut view: PreferencesView, language: InterfaceLanguageValue) -> PreferencesView {
    view.system.interface_language = language;
    view
}

/// What the last of a few passes of `window` drew on a big screen, once its grids took their widths.
fn settled(ctx: &egui::Context, window: &mut SettingsWindow) -> Shown {
    let mut shown = show(ctx, window, BIG_SCREEN, Vec::new());
    for _ in 0..2 {
        shown = show(ctx, window, BIG_SCREEN, Vec::new());
    }
    shown
}

/// Where each control used every day starts, in a window drawn in the language of `text`, named for the message of a
/// failed comparison. A box or radio sits a fixed step before its label and a key is centred in its field, so these
/// labels move with their controls.
fn control_starts(shown: &Shown, text: &Labels) -> Vec<(&'static str, f32)> {
    let row = |label: &str| place_of(shown, label).unwrap_or_else(|| panic!("{label} is drawn"));
    let first_choice = |label: &str, choice: &str| {
        let row = row(label);
        places_of(shown, choice)
            .into_iter()
            .find(|place| near(place.center().y, row.center().y))
            .unwrap_or_else(|| panic!("{choice} in the {label} row"))
            .left()
    };
    let ctrl = places_of(shown, text.key_ctrl);
    let key = places_of(shown, "F5");
    vec![
        ("input method", button_of(shown, text.input_method_telex).left()),
        ("code table", button_of(shown, text.code_table_unicode).left()),
        ("switch hotkey's first box", ctrl[0].left()),
        ("switch hotkey's key", key[0].left()),
        ("beep", row(text.beep).left()),
        ("settings hotkey's first box", ctrl[1].left()),
        ("settings hotkey's key", key[1].left()),
        (
            "typing's first radio",
            first_choice(text.language, text.language_vietnamese),
        ),
        (
            "interface's first radio",
            first_choice(text.interface_language, text.interface_vietnamese),
        ),
    ]
}

/// The bar's buttons in a window drawn in the language of `text`, left to right.
fn bar_of(shown: &Shown, text: &Labels, expanded: bool) -> Vec<egui::Rect> {
    let labels: &[&str] = if expanded {
        &[
            text.button_reset,
            text.button_collapse,
            text.button_exit,
            text.button_close,
        ]
    } else {
        &[text.button_expand, text.button_exit, text.button_close]
    };
    labels.iter().map(|label| button_of(shown, label)).collect()
}

/// Where each option of the expanded sections starts, in the order the sections list them.
fn option_starts(shown: &Shown, text: &Labels) -> Vec<(&'static str, f32)> {
    SECTIONS
        .iter()
        .flat_map(|section| section.options)
        .map(|option| {
            let label = (option.label)(&ENGLISH);
            let place = place_of(shown, (option.label)(text)).unwrap_or_else(|| panic!("{label} is drawn"));
            (label, place.left())
        })
        .collect()
}

fn assert_same_starts(vietnamese: &[(&str, f32)], english: &[(&str, f32)]) {
    assert_eq!(vietnamese.len(), english.len());
    for ((name, vi), (_, en)) in vietnamese.iter().zip(english) {
        assert!(near(*vi, *en), "{name} starts at {vi} in Vietnamese, {en} in English");
    }
}

fn assert_same_buttons(vietnamese: &[egui::Rect], english: &[egui::Rect]) {
    assert_eq!(vietnamese.len(), english.len());
    for (vi, en) in vietnamese.iter().zip(english) {
        assert!(
            near(vi.left(), en.left()) && near(vi.width(), en.width()),
            "a bar button at {vi:?} in Vietnamese is at {en:?} in English"
        );
    }
}

#[test]
fn switching_language_moves_no_control() {
    let mut view = vietnamese();
    view.switching.hotkey.key = Some(KeyName::F5);
    view.behaviours.settings_hotkey.key = Some(KeyName::F5);
    let english = speaking(view.clone(), InterfaceLanguageValue::English);
    let ctx = themed();
    let mut window = SettingsWindow::new(view.clone(), Platform::Windows);

    let compact_vi = settled(&ctx, &mut window);
    window.set_preferences(english);
    let compact_en = settled(&ctx, &mut window);
    assert_same_starts(
        &control_starts(&compact_vi, &VIETNAMESE),
        &control_starts(&compact_en, &ENGLISH),
    );
    assert_same_buttons(
        &bar_of(&compact_vi, &VIETNAMESE, false),
        &bar_of(&compact_en, &ENGLISH, false),
    );

    window.expand();
    let expanded_en = settled(&ctx, &mut window);
    window.set_preferences(view);
    let expanded_vi = settled(&ctx, &mut window);
    assert_same_starts(
        &option_starts(&expanded_vi, &VIETNAMESE),
        &option_starts(&expanded_en, &ENGLISH),
    );
    assert_same_buttons(
        &bar_of(&expanded_vi, &VIETNAMESE, true),
        &bar_of(&expanded_en, &ENGLISH, true),
    );
}

#[test]
fn the_bar_measures_every_label_it_can_show() {
    for text in [&VIETNAMESE, &ENGLISH] {
        let measured = bar_labels(text);
        for label in [
            text.button_expand,
            text.button_collapse,
            text.button_exit,
            text.button_close,
            text.button_reset,
        ] {
            assert!(measured.contains(&label), "{label} in {measured:?}");
        }
    }
}
