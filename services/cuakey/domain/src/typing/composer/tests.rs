use alloc::string::String;

use super::Composer;
use crate::{
    ComposerSettings, Edit, InputMethod, KeyCode, KeyMods, KeyPress, Language, OnOff, SettingChange, Settings,
};

const LETTER_KEYS: [KeyCode; 26] = [
    KeyCode::A,
    KeyCode::B,
    KeyCode::C,
    KeyCode::D,
    KeyCode::E,
    KeyCode::F,
    KeyCode::G,
    KeyCode::H,
    KeyCode::I,
    KeyCode::J,
    KeyCode::K,
    KeyCode::L,
    KeyCode::M,
    KeyCode::N,
    KeyCode::O,
    KeyCode::P,
    KeyCode::Q,
    KeyCode::R,
    KeyCode::S,
    KeyCode::T,
    KeyCode::U,
    KeyCode::V,
    KeyCode::W,
    KeyCode::X,
    KeyCode::Y,
    KeyCode::Z,
];

const DIGIT_KEYS: [KeyCode; 10] = [
    KeyCode::Digit0,
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
    KeyCode::Digit9,
];

/// Backspace, written into a typed string.
const BACKSPACE: char = '\u{8}';

/// The settings with spell check off, so the engine is tested on its own.
fn settings() -> Settings {
    Settings::default().with(SettingChange::SpellCheck(OnOff::Off))
}

fn composer_with(settings: &Settings) -> Composer {
    Composer::new(ComposerSettings::from_settings(settings, Language::Vietnamese))
}

fn telex() -> Composer {
    composer_with(&settings())
}

fn simple_telex() -> Composer {
    composer_with(&settings().with(SettingChange::InputMethod(InputMethod::SimpleTelex)))
}

fn vni() -> Composer {
    composer_with(&settings().with(SettingChange::InputMethod(InputMethod::Vni)))
}

/// The US-layout key that types `c`, and whether it takes Shift.
fn key_for(c: char) -> (KeyCode, OnOff) {
    let shift = if c.is_ascii_uppercase() { OnOff::On } else { OnOff::Off };
    let letter = ('a'..='z')
        .zip(LETTER_KEYS)
        .find(|(l, _)| *l == c.to_ascii_lowercase())
        .map(|(_, key)| (key, shift));
    let digit = ('0'..='9')
        .zip(DIGIT_KEYS)
        .find(|(d, _)| *d == c)
        .map(|(_, key)| (key, OnOff::Off));
    let symbol = match c {
        ' ' => Some((KeyCode::Space, OnOff::Off)),
        BACKSPACE => Some((KeyCode::Backspace, OnOff::Off)),
        '\n' => Some((KeyCode::Enter, OnOff::Off)),
        '\t' => Some((KeyCode::Tab, OnOff::Off)),
        ',' => Some((KeyCode::Comma, OnOff::Off)),
        '.' => Some((KeyCode::Period, OnOff::Off)),
        '[' => Some((KeyCode::LeftBracket, OnOff::Off)),
        ']' => Some((KeyCode::RightBracket, OnOff::Off)),
        '!' => Some((KeyCode::Digit1, OnOff::On)),
        _ => None,
    };
    letter
        .or(digit)
        .or(symbol)
        .unwrap_or_else(|| panic!("no key types {c:?}"))
}

/// What the key for `c` types when let through: Caps Lock turns a letter's case around.
fn typed_by_key(c: char, caps_lock: OnOff) -> char {
    match (c.is_ascii_alphabetic(), caps_lock) {
        (true, OnOff::On) if c.is_ascii_uppercase() => c.to_ascii_lowercase(),
        (true, OnOff::On) => c.to_ascii_uppercase(),
        _ => c,
    }
}

/// Applies an edit to the screen as the app being typed into would.
fn apply(edit: Edit, screen: &mut String, typed: Option<char>) {
    match edit {
        Edit::PassThrough => match typed {
            Some(c) => screen.push(c),
            None => {
                screen.pop();
            }
        },
        Edit::Replace {
            delete,
            insert,
            resend_key,
        } => {
            for _ in 0..delete {
                screen.pop();
            }
            screen.push_str(&insert);
            assert!(!resend_key, "no key typed in these tests is sent again");
        }
    }
}

/// Presses the key for `c` and applies the edit to `screen`.
fn type_key(composer: &mut Composer, screen: &mut String, c: char, caps_lock: OnOff) {
    let (code, shift) = key_for(c);
    let mods = KeyMods::none().with_shift(shift).with_caps_lock(caps_lock);
    let typed = (c != BACKSPACE).then(|| typed_by_key(c, caps_lock));
    apply(composer.press(KeyPress::new(code, mods)), screen, typed);
}

fn type_keys(composer: &mut Composer, text: &str, caps_lock: OnOff) -> String {
    let mut screen = String::new();
    for c in text.chars() {
        type_key(composer, &mut screen, c, caps_lock);
    }
    screen
}

/// Types `text` key by key on a US layout, Shift for capitals, and returns what the screen shows.
fn type_text(composer: &mut Composer, text: &str) -> String {
    type_keys(composer, text, OnOff::Off)
}

#[test]
fn telex_basic() {
    assert_eq!(type_text(&mut telex(), "tieengs"), "tiếng", "ee makes ê, s the acute");
}

#[test]
fn telex_viet() {
    assert_eq!(type_text(&mut telex(), "vieetj"), "việt", "j the dot below");
}

#[test]
fn telex_duoc() {
    assert_eq!(
        type_text(&mut telex(), "dduowcj"),
        "được",
        "dd makes đ, w on uo makes ươ"
    );
}

#[test]
fn telex_nguoi() {
    assert_eq!(type_text(&mut telex(), "nguowif"), "người", "the tone goes on ơ");
}

#[test]
fn telex_sentence() {
    // Split, or the spelling check reads the second word of the country's name as a misspelt English word.
    let typed = concat!("Vieetj N", "am");
    let shown = concat!("Việt N", "am");

    assert_eq!(type_text(&mut telex(), typed), shown, "space ends the word");
}

#[test]
fn old_tone_placement() {
    assert_eq!(type_text(&mut telex(), "hoaf"), "hòa", "old style marks the o of oa");
}

#[test]
fn new_tone_placement() {
    let mut composer = composer_with(&settings().with(SettingChange::NewToneStyle(OnOff::On)));

    assert_eq!(type_text(&mut composer, "hoaf"), "hoà", "new style marks the a of oa");
}

#[test]
fn lone_w_telex() {
    assert_eq!(
        type_text(&mut telex(), "nhw"),
        "như",
        "a w with no vowel to mark types ư"
    );
}

#[test]
fn lone_w_simple_telex() {
    assert_eq!(
        type_text(&mut simple_telex(), "nhw"),
        "nhw",
        "Simple Telex keeps a lone w"
    );
}

#[test]
fn uw_simple_telex() {
    assert_eq!(
        type_text(&mut simple_telex(), "nhuw"),
        "như",
        "Simple Telex still marks u with w"
    );
}

#[test]
fn double_key_undoes() {
    assert_eq!(
        type_text(&mut telex(), "aaa"),
        "aa",
        "a third a takes the circumflex back"
    );
}

#[test]
fn z_removes_tone() {
    assert_eq!(type_text(&mut telex(), "asz"), "a", "z takes the tone off");
}

#[test]
fn tone_after_final_consonant() {
    assert_eq!(
        type_text(&mut telex(), "toans"),
        "toán",
        "a tone typed last goes on its vowel"
    );
}

#[test]
fn vni_basic() {
    assert_eq!(
        type_text(&mut vni(), "tie61ng"),
        "tiếng",
        "6 the circumflex, 1 the acute"
    );
}

#[test]
fn vni_viet() {
    assert_eq!(type_text(&mut vni(), "vie65t"), "việt", "5 the dot below");
}

#[test]
fn vni_duoc() {
    assert_eq!(type_text(&mut vni(), "d9u7o7c5"), "được", "9 makes đ, 7 the horn");
}

#[test]
fn caps_lock_and_shift() {
    assert_eq!(
        type_keys(&mut telex(), "vieetj", OnOff::On),
        "VIỆT",
        "Caps Lock types the whole word in capitals"
    );
    assert_eq!(
        type_text(&mut telex(), "Vieetj"),
        "Việt",
        "Shift on the first letter only"
    );
}

#[test]
fn english_mode_passes() {
    let mut composer = Composer::new(ComposerSettings::from_settings(&settings(), Language::English));

    assert_eq!(type_text(&mut composer, "aa"), "aa", "English mode changes nothing");
}

#[test]
fn screen_matches_word_after_every_key() {
    let mut composer = telex();
    let mut screen = String::new();

    for c in "vieet\u{8}js\u{8}\u{8}eej".chars() {
        type_key(&mut composer, &mut screen, c, OnOff::Off);
        assert_eq!(screen, composer.word(), "after {c:?} the word is what the screen shows");
    }
    assert_eq!(screen, "vệ", "the word typed again after deleting");
}

#[test]
fn a_tone_typed_early_moves_as_the_word_grows() {
    assert_eq!(type_text(&mut telex(), "tosan"), "toán", "the tone moves from o to a");
    assert_eq!(
        type_text(&mut vni(), "ho2a"),
        "hòa",
        "the tone stays on o of an open oa"
    );
}

#[test]
fn typing_a_tone_again_takes_it_back() {
    assert_eq!(
        type_text(&mut telex(), "ass"),
        "as",
        "the second s is typed as a letter"
    );
    assert_eq!(type_text(&mut vni(), "a11"), "a1", "the second 1 is typed as a digit");
}

#[test]
fn after_a_change_is_taken_back_the_word_is_typed_as_it_is() {
    assert_eq!(
        type_text(&mut telex(), "aaaa"),
        "aaa",
        "no circumflex after taking it back"
    );
    assert_eq!(type_text(&mut telex(), "ddd"), "dd", "a third d takes the stroke back");
}

#[test]
fn w_again_takes_back_the_vowel_it_typed() {
    assert_eq!(type_text(&mut telex(), "nhww"), "nhw", "the second w takes ư back");
    assert_eq!(type_text(&mut telex(), "uww"), "uw", "the second w takes the horn back");
}

#[test]
fn a_mark_can_change_into_another() {
    assert_eq!(type_text(&mut telex(), "aaw"), "ă", "w turns â into ă");
    assert_eq!(type_text(&mut telex(), "owo"), "ô", "o turns ơ into ô");
}

#[test]
fn w_marks_the_vowel_that_carries_it() {
    assert_eq!(type_text(&mut telex(), "hoawcj"), "hoặc", "the a of oa");
    assert_eq!(type_text(&mut telex(), "muaw"), "mưa", "the u of ua");
    assert_eq!(type_text(&mut telex(), "quaws"), "quắ", "the a after qu");
}

#[test]
fn brackets_type_horned_vowels_in_telex_only() {
    assert_eq!(type_text(&mut telex(), "t]"), "tư", "] types ư");
    assert_eq!(type_text(&mut telex(), "m[f"), "mờ", "[ types ơ");
    assert_eq!(type_text(&mut telex(), "[["), "[", "a second [ takes ơ back");
    assert_eq!(
        type_text(&mut simple_telex(), "t]"),
        "t]",
        "Simple Telex types the bracket"
    );
}

#[test]
fn capitals_keep_their_case_when_marked() {
    assert_eq!(
        type_text(&mut telex(), "Dduowcj"),
        "Được",
        "the stroke keeps the capital D"
    );
    assert_eq!(type_text(&mut telex(), "VIEETJ"), "VIỆT", "Shift on every letter");
}

#[test]
fn a_vowel_after_the_final_consonant_stops_the_marks() {
    assert_eq!(type_text(&mut telex(), "hellos"), "hellos", "not a Vietnamese syllable");
}

#[test]
fn vni_digits_with_nothing_to_mark_are_typed() {
    assert_eq!(type_text(&mut vni(), "1a"), "1a", "a tone with no vowel");
    assert_eq!(type_text(&mut vni(), "a10"), "a", "0 takes the tone off");
    assert_eq!(type_text(&mut vni(), "a!"), "a!", "a shifted digit is a symbol");
}

#[test]
fn keys_that_type_no_letter_end_the_word() {
    for text in ["vie j", "vie,j", "vie.j", "vie\nj", "vie\tj"] {
        let mut composer = telex();

        let screen = type_text(&mut composer, text);

        assert!(
            screen.ends_with('j'),
            "{text:?}: the tone key starts a new word, got {screen:?}"
        );
        assert_eq!(
            composer.word(),
            "j",
            "{text:?}: the word is only what came after the break"
        );
    }
}

#[test]
fn a_chord_or_a_key_with_no_text_ends_the_word() {
    let chord = KeyPress::new(KeyCode::A, KeyMods::none().with_ctrl_or_alt(OnOff::On));
    for key in [
        chord,
        KeyPress::new(KeyCode::Escape, KeyMods::none()),
        KeyPress::new(KeyCode::Other, KeyMods::none()),
    ] {
        let mut composer = telex();
        type_text(&mut composer, "vie");

        let edit = composer.press(key);

        assert_eq!(edit, Edit::PassThrough, "{key:?} goes through");
        assert_eq!(composer.word(), "", "{key:?} ends the word");
    }
}

#[test]
fn breaking_the_word_starts_a_new_one() {
    let mut composer = telex();
    type_text(&mut composer, "vie");

    composer.break_word();

    assert_eq!(type_text(&mut composer, "j"), "j", "the tone has no word to go on");
}

#[test]
fn new_settings_start_a_new_word() {
    let mut composer = telex();
    type_text(&mut composer, "vie");

    composer.set_settings(ComposerSettings::from_settings(
        &settings().with(SettingChange::InputMethod(InputMethod::Vni)),
        Language::Vietnamese,
    ));

    assert_eq!(composer.word(), "", "the word is dropped");
    assert_eq!(type_text(&mut composer, "a1"), "á", "the new input method types");
}

#[test]
fn backspace_with_no_word_goes_through() {
    let mut composer = telex();

    assert_eq!(
        composer.press(KeyPress::new(KeyCode::Backspace, KeyMods::none())),
        Edit::PassThrough,
        "it deletes text before the word"
    );
}
