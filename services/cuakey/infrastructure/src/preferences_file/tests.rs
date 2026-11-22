use cuakey_application::PreferencesStore as _;
use cuakey_domain::{
    AppName, CodeTable, ConvertCase, ConvertOptions, Hotkey, InputMethod, KeyCode, Language, Modifier, ModifierSet,
    OnOff, Preferences, ProfileId, SettingChange, Settings,
};
use platform_clock::{FixedClock, Timestamp};

use super::JsonPreferencesStore;
use crate::test_dir::TestDir;

fn store(dir: &TestDir) -> JsonPreferencesStore<FixedClock> {
    let clock = FixedClock(Timestamp::from_second(1_700_000_000).unwrap());
    JsonPreferencesStore::new(dir.path().to_path_buf(), clock)
}

fn defaults() -> Preferences {
    Preferences::new(ProfileId::new("default").unwrap(), Settings::default()).unwrap()
}

fn write_settings(dir: &TestDir, json: &str) {
    std::fs::write(dir.path().join("settings.json"), json).unwrap();
}

fn hotkey(modifiers: &[Modifier], key: Option<KeyCode>) -> Hotkey {
    let set = modifiers.iter().fold(ModifierSet::empty(), |set, m| set.with(*m));
    Hotkey::new(set, key).unwrap()
}

/// Preferences with something other than the default in every part that is stored.
fn customised() -> Preferences {
    let mut preferences = defaults();
    let changes = [
        SettingChange::InputMethod(InputMethod::Vni),
        SettingChange::CodeTable(CodeTable::Tcvn3),
        SettingChange::SpellCheck(OnOff::Off),
        SettingChange::UseMacros(OnOff::Off),
        SettingChange::SwitchHotkey(hotkey(&[Modifier::Alt], Some(KeyCode::Z))),
        SettingChange::SettingsHotkey(hotkey(&[Modifier::Ctrl, Modifier::Win], Some(KeyCode::Digit0))),
        SettingChange::QuickConvertHotkey(Some(hotkey(&[Modifier::Ctrl, Modifier::Alt], Some(KeyCode::F12)))),
        SettingChange::BeepOnSwitch(OnOff::On),
        SettingChange::ChromiumFix(OnOff::On),
        SettingChange::StartWithWindows(OnOff::Off),
        SettingChange::KeepW(OnOff::Off),
        SettingChange::NotifyWhenConvertDone(OnOff::Off),
        SettingChange::ConvertOptions(
            ConvertOptions::new(CodeTable::Cp1258, CodeTable::Unicode)
                .with_case(ConvertCase::EachWord)
                .with_remove_accents(OnOff::On),
        ),
    ];
    for change in changes {
        preferences.update(change).unwrap();
    }
    preferences.set_always_on(AppName::new("Game.EXE").unwrap(), OnOff::On);
    preferences.set_language(Language::English);
    preferences.remember_app(AppName::new("notepad.exe").unwrap());
    preferences.set_language(Language::Vietnamese);
    preferences
}

#[tokio::test]
async fn missing_file_gives_defaults() {
    let dir = TestDir::new("missing_file");

    let loaded = store(&dir).load().await.unwrap();

    assert!(
        loaded.is_none(),
        "nothing saved yet means the application applies the defaults"
    );
}

#[tokio::test]
async fn round_trip() {
    let dir = TestDir::new("round_trip");
    let saved = customised();

    store(&dir).save(&saved).await.unwrap();
    let loaded = store(&dir).load().await.unwrap();

    assert_eq!(loaded, Some(saved), "every stored part comes back as it was saved");
    assert_eq!(
        dir.file_names(),
        ["settings.json"],
        "the temporary file is gone after the rename"
    );
}

#[tokio::test]
async fn saving_again_replaces_the_file() {
    let dir = TestDir::new("save_twice");
    let store = store(&dir);
    store.save(&customised()).await.unwrap();

    store.save(&defaults()).await.unwrap();

    assert_eq!(store.load().await.unwrap(), Some(defaults()));
}

#[tokio::test]
async fn older_file_gets_defaults() {
    let dir = TestDir::new("older_file");
    write_settings(&dir, r#"{"switching":{}}"#);

    let loaded = store(&dir).load().await.unwrap();

    assert_eq!(
        loaded,
        Some(defaults()),
        "a group with no fields, and every missing group, take defaults"
    );
}

#[tokio::test]
async fn an_empty_object_gets_defaults() {
    let dir = TestDir::new("empty_object");
    write_settings(&dir, "{}");

    assert_eq!(store(&dir).load().await.unwrap(), Some(defaults()));
}

#[tokio::test]
async fn unknown_fields_ignored() {
    let dir = TestDir::new("unknown_fields");
    write_settings(
        &dir,
        r#"{"from_the_future":1,"typing":{"input_method":"vni","hologram":true}}"#,
    );

    let loaded = store(&dir).load().await.unwrap().unwrap();

    assert_eq!(
        loaded.settings().typing().input_method(),
        InputMethod::Vni,
        "known fields are kept"
    );
    assert_eq!(
        loaded.settings().typing().code_table(),
        CodeTable::Unicode,
        "missing ones take defaults"
    );
}

#[tokio::test]
async fn corrupt_file_kept_aside() {
    let dir = TestDir::new("corrupt_file");
    write_settings(&dir, "{ this is not json");

    let loaded = store(&dir).load().await.unwrap();

    assert!(loaded.is_none(), "a corrupt file means defaults");
    assert_eq!(
        dir.file_names(),
        ["settings.corrupt-1700000000.json"],
        "the file is renamed with the clock's time, and no settings.json remains"
    );
    let kept = std::fs::read(dir.path().join("settings.corrupt-1700000000.json")).unwrap();
    assert_eq!(kept, b"{ this is not json", "the original bytes are kept");
}

#[tokio::test]
async fn an_unknown_choice_is_a_corrupt_file_not_a_silent_reset() {
    let dir = TestDir::new("unknown_choice");
    write_settings(&dir, r#"{"typing":{"input_method":"morse"}}"#);

    let loaded = store(&dir).load().await.unwrap();

    assert!(loaded.is_none());
    assert_eq!(
        dir.file_names(),
        ["settings.corrupt-1700000000.json"],
        "the user's file is kept"
    );
}

type Hotkeys = (Hotkey, Hotkey, Option<Hotkey>);

/// What loading `hotkeys_json` as the `hotkeys` object gives for the three hotkeys.
async fn hotkeys_after_loading(test: &str, hotkeys_json: &str) -> Hotkeys {
    let dir = TestDir::new(test);
    write_settings(&dir, &format!(r#"{{"hotkeys":{hotkeys_json}}}"#));
    let loaded = store(&dir).load().await.unwrap().unwrap();
    hotkeys_of(loaded.settings())
}

fn hotkeys_of(settings: &Settings) -> Hotkeys {
    (
        settings.switching().hotkey(),
        settings.behaviours().settings_hotkey(),
        settings.convert().quick_convert_hotkey(),
    )
}

#[tokio::test]
async fn an_empty_hotkey_falls_back_to_its_default() {
    let got = hotkeys_after_loading("empty_hotkey", r#"{"switch":{"modifiers":[],"key":null}}"#).await;

    assert_eq!(got, hotkeys_of(&Settings::default()));
}

#[tokio::test]
async fn a_hotkey_on_an_unknown_or_other_key_falls_back_to_its_default() {
    let unknown = hotkeys_after_loading("unknown_key", r#"{"settings":{"modifiers":["ctrl"],"key":"Banana"}}"#).await;
    let other = hotkeys_after_loading("other_key", r#"{"settings":{"modifiers":["ctrl"],"key":"Other"}}"#).await;

    assert_eq!(unknown, hotkeys_of(&Settings::default()));
    assert_eq!(other, hotkeys_of(&Settings::default()));
}

#[tokio::test]
async fn a_valid_hotkey_is_kept_beside_one_that_falls_back() {
    let got = hotkeys_after_loading(
        "partial",
        r#"{"switch":{"modifiers":["alt"],"key":"Z"},"settings":{"modifiers":[],"key":null}}"#,
    )
    .await;

    assert_eq!(got.0, hotkey(&[Modifier::Alt], Some(KeyCode::Z)));
    assert_eq!(got.1, hotkeys_of(&Settings::default()).1);
}

#[tokio::test]
async fn a_settings_hotkey_equal_to_the_switch_hotkey_falls_back() {
    let same = r#"{"modifiers":["alt"],"key":"Z"}"#;

    let got = hotkeys_after_loading("settings_clash", &format!(r#"{{"switch":{same},"settings":{same}}}"#)).await;

    assert_eq!(
        got.0,
        hotkey(&[Modifier::Alt], Some(KeyCode::Z)),
        "the switch hotkey stays"
    );
    assert_eq!(
        got.1,
        hotkeys_of(&Settings::default()).1,
        "the settings hotkey takes its default"
    );
}

#[tokio::test]
async fn a_quick_convert_hotkey_equal_to_another_is_dropped() {
    let default_settings = r#"{"modifiers":["ctrl","shift"],"key":"F5"}"#;

    let got = hotkeys_after_loading("quick_clash", &format!(r#"{{"quick_convert":{default_settings}}}"#)).await;

    assert_eq!(
        got,
        hotkeys_of(&Settings::default()),
        "quick convert has no default, so it is off"
    );
}

#[tokio::test]
async fn when_the_fallback_collides_every_hotkey_takes_its_default() {
    // The stored switch hotkey is the default settings hotkey, and the stored settings hotkey is the same again.
    let clash = r#"{"modifiers":["ctrl","shift"],"key":"F5"}"#;

    let got = hotkeys_after_loading("fallback_clash", &format!(r#"{{"switch":{clash},"settings":{clash}}}"#)).await;

    assert_eq!(got, hotkeys_of(&Settings::default()));
}
