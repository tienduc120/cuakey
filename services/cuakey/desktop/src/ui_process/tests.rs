use cuakey_application::{AppMemoryView, CodeTableValue, LanguageValue, PreferencesView, Toggle};
use cuakey_interfaces::{COMPACT_SIZE, EXPANDED_SIZE, WindowKind, labels};
use eframe::egui;

use super::window::Window;
use super::{Edit, edit_of, expand_for_about, largest_png, move_into_root, rebase, shown_kind};
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

/// Stores on a folder that does not exist: every window shows the defaults, and opening one writes nothing.
fn empty_stores(test: &str) -> Stores {
    let dir = std::env::temp_dir().join(format!("cuakey-ui-process-{}-{test}", std::process::id()));
    Stores::in_folder(dir).unwrap()
}

/// What `send` tells the root viewport in one pass.
fn root_commands(send: impl FnOnce(&egui::Context)) -> Vec<egui::ViewportCommand> {
    let ctx = egui::Context::default();
    let mut send = Some(send);
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
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

#[test]
fn about_expands_an_open_settings_window() {
    let stores = empty_stores("expand");
    let mut window = Window::open(WindowKind::Settings, &stores);

    let commands = root_commands(|ctx| expand_for_about(ctx, &mut window, egui::ViewportId::ROOT));
    assert_eq!(window.size(), EXPANDED_SIZE, "the about text is in the expanded part");
    assert!(
        commands.contains(&egui::ViewportCommand::InnerSize(EXPANDED_SIZE)),
        "the native window grows with it: {commands:?}"
    );

    let again = root_commands(|ctx| expand_for_about(ctx, &mut window, egui::ViewportId::ROOT));
    assert_eq!(window.size(), EXPANDED_SIZE);
    assert!(
        !again
            .iter()
            .any(|command| matches!(command, egui::ViewportCommand::InnerSize(_))),
        "an expanded window keeps the size the user gave it: {again:?}"
    );
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
        ends_in_view(&mut opened, labels::SECTION_ABOUT),
        "a window opened for About shows the about text"
    );

    let mut open_compact = Window::open(WindowKind::Settings, &stores);
    drop(root_commands(|ctx| {
        expand_for_about(ctx, &mut open_compact, egui::ViewportId::ROOT);
    }));
    assert!(
        ends_in_view(&mut open_compact, labels::SECTION_ABOUT),
        "a window About expanded shows the about text"
    );
}
