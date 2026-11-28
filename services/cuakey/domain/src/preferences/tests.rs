use alloc::collections::{BTreeMap, BTreeSet};

use super::Preferences;
use crate::{
    AppName, AppState, CodeTable, ConvertCase, ConvertOptions, Hotkey, InputMethod, InterfaceLanguage, KeyCode,
    Language, Modifier, ModifierSet, OnOff, PreferencesError, ProfileId, SettingChange, Settings, Theme,
};

fn mods(list: &[Modifier]) -> ModifierSet {
    list.iter().fold(ModifierSet::empty(), |set, m| set.with(*m))
}

fn app(raw: &str) -> AppName {
    AppName::new(raw).unwrap()
}

fn fresh() -> Preferences {
    Preferences::new(ProfileId::new("default").unwrap(), Settings::default()).unwrap()
}

/// Preferences with the per-app memory switched on, as a user who turned it on has it.
fn remembering() -> Preferences {
    let mut preferences = fresh();
    preferences.update(SettingChange::SmartSwitch(OnOff::On)).unwrap();
    preferences.update(SettingChange::RememberCodeTable(OnOff::On)).unwrap();
    preferences
}

#[test]
fn typing_defaults_match_the_documented_ones() {
    let preferences = fresh();
    let typing = preferences.settings().typing();

    assert_eq!(typing.input_method(), InputMethod::Telex, "input method");
    assert_eq!(typing.code_table(), CodeTable::Unicode, "code table");
    assert_eq!(typing.new_tone_style(), OnOff::Off, "new-style tone placement");
    assert_eq!(typing.spell_check(), OnOff::On, "spell check");
    assert_eq!(typing.restore_invalid_word(), OnOff::On, "restore typed keys");
    assert_eq!(typing.allow_zwjf_consonants(), OnOff::Off, "z, w, j, f first");
    assert_eq!(typing.capitalize_sentences(), OnOff::Off, "capitalize sentences");
    assert_eq!(typing.quick_telex(), OnOff::Off, "quick telex");
    assert_eq!(typing.quick_first_consonant(), OnOff::Off, "quick first consonant");
    assert_eq!(typing.quick_last_consonant(), OnOff::Off, "quick last consonant");
}

#[test]
fn macro_and_switching_defaults_match_the_documented_ones() {
    let preferences = fresh();
    let (macros, switching) = (preferences.settings().macros(), preferences.settings().switching());
    let ctrl_shift = Hotkey::new(mods(&[Modifier::Ctrl, Modifier::Shift]), None).unwrap();

    assert_eq!(macros.use_macros(), OnOff::On, "macros");
    assert_eq!(macros.macros_in_english(), OnOff::Off, "macros in english");
    assert_eq!(macros.match_case(), OnOff::Off, "macro capitals");
    assert_eq!(switching.hotkey(), ctrl_shift, "switch hotkey");
    assert_eq!(switching.beep(), OnOff::Off, "beep");
    assert_eq!(switching.smart_switch(), OnOff::Off, "smart switch");
    assert_eq!(switching.remember_code_table(), OnOff::Off, "remember code table");
    assert_eq!(
        switching.work_in_other_languages(),
        OnOff::On,
        "other keyboard languages"
    );
}

#[test]
fn compatibility_and_system_defaults_match_the_documented_ones() {
    let preferences = fresh();
    let (compat, system) = (preferences.settings().compatibility(), preferences.settings().system());

    assert_eq!(compat.fix_autocomplete(), OnOff::On, "autocomplete fix");
    assert_eq!(compat.chromium_fix(), OnOff::Off, "chromium fix");
    assert_eq!(compat.clipboard_input(), OnOff::Off, "clipboard input");
    assert_eq!(compat.store_app_support(), OnOff::Off, "store app support");
    assert_eq!(system.start_with_windows(), OnOff::On, "start with windows");
    assert_eq!(
        system.show_settings_at_startup(),
        OnOff::Off,
        "the settings window stays closed at startup; the tray icon opens it"
    );
    assert_eq!(system.run_as_administrator(), OnOff::Off, "run as administrator");
    assert_eq!(system.desktop_shortcut(), OnOff::Off, "desktop shortcut");
}

#[test]
fn interface_language_defaults_to_english() {
    assert_eq!(
        fresh().settings().system().interface_language(),
        InterfaceLanguage::English
    );
}

#[test]
fn the_theme_defaults_to_dark() {
    assert_eq!(fresh().settings().system().theme(), Theme::Dark);
}

#[test]
fn a_theme_change_is_kept() {
    let mut preferences = fresh();

    preferences.update(SettingChange::Theme(Theme::Light)).unwrap();

    assert_eq!(preferences.settings().system().theme(), Theme::Light);
}

#[test]
fn behaviour_and_convert_defaults_match_the_documented_ones() {
    let preferences = fresh();
    let (behaviours, convert) = (preferences.settings().behaviours(), preferences.settings().convert());
    let settings_hotkey = Hotkey::new(mods(&[Modifier::Ctrl, Modifier::Shift]), Some(KeyCode::F5)).unwrap();

    assert_eq!(behaviours.settings_hotkey(), settings_hotkey, "settings hotkey");
    assert_eq!(behaviours.keep_w(), OnOff::On, "w at word start stays w");
    assert_eq!(behaviours.only_text_boxes(), OnOff::On, "only in text boxes");
    assert_eq!(convert.quick_convert_hotkey(), None, "quick convert hotkey");
    assert_eq!(convert.notify_when_done(), OnOff::On, "convert notification");
    assert!(preferences.always_on_apps().is_empty(), "always-on list");
    assert_eq!(preferences.language(), Language::English, "language at start");
}

#[test]
fn equal_hotkeys_are_refused() {
    let ctrl_shift = Hotkey::new(mods(&[Modifier::Ctrl, Modifier::Shift]), None).unwrap();
    let settings = Settings::default().with(SettingChange::SettingsHotkey(ctrl_shift));

    let made = Preferences::new(ProfileId::new("default").unwrap(), settings);

    assert_eq!(made, Err(PreferencesError::SwitchEqualsSettings));
}

#[test]
fn a_quick_convert_hotkey_equal_to_another_is_refused() {
    let ctrl_shift = Hotkey::new(mods(&[Modifier::Ctrl, Modifier::Shift]), None).unwrap();
    let mut preferences = fresh();

    let outcome = preferences.update(SettingChange::QuickConvertHotkey(Some(ctrl_shift)));

    assert_eq!(outcome, Err(PreferencesError::QuickConvertClashes));
    assert_eq!(
        preferences.settings().convert().quick_convert_hotkey(),
        None,
        "a refused change leaves no trace"
    );
}

#[test]
fn an_update_that_makes_two_hotkeys_equal_is_refused() {
    let mut preferences = fresh();
    let settings_hotkey = preferences.settings().behaviours().settings_hotkey();

    let outcome = preferences.update(SettingChange::SwitchHotkey(settings_hotkey));

    assert_eq!(outcome, Err(PreferencesError::SwitchEqualsSettings));
}

#[test]
fn an_update_changes_the_setting() {
    let mut preferences = fresh();

    preferences
        .update(SettingChange::InputMethod(InputMethod::Vni))
        .unwrap();
    preferences.update(SettingChange::SpellCheck(OnOff::Off)).unwrap();

    assert_eq!(
        (
            preferences.settings().typing().input_method(),
            preferences.settings().typing().spell_check()
        ),
        (InputMethod::Vni, OnOff::Off)
    );
}

#[test]
fn empty_hotkey_is_refused() {
    assert_eq!(
        Hotkey::new(ModifierSet::empty(), None),
        Err(PreferencesError::EmptyHotkey)
    );
}

#[test]
fn a_key_alone_is_a_hotkey() {
    assert_eq!(
        Hotkey::new(ModifierSet::empty(), Some(KeyCode::F9)).unwrap().key(),
        Some(KeyCode::F9)
    );
}

#[test]
fn app_names_normalise() {
    let by_path = app(r"C:\Games\Game.EXE");
    let by_name = app("GAME.exe");
    let mut preferences = fresh();

    preferences.set_always_on(by_path.clone(), OnOff::On);

    assert_eq!(by_path, by_name, "a path and a name in another case are the same app");
    assert_eq!(by_path.as_str(), "game.exe");
    assert!(preferences.is_always_on(&by_path) && preferences.is_always_on(&by_name));
}

#[test]
fn an_empty_app_name_is_refused() {
    assert_eq!(AppName::new(r"C:\Games\"), Err(PreferencesError::EmptyAppName));
}

#[test]
fn always_on_can_be_turned_off_again() {
    let mut preferences = fresh();
    preferences.set_always_on(app("game.exe"), OnOff::On);

    preferences.set_always_on(app("game.exe"), OnOff::Off);

    assert!(!preferences.is_always_on(&app("game.exe")));
}

#[test]
fn app_switch_restores_memory() {
    let mut preferences = remembering();
    preferences.set_language(Language::English);
    preferences.remember_app(app("zalo.exe"));
    preferences.set_language(Language::Vietnamese);

    let known_chrome = preferences.apply_app_switch(&app("chrome.exe"));
    let known_zalo = preferences.apply_app_switch(&app("zalo.exe"));

    assert!(!known_chrome, "an app never seen is only remembered");
    assert!(known_zalo, "an app seen before is restored");
    assert_eq!(preferences.language(), Language::English);
}

#[test]
fn an_unknown_app_is_remembered_with_the_current_state() {
    let mut preferences = remembering();
    preferences.set_language(Language::English);
    preferences.apply_app_switch(&app("chrome.exe"));
    preferences.set_language(Language::Vietnamese);

    let known = preferences.apply_app_switch(&app("chrome.exe"));

    assert!(known);
    assert_eq!(preferences.language(), Language::English);
}

#[test]
fn explorer_is_ignored_by_app_switch() {
    let mut preferences = fresh();
    preferences.set_language(Language::English);

    let known = preferences.apply_app_switch(&app("Explorer.EXE"));
    preferences.set_language(Language::Vietnamese);
    let known_again = preferences.apply_app_switch(&app("explorer.exe"));

    assert_eq!((known, known_again), (false, false));
    assert_eq!(
        preferences.language(),
        Language::Vietnamese,
        "nothing was stored or adopted"
    );
}

#[test]
fn the_code_table_is_restored_only_when_remembering_it_is_on() {
    let mut preferences = remembering();
    preferences.update(SettingChange::CodeTable(CodeTable::Tcvn3)).unwrap();
    preferences.remember_app(app("a.exe"));
    preferences
        .update(SettingChange::CodeTable(CodeTable::Unicode))
        .unwrap();
    preferences.apply_app_switch(&app("a.exe"));
    let restored = preferences.settings().typing().code_table();
    preferences
        .update(SettingChange::RememberCodeTable(OnOff::Off))
        .unwrap();
    preferences
        .update(SettingChange::CodeTable(CodeTable::Unicode))
        .unwrap();
    preferences.apply_app_switch(&app("a.exe"));

    assert_eq!(restored, CodeTable::Tcvn3);
    assert_eq!(preferences.settings().typing().code_table(), CodeTable::Unicode);
}

#[test]
fn the_language_is_kept_when_smart_switch_is_off() {
    let mut preferences = fresh();
    preferences.set_language(Language::English);
    preferences.remember_app(app("a.exe"));
    preferences.set_language(Language::Vietnamese);
    preferences.update(SettingChange::SmartSwitch(OnOff::Off)).unwrap();

    preferences.apply_app_switch(&app("a.exe"));

    assert_eq!(preferences.language(), Language::Vietnamese);
}

#[test]
fn own_window_is_never_adopted_or_remembered() {
    let own = app("cuakey.exe");
    let stale = BTreeMap::from([(own.clone(), AppState::new(Language::Vietnamese, CodeTable::Tcvn3))]);
    for memory in [stale, BTreeMap::new()] {
        let mut preferences = Preferences::restore(
            ProfileId::new("default").unwrap(),
            Settings::default(),
            Language::English,
            BTreeSet::new(),
            memory.clone(),
        );

        let adopted = preferences.apply_app_switch(&own);
        preferences.remember_app(own.clone());

        assert!(!adopted, "switch");
        assert_eq!(preferences.language(), Language::English, "language");
        assert_eq!(
            preferences.settings().typing().code_table(),
            CodeTable::Unicode,
            "code table"
        );
        assert_eq!(preferences.app_memory(), &memory, "app memory");
    }
}

#[test]
fn restore_round_trip() {
    let mut preferences = fresh();
    preferences
        .update(SettingChange::InputMethod(InputMethod::SimpleTelex))
        .unwrap();
    preferences.set_always_on(app("game.exe"), OnOff::On);
    preferences.set_language(Language::English);
    preferences.remember_app(app("zalo.exe"));

    let always_on: BTreeSet<AppName> = preferences.always_on_apps().clone();
    let memory: BTreeMap<AppName, AppState> = preferences.app_memory().clone();
    let restored = Preferences::restore(
        preferences.id().clone(),
        preferences.settings().clone(),
        preferences.language(),
        always_on,
        memory,
    );

    assert_eq!(restored, preferences);
}

#[test]
fn restore_does_not_replace_stored_values_with_defaults() {
    let settings = Settings::default().with(SettingChange::SpellCheck(OnOff::Off));

    let restored = Preferences::restore(
        ProfileId::new("default").unwrap(),
        settings,
        Language::English,
        BTreeSet::new(),
        BTreeMap::new(),
    );

    assert_eq!(restored.settings().typing().spell_check(), OnOff::Off);
    assert_eq!(restored.language(), Language::English);
}

#[test]
fn a_blank_profile_id_is_refused() {
    assert_eq!(ProfileId::new("  "), Err(PreferencesError::EmptyId));
}

#[test]
fn the_conversion_options_default_to_unicode_with_nothing_changed() {
    let preferences = fresh();

    let options = preferences.settings().convert().options();

    assert_eq!(
        options,
        ConvertOptions::new(CodeTable::Unicode, CodeTable::Unicode),
        "unicode to unicode, case and accents kept"
    );
}

#[test]
fn the_conversion_options_can_be_set() {
    let mut preferences = fresh();
    let options = ConvertOptions::new(CodeTable::VniWindows, CodeTable::Unicode)
        .with_case(ConvertCase::Upper)
        .with_remove_accents(OnOff::On);

    preferences.update(SettingChange::ConvertOptions(options)).unwrap();

    assert_eq!(preferences.settings().convert().options(), options);
}

#[test]
fn defaults_are_english_without_per_app_memory() {
    let preferences = fresh();

    assert_eq!(preferences.language(), Language::English, "language");
    assert_eq!(
        preferences.settings().switching().smart_switch(),
        OnOff::Off,
        "smart switch"
    );
    assert_eq!(
        preferences.settings().switching().remember_code_table(),
        OnOff::Off,
        "remember code table"
    );
}
