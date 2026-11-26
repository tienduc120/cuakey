use cuakey_application::{
    AppMemoryView, CodeTableValue, InterfaceLanguageValue, LanguageValue, PreferencesView, Toggle,
};
use cuakey_interfaces::MenuId;

use super::{rebase_menu_change, rebase_session};

/// What the window process stored while the session still had the older settings: spell checking off, game.exe always
/// on.
fn stored() -> PreferencesView {
    let mut stored = PreferencesView {
        always_on: vec!["game.exe".to_owned()],
        ..PreferencesView::default()
    };
    stored.typing.spell_check = Toggle::Off;
    stored
}

/// The session's preferences: the older settings, with what the session itself changed since.
fn session() -> PreferencesView {
    let mut session = PreferencesView {
        language: LanguageValue::English,
        app_memory: vec![AppMemoryView {
            app: "notepad.exe".to_owned(),
            language: LanguageValue::English,
            code_table: CodeTableValue::Tcvn3,
        }],
        ..PreferencesView::default()
    };
    session.typing.code_table = CodeTableValue::Tcvn3;
    session
}

#[test]
fn a_session_save_takes_only_what_the_session_changes() {
    let saved = rebase_session(stored(), &session());

    let mut expected = PreferencesView {
        language: LanguageValue::English,
        app_memory: session().app_memory,
        ..stored()
    };
    expected.typing.code_table = CodeTableValue::Tcvn3;
    assert_eq!(saved, expected);
}

#[test]
fn a_menu_change_applies_to_the_rebased_preferences() {
    let saved = rebase_menu_change(stored(), &session(), MenuId::UseMacros);

    assert_eq!(saved.macros.use_macros, Toggle::Off, "the click");
    assert_eq!(saved.typing.spell_check, Toggle::Off, "kept from the store");
    assert_eq!(saved.always_on, stored().always_on, "kept from the store");
    assert_eq!(saved.language, LanguageValue::English, "the session's");
}

#[test]
fn a_session_save_keeps_the_stored_interface_language() {
    let mut stored = stored();
    stored.system.interface_language = InterfaceLanguageValue::Vietnamese;
    // The session switched the language with the preferences it started on, which hold the default interface.
    let switched = PreferencesView {
        language: LanguageValue::Vietnamese,
        ..session()
    };
    assert_eq!(switched.system.interface_language, InterfaceLanguageValue::English);

    let saved = rebase_session(stored, &switched);

    assert_eq!(saved.language, LanguageValue::Vietnamese, "the switch");
    assert_eq!(
        saved.system.interface_language,
        InterfaceLanguageValue::Vietnamese,
        "the interface the store holds"
    );
}
