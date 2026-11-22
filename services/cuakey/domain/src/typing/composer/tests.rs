use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use super::Composer;
use crate::{
    BaseLetter, CodeTable, ComposerSettings, Edit, InputMethod, KeyCode, KeyMods, KeyPress, Language, Letter,
    LetterCase, OnOff, SettingChange, Settings, Tone,
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
    let let_through = |screen: &mut String| match typed {
        Some(c) => screen.push(c),
        None => {
            screen.pop();
        }
    };
    match edit {
        Edit::PassThrough => let_through(screen),
        Edit::Replace {
            delete,
            insert,
            resend_key,
        } => {
            for _ in 0..delete {
                screen.pop();
            }
            screen.push_str(&insert);
            if resend_key {
                let_through(screen);
            }
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
    assert_eq!(type_text(&mut telex(), "Vieetj Nam"), "Việt Nam", "space ends the word");
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
fn with_spell_check_deleting_the_key_that_took_a_change_back_lets_the_word_change_again() {
    let cases = [
        ("vieee\u{8}ej", "việ", "the circumflex comes back"),
        (
            "vieee\u{8}\u{8}eej",
            "việ",
            "deleting further, the circumflex comes back",
        ),
        ("ddd\u{8}di", "đi", "the stroke comes back"),
        ("vieejj\u{8}j", "việ", "the tone comes back"),
        ("ass\u{8}s", "á", "the acute comes back"),
        ("nhww\u{8}w", "như", "a lone w types ư again"),
    ];

    for (keys, shown, why) in cases {
        let mut composer = composer_with(&vietnamese());
        let mut screen = String::new();

        for c in keys.chars() {
            type_key(&mut composer, &mut screen, c, OnOff::Off);
            assert_eq!(
                screen,
                composer.word(),
                "{keys:?}: after {c:?} the word is what the screen shows"
            );
        }

        assert_eq!(screen, shown, "{keys:?}: {why}");
    }
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
fn a_bracket_types_its_vowel_after_a_u() {
    for (keys, shown) in [
        ("ng][if", "người"),
        ("nu[cs", "nước"),
        ("nw[", "nươ"),
        ("thu[r", "thuở"),
    ] {
        assert_eq!(
            type_text(&mut telex(), keys),
            shown,
            "{keys:?}: ươ and uơ typed with brackets"
        );
    }
}

#[test]
fn a_bracket_types_its_vowel_after_a_word_of_one_vowel_but_e_or_y() {
    for (keys, shown) in [
        ("a[", "aơ"),
        ("i]", "iư"),
        ("os]", "oứ"),
        ("e[", "e["),
        ("y]", "y]"),
        ("nu]", "nu]"),
    ] {
        assert_eq!(type_text(&mut telex(), keys), shown, "{keys:?}");
    }
}

#[test]
fn a_key_that_types_the_same_vowel_takes_it_back() {
    assert_eq!(type_text(&mut telex(), "nw]"), "n]", "] after the ư of w");
    assert_eq!(type_text(&mut telex(), "n]w"), "nw", "w after the ư of ]");
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
fn without_spell_check_marks_go_on_the_last_syllable_of_a_longer_word() {
    assert_eq!(type_text(&mut telex(), "hellos"), "helló", "the o of lo");
    assert_eq!(
        type_text(&mut telex(), "after"),
        "àtẻ",
        "each syllable keeps its own tone"
    );
    assert_eq!(
        type_text(&mut composer_with(&vietnamese()), "hellos"),
        "hellos",
        "with spell check, a word of two syllables is not Vietnamese"
    );
}

#[test]
fn vni_digits_with_nothing_to_mark_are_typed() {
    assert_eq!(type_text(&mut vni(), "1a"), "1a", "a tone with no vowel");
    assert_eq!(type_text(&mut vni(), "a10"), "a", "0 takes the tone off");
    assert_eq!(type_text(&mut vni(), "a!"), "a!", "a shifted digit is a symbol");
}

#[test]
fn marks_and_tones_go_on_the_letters_after_a_vni_digit_that_types_itself() {
    let vni_spelled = vietnamese().with(SettingChange::InputMethod(InputMethod::Vni));
    for (composer, keys, shown) in [
        (vni(), "7u1", "7ú"),
        (vni(), "5bru5", "5brụ"),
        (vni(), "t1ad5g3a2", "t1ad5g3à"),
        (composer_with(&vni_spelled), "7u1 ", "7ú "),
    ] {
        let mut composer = composer;
        assert_eq!(
            type_text(&mut composer, keys),
            shown,
            "{keys:?}: the letters after the digit take the marks"
        );
    }
}

#[test]
fn with_spell_check_a_vni_digit_after_letters_makes_the_word_foreign() {
    let vni_spelled = vietnamese().with(SettingChange::InputMethod(InputMethod::Vni));
    for (keys, shown) in [("i8a4", "i8a4"), ("a1b2 ", "a1b2 ")] {
        assert_eq!(
            type_text(&mut composer_with(&vni_spelled), keys),
            shown,
            "{keys:?}: the word mixes letters and digits, so it stays or goes back to its keys"
        );
    }
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
fn a_chord_ends_the_word_as_the_screen_shows_it() {
    let quick_last = vietnamese().with(SettingChange::QuickLastConsonant(OnOff::On));
    for (settings, keys) in [(vietnamese(), "asno"), (quick_last, "ok")] {
        let mut composer = composer_with(&settings);
        type_text(&mut composer, keys);

        let edit = composer.press(KeyPress::new(KeyCode::S, KeyMods::none().with_ctrl_or_alt(OnOff::On)));

        assert_eq!(
            edit,
            Edit::PassThrough,
            "{keys:?}: Ctrl+S neither restores the word nor writes out a quick consonant"
        );
        assert_eq!(composer.word(), "", "{keys:?}: the chord ends the word");
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

/// The settings as they ship: spell check, restoring an invalid word, and keeping `w` at the start of a word on.
fn vietnamese() -> Settings {
    Settings::default()
}

#[test]
fn ass_restores() {
    assert_eq!(
        type_text(&mut composer_with(&vietnamese()), "ass"),
        "as",
        "the second s takes the acute back and types itself"
    );
}

#[test]
fn hello_stays_english() {
    assert_eq!(
        type_text(&mut composer_with(&vietnamese()), "hello "),
        "hello ",
        "a word that cannot be Vietnamese is left as typed"
    );
}

#[test]
fn windows_with_keep_w() {
    assert_eq!(
        type_text(&mut composer_with(&vietnamese()), "windows"),
        "windows",
        "the w that starts the word stays w, and spell check leaves the rest alone"
    );
}

#[test]
fn keep_w_types_w_only_at_the_start_of_a_word() {
    let cases = [
        ("w", "w", "a lone w"),
        ("W", "W", "a capital W"),
        ("ww", "ww", "a second w after it"),
        ("nhw", "như", "a w after consonants still types ư"),
        ("uw", "ư", "a w after u puts the horn on it"),
        ("aw ow", "ă ơ", "a w after a or o marks it"),
        ("tooi w", "tôi w", "a w after a space starts a new word"),
    ];

    for (keys, shown, why) in cases {
        assert_eq!(
            type_text(&mut composer_with(&vietnamese()), keys),
            shown,
            "{keys:?}: {why}"
        );
    }
}

#[test]
fn keep_w_after_a_click() {
    let mut composer = composer_with(&vietnamese());
    type_text(&mut composer, "tooi");

    composer.break_word();

    assert_eq!(type_text(&mut composer, "w"), "w", "a click starts a new word");
}

#[test]
fn keep_w_leaves_simple_telex_alone_and_can_be_turned_off() {
    let simple = vietnamese().with(SettingChange::InputMethod(InputMethod::SimpleTelex));
    let off = vietnamese().with(SettingChange::KeepW(OnOff::Off));

    assert_eq!(
        type_text(&mut composer_with(&simple), "w"),
        "w",
        "Simple Telex keeps a lone w"
    );
    assert_eq!(
        type_text(&mut composer_with(&simple), "uw"),
        "ư",
        "Simple Telex still marks u"
    );
    assert_eq!(
        type_text(&mut composer_with(&off), "w"),
        "ư",
        "with the option off, a lone w types ư"
    );
}

#[test]
fn quick_telex() {
    let mut composer = composer_with(&vietnamese().with(SettingChange::QuickTelex(OnOff::On)));

    assert_eq!(type_text(&mut composer, "ccaof"), "chào", "cc types ch");
}

#[test]
fn quick_first_consonant() {
    let mut composer = composer_with(&vietnamese().with(SettingChange::QuickFirstConsonant(OnOff::On)));

    assert_eq!(
        type_text(&mut composer, "fanh "),
        "phanh ",
        "an f that starts the word becomes ph when the word ends"
    );
}

#[test]
fn without_spell_check_a_quick_first_consonant_is_written_out_before_no_vowel_too() {
    let quick_first = settings().with(SettingChange::QuickFirstConsonant(OnOff::On));
    let simple_quick_first = quick_first
        .clone()
        .with(SettingChange::InputMethod(InputMethod::SimpleTelex));
    for (settings, keys, shown) in [
        (&simple_quick_first, "ww ", "quw "),
        (&simple_quick_first, "jp ", "gip "),
        (&quick_first, "fr ", "phr "),
    ] {
        assert_eq!(
            type_text(&mut composer_with(settings), keys),
            shown,
            "{keys:?}: the word has no vowel yet, but spell check is off"
        );
    }
}

#[test]
fn a_quick_first_consonant_typed_alone_stays_a_letter() {
    let quick_first = settings().with(SettingChange::QuickFirstConsonant(OnOff::On));
    let simple_quick_first = quick_first
        .clone()
        .with(SettingChange::InputMethod(InputMethod::SimpleTelex));
    for (settings, keys) in [
        (&simple_quick_first, "w "),
        (&simple_quick_first, "W "),
        (&quick_first, "f "),
    ] {
        assert_eq!(
            type_text(&mut composer_with(settings), keys),
            keys,
            "{keys:?}: a letter alone is the letter itself"
        );
    }
}

#[test]
fn a_w_typed_again_to_take_back_its_vowel_is_no_quick_first_consonant() {
    let quick_first = settings()
        .with(SettingChange::QuickFirstConsonant(OnOff::On))
        .with(SettingChange::KeepW(OnOff::Off));

    assert_eq!(
        type_text(&mut composer_with(&quick_first), "wwu "),
        "wu ",
        "the second w asked for the letter w itself"
    );
}

#[test]
fn a_chord_ends_the_sentence_as_far_as_capitals_go() {
    let mut composer = composer_with(&settings().with(SettingChange::CapitalizeSentences(OnOff::On)));
    type_text(&mut composer, "tooi. ");

    composer.press(KeyPress::new(KeyCode::S, KeyMods::none().with_ctrl_or_alt(OnOff::On)));

    assert_eq!(
        type_text(&mut composer, "ban"),
        "ban",
        "the chord may have moved the caret, so no sentence is known to start"
    );
}

#[test]
fn backspace_after_a_simple_telex_bracket_takes_the_word_up_again() {
    let simple = settings().with(SettingChange::InputMethod(InputMethod::SimpleTelex));

    assert_eq!(
        type_text(&mut composer_with(&simple), "ana[\u{8}r"),
        "ẩn",
        "the hook goes on the word before the deleted bracket"
    );
}

#[test]
fn simple_telex_brackets_end_the_word_as_it_shows() {
    let quick = settings()
        .with(SettingChange::InputMethod(InputMethod::SimpleTelex))
        .with(SettingChange::QuickFirstConsonant(OnOff::On))
        .with(SettingChange::QuickLastConsonant(OnOff::On));
    let spelled = vietnamese().with(SettingChange::InputMethod(InputMethod::SimpleTelex));
    for (settings, keys, shown) in [
        (&quick, "ju]", "ju]"),
        (&quick, "uh[", "uh["),
        (&spelled, "asd[", "ád["),
    ] {
        assert_eq!(
            type_text(&mut composer_with(settings), keys),
            shown,
            "{keys:?}: no quick consonant written out and no restore"
        );
    }
}

#[test]
fn quick_last_consonant() {
    let mut composer = composer_with(&vietnamese().with(SettingChange::QuickLastConsonant(OnOff::On)));

    assert_eq!(
        type_text(&mut composer, "hag "),
        "hang ",
        "a g that ends the word becomes ng when the word ends"
    );
}

#[test]
fn with_spell_check_a_quick_last_consonant_is_judged_as_the_pair_it_stands_for() {
    let quick_last = vietnamese().with(SettingChange::QuickLastConsonant(OnOff::On));
    for (keys, shown) in [
        ("book ", "book "),
        ("look ", "look "),
        ("aak ", "aak "),
        ("uhr", "uhr"),
        ("ahs ", "ánh "),
    ] {
        assert_eq!(
            type_text(&mut composer_with(&quick_last), keys),
            shown,
            "{keys:?}: ôch, âch and unh are no syllables, anh is"
        );
    }
}

#[test]
fn with_spell_check_no_mark_or_tone_goes_where_no_syllable_can_follow() {
    let quick_last = vietnamese().with(SettingChange::QuickLastConsonant(OnOff::On));
    for (keys, shown) in [("aka", "aka"), ("nwohr", "nưohr"), ("tienge", "tiêng")] {
        assert_eq!(
            type_text(&mut composer_with(&quick_last), keys),
            shown,
            "{keys:?}: âch and ươnh are no syllables, whatever marks come later; iêng is"
        );
    }
}

#[test]
fn a_mark_typed_after_the_last_consonant_of_gieng_still_lands() {
    let no_restore = vietnamese().with(SettingChange::RestoreInvalidWord(OnOff::Off));

    assert_eq!(
        type_text(&mut composer_with(&no_restore), "gienges "),
        "giếng ",
        "gi then ê reads as iê, which ng follows"
    );
}

#[test]
fn a_quick_last_consonant_is_written_out_only_where_that_makes_a_syllable() {
    let quick_last = settings().with(SettingChange::QuickLastConsonant(OnOff::On));
    for (keys, shown) in [
        ("ah ", "anh "),
        ("eeh ", "ênh "),
        ("aak ", "âk "),
        ("uhf ", "ùh "),
        ("thuowg ", "thương "),
        ("aig ", "aig "),
    ] {
        assert_eq!(
            type_text(&mut composer_with(&quick_last), keys),
            shown,
            "{keys:?}: with spell check off too"
        );
    }
}

#[test]
fn capitalise_sentence() {
    let mut composer = composer_with(&vietnamese().with(SettingChange::CapitalizeSentences(OnOff::On)));

    assert_eq!(
        type_text(&mut composer, "tooi. ban"),
        "tôi. Ban",
        "the first letter after a full stop is a capital"
    );
}

#[test]
fn restore_before_tab_resends_key() {
    let mut composer = composer_with(&vietnamese());
    let shown = type_text(&mut composer, "asd");

    let edit = composer.press(KeyPress::new(KeyCode::Tab, KeyMods::none()));

    let Edit::Replace {
        delete,
        insert,
        resend_key,
    } = edit
    else {
        panic!("the word is typed again before the Tab, got {edit:?}");
    };
    assert_eq!(shown, "ád", "d is no last consonant, so the word is not Vietnamese");
    assert_eq!(delete, 2, "the word on screen goes");
    assert_eq!(insert, "asd", "the keys as typed");
    assert!(resend_key, "the Tab itself follows");
}

#[test]
fn a_restore_that_changes_nothing_lets_the_key_through() {
    let mut composer = composer_with(&vietnamese());
    type_text(&mut composer, "ass");

    assert_eq!(
        composer.press(KeyPress::new(KeyCode::Tab, KeyMods::none())),
        Edit::PassThrough,
        "the screen already shows the keys as typed, as"
    );
}

#[test]
fn backspace_deletes_every_code_unit_of_the_last_letter() {
    let compound = settings().with(SettingChange::CodeTable(CodeTable::UnicodeCompound));
    let mut composer = composer_with(&compound);
    type_text(&mut composer, "taas");

    let edit = composer.press(KeyPress::new(KeyCode::Backspace, KeyMods::none()));

    assert_eq!(
        edit,
        Edit::Replace {
            delete: 1,
            insert: String::new(),
            resend_key: false,
        },
        "ấ is two code units in Unicode compound, so the edit deletes the letter"
    );
    assert_eq!(
        composer.press(KeyPress::new(KeyCode::Backspace, KeyMods::none())),
        Edit::PassThrough,
        "a letter of one code unit goes with the key itself"
    );
}

#[test]
fn backspace_moves_the_tone_onto_the_vowel_that_carries_it_now() {
    let mut composer = telex();
    let mut screen = String::new();

    for c in "toans\u{8}".chars() {
        type_key(&mut composer, &mut screen, c, OnOff::Off);
    }

    assert_eq!(
        screen, "tóa",
        "with the n gone, an open oa carries the tone on o, old style"
    );
    assert_eq!(composer.word(), screen, "the word is what the screen shows");
}

#[test]
fn backspace_right_after_a_space_takes_the_word_up_again() {
    let mut composer = telex();
    let mut screen = String::new();

    for c in "chaof \u{8}".chars() {
        type_key(&mut composer, &mut screen, c, OnOff::Off);
    }
    let taken_up = String::from(composer.word());
    type_key(&mut composer, &mut screen, 'f', OnOff::Off);

    assert_eq!(
        taken_up, "chào",
        "the Space is gone and the word is the one being typed"
    );
    assert_eq!(screen, "chaof", "f again takes the grave back");
}

#[test]
fn backspace_back_across_a_word_end_takes_the_word_before_up_again() {
    assert_eq!(
        type_text(&mut telex(), "tieeng,x\u{8}\u{8}s"),
        "tiếng",
        "the x and the comma are gone, so the tone goes on the word before them"
    );
}

#[test]
fn backspace_right_after_punctuation_takes_the_word_up_again() {
    for (keys, shown) in [("tieeng,\u{8}s", "tiếng"), ("o,\u{8}o", "ô"), ("xu.\u{8}j", "xụ")] {
        assert_eq!(
            type_text(&mut telex(), keys),
            shown,
            "{keys:?}: the tone goes on the word before"
        );
    }
}

#[test]
fn backspace_over_any_key_that_ended_a_word_takes_the_sentence_back() {
    let capitals = settings().with(SettingChange::CapitalizeSentences(OnOff::On));
    for (keys, shown) in [
        ("tooi\n\u{8}ban", "tôiban"),
        ("tooi\n\u{8} ban", "tôi ban"),
        ("tooi. \u{8}\u{8} ban", "tôi ban"),
    ] {
        assert_eq!(
            type_text(&mut composer_with(&capitals), keys),
            shown,
            "{keys:?}: the line break or full stop is gone, so no sentence starts"
        );
    }
}

#[test]
fn taking_a_word_up_again_takes_its_sentence_back() {
    let capitals = settings().with(SettingChange::CapitalizeSentences(OnOff::On));

    assert_eq!(
        type_text(&mut composer_with(&capitals), "tooi.\u{8} ban"),
        "tôi ban",
        "the full stop is gone, so no sentence starts after it"
    );
}

#[test]
fn a_first_letter_deleted_at_a_sentence_start_is_raised_again_when_retyped() {
    let capitals = settings().with(SettingChange::CapitalizeSentences(OnOff::On));

    assert_eq!(
        type_text(&mut composer_with(&capitals), "tooi. k\u{8}ko"),
        "tôi. Ko",
        "the sentence still starts with the word typed again"
    );
}

#[test]
fn only_backspace_right_after_the_space_takes_the_word_up() {
    let mut composer = telex();

    assert_eq!(
        type_text(&mut composer, "chaof x\u{8}f"),
        "chào f",
        "the f starts a word of its own"
    );
}

#[test]
fn a_restored_word_keeps_the_tone_key_of_a_tone_that_moved() {
    assert_eq!(
        type_text(&mut composer_with(&vietnamese()), "hoans\u{8}\u{8}k "),
        "hosk ",
        "Backspace moved the tone onto the o, so the s still shows"
    );
}

#[test]
fn a_restored_word_is_taken_up_as_it_was_typed() {
    assert_eq!(
        type_text(&mut composer_with(&vietnamese()), "ass \u{8}s"),
        "ass",
        "the word the Space restored takes more keys as they are"
    );
}

#[test]
fn without_spell_check_a_word_typed_as_it_is_stays_so_after_backspace() {
    let cases = [
        ("vieee\u{8}ej", "vieej", "the e and the j are typed as they are"),
        ("ass\u{8}s", "as", "the s is typed as it is"),
    ];

    for (keys, shown, why) in cases {
        assert_eq!(type_text(&mut telex(), keys), shown, "{keys:?}: {why}");
    }
}

#[test]
fn a_syllable_ending_in_c_ch_p_or_t_takes_only_the_acute_or_the_dot_below() {
    for (keys, shown) in [("matf", "matf"), ("hocr", "hocr"), ("sachx", "sachx"), ("hapf", "hapf")] {
        assert_eq!(
            type_text(&mut telex(), keys),
            shown,
            "{keys:?}: no Vietnamese syllable reads so"
        );
    }
    for (keys, shown) in [("mats", "mát"), ("hocj", "học"), ("sachs", "sách"), ("hapj", "hạp")] {
        assert_eq!(type_text(&mut telex(), keys), shown, "{keys:?}: a stop takes these two");
    }
    assert_eq!(type_text(&mut vni(), "mat4"), "mat4", "VNI alike");
}

#[test]
fn a_falling_tone_typed_before_a_stop_keeps_the_word_from_being_a_syllable() {
    let quick_last = settings().with(SettingChange::QuickLastConsonant(OnOff::On));
    assert_eq!(
        type_text(&mut composer_with(&quick_last), "axk "),
        "ãk ",
        "ãch is no syllable, so the quick k is not written out"
    );
    assert_eq!(
        type_text(&mut composer_with(&vietnamese()), "hofc "),
        "hofc ",
        "hòc is no syllable, so restore gives the keys back"
    );
}

#[test]
fn marks_and_tones_never_reach_past_consonants_that_cannot_end_a_syllable() {
    assert_eq!(type_text(&mut telex(), "java"), "java", "v ends no syllable");
    assert_eq!(
        type_text(&mut telex(), "alsof"),
        "alsò",
        "the o after ls starts its own syllable"
    );
    assert_eq!(type_text(&mut vni(), "abc1"), "abc1", "nor does bc");
    assert_eq!(type_text(&mut vni(), "a1b2"), "áb2", "the 2 types itself");
    assert_eq!(type_text(&mut telex(), "toans"), "toán", "n ends a syllable");
}

#[test]
fn a_lone_w_types_u_horn_only_where_the_syllable_has_no_vowel_yet() {
    assert_eq!(type_text(&mut telex(), "ew"), "ew", "e takes no mark from w");
    assert_eq!(type_text(&mut telex(), "new"), "new", "nor does the e of ne");
    assert_eq!(type_text(&mut telex(), "e]"), "e]", "nor a bracket");
    assert_eq!(type_text(&mut telex(), "tw"), "tư", "after consonants alone, w types ư");
}

#[test]
fn a_lone_w_types_u_horn_only_after_a_first_consonant_that_goes_before_it() {
    for (keys, shown) in [
        ("nhw", "như"),
        ("trw", "trư"),
        ("ngw", "ngư"),
        ("pw", "pư"),
        ("qw", "qư"),
        ("h[", "hơ"),
    ] {
        assert_eq!(
            type_text(&mut telex(), keys),
            shown,
            "{keys:?}: a Vietnamese first consonant"
        );
    }
    let quick_first = settings().with(SettingChange::QuickFirstConsonant(OnOff::On));
    for (composer, keys) in [
        (telex(), "btw"),
        (telex(), "kw"),
        (telex(), "ghw"),
        (telex(), "jw"),
        (telex(), "zw"),
        (composer_with(&quick_first), "fw"),
        (telex(), "k["),
        (telex(), "pr]"),
    ] {
        let mut composer = composer;
        assert_eq!(
            type_text(&mut composer, keys),
            keys,
            "{keys:?}: no Vietnamese syllable starts so, so the key types itself"
        );
    }
}

#[test]
fn the_u_after_q_is_never_the_vowel_of_the_syllable() {
    assert_eq!(type_text(&mut telex(), "qus"), "qus", "qu has no vowel for the tone");
    assert_eq!(type_text(&mut telex(), "quas"), "quá", "the a after qu takes it");
    assert_eq!(type_text(&mut telex(), "quys"), "quý", "as does the y");
}

#[test]
fn after_th_w_on_an_open_uo_marks_the_o_alone() {
    assert_eq!(type_text(&mut telex(), "thuowr"), "thuở", "thuở");
    assert_eq!(type_text(&mut telex(), "huow"), "hươ", "after h, both take the horn");
    assert_eq!(type_text(&mut vni(), "thuo73"), "thuở", "VNI 7 alike");
}

#[test]
fn a_consonant_after_uo_with_a_horn_on_the_o_puts_one_on_the_u() {
    assert_eq!(type_text(&mut telex(), "thuowng"), "thương", "ươ before a consonant");
    assert_eq!(type_text(&mut telex(), "thuowrng"), "thưởng", "with its tone");
}

#[test]
fn half_a_horned_uo_takes_the_other_horn_once_the_syllable_goes_on() {
    for (keys, shown) in [
        ("ngu[if", "người"),
        ("bwoms", "bướm"),
        ("nwoc", "nươc"),
        ("ngwoi", "ngươi"),
    ] {
        assert_eq!(
            type_text(&mut telex(), keys),
            shown,
            "{keys:?}: ươ before a consonant or i"
        );
    }
    assert_eq!(type_text(&mut telex(), "thu[r"), "thuở", "an open uơ stays");
}

#[test]
fn d_strokes_the_d_that_starts_the_last_syllable() {
    assert_eq!(
        type_text(&mut telex(), "download"),
        "dơnload",
        "the last syllable starts with l"
    );
    assert_eq!(type_text(&mut telex(), "dongd"), "đong", "the d that starts the word");
}

#[test]
fn without_spell_check_d_strokes_a_d_typed_after_a_vowel() {
    assert_eq!(type_text(&mut telex(), "add"), "ađ", "a d after a");
    assert_eq!(
        type_text(&mut composer_with(&vietnamese()), "add"),
        "add",
        "with spell check, đ only starts a syllable"
    );
}

#[test]
fn a_consonant_that_ends_no_syllable_leaves_uo_as_it_is() {
    assert_eq!(type_text(&mut telex(), "thuowg"), "thuơg", "g ends no syllable");
}

#[test]
fn a_mark_goes_on_a_vowel_before_another_only_where_vietnamese_writes_that_pair() {
    assert_eq!(
        type_text(&mut telex(), "beoe"),
        "beoe",
        "êo is no Vietnamese vowel group"
    );
    assert_eq!(type_text(&mut telex(), "keue"), "kêu", "êu is one");
    assert_eq!(type_text(&mut telex(), "muaw"), "mưa", "as is ưa");
    assert_eq!(type_text(&mut telex(), "tuoio"), "tuôi", "and uôi");
}

#[test]
fn without_spell_check_restoring_waits_for_a_word_that_still_shows_a_mark() {
    let quick = settings().with(SettingChange::QuickTelex(OnOff::On));

    assert_eq!(
        type_text(&mut composer_with(&quick), "success "),
        "suches ",
        "the tone taken back leaves no mark, so the word stays"
    );
    assert_eq!(
        type_text(&mut telex(), "taass "),
        "taas ",
        "â is still shown, so the keys are typed again"
    );
    assert_eq!(
        type_text(
            &mut composer_with(&vietnamese().with(SettingChange::QuickTelex(OnOff::On))),
            "vieejtt "
        ),
        "vieejtt ",
        "with spell check, restoring types the doubled consonant as typed"
    );
}

#[test]
fn a_quick_last_consonant_leaves_uo_as_it_is_too() {
    let quick = settings().with(SettingChange::QuickLastConsonant(OnOff::On));

    assert_eq!(
        type_text(&mut composer_with(&quick), "thuowg"),
        "thuơg",
        "g is no ng yet"
    );
}

#[test]
fn an_exclamation_mark_ends_the_sentence_too() {
    let capitals = vietnamese().with(SettingChange::CapitalizeSentences(OnOff::On));
    let vni_capitals = capitals.clone().with(SettingChange::InputMethod(InputMethod::Vni));

    assert_eq!(
        type_text(&mut composer_with(&capitals), "tooi! ban"),
        "tôi! Ban",
        "Telex: the ! ends the word and the sentence"
    );
    assert_eq!(
        type_text(&mut composer_with(&vni_capitals), "to6i! ban"),
        "tôi! Ban",
        "VNI: Shift with 1 types ! and ends the sentence"
    );
}

#[test]
fn a_closed_oo_takes_the_tone_on_its_second_o() {
    assert_eq!(type_text(&mut vni(), "mooc1"), "moóc", "moóc");
    assert_eq!(
        type_text(&mut vni(), "soo1c"),
        "soóc",
        "a tone typed early moves onto the second o"
    );
    assert_eq!(type_text(&mut vni(), "goo2ng"), "goòng", "goòng");
}

#[test]
fn a_lone_y_before_a_consonant_takes_no_tone() {
    for composer in [telex(), composer_with(&vietnamese())] {
        let mut composer = composer;
        assert_eq!(
            type_text(&mut composer, "cyns"),
            "cyns",
            "y never comes before a last consonant"
        );
    }
    assert_eq!(type_text(&mut telex(), "kyf"), "kỳ", "an open y takes it");
    assert_eq!(
        type_text(&mut telex(), "huyts"),
        "huýt",
        "as does the y of uy before a consonant"
    );
}

#[test]
fn one_vowel_written_twice_takes_no_tone_but_oo_and_uu() {
    let vni_new = settings()
        .with(SettingChange::InputMethod(InputMethod::Vni))
        .with(SettingChange::NewToneStyle(OnOff::On));
    let quick = settings().with(SettingChange::QuickTelex(OnOff::On));

    assert_eq!(type_text(&mut vni(), "the6e1"), "thêe1", "êe is no vowel group");
    assert_eq!(
        type_text(&mut composer_with(&vni_new), "the6e1"),
        "thêe1",
        "in either style"
    );
    assert_eq!(
        type_text(&mut composer_with(&quick), "ggif"),
        "giif",
        "nor is the ii after gi"
    );
    assert_eq!(type_text(&mut telex(), "nguur"), "ngủu", "uu is written as ưu");
    assert_eq!(type_text(&mut vni(), "mooc1"), "moóc", "oo is Vietnamese");
}

#[test]
fn the_y_after_qu_takes_the_tone_before_a_consonant() {
    assert_eq!(
        type_text(&mut composer_with(&vietnamese()), "quynhf"),
        "quỳnh",
        "the u of qu is no vowel of its own"
    );
    assert_eq!(type_text(&mut vni(), "quyt5"), "quỵt", "VNI alike");
}

/// Where a word is typed with its tone key.
#[derive(Debug, Clone, Copy)]
enum ToneKeyAt {
    /// Right after the vowel that carries it, before any last consonant.
    AfterVowel,
    /// After the whole word.
    End,
}

/// An input method as a typist uses it: the key typed after each plain letter for its mark, and the key for each tone.
#[derive(Debug)]
struct Keyboard {
    method: InputMethod,
    marks: [(BaseLetter, char, char); 7],
    tones: [(Tone, char); 5],
}

const TELEX_KEYBOARD: Keyboard = Keyboard {
    method: InputMethod::Telex,
    marks: [
        (BaseLetter::ABreve, 'a', 'w'),
        (BaseLetter::ACircumflex, 'a', 'a'),
        (BaseLetter::ECircumflex, 'e', 'e'),
        (BaseLetter::OCircumflex, 'o', 'o'),
        (BaseLetter::OHorn, 'o', 'w'),
        (BaseLetter::UHorn, 'u', 'w'),
        (BaseLetter::DStroke, 'd', 'd'),
    ],
    tones: [
        (Tone::Sac, 's'),
        (Tone::Huyen, 'f'),
        (Tone::Hoi, 'r'),
        (Tone::Nga, 'x'),
        (Tone::Nang, 'j'),
    ],
};

const VNI_KEYBOARD: Keyboard = Keyboard {
    method: InputMethod::Vni,
    marks: [
        (BaseLetter::ABreve, 'a', '8'),
        (BaseLetter::ACircumflex, 'a', '6'),
        (BaseLetter::ECircumflex, 'e', '6'),
        (BaseLetter::OCircumflex, 'o', '6'),
        (BaseLetter::OHorn, 'o', '7'),
        (BaseLetter::UHorn, 'u', '7'),
        (BaseLetter::DStroke, 'd', '9'),
    ],
    tones: [
        (Tone::Sac, '1'),
        (Tone::Huyen, '2'),
        (Tone::Hoi, '3'),
        (Tone::Nga, '4'),
        (Tone::Nang, '5'),
    ],
};

/// Real Vietnamese words, each with the tone style it is written in (on: new style).
const REAL_WORDS: &[(&str, OnOff)] = &[
    ("giêng", OnOff::Off),
    ("giếng", OnOff::Off),
    ("giềng", OnOff::Off),
    ("giếc", OnOff::Off),
    ("giữa", OnOff::Off),
    ("giấc", OnOff::Off),
    ("giặt", OnOff::Off),
    ("thương", OnOff::Off),
    ("thường", OnOff::Off),
    ("sách", OnOff::Off),
    ("bếch", OnOff::Off),
    ("thích", OnOff::Off),
    ("huých", OnOff::Off),
    ("khuếch", OnOff::Off),
    ("oách", OnOff::Off),
    ("anh", OnOff::Off),
    ("lênh", OnOff::Off),
    ("xinh", OnOff::Off),
    ("khuếnh", OnOff::Off),
    ("oanh", OnOff::Off),
    ("quỳnh", OnOff::Off),
    ("quýnh", OnOff::Off),
    ("quỵt", OnOff::Off),
    ("quýt", OnOff::Off),
    ("quyết", OnOff::Off),
    ("quyền", OnOff::Off),
    ("quạ", OnOff::Off),
    ("quốc", OnOff::Off),
    ("gìn", OnOff::Off),
    ("giết", OnOff::Off),
    ("giữ", OnOff::Off),
    ("gió", OnOff::Off),
    ("giường", OnOff::Off),
    ("ý", OnOff::Off),
    ("kỷ", OnOff::Off),
    ("mỹ", OnOff::Off),
    ("tỵ", OnOff::Off),
    ("huỳnh", OnOff::Off),
    ("khuỷu", OnOff::Off),
    ("tuýt", OnOff::Off),
    ("moóc", OnOff::Off),
    ("goòng", OnOff::Off),
    ("soóc", OnOff::Off),
    ("cừu", OnOff::Off),
    ("hưu", OnOff::Off),
    ("thuở", OnOff::Off),
    ("được", OnOff::Off),
    ("người", OnOff::Off),
    ("hòa", OnOff::Off),
    ("hoà", OnOff::On),
    ("khỏe", OnOff::Off),
    ("khoẻ", OnOff::On),
    ("thúy", OnOff::Off),
    ("thuý", OnOff::On),
    ("má", OnOff::Off),
    ("mắt", OnOff::Off),
    ("mất", OnOff::Off),
    ("mẹ", OnOff::Off),
    ("lễ", OnOff::Off),
    ("tím", OnOff::Off),
    ("có", OnOff::Off),
    ("cố", OnOff::Off),
    ("mở", OnOff::Off),
    ("thú", OnOff::Off),
    ("từ", OnOff::Off),
    ("hái", OnOff::Off),
    ("cháo", OnOff::Off),
    ("sáu", OnOff::Off),
    ("máy", OnOff::Off),
    ("dấu", OnOff::Off),
    ("đấy", OnOff::Off),
    ("kéo", OnOff::Off),
    ("nếu", OnOff::Off),
    ("mía", OnOff::Off),
    ("dịu", OnOff::Off),
    ("tiếng", OnOff::Off),
    ("toán", OnOff::Off),
    ("hoặc", OnOff::Off),
    ("khoét", OnOff::Off),
    ("gói", OnOff::Off),
    ("tối", OnOff::Off),
    ("mới", OnOff::Off),
    ("múa", OnOff::Off),
    ("tuần", OnOff::Off),
    ("thuế", OnOff::Off),
    ("múi", OnOff::Off),
    ("muốn", OnOff::Off),
    ("huýt", OnOff::Off),
    ("cửa", OnOff::Off),
    ("ngửi", OnOff::Off),
    ("yến", OnOff::Off),
    ("ngoài", OnOff::Off),
    ("xoáy", OnOff::Off),
    ("ngoẹo", OnOff::Off),
    ("khuấy", OnOff::Off),
    ("tuổi", OnOff::Off),
    ("chuyện", OnOff::Off),
    ("nhiều", OnOff::Off),
    ("rượu", OnOff::Off),
    ("yếu", OnOff::Off),
    ("khuya", OnOff::Off),
];

/// The keys that type `word` on `keyboard`, its tone key typed `at` that place.
fn keys_for(word: &str, keyboard: &Keyboard, at: ToneKeyAt) -> String {
    let letters: Vec<Letter> = word.chars().map(|c| Letter::from_char(c).unwrap()).collect();
    let tone_key = letters
        .iter()
        .find_map(|letter| keyboard.tones.iter().find(|(tone, _)| *tone == letter.tone()))
        .map(|(_, key)| *key);
    let last_vowel = letters.iter().rposition(|letter| letter.with_tone(Tone::Sac).is_some());
    let mut keys = String::new();
    for (place, letter) in letters.iter().enumerate() {
        match keyboard.marks.iter().find(|(base, ..)| *base == letter.base()) {
            Some((_, plain, mark)) => {
                keys.push(*plain);
                keys.push(*mark);
            }
            None => keys.extend(Letter::from_parts(letter.base(), Tone::Ngang, LetterCase::Lower).map(Letter::to_char)),
        }
        if matches!(at, ToneKeyAt::AfterVowel) && Some(place) == last_vowel {
            keys.extend(tone_key);
        }
    }
    if matches!(at, ToneKeyAt::End) {
        keys.extend(tone_key);
    }
    keys
}

/// Whether Telex can type `word`: a second o always makes ô, so a plain oo (moóc) is typed in VNI.
fn telex_types(word: &str) -> bool {
    let bases: Vec<BaseLetter> = word.chars().map(|c| Letter::from_char(c).unwrap().base()).collect();
    !bases.windows(2).any(|pair| pair == [BaseLetter::O, BaseLetter::O])
}

/// What typing `word` on `keyboard`, its tone key `at` that place, shows when it is not the word.
fn typed_wrong(keyboard: &Keyboard, (word, new_style): (&str, OnOff), at: ToneKeyAt) -> Option<String> {
    let settings = vietnamese()
        .with(SettingChange::InputMethod(keyboard.method))
        .with(SettingChange::NewToneStyle(new_style));
    let keys = keys_for(word, keyboard, at);
    let shown = type_text(&mut composer_with(&settings), &format!("{keys} "));
    (shown != format!("{word} ")).then(|| format!("{:?} {keys:?} shows {shown:?}, not {word:?}", keyboard.method))
}

#[test]
fn real_words_type_with_the_tone_key_before_or_after_the_last_consonant() {
    let wrong: Vec<String> = [&TELEX_KEYBOARD, &VNI_KEYBOARD]
        .into_iter()
        .flat_map(|keyboard| REAL_WORDS.iter().map(move |entry| (keyboard, *entry)))
        .filter(|(keyboard, (word, _))| keyboard.method != InputMethod::Telex || telex_types(word))
        .flat_map(|(keyboard, entry)| {
            [ToneKeyAt::AfterVowel, ToneKeyAt::End].map(|at| typed_wrong(keyboard, entry, at))
        })
        .flatten()
        .collect();

    assert_eq!(
        wrong,
        Vec::<String>::new(),
        "every real word types as Vietnamese writes it"
    );
}

/// What typing `word` with its last ng, nh, or ch as a quick g, h, or k shows when it is not the word, with the shipped
/// settings and quick last consonant on.
fn typed_wrong_with_quick_last(keyboard: &Keyboard, (word, new_style): (&str, OnOff), at: ToneKeyAt) -> Option<String> {
    let quick = [("ng", "g"), ("nh", "h"), ("ch", "k")]
        .into_iter()
        .find_map(|(pair, single)| word.strip_suffix(pair).map(|stem| format!("{stem}{single}")))?;
    let settings = vietnamese()
        .with(SettingChange::InputMethod(keyboard.method))
        .with(SettingChange::NewToneStyle(new_style))
        .with(SettingChange::QuickLastConsonant(OnOff::On));
    let keys = keys_for(&quick, keyboard, at);
    let shown = type_text(&mut composer_with(&settings), &format!("{keys} "));
    (shown != format!("{word} ")).then(|| format!("{:?} {keys:?} shows {shown:?}, not {word:?}", keyboard.method))
}

#[test]
fn real_words_type_with_a_quick_last_consonant() {
    let wrong: Vec<String> = [&TELEX_KEYBOARD, &VNI_KEYBOARD]
        .into_iter()
        .flat_map(|keyboard| REAL_WORDS.iter().map(move |entry| (keyboard, *entry)))
        .filter(|(keyboard, (word, _))| keyboard.method != InputMethod::Telex || telex_types(word))
        .flat_map(|(keyboard, entry)| {
            [ToneKeyAt::AfterVowel, ToneKeyAt::End].map(|at| typed_wrong_with_quick_last(keyboard, entry, at))
        })
        .flatten()
        .collect();

    assert_eq!(
        wrong,
        Vec::<String>::new(),
        "every real word ending in ng, nh, or ch types with g, h, or k under quick last consonant"
    );
}

mod macros;
