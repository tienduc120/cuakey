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
fn a_chord_never_expands_a_macro() {
    let in_english = settings().with(SettingChange::MacrosInEnglish(OnOff::On));
    for language in [Language::Vietnamese, Language::English] {
        let mut composer = composer_for(&in_english, language, &[("ko", "không")]);
        type_text(&mut composer, "ko");

        let edit = composer.press(KeyPress::new(KeyCode::A, KeyMods::none().with_ctrl_or_alt(OnOff::On)));

        assert_eq!(
            edit,
            Edit::PassThrough,
            "{language:?}: Ctrl+A acts on the text the screen shows, so the shortcut stays"
        );
    }
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
fn enter_is_sent_after_the_expansion_not_typed_into_it() {
    let mut composer = telex_with(&[("ko", "không")]);
    type_text(&mut composer, "ko");

    let edit = press(&mut composer, KeyCode::Enter);

    assert_eq!(
        edit,
        Edit::Replace {
            delete: 2,
            insert: "không".into(),
            resend_key: true,
        },
        "Enter must still act as the key"
    );
}

#[test]
fn a_tab_ends_the_word_without_expanding_a_macro() {
    let in_english = settings().with(SettingChange::MacrosInEnglish(OnOff::On));
    for language in [Language::Vietnamese, Language::English] {
        let mut composer = composer_for(&in_english, language, &[("ko", "không")]);
        type_text(&mut composer, "ko");

        let edit = press(&mut composer, KeyCode::Tab);

        assert_eq!(
            edit,
            Edit::PassThrough,
            "{language:?}: Tab moves on and leaves the shortcut"
        );
        assert_eq!(
            type_text(&mut composer, " "),
            " ",
            "{language:?}: the Tab ended the word"
        );
    }
}

#[test]
fn a_shortcut_with_a_digit_expands_in_telex() {
    for settings in [settings(), Settings::default()] {
        let mut composer = composer_for(&settings, Language::Vietnamese, &[("vn1", "Việt Nam")]);

        assert_eq!(
            type_text(&mut composer, "vn1 "),
            "Việt Nam ",
            "a digit ends the Telex word, but the shortcut is the whole of vn1"
        );
    }
}

#[test]
fn backspace_takes_a_digit_back_out_of_the_shortcut() {
    let mut composer = telex_with(&[("vn1", "Việt Nam")]);
    let mut text = String::from("vn2");
    text.push(BACKSPACE);
    text.push_str("1 ");

    assert_eq!(type_text(&mut composer, &text), "Việt Nam ", "the 2 was deleted");
}

#[test]
fn a_digit_inside_a_shortcut_expands_nothing_yet() {
    let mut composer = telex_with(&[("vn", "Việt Nam"), ("vn1", "Việt Nam 1")]);

    assert_eq!(
        type_text(&mut composer, "vn1 "),
        "Việt Nam 1 ",
        "the digit goes on the shortcut, so vn alone does not fire"
    );
}

fn capitalising(match_caps: OnOff) -> Settings {
    settings()
        .with(SettingChange::CapitalizeSentences(OnOff::On))
        .with(SettingChange::MacroMatchCase(match_caps))
}

#[test]
fn a_macro_fires_on_the_first_word_of_a_sentence_the_engine_capitalised() {
    let mut composer = composer_for(&capitalising(OnOff::Off), Language::Vietnamese, &[("ko", "không")]);

    assert_eq!(
        type_text(&mut composer, "tooi. ko "),
        "tôi. Không ",
        "the engine raised the k, so the expansion is raised too"
    );
}

#[test]
fn a_raised_word_taken_up_again_after_a_space_is_still_raised() {
    let mut composer = composer_for(&capitalising(OnOff::Off), Language::Vietnamese, &[("ko", "không")]);

    assert_eq!(
        type_text(&mut composer, "tooi. kop \u{8}\u{8} "),
        "tôi. Không ",
        "Backspace took Kop up again and deleted its p; the engine had raised its k"
    );
}

#[test]
fn a_macro_fires_on_a_capitalised_first_word_with_match_caps_on() {
    let mut composer = composer_for(&capitalising(OnOff::On), Language::Vietnamese, &[("ko", "không")]);

    assert_eq!(type_text(&mut composer, "tooi. ko "), "tôi. Không ", "match caps on");
}

#[test]
fn a_capital_the_user_typed_still_needs_match_caps() {
    let mut composer = composer_for(&capitalising(OnOff::Off), Language::Vietnamese, &[("ko", "không")]);

    assert_eq!(
        type_text(&mut composer, "tooi Ko "),
        "tôi Ko ",
        "no match: the user typed the capital"
    );
}

#[test]
fn a_shortcut_that_matches_only_by_its_keys_fires_at_a_capitalised_sentence_start() {
    let mut composer = composer_for(
        &capitalising(OnOff::Off),
        Language::Vietnamese,
        &[("btw", "by the way")],
    );

    assert_eq!(
        type_text(&mut composer, "tooi. btw "),
        "tôi. By the way ",
        "w became ư on screen; the keys were btw"
    );
}

#[test]
fn a_shortcut_never_joins_text_typed_before_enter() {
    let mut digits = telex_with(&[("t1w", "x")]);
    assert_eq!(
        type_text(&mut digits, "t1\n\u{8}w "),
        "t1w ",
        "the t1 typed before Enter is no part of the shortcut typed after it"
    );
    let mut letters = telex_with(&[("btw", "by the way")]);
    assert_eq!(type_text(&mut letters, "bt\n\u{8}w "), "btw ", "nor is bt");
}
