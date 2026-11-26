use std::path::PathBuf;
use std::sync::Arc;

use cuakey_application::{
    AppMemoryView, CodeTableValue, InputMethodValue, InterfaceLanguageValue, LanguageValue, PlacedWindow,
    PreferencesView, Toggle, WindowPlace, WindowPlaces,
};
use cuakey_interfaces::{COMPACT_SIZE, EXPANDED_SIZE, WindowKind, labels};
use eframe::egui;
use platform_windows::ScreenRect;

use super::window::Window;
use super::{Edit, edit_of, keep_places, largest_png, move_into_root, native_options, rebase, shown_kind};
use crate::stores::Stores;

const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";

/// What the store holds once the background remembered an app and the user listed one as always on.
fn stored() -> PreferencesView {
    PreferencesView {
        language: LanguageValue::English,
        always_on: vec!["game.exe".to_owned()],
        app_memory: vec![AppMemoryView {
            app: "notepad.exe".to_owned(),
            language: LanguageValue::English,
            code_table: CodeTableValue::Unicode,
        }],
        ..PreferencesView::default()
    }
}

#[test]
fn a_settings_save_keeps_the_stored_apps() {
    // The window opened before the background remembered notepad and before game.exe was listed.
    let mut edited = PreferencesView {
        language: LanguageValue::Vietnamese,
        ..PreferencesView::default()
    };
    edited.typing.spell_check = Toggle::Off;
    edited.convert.to = CodeTableValue::Tcvn3;

    let saved = rebase(stored(), edited, Edit::Settings);

    assert_eq!(saved.typing.spell_check, Toggle::Off);
    assert_eq!(saved.convert.to, CodeTableValue::Tcvn3);
    assert_eq!(
        saved.language,
        LanguageValue::Vietnamese,
        "the language comes from the window"
    );
    assert_eq!(saved.always_on, stored().always_on);
    assert_eq!(saved.app_memory, stored().app_memory);
}

#[test]
fn a_settings_save_carries_the_interface_language() {
    let mut edited = PreferencesView::default();
    edited.system.interface_language = InterfaceLanguageValue::Vietnamese;
    edited.system.desktop_shortcut = Toggle::On;

    let saved = rebase(stored(), edited, Edit::Settings);

    assert_eq!(saved.system.interface_language, InterfaceLanguageValue::Vietnamese);
    assert_eq!(saved.system.desktop_shortcut, Toggle::On, "with the rest of the group");
}

#[test]
fn an_always_on_save_takes_only_the_list() {
    // The window opened before spell checking was turned off elsewhere.
    let mut edited = PreferencesView {
        always_on: vec!["chrome.exe".to_owned()],
        ..PreferencesView::default()
    };
    edited.typing.spell_check = Toggle::On;
    let mut stored_now = stored();
    stored_now.typing.spell_check = Toggle::Off;

    let saved = rebase(stored_now.clone(), edited, Edit::AlwaysOn);

    assert_eq!(
        saved,
        PreferencesView {
            always_on: vec!["chrome.exe".to_owned()],
            ..stored_now
        }
    );
}

#[test]
fn only_the_always_on_window_saves_the_list() {
    assert_eq!(edit_of(WindowKind::AlwaysOn), Edit::AlwaysOn);
    for kind in [WindowKind::Settings, WindowKind::Macros, WindowKind::Convert] {
        assert_eq!(edit_of(kind), Edit::Settings, "{kind:?}");
    }
}

#[test]
fn the_about_window_is_the_settings_window() {
    assert_eq!(shown_kind(WindowKind::About), WindowKind::Settings);
    for kind in [
        WindowKind::Settings,
        WindowKind::Macros,
        WindowKind::Convert,
        WindowKind::AlwaysOn,
    ] {
        assert_eq!(shown_kind(kind), kind);
    }
}

#[test]
fn the_window_icon_is_the_largest_picture_in_the_icon_file() {
    let png = largest_png(include_bytes!("../../../../../icon/icon_V.ico")).unwrap();

    assert!(png.starts_with(PNG_SIGNATURE));
    // The width is the first field of the header chunk, after the signature, the chunk length and its name.
    assert_eq!(png.get(16..20), Some(256_u32.to_be_bytes().as_slice()));
}

#[test]
fn an_icon_file_without_a_png_picture_gives_none() {
    let mut ico = vec![0, 0, 1, 0, 1, 0];
    // One 16 by 16 picture of 32 bits, 4 bytes long, right after the directory.
    ico.extend([16, 16, 0, 0, 1, 0, 32, 0]);
    ico.extend(4_u32.to_le_bytes());
    ico.extend(22_u32.to_le_bytes());
    // The start of a bitmap header, which is not a PNG.
    ico.extend([0x28, 0, 0, 0]);

    assert_eq!(largest_png(&ico), None);
    assert_eq!(largest_png(&[]), None, "an empty file");
    assert_eq!(largest_png(&ico[..10]), None, "a directory cut short");
}

/// A folder of its own in the temp folder for one test, removed when the test ends, whether it passed or not.
struct TempFolder(PathBuf);

impl TempFolder {
    fn new(test: &str) -> Self {
        let path = std::env::temp_dir().join(format!("cuakey-ui-process-{}-{test}", std::process::id()));
        // What an earlier run left would be read as stored.
        drop(std::fs::remove_dir_all(&path));
        Self(path)
    }
}

impl Drop for TempFolder {
    fn drop(&mut self) {
        // A folder left behind is not a failed test.
        drop(std::fs::remove_dir_all(&self.0));
    }
}

/// Stores on a folder that does not exist: every window shows the defaults, and opening one writes nothing.
fn empty_stores(test: &str) -> Stores {
    let dir = std::env::temp_dir().join(format!("cuakey-ui-process-{}-{test}", std::process::id()));
    Stores::in_folder(dir).unwrap()
}

/// What `send` tells the root viewport in one pass.
fn root_commands(send: impl FnOnce(&egui::Context)) -> Vec<egui::ViewportCommand> {
    root_commands_in(egui::ViewportInfo::default(), send)
}

/// What `send` tells the root viewport in one pass, its native window being the one `native` describes.
fn root_commands_in(native: egui::ViewportInfo, send: impl FnOnce(&egui::Context)) -> Vec<egui::ViewportCommand> {
    let ctx = egui::Context::default();
    let mut send = Some(send);
    let mut input = egui::RawInput::default();
    input.viewports.insert(egui::ViewportId::ROOT, native);
    let mut output = ctx.run_ui(input, |ui| {
        if let Some(send) = send.take() {
            send(ui.ctx());
        }
    });
    // No renderer takes the font texture here.
    output.textures_delta.clear();
    output
        .viewport_output
        .get(&egui::ViewportId::ROOT)
        .map(|viewport| viewport.commands.clone())
        .unwrap_or_default()
}

/// What moving `window` into the root viewport tells that viewport.
fn root_commands_moving(window: &Window) -> Vec<egui::ViewportCommand> {
    root_commands(|ctx| move_into_root(ctx, window))
}

/// Whether `commands` resize the native window.
fn resizes(commands: &[egui::ViewportCommand]) -> bool {
    commands
        .iter()
        .any(|command| matches!(command, egui::ViewportCommand::InnerSize(_)))
}

#[test]
fn about_expands_an_open_settings_window() {
    let stores = empty_stores("expand");
    let mut window = Window::open(WindowKind::Settings, &stores);

    window.reveal_about();
    assert_eq!(window.size(), EXPANDED_SIZE, "the about text is in the expanded part");
    let commands = commands_drawing(&mut window);
    assert!(
        commands.contains(&egui::ViewportCommand::InnerSize(EXPANDED_SIZE)),
        "the native window grows with it as it is drawn: {commands:?}"
    );

    window.reveal_about();
    assert_eq!(window.size(), EXPANDED_SIZE);
    let again = commands_drawing(&mut window);
    assert!(
        !resizes(&again),
        "an expanded window keeps the size the user gave it: {again:?}"
    );
}

#[test]
fn a_window_of_its_own_is_resized_only_by_what_it_draws() {
    let stores = empty_stores("own-viewport");
    let icon = Arc::new(egui::IconData::default());
    let mut window = Window::open(WindowKind::Settings, &stores);
    assert_eq!(
        window.viewport(&icon).inner_size,
        Some(COMPACT_SIZE),
        "it opens compact"
    );

    window.reveal_about();

    assert_eq!(
        window.viewport(&icon).inner_size,
        Some(COMPACT_SIZE),
        "a new size here would resize the native window again, past where the window keeps itself on screen"
    );
}

#[test]
fn every_window_is_fixed_size() {
    let stores = empty_stores("fixed-size");
    let icon = Arc::new(egui::IconData::default());
    for kind in [
        WindowKind::Settings,
        WindowKind::About,
        WindowKind::Macros,
        WindowKind::Convert,
        WindowKind::AlwaysOn,
    ] {
        let viewport = Window::open(kind, &stores).viewport(&icon);
        assert_eq!(viewport.resizable, Some(false), "the user cannot resize {kind:?}");
        assert_eq!(viewport.maximize_button, Some(false), "{kind:?} has no maximize button");
    }
}

#[test]
fn the_about_menu_opens_the_settings_window_expanded() {
    let stores = empty_stores("about");

    let about = Window::open(WindowKind::About, &stores);
    assert_eq!(about.kind(), WindowKind::Settings);
    assert_eq!(about.size(), EXPANDED_SIZE, "the about text is in the expanded part");

    let settings = Window::open(WindowKind::Settings, &stores);
    assert_eq!(settings.size(), COMPACT_SIZE, "every other way in opens compact");
}

#[test]
fn a_settings_window_moved_into_the_root_keeps_its_size() {
    let stores = empty_stores("root");

    let expanded = Window::open(WindowKind::About, &stores);
    let commands = root_commands_moving(&expanded);
    assert!(
        commands.contains(&egui::ViewportCommand::InnerSize(EXPANDED_SIZE)),
        "{commands:?}"
    );

    let compact = Window::open(WindowKind::Settings, &stores);
    let commands = root_commands_moving(&compact);
    assert!(
        commands.contains(&egui::ViewportCommand::InnerSize(COMPACT_SIZE)),
        "{commands:?}"
    );
}

/// Whether `shape` paints `text` whole inside `clip`, the area it is clipped to.
fn paints_whole(shape: &egui::Shape, clip: egui::Rect, text: &str) -> bool {
    if let egui::Shape::Vec(inner) = shape {
        inner.iter().any(|shape| paints_whole(shape, clip, text))
    } else if let egui::Shape::Text(shown) = shape {
        let place = egui::Rect::from_min_size(shown.pos, shown.galley.size());
        shown.galley.text() == text && clip.contains_rect(place)
    } else {
        false
    }
}

/// Whether a few passes of `window` at its own size end with `text` in view, not scrolled out of it.
fn ends_in_view(window: &mut Window, text: &str) -> bool {
    let ctx = egui::Context::default();
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, window.size());
    let mut in_view = false;
    for _ in 0..3 {
        let input = egui::RawInput {
            screen_rect: Some(screen),
            ..egui::RawInput::default()
        };
        let mut output = ctx.run_ui(input, |ui| drop(window.show(ui)));
        output.textures_delta.clear();
        in_view = output
            .shapes
            .iter()
            .any(|clipped| paints_whole(&clipped.shape, clipped.clip_rect.intersect(screen), text));
    }
    in_view
}

#[test]
fn the_about_menu_brings_the_about_text_into_view() {
    let stores = empty_stores("about-text");

    let mut opened = Window::open(WindowKind::About, &stores);
    assert!(
        ends_in_view(&mut opened, labels::ENGLISH.section_about),
        "a window opened for About shows the about text"
    );

    let mut open_compact = Window::open(WindowKind::Settings, &stores);
    open_compact.reveal_about();
    assert!(
        ends_in_view(&mut open_compact, labels::ENGLISH.section_about),
        "a window About expanded shows the about text"
    );
}

/// What one pass drawing `window` tells the viewport it is drawn in, the root one here.
fn commands_drawing(window: &mut Window) -> Vec<egui::ViewportCommand> {
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| drop(window.show(ui)));
    // No renderer takes the font texture here.
    output.textures_delta.clear();
    output
        .viewport_output
        .get(&egui::ViewportId::ROOT)
        .map(|viewport| viewport.commands.clone())
        .unwrap_or_default()
}

/// The titles `commands` give the native window, in order.
fn titles(commands: &[egui::ViewportCommand]) -> Vec<String> {
    commands
        .iter()
        .filter_map(|command| {
            if let egui::ViewportCommand::Title(title) = command {
                Some(title.clone())
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn a_window_takes_the_title_of_its_interface_language() {
    let stores = empty_stores("title");
    let mut vietnamese = PreferencesView::default();
    vietnamese.system.interface_language = InterfaceLanguageValue::Vietnamese;

    for (kind, english, translated) in [
        (
            WindowKind::Settings,
            labels::ENGLISH.window_title,
            labels::VIETNAMESE.window_title,
        ),
        (
            WindowKind::Macros,
            labels::ENGLISH.macro_window_title,
            labels::VIETNAMESE.macro_window_title,
        ),
        (
            WindowKind::Convert,
            labels::ENGLISH.convert_window_title,
            labels::VIETNAMESE.convert_window_title,
        ),
        (
            WindowKind::AlwaysOn,
            labels::ENGLISH.always_on_window_title,
            labels::VIETNAMESE.always_on_window_title,
        ),
    ] {
        let mut window = Window::open(kind, &stores);
        assert_eq!(window.title(), english, "{kind:?} opens in English, the default");
        assert!(
            titles(&commands_drawing(&mut window)).is_empty(),
            "{kind:?} keeps the title it opened with"
        );

        window.set_preferences(vietnamese.clone());
        assert_eq!(
            titles(&commands_drawing(&mut window)),
            [translated],
            "{kind:?} is retitled in Vietnamese"
        );
        assert!(
            titles(&commands_drawing(&mut window)).is_empty(),
            "{kind:?} is retitled once, not every frame"
        );
    }
}

#[test]
fn a_reset_keeps_the_open_window_in_its_language() {
    let folder = TempFolder::new("reset-language");
    let stores = Stores::in_folder(folder.0.clone()).unwrap();
    let mut vietnamese = PreferencesView::default();
    vietnamese.system.interface_language = InterfaceLanguageValue::Vietnamese;
    vietnamese.typing.input_method = InputMethodValue::Vni;
    stores.save_preferences(&vietnamese).unwrap();
    let mut window = Window::open(WindowKind::Settings, &stores);
    assert_eq!(window.title(), labels::VIETNAMESE.window_title);

    // What the window process does for the reset button: reset what is stored, then show it in every window.
    let reset = stores.reset_preferences(&stores.load_preferences().unwrap()).unwrap();
    window.set_preferences(reset);

    assert_eq!(
        stores.load_preferences().unwrap().typing.input_method,
        PreferencesView::default().typing.input_method,
        "the settings went back to their defaults"
    );
    assert_eq!(window.interface_language(), InterfaceLanguageValue::Vietnamese);
    assert!(
        titles(&commands_drawing(&mut window)).is_empty(),
        "the window keeps its Vietnamese title"
    );
}

fn place(x: f32, y: f32, scale: f32) -> WindowPlace {
    WindowPlace { x, y, scale }
}

/// A monitor check that finds every place on a monitor.
const fn on_screen(_: ScreenRect) -> bool {
    true
}

/// Draws `window` once in a native window whose frame's top-left corner is at `corner`, in the points of a monitor of
/// `scale`, and returns what it told that native window.
fn draw_at(window: &mut Window, corner: egui::Pos2, scale: f32) -> Vec<egui::ViewportCommand> {
    let ctx = egui::Context::default();
    let mut input = egui::RawInput::default();
    let native = egui::ViewportInfo {
        native_pixels_per_point: Some(scale),
        outer_rect: Some(egui::Rect::from_min_size(corner, window.size())),
        ..egui::ViewportInfo::default()
    };
    input.viewports.insert(egui::ViewportId::ROOT, native);
    let mut output = ctx.run_ui(input, |ui| drop(window.show(ui)));
    // No renderer takes the font texture here.
    output.textures_delta.clear();
    output
        .viewport_output
        .get(&egui::ViewportId::ROOT)
        .map(|viewport| viewport.commands.clone())
        .unwrap_or_default()
}

/// Whether `commands` move the native window.
fn moves(commands: &[egui::ViewportCommand]) -> bool {
    commands
        .iter()
        .any(|command| matches!(command, egui::ViewportCommand::OuterPosition(_)))
}

#[test]
fn a_window_opens_at_its_saved_place() {
    let stores = empty_stores("saved-place");
    let icon = Arc::new(egui::IconData::default());
    let mut places = WindowPlaces::default();
    // Saved on a monitor at 150%, where 1500 by 300 pixels are 1000 by 200 points.
    places.set(PlacedWindow::Settings, place(1500.0, 300.0, 1.5));
    places.set(PlacedWindow::Macros, place(1500.0, 300.0, 1.5));

    let first = Window::open(WindowKind::Settings, &stores).at_place(&places, 1.0, on_screen);
    let options = native_options(&first, &icon);
    assert_eq!(options.viewport.position, Some(egui::pos2(1000.0, 200.0)));
    assert!(!options.centered, "eframe would move a centred window off its place");
    assert_eq!(
        options.viewport.inner_size,
        Some(COMPACT_SIZE),
        "it still opens compact"
    );

    let own = Window::open(WindowKind::Macros, &stores).at_place(&places, 1.0, on_screen);
    assert_eq!(
        own.viewport(&icon).position,
        Some(egui::pos2(1000.0, 200.0)),
        "a window in a viewport of its own too"
    );

    let zoomed = Window::open(WindowKind::Macros, &stores).at_place(&places, 1.25, on_screen);
    assert_eq!(
        zoomed.viewport(&icon).position,
        Some(egui::pos2(800.0, 160.0)),
        "the zoom makes egui's points larger"
    );
}

#[test]
fn an_off_screen_place_is_ignored() {
    let stores = empty_stores("off-screen");
    let icon = Arc::new(egui::IconData::default());
    let mut places = WindowPlaces::default();
    places.set(PlacedWindow::Settings, place(1500.0, 300.0, 1.5));
    let mut asked = None;

    let first = Window::open(WindowKind::Settings, &stores).at_place(&places, 1.0, |title_bar| {
        asked = Some(title_bar);
        false
    });

    let title_bar = ScreenRect {
        left: 1500,
        top: 300,
        right: 1500 + 840,
        bottom: 300 + 45,
    };
    assert_eq!(asked, Some(title_bar), "the check is of its title bar, in pixels");
    let options = native_options(&first, &icon);
    assert_eq!(options.viewport.position, None);
    assert!(options.centered, "it opens centred, as with no place");

    let unplaced = Window::open(WindowKind::Settings, &stores)
        .at_place(&WindowPlaces::default(), 1.0, |_| panic!("no place, nothing to check"));
    let options = native_options(&unplaced, &icon);
    assert_eq!(options.viewport.position, None);
    assert!(options.centered, "a first run opens centred");
}

#[test]
fn a_place_without_a_scale_is_ignored() {
    let stores = empty_stores("no-scale");
    let icon = Arc::new(egui::IconData::default());
    for scale in [0.0, -1.5] {
        let mut places = WindowPlaces::default();
        // A file edited by hand can hold anything.
        places.set(PlacedWindow::Macros, place(1500.0, 300.0, scale));

        let window = Window::open(WindowKind::Macros, &stores).at_place(&places, 1.0, on_screen);

        assert_eq!(window.viewport(&icon).position, None, "scale {scale}");
    }
}

#[test]
fn closing_a_window_that_never_said_where_it_was_writes_nothing() {
    let folder = TempFolder::new("close-unplaced");
    let stores = Stores::in_folder(folder.0.clone()).unwrap();
    let mut places = stores.window_places();
    let window = Window::open(WindowKind::Macros, &stores);

    keep_places(&stores, &mut places, [&window]);

    assert!(!folder.0.exists(), "no file, not even a folder, for nothing to keep");
}

#[test]
fn closing_a_window_saves_its_place() {
    let folder = TempFolder::new("close-place");
    let stores = Stores::in_folder(folder.0.clone()).unwrap();
    let mut places = stores.window_places();
    let mut window = Window::open(WindowKind::Macros, &stores);
    drop(draw_at(&mut window, egui::pos2(100.0, 50.0), 1.5));
    // The user moves it before closing it.
    drop(draw_at(&mut window, egui::pos2(200.0, 80.0), 1.5));

    keep_places(&stores, &mut places, [&window]);

    assert_eq!(
        stores.window_places().get(PlacedWindow::Macros),
        Some(place(300.0, 120.0, 1.5)),
        "the last place it reported, in pixels"
    );
}

#[test]
fn quitting_saves_every_open_window() {
    let folder = TempFolder::new("quit-places");
    let stores = Stores::in_folder(folder.0.clone()).unwrap();
    let mut places = stores.window_places();
    let mut settings = Window::open(WindowKind::Settings, &stores);
    let mut convert = Window::open(WindowKind::Convert, &stores);
    let never_drawn = Window::open(WindowKind::AlwaysOn, &stores);
    drop(draw_at(&mut settings, egui::pos2(10.0, 20.0), 1.0));
    drop(draw_at(&mut convert, egui::pos2(-900.0, 40.0), 1.25));

    keep_places(&stores, &mut places, &[settings, convert, never_drawn]);

    let saved = stores.window_places();
    assert_eq!(saved.get(PlacedWindow::Settings), Some(place(10.0, 20.0, 1.0)));
    assert_eq!(saved.get(PlacedWindow::Convert), Some(place(-1125.0, 50.0, 1.25)));
    assert_eq!(
        saved.get(PlacedWindow::AlwaysOn),
        None,
        "a window that never said where it was keeps no place"
    );
}

#[test]
fn each_kind_remembers_its_own_place() {
    let folder = TempFolder::new("own-places");
    let stores = Stores::in_folder(folder.0.clone()).unwrap();
    let icon = Arc::new(egui::IconData::default());
    let mut places = stores.window_places();
    for (kind, corner) in [
        (WindowKind::Macros, egui::pos2(100.0, 100.0)),
        (WindowKind::AlwaysOn, egui::pos2(700.0, 300.0)),
        (WindowKind::Settings, egui::pos2(400.0, 20.0)),
    ] {
        let mut window = Window::open(kind, &stores);
        drop(draw_at(&mut window, corner, 1.0));
        keep_places(&stores, &mut places, [&window]);
    }

    // The next run reads them all.
    let saved = stores.window_places();
    for (kind, corner) in [
        (WindowKind::Macros, Some(egui::pos2(100.0, 100.0))),
        (WindowKind::AlwaysOn, Some(egui::pos2(700.0, 300.0))),
        (WindowKind::About, Some(egui::pos2(400.0, 20.0))),
        (WindowKind::Convert, None),
    ] {
        let reopened = Window::open(kind, &stores).at_place(&saved, 1.0, on_screen);
        assert_eq!(reopened.viewport(&icon).position, corner, "{kind:?}");
    }
}

#[test]
fn a_window_moved_into_the_root_keeps_its_own_place() {
    let stores = empty_stores("root-place");
    let mut window = Window::open(WindowKind::Macros, &stores);
    // Its own native window was at 300 by 120 pixels, on a monitor at 150%.
    drop(draw_at(&mut window, egui::pos2(200.0, 80.0), 1.5));
    // The root's native window is on a monitor at 125%, where those pixels are 240 by 96 points.
    let root = egui::ViewportInfo {
        native_pixels_per_point: Some(1.25),
        ..egui::ViewportInfo::default()
    };

    let commands = root_commands_in(root, |ctx| move_into_root(ctx, &window));

    assert!(
        commands.contains(&egui::ViewportCommand::OuterPosition(egui::pos2(240.0, 96.0))),
        "the root moves to where the window was, so it does not jump: {commands:?}"
    );

    let unplaced = Window::open(WindowKind::Convert, &stores);
    let commands = root_commands_moving(&unplaced);
    assert!(
        !moves(&commands),
        "a window that never said where it was leaves the root where it is: {commands:?}"
    );
}

#[test]
fn a_window_that_lands_off_its_place_is_moved_onto_it() {
    let stores = empty_stores("settle");
    let mut places = WindowPlaces::default();
    places.set(PlacedWindow::Macros, place(1500.0, 300.0, 1.5));

    let mut landed_on = Window::open(WindowKind::Macros, &stores).at_place(&places, 1.0, on_screen);
    let commands = draw_at(&mut landed_on, egui::pos2(1000.0, 200.0), 1.5);
    assert!(!moves(&commands), "on its place already: {commands:?}");

    // Created on a monitor at 100%, the window turned the points of its place into pixels at that scale.
    let mut landed_off = Window::open(WindowKind::Macros, &stores).at_place(&places, 1.0, on_screen);
    let commands = draw_at(&mut landed_off, egui::pos2(1000.0, 200.0), 1.0);
    assert!(
        commands.contains(&egui::ViewportCommand::OuterPosition(egui::pos2(1500.0, 300.0))),
        "moved onto its place, in the points of the monitor it is on: {commands:?}"
    );
    let again = draw_at(&mut landed_off, egui::pos2(1600.0, 200.0), 1.0);
    assert!(!moves(&again), "only once, so the user can move it: {again:?}");
}
