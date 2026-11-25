use super::{
    LoadPreferences, LoadPreferencesError, ResetPreferences, ResetPreferencesError, SavePreferences,
    SavePreferencesError, app_name,
};
use crate::testing::{FakePreferencesStore, Health};
use crate::{
    AppMemoryView, BehaviourPrefs, CodeTableValue, CompatibilityPrefs, ConvertCaseValue, ConvertPrefs, HotkeyView,
    InputMethodValue, KeyName, LanguageValue, MacroPrefs, PreferencesView, StoreError, SwitchPrefs, SystemPrefs,
    Toggle, TypingPrefs, UpdateError,
};

fn hotkey(modifiers: [Toggle; 4], key: Option<KeyName>) -> HotkeyView {
    let [ctrl, alt, win, shift] = modifiers;
    HotkeyView {
        ctrl,
        alt,
        win,
        shift,
        key,
    }
}

fn memory(app: &str, language: LanguageValue, code_table: CodeTableValue) -> AppMemoryView {
    AppMemoryView {
        app: app.to_owned(),
        language,
        code_table,
    }
}

/// Every preference away from its default, with app memory unlike the current language and code table.
fn everything_changed() -> PreferencesView {
    use Toggle::{Off, On};
    PreferencesView {
        profile_id: "work".to_owned(),
        language: LanguageValue::English,
        typing: TypingPrefs {
            input_method: InputMethodValue::Vni,
            code_table: CodeTableValue::Tcvn3,
            new_tone_style: On,
            spell_check: Off,
            restore_invalid_word: Off,
            allow_zwjf_consonants: On,
            capitalize_sentences: On,
            quick_telex: On,
            quick_first_consonant: On,
            quick_last_consonant: On,
        },
        macros: MacroPrefs {
            use_macros: Off,
            macros_in_english: On,
            match_case: On,
        },
        switching: SwitchPrefs {
            hotkey: hotkey([On, On, Off, Off], Some(KeyName::Z)),
            beep: On,
            smart_switch: Off,
            remember_code_table: Off,
            work_in_other_languages: Off,
        },
        compatibility: CompatibilityPrefs {
            fix_autocomplete: Off,
            chromium_fix: On,
            clipboard_input: On,
            store_app_support: On,
        },
        system: SystemPrefs {
            start_with_windows: Off,
            show_settings_at_startup: Off,
            run_as_administrator: On,
            desktop_shortcut: On,
        },
        behaviours: BehaviourPrefs {
            settings_hotkey: hotkey([Off, Off, On, Off], Some(KeyName::F9)),
            keep_w: Off,
            only_text_boxes: Off,
        },
        convert: ConvertPrefs {
            quick_convert_hotkey: Some(hotkey([On, Off, Off, On], Some(KeyName::Q))),
            notify_when_done: Off,
            from: CodeTableValue::VniWindows,
            to: CodeTableValue::UnicodeCompound,
            case: ConvertCaseValue::EachWord,
            remove_accents: On,
        },
        always_on: vec!["game.exe".to_owned(), "zalo.exe".to_owned()],
        app_memory: vec![
            memory("chrome.exe", LanguageValue::Vietnamese, CodeTableValue::VniWindows),
            memory("word.exe", LanguageValue::English, CodeTableValue::Cp1258),
        ],
    }
}

/// The defaults with the settings hotkey made equal to the switch hotkey.
fn equal_hotkeys() -> PreferencesView {
    let mut prefs = PreferencesView::default();
    prefs.behaviours.settings_hotkey = prefs.switching.hotkey;
    prefs
}

#[test]
fn a_view_survives_the_trip_through_the_aggregate() {
    let prefs = everything_changed();

    let back = PreferencesView::of(&prefs.to_preferences().unwrap());

    assert_eq!(back, prefs, "every field is carried both ways");
}

#[test]
fn app_names_are_stored_as_lower_case_file_names() {
    let prefs = PreferencesView {
        always_on: vec![r"C:\Games\Game.EXE".to_owned()],
        ..PreferencesView::default()
    };

    let back = PreferencesView::of(&prefs.to_preferences().unwrap());

    assert_eq!(back.always_on, vec!["game.exe".to_owned()]);
}

#[test]
fn a_modifier_cannot_be_the_key_of_a_hotkey() {
    let mut prefs = PreferencesView::default();
    prefs.behaviours.settings_hotkey.key = Some(KeyName::Shift);

    assert_eq!(prefs.to_preferences(), Err(UpdateError::InvalidHotkeyKey));
}

#[tokio::test]
async fn load_gives_the_defaults_when_nothing_was_saved() {
    let load = LoadPreferences::new(FakePreferencesStore::new(Health::Working));

    let prefs = load.execute().await.unwrap();

    assert_eq!(prefs, PreferencesView::default());
}

#[tokio::test]
async fn load_gives_what_was_saved() {
    let load = LoadPreferences::new(FakePreferencesStore::holding(&everything_changed()).unwrap());

    let prefs = load.execute().await.unwrap();

    assert_eq!(prefs, everything_changed());
}

#[tokio::test]
async fn load_keeps_the_cause_of_a_storage_failure() {
    let load = LoadPreferences::new(FakePreferencesStore::new(Health::Broken));

    let err = load.execute().await.unwrap_err();

    assert!(
        matches!(err, LoadPreferencesError::Storage(StoreError::Storage(_))),
        "{err:?}"
    );
}

#[tokio::test]
async fn save_stores_valid_preferences() {
    let save = SavePreferences::new(FakePreferencesStore::new(Health::Working));

    save.execute(&everything_changed()).await.unwrap();

    assert_eq!(save.store().saved(), vec![everything_changed()]);
}

#[tokio::test]
async fn save_refuses_equal_hotkeys() {
    let save = SavePreferences::new(FakePreferencesStore::new(Health::Working));

    let err = save.execute(&equal_hotkeys()).await.unwrap_err();

    assert!(
        matches!(err, SavePreferencesError::Invalid(UpdateError::SwitchEqualsSettings)),
        "{err:?}"
    );
    assert_eq!(save.store().saved(), vec![], "nothing saved");
}

#[tokio::test]
async fn save_keeps_the_cause_of_a_storage_failure() {
    let save = SavePreferences::new(FakePreferencesStore::new(Health::Broken));

    let err = save.execute(&PreferencesView::default()).await.unwrap_err();

    assert!(
        matches!(err, SavePreferencesError::Storage(StoreError::Storage(_))),
        "{err:?}"
    );
}

#[tokio::test]
async fn reset_keeps_always_on_and_memory() {
    let reset = ResetPreferences::new(FakePreferencesStore::new(Health::Working));
    let current = everything_changed();

    let prefs = reset.execute(&current).await.unwrap();

    let expected = PreferencesView {
        profile_id: current.profile_id.clone(),
        always_on: current.always_on.clone(),
        app_memory: current.app_memory.clone(),
        ..PreferencesView::default()
    };
    assert_eq!(prefs, expected, "defaults, with the app lists kept");
    assert_eq!(reset.store().saved(), vec![expected], "saved");
}

#[tokio::test]
async fn reset_keeps_the_cause_of_a_storage_failure() {
    let reset = ResetPreferences::new(FakePreferencesStore::new(Health::Broken));

    let err = reset.execute(&PreferencesView::default()).await.unwrap_err();

    assert!(
        matches!(err, ResetPreferencesError::Storage(StoreError::Storage(_))),
        "{err:?}"
    );
}

#[test]
fn app_name_keeps_the_lower_case_file_name() {
    assert_eq!(app_name(r"C:\Games\Game.EXE").unwrap(), "game.exe");
    assert_eq!(app_name("  Notes.exe ").unwrap(), "notes.exe");
}

#[test]
fn app_name_refuses_blank_text() {
    assert_eq!(app_name("   ").unwrap_err(), UpdateError::EmptyAppName);
    assert_eq!(app_name(r"C:\Games\").unwrap_err(), UpdateError::EmptyAppName);
}

#[test]
fn default_view_is_english() {
    assert_eq!(PreferencesView::default().language, LanguageValue::English);
}
