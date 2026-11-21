use alloc::string::String;

use super::Composer;
use super::{BACKSPACE, composer_with, settings, type_text};
use crate::{
    ComposerSettings, Edit, KeyCode, KeyMods, KeyPress, Language, MacroEntry, MacroTable, OnOff, SettingChange,
    Settings,
};

fn table(pairs: &[(&str, &str)]) -> MacroTable {
    let mut table = MacroTable::default();
    for (shortcut, expansion) in pairs {
        table.add(MacroEntry::new(shortcut, expansion).unwrap());
    }
    table
}

fn composer_for(settings: &Settings, language: Language, pairs: &[(&str, &str)]) -> Composer {
    let mut composer = Composer::new(ComposerSettings::from_settings(settings, language));
    composer.set_macros(table(pairs));
    composer
}

fn telex_with(pairs: &[(&str, &str)]) -> Composer {
    composer_for(&settings(), Language::Vietnamese, pairs)
}

fn press(composer: &mut Composer, code: KeyCode) -> Edit {
    composer.press(KeyPress::new(code, KeyMods::none()))
}

#[test]
fn macro_expands_on_space() {
    let mut composer = telex_with(&[("ko", "không")]);

    assert_eq!(
        type_text(&mut composer, "ko "),
        "không ",
        "the shortcut becomes the expansion"
    );
}

#[test]
fn the_edit_deletes_the_word_and_types_the_expansion_and_the_space() {
    let mut composer = telex_with(&[("ko", "không")]);
    type_text(&mut composer, "ko");

    let edit = press(&mut composer, KeyCode::Space);

    assert_eq!(
        edit,
        Edit::Replace {
            delete: 2,
            insert: "không ".into(),
            resend_key: false,
        },
        "the break character is typed with the expansion"
    );
}

#[test]
fn a_break_key_with_no_text_is_sent_after_the_expansion() {
    let mut composer = telex_with(&[("ko", "không")]);
    type_text(&mut composer, "ko");

    let edit = press(&mut composer, KeyCode::Escape);

    assert_eq!(
        edit,
        Edit::Replace {
            delete: 2,
            insert: "không".into(),
            resend_key: true,
        },
        "the key follows the replacement"
    );
}

#[test]
fn macro_expands_on_punctuation_and_enter() {
    let mut composer = telex_with(&[("ko", "không")]);

    assert_eq!(type_text(&mut composer, "ko,"), "không,", "a comma");
    assert_eq!(type_text(&mut composer, "ko\n"), "không\n", "Enter");
}

#[test]
fn a_shifted_break_key_types_its_shifted_character() {
    let mut composer = telex_with(&[("ko", "không")]);
    type_text(&mut composer, "ko");

    let edit = composer.press(KeyPress::new(KeyCode::Slash, KeyMods::none().with_shift(OnOff::On)));

    assert_eq!(
        edit,
        Edit::Replace {
            delete: 2,
            insert: "không?".into(),
            resend_key: false,
        },
        "Shift+/ is a question mark"
    );
}

#[test]
fn a_chord_ends_the_word_and_is_sent_after_the_expansion() {
    let mut composer = telex_with(&[("ko", "không")]);
    type_text(&mut composer, "ko");

    let edit = composer.press(KeyPress::new(KeyCode::A, KeyMods::none().with_ctrl_or_alt(OnOff::On)));

    assert_eq!(
        edit,
        Edit::Replace {
            delete: 2,
            insert: "không".into(),
            resend_key: true,
        },
        "Ctrl+A goes on to the app"
    );
}

#[test]
fn the_word_is_matched_as_the_screen_shows_it() {
    let mut composer = telex_with(&[("tiếng", "tiếng Việt")]);

    assert_eq!(
        type_text(&mut composer, "tieengs "),
        "tiếng Việt ",
        "the typed keys make the shortcut"
    );
}

#[test]
fn macro_wins_over_restoring_an_invalid_word() {
    let spelled = settings().with(SettingChange::SpellCheck(OnOff::On));
    let without = composer_for(&spelled, Language::Vietnamese, &[]);
    let mut with = composer_for(&spelled, Language::Vietnamese, &[("bcdz", "đúng")]);
    let mut without = without;

    assert_eq!(
        type_text(&mut without, "bcdz "),
        "bcdz ",
        "restore leaves the keys as typed"
    );
    assert_eq!(type_text(&mut with, "bcdz "), "đúng ", "the macro is looked up first");
}

#[test]
fn a_word_that_is_no_macro_is_left_alone() {
    let mut composer = telex_with(&[("ko", "không")]);

    assert_eq!(type_text(&mut composer, "kho "), "kho ", "another word");
    assert_eq!(type_text(&mut composer, "k "), "k ", "a prefix of a shortcut");
}

#[test]
fn a_space_with_no_word_expands_nothing() {
    let mut composer = telex_with(&[("ko", "không")]);

    assert_eq!(
        press(&mut composer, KeyCode::Space),
        Edit::PassThrough,
        "no word, no macro"
    );
}

#[test]
fn macros_off_expands_nothing() {
    let settings = settings().with(SettingChange::UseMacros(OnOff::Off));
    let mut composer = composer_for(&settings, Language::Vietnamese, &[("ko", "không")]);

    assert_eq!(type_text(&mut composer, "ko "), "ko ", "the switch is off");
}

#[test]
fn macro_match_caps() {
    let settings = settings().with(SettingChange::MacroMatchCase(OnOff::On));
    let mut composer = composer_for(&settings, Language::Vietnamese, &[("btw", "by the way")]);

    assert_eq!(type_text(&mut composer, "btw "), "by the way ", "lower case");
    assert_eq!(type_text(&mut composer, "Btw "), "By the way ", "Capitalised");
    assert_eq!(type_text(&mut composer, "BTW "), "BY THE WAY ", "ALL CAPS");
}

#[test]
fn without_match_caps_a_capital_is_no_match() {
    let mut composer = composer_for(&Settings::default(), Language::Vietnamese, &[("btw", "by the way")]);

    assert_eq!(type_text(&mut composer, "Btw "), "Btw ", "the case must match");
}

#[test]
fn macro_in_english_mode() {
    let settings = settings().with(SettingChange::MacrosInEnglish(OnOff::On));
    let mut composer = composer_for(&settings, Language::English, &[("ko", "không")]);

    assert_eq!(type_text(&mut composer, "ko "), "không ", "the option is on");
}

#[test]
fn english_mode_is_pass_through_unless_macros_in_english_is_on() {
    let mut composer = composer_for(&settings(), Language::English, &[("ko", "không")]);

    assert_eq!(type_text(&mut composer, "ko "), "ko ", "the option is off");
    assert_eq!(
        press(&mut composer, KeyCode::A),
        Edit::PassThrough,
        "a key goes through"
    );
}

#[test]
fn english_mode_with_macros_never_composes_vietnamese() {
    let settings = settings().with(SettingChange::MacrosInEnglish(OnOff::On));
    let mut composer = composer_for(&settings, Language::English, &[("ko", "không")]);

    assert_eq!(type_text(&mut composer, "tieengs "), "tieengs ", "no tones or marks");
}

#[test]
fn english_mode_follows_backspace_inside_the_word() {
    let settings = settings().with(SettingChange::MacrosInEnglish(OnOff::On));
    let mut composer = composer_for(&settings, Language::English, &[("ko", "không")]);

    let mut text = String::from("kx");
    text.push(BACKSPACE);
    text.push_str("o ");

    assert_eq!(
        type_text(&mut composer, &text),
        "không ",
        "the deleted letter is not part of the word"
    );
}

#[test]
fn english_mode_macro_matches_caps_too() {
    let settings = settings()
        .with(SettingChange::MacrosInEnglish(OnOff::On))
        .with(SettingChange::MacroMatchCase(OnOff::On));
    let mut composer = composer_for(&settings, Language::English, &[("btw", "by the way")]);

    assert_eq!(type_text(&mut composer, "BTW "), "BY THE WAY ", "ALL CAPS");
}

#[test]
fn backspace_after_an_expansion_does_not_take_the_shortcut_back_up() {
    let mut composer = telex_with(&[("ko", "không")]);
    type_text(&mut composer, "ko ");

    let edit = press(&mut composer, KeyCode::Backspace);

    assert_eq!(edit, Edit::PassThrough, "it deletes the space on screen");
    assert_eq!(composer.word(), "", "the screen shows the expansion, not the shortcut");
    assert_eq!(type_text(&mut composer, "s"), "s", "a tone key finds no word to change");
}

#[test]
fn backspace_after_an_expansion_in_english_mode_does_not_take_the_shortcut_back_up() {
    let settings = settings().with(SettingChange::MacrosInEnglish(OnOff::On));
    let mut composer = composer_for(&settings, Language::English, &[("ko", "không")]);
    type_text(&mut composer, "ko ");

    press(&mut composer, KeyCode::Backspace);

    assert_eq!(composer.word(), "", "nothing to take up");
}

#[test]
fn backspace_after_a_space_that_expanded_nothing_still_takes_the_word_up() {
    let mut composer = telex_with(&[("ko", "không")]);
    type_text(&mut composer, "vieet ");

    press(&mut composer, KeyCode::Backspace);

    assert_eq!(composer.word(), "viêt", "the ended word is back");
}

#[test]
fn set_macros_replaces_the_table() {
    let mut composer = telex_with(&[("ko", "không")]);

    composer.set_macros(table(&[("dc", "được")]));

    assert_eq!(type_text(&mut composer, "ko "), "ko ", "the old macro is gone");
    assert_eq!(type_text(&mut composer, "dc "), "được ", "the new one works");
}

#[test]
fn a_new_composer_has_no_macros() {
    let mut composer = composer_with(&settings());

    assert_eq!(type_text(&mut composer, "ko "), "ko ", "no table, no expansion");
}

#[test]
fn a_shortcut_matches_the_keys_as_typed_with_spell_check_on() {
    let mut composer = composer_for(&Settings::default(), Language::Vietnamese, &[("btw", "by the way")]);

    assert_eq!(
        type_text(&mut composer, "btw "),
        "by the way ",
        "w became ư on screen, the keys were btw"
    );
}

#[test]
fn a_shortcut_matches_the_keys_as_typed_with_spell_check_off() {
    let mut composer = telex_with(&[("btw", "by the way")]);

    assert_eq!(type_text(&mut composer, "btw "), "by the way ", "spell check off");
}

#[test]
fn match_caps_applies_to_the_keys_as_typed() {
    let settings = Settings::default().with(SettingChange::MacroMatchCase(OnOff::On));
    let mut composer = composer_for(&settings, Language::Vietnamese, &[("btw", "by the way")]);

    assert_eq!(type_text(&mut composer, "Btw "), "By the way ", "Capitalised");
    assert_eq!(type_text(&mut composer, "BTW "), "BY THE WAY ", "ALL CAPS");
}

#[test]
fn a_shortcut_in_screen_form_still_matches() {
    let mut composer = composer_for(&Settings::default(), Language::Vietnamese, &[("đc", "được")]);

    assert_eq!(type_text(&mut composer, "ddc "), "được ", "dd typed đ");
}

#[test]
fn the_edit_deletes_the_word_as_shown_when_the_keys_matched() {
    let mut composer = telex_with(&[("btw", "by the way")]);
    type_text(&mut composer, "btw");

    let edit = press(&mut composer, KeyCode::Space);

    assert_eq!(
        edit,
        Edit::Replace {
            delete: 3,
            insert: "by the way ".into(),
            resend_key: false,
        },
        "the screen shows btư, three characters"
    );
}

#[test]
fn enter_and_tab_are_sent_after_the_expansion_not_typed_into_it() {
    for code in [KeyCode::Enter, KeyCode::Tab] {
        let mut composer = telex_with(&[("ko", "không")]);
        type_text(&mut composer, "ko");

        let edit = press(&mut composer, code);

        assert_eq!(
            edit,
            Edit::Replace {
                delete: 2,
                insert: "không".into(),
                resend_key: true,
            },
            "{code:?} must still act as the key"
        );
    }
}
