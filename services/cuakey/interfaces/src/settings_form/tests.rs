use cuakey_application::{
    AppMemoryView, CodeTableValue, HotkeyView, InputMethodValue, InterfaceLanguageValue, KeyName, LanguageValue,
    PreferencesView, ThemeValue, Toggle,
};

use super::{ButtonBar, GatedControl, SettingsForm};
use crate::{Button, Platform};

const WINDOWS_ONLY: [GatedControl; 6] = [
    GatedControl::FixAutocomplete,
    GatedControl::ChromiumFix,
    GatedControl::DesktopShortcut,
    GatedControl::RunAsAdministrator,
    GatedControl::StoreAppSupport,
    GatedControl::ClipboardInput,
];

fn flip(toggle: Toggle) -> Toggle {
    match toggle {
        Toggle::On => Toggle::Off,
        Toggle::Off => Toggle::On,
    }
}

fn hotkey(key: KeyName) -> HotkeyView {
    HotkeyView {
        ctrl: Toggle::Off,
        alt: Toggle::On,
        win: Toggle::On,
        shift: Toggle::Off,
        key: Some(key),
    }
}

/// The base with every option changed.
fn flipped(base: &PreferencesView) -> PreferencesView {
    let mut view = base.clone();
    view.language = LanguageValue::English;
    view.typing.input_method = InputMethodValue::SimpleTelex;
    view.typing.code_table = CodeTableValue::Cp1258;
    view.typing.new_tone_style = flip(base.typing.new_tone_style);
    view.typing.spell_check = flip(base.typing.spell_check);
    view.typing.restore_invalid_word = flip(base.typing.restore_invalid_word);
    view.typing.allow_zwjf_consonants = flip(base.typing.allow_zwjf_consonants);
    view.typing.capitalize_sentences = flip(base.typing.capitalize_sentences);
    view.typing.quick_telex = flip(base.typing.quick_telex);
    view.typing.quick_first_consonant = flip(base.typing.quick_first_consonant);
    view.typing.quick_last_consonant = flip(base.typing.quick_last_consonant);
    view.macros.use_macros = flip(base.macros.use_macros);
    view.macros.macros_in_english = flip(base.macros.macros_in_english);
    view.macros.match_case = flip(base.macros.match_case);
    view.switching.hotkey = hotkey(KeyName::F9);
    view.switching.beep = flip(base.switching.beep);
    view.switching.smart_switch = flip(base.switching.smart_switch);
    view.switching.remember_code_table = flip(base.switching.remember_code_table);
    view.switching.work_in_other_languages = flip(base.switching.work_in_other_languages);
    view.compatibility.fix_autocomplete = flip(base.compatibility.fix_autocomplete);
    view.compatibility.chromium_fix = flip(base.compatibility.chromium_fix);
    view.compatibility.clipboard_input = flip(base.compatibility.clipboard_input);
    view.compatibility.store_app_support = flip(base.compatibility.store_app_support);
    view.system.start_with_windows = flip(base.system.start_with_windows);
    view.system.show_settings_at_startup = flip(base.system.show_settings_at_startup);
    view.system.run_as_administrator = flip(base.system.run_as_administrator);
    view.system.desktop_shortcut = flip(base.system.desktop_shortcut);
    view.system.interface_language = InterfaceLanguageValue::Vietnamese;
    view.system.theme = ThemeValue::Light;
    view.behaviours.settings_hotkey = hotkey(KeyName::F10);
    view.behaviours.keep_w = flip(base.behaviours.keep_w);
    view.behaviours.only_text_boxes = flip(base.behaviours.only_text_boxes);
    view
}

/// Drives each control with the value `target` holds.
fn apply_every_control(form: &mut SettingsForm, target: &PreferencesView) {
    let on = |toggle: Toggle| toggle == Toggle::On;
    form.set_language(target.language);
    form.set_input_method(target.typing.input_method);
    form.set_code_table(target.typing.code_table);
    form.set_new_tone_style(on(target.typing.new_tone_style));
    form.set_spell_check(on(target.typing.spell_check));
    form.set_restore_invalid_word(on(target.typing.restore_invalid_word));
    form.set_allow_zwjf_consonants(on(target.typing.allow_zwjf_consonants));
    form.set_capitalize_sentences(on(target.typing.capitalize_sentences));
    form.set_quick_telex(on(target.typing.quick_telex));
    form.set_quick_first_consonant(on(target.typing.quick_first_consonant));
    form.set_quick_last_consonant(on(target.typing.quick_last_consonant));
    form.set_use_macros(on(target.macros.use_macros));
    form.set_macros_in_english(on(target.macros.macros_in_english));
    form.set_macro_match_case(on(target.macros.match_case));
    form.set_switch_hotkey(target.switching.hotkey);
    form.set_beep(on(target.switching.beep));
    form.set_smart_switch(on(target.switching.smart_switch));
    form.set_remember_code_table(on(target.switching.remember_code_table));
    form.set_work_in_other_languages(on(target.switching.work_in_other_languages));
    form.set_fix_autocomplete(on(target.compatibility.fix_autocomplete));
    form.set_chromium_fix(on(target.compatibility.chromium_fix));
    form.set_clipboard_input(on(target.compatibility.clipboard_input));
    form.set_store_app_support(on(target.compatibility.store_app_support));
    form.set_start_with_windows(on(target.system.start_with_windows));
    form.set_show_settings_at_startup(on(target.system.show_settings_at_startup));
    form.set_run_as_administrator(on(target.system.run_as_administrator));
    form.set_desktop_shortcut(on(target.system.desktop_shortcut));
    form.set_interface_language(target.system.interface_language);
    if form.view().system.theme != target.system.theme {
        form.toggle_theme();
    }
    form.set_settings_hotkey(target.behaviours.settings_hotkey);
    form.set_keep_w(on(target.behaviours.keep_w));
    form.set_only_text_boxes(on(target.behaviours.only_text_boxes));
}

#[test]
fn every_control_round_trips() {
    let base = PreferencesView::default();
    let target = flipped(&base);
    assert_ne!(target, base);
    let mut form = SettingsForm::new(base, Platform::Windows);
    apply_every_control(&mut form, &target);
    assert_eq!(form.take_save(), Some(target));
}

#[test]
fn switching_the_theme_saves_the_other_one() {
    let mut form = SettingsForm::new(PreferencesView::default(), Platform::Windows);

    form.toggle_theme();
    let first = form.take_save().map(|view| view.system.theme);
    form.toggle_theme();
    let second = form.take_save().map(|view| view.system.theme);

    assert_eq!(first, Some(ThemeValue::Light), "a dark window switches to light");
    assert_eq!(second, Some(ThemeValue::Dark), "and back to dark");
}

#[test]
fn nothing_to_save_until_a_control_changes() {
    let mut form = SettingsForm::new(PreferencesView::default(), Platform::Windows);
    assert_eq!(form.take_save(), None);
    form.set_beep(true);
    assert!(form.take_save().is_some());
    assert_eq!(form.take_save(), None);
}

#[test]
fn windows_only_hidden_elsewhere() {
    let windows = SettingsForm::new(PreferencesView::default(), Platform::Windows);
    let other = SettingsForm::new(PreferencesView::default(), Platform::Other);
    for control in WINDOWS_ONLY {
        assert!(windows.is_visible(control), "{control:?} on Windows");
        assert!(!other.is_visible(control), "{control:?} elsewhere");
    }
    assert!(other.is_visible(GatedControl::RestoreInvalidWord));
    assert!(other.is_visible(GatedControl::AllowZwjfConsonants));
}

#[test]
fn the_spelling_options_follow_spell_check_and_keep_their_value() {
    let mut view = PreferencesView::default();
    view.typing.restore_invalid_word = Toggle::On;
    view.typing.allow_zwjf_consonants = Toggle::On;
    view.typing.spell_check = Toggle::On;
    let mut form = SettingsForm::new(view, Platform::Windows);
    assert!(form.is_enabled(GatedControl::RestoreInvalidWord));
    assert!(form.is_enabled(GatedControl::AllowZwjfConsonants));

    form.set_spell_check(false);
    assert!(!form.is_enabled(GatedControl::RestoreInvalidWord));
    assert!(!form.is_enabled(GatedControl::AllowZwjfConsonants));
    assert_eq!(form.view().typing.restore_invalid_word, Toggle::On);
    assert_eq!(form.view().typing.allow_zwjf_consonants, Toggle::On);
}

#[test]
fn the_chromium_fix_follows_the_suggestion_fix_and_keeps_its_value() {
    let mut view = PreferencesView::default();
    view.compatibility.fix_autocomplete = Toggle::On;
    view.compatibility.chromium_fix = Toggle::On;
    let mut form = SettingsForm::new(view, Platform::Windows);
    assert!(form.is_enabled(GatedControl::ChromiumFix));

    form.set_fix_autocomplete(false);
    assert!(!form.is_enabled(GatedControl::ChromiumFix));
    assert_eq!(form.view().compatibility.chromium_fix, Toggle::On);
}

#[test]
fn save_carries_always_on_and_app_memory_unchanged() {
    let view = PreferencesView {
        always_on: vec!["game.exe".to_owned()],
        app_memory: vec![AppMemoryView {
            app: "chrome.exe".to_owned(),
            language: LanguageValue::English,
            code_table: CodeTableValue::Tcvn3,
        }],
        ..PreferencesView::default()
    };
    let mut form = SettingsForm::new(view.clone(), Platform::Windows);
    form.set_beep(true);
    let saved = form.take_save().unwrap();
    assert_eq!(saved.always_on, view.always_on);
    assert_eq!(saved.app_memory, view.app_memory);
}

#[test]
fn set_preferences_replaces_the_view_without_saving() {
    let mut form = SettingsForm::new(PreferencesView::default(), Platform::Windows);
    let target = flipped(&PreferencesView::default());
    form.set_preferences(target.clone());
    assert_eq!(form.view(), &target);
    assert_eq!(form.take_save(), None);
}

#[test]
fn an_edit_stays_after_a_refused_save() {
    let mut form = SettingsForm::new(PreferencesView::default(), Platform::Windows);
    form.set_switch_hotkey(hotkey(KeyName::F9));
    drop(form.take_save());
    form.show_error(Some("Hai phím tắt không được trùng nhau"));
    assert_eq!(form.error(), Some("Hai phím tắt không được trùng nhau"));
    assert_eq!(form.view().switching.hotkey, hotkey(KeyName::F9));
    form.show_error(None);
    assert_eq!(form.error(), None);
}

#[test]
fn opens_compact() {
    let form = SettingsForm::new(PreferencesView::default(), Platform::Windows);
    assert!(
        !form.is_expanded(),
        "the window opens with the controls used every day only"
    );
}

#[test]
fn toggle_expands_and_collapses() {
    let mut form = SettingsForm::new(PreferencesView::default(), Platform::Windows);

    form.toggle_expanded();
    assert!(form.is_expanded());
    assert_eq!(
        Button::ToggleExpanded.action(),
        None,
        "the window resizes itself and asks the process nothing"
    );
    assert_eq!(form.take_save(), None, "showing more of the window changes no setting");

    form.toggle_expanded();
    assert!(!form.is_expanded(), "a second toggle collapses it again");
    assert_eq!(form.take_save(), None);
}

/// The right end of the bottom bar, left to right, in either mode.
const BAR_RIGHT: [Button; 3] = [Button::ToggleExpanded, Button::ExitApp, Button::CloseWindow];

#[test]
fn bar_in_compact_mode() {
    let form = SettingsForm::new(PreferencesView::default(), Platform::Windows);
    assert_eq!(
        form.bar(),
        ButtonBar {
            left: None,
            right: BAR_RIGHT,
        },
        "compact: nothing at the left end"
    );
}

#[test]
fn bar_in_expanded_mode() {
    let mut form = SettingsForm::new(PreferencesView::default(), Platform::Windows);
    form.toggle_expanded();
    assert_eq!(
        form.bar(),
        ButtonBar {
            left: Some(Button::ResetToDefaults),
            right: BAR_RIGHT,
        },
        "expanded: the reset at the left end, away from the buttons clicked every day"
    );
}
