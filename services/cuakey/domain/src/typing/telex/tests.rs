use super::{Style, effects};
use crate::typing::{Effect, Mark};
use crate::{BaseLetter, KeyCode, Tone};

fn mark_of(code: KeyCode, style: Style) -> Mark {
    match effects(code, style) {
        Some([Effect::Mark(mark), ..]) => *mark,
        other => panic!("{code:?} puts no mark first: {other:?}"),
    }
}

#[test]
fn tone_keys_give_their_tones() {
    let tones = [
        (KeyCode::S, Tone::Sac),
        (KeyCode::F, Tone::Huyen),
        (KeyCode::R, Tone::Hoi),
        (KeyCode::X, Tone::Nga),
        (KeyCode::J, Tone::Nang),
        (KeyCode::Z, Tone::Ngang),
    ];

    for (code, tone) in tones {
        assert_eq!(
            effects(code, Style::Full),
            Some(&[Effect::Tone(tone)][..]),
            "{code:?} gives {tone:?}"
        );
    }
}

#[test]
fn a_vowel_typed_again_puts_a_circumflex_on_it() {
    let doubled = [
        (KeyCode::A, BaseLetter::A, BaseLetter::ACircumflex),
        (KeyCode::E, BaseLetter::E, BaseLetter::ECircumflex),
        (KeyCode::O, BaseLetter::O, BaseLetter::OCircumflex),
    ];

    for (code, plain, marked) in doubled {
        let mark = mark_of(code, Style::Full);

        assert_eq!(mark.on(plain), Some(marked), "{code:?} on {plain:?}");
        assert_eq!(mark.on(BaseLetter::U), None, "{code:?} leaves u alone");
    }
}

#[test]
fn w_puts_a_breve_on_a_and_a_horn_on_o_and_u() {
    let mark = mark_of(KeyCode::W, Style::Simple);

    assert_eq!(mark.on(BaseLetter::A), Some(BaseLetter::ABreve), "ă");
    assert_eq!(mark.on(BaseLetter::O), Some(BaseLetter::OHorn), "ơ");
    assert_eq!(mark.on(BaseLetter::U), Some(BaseLetter::UHorn), "ư");
    assert_eq!(mark.on(BaseLetter::E), None, "nothing on e");
}

#[test]
fn a_lone_w_types_u_horn_in_telex_but_not_in_simple_telex() {
    let full = effects(KeyCode::W, Style::Full).unwrap();
    let simple = effects(KeyCode::W, Style::Simple).unwrap();

    assert_eq!(full.len(), 2, "a mark, then a vowel: {full:?}");
    assert_eq!(
        full.get(1),
        Some(&Effect::Vowel(BaseLetter::UHorn)),
        "ư when there is nothing to mark"
    );
    assert_eq!(simple.len(), 1, "only the mark: {simple:?}");
}

#[test]
fn d_strokes_the_d_that_starts_the_word() {
    assert_eq!(
        effects(KeyCode::D, Style::Full),
        Some(&[Effect::Stroke][..]),
        "dd makes đ"
    );
}

#[test]
fn brackets_type_horned_vowels_in_telex_and_end_the_word_in_simple_telex() {
    assert_eq!(
        effects(KeyCode::LeftBracket, Style::Full),
        Some(&[Effect::Vowel(BaseLetter::OHorn)][..]),
        "[ types ơ"
    );
    assert_eq!(
        effects(KeyCode::RightBracket, Style::Full),
        Some(&[Effect::Vowel(BaseLetter::UHorn)][..]),
        "] types ư"
    );
    assert_eq!(effects(KeyCode::LeftBracket, Style::Simple), None, "[ is punctuation");
    assert_eq!(effects(KeyCode::RightBracket, Style::Simple), None, "] is punctuation");
}

#[test]
fn other_letters_belong_to_the_word_and_other_keys_end_it() {
    assert_eq!(effects(KeyCode::B, Style::Full), Some(&[][..]), "b is a plain letter");
    for code in [KeyCode::Space, KeyCode::Digit1, KeyCode::Comma, KeyCode::Enter] {
        assert_eq!(effects(code, Style::Full), None, "{code:?} ends the word");
    }
}
