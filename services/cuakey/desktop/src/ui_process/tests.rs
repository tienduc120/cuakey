use cuakey_application::{AppMemoryView, CodeTableValue, LanguageValue, PreferencesView, Toggle};
use cuakey_interfaces::WindowKind;

use super::{Edit, edit_of, largest_png, rebase, shown_kind};

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
    let mut edited = PreferencesView::default();
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
