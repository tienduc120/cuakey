use super::effects;
use crate::typing::{Effect, Mark};
use crate::{BaseLetter, KeyCode, OnOff, Tone};

fn mark_of(code: KeyCode) -> Mark {
    match effects(code, OnOff::Off) {
        Some([Effect::Mark(mark)]) => *mark,
        other => panic!("{code:?} puts no mark: {other:?}"),
    }
}

#[test]
fn digits_one_to_five_give_the_tones_and_zero_takes_it_off() {
    let tones = [
        (KeyCode::Digit1, Tone::Sac),
        (KeyCode::Digit2, Tone::Huyen),
        (KeyCode::Digit3, Tone::Hoi),
        (KeyCode::Digit4, Tone::Nga),
        (KeyCode::Digit5, Tone::Nang),
        (KeyCode::Digit0, Tone::Ngang),
    ];

    for (code, tone) in tones {
        assert_eq!(
            effects(code, OnOff::Off),
            Some(&[Effect::Tone(tone)][..]),
            "{code:?} gives {tone:?}"
        );
    }
}

#[test]
fn six_puts_a_circumflex_on_a_e_and_o() {
    let mark = mark_of(KeyCode::Digit6);

    assert_eq!(mark.on(BaseLetter::A), Some(BaseLetter::ACircumflex), "â");
    assert_eq!(mark.on(BaseLetter::E), Some(BaseLetter::ECircumflex), "ê");
    assert_eq!(mark.on(BaseLetter::O), Some(BaseLetter::OCircumflex), "ô");
    assert_eq!(mark.on(BaseLetter::U), None, "nothing on u");
}

#[test]
fn seven_puts_a_horn_on_o_and_u() {
    let mark = mark_of(KeyCode::Digit7);

    assert_eq!(mark.on(BaseLetter::O), Some(BaseLetter::OHorn), "ơ");
    assert_eq!(mark.on(BaseLetter::U), Some(BaseLetter::UHorn), "ư");
    assert_eq!(mark.on(BaseLetter::A), None, "nothing on a");
}

#[test]
fn eight_puts_a_breve_on_a() {
    let mark = mark_of(KeyCode::Digit8);

    assert_eq!(mark.on(BaseLetter::A), Some(BaseLetter::ABreve), "ă");
    assert_eq!(mark.on(BaseLetter::O), None, "nothing on o");
}

#[test]
fn nine_strokes_the_d_that_starts_the_word() {
    assert_eq!(
        effects(KeyCode::Digit9, OnOff::Off),
        Some(&[Effect::Stroke][..]),
        "d9 makes đ"
    );
}

#[test]
fn letters_belong_to_the_word_and_symbols_end_it() {
    assert_eq!(
        effects(KeyCode::W, OnOff::Off),
        Some(&[][..]),
        "w is a plain letter in VNI"
    );
    assert_eq!(
        effects(KeyCode::A, OnOff::On),
        Some(&[][..]),
        "a capital is a letter too"
    );
    assert_eq!(effects(KeyCode::Digit1, OnOff::On), None, "Shift and 1 types !");
    for code in [KeyCode::Space, KeyCode::LeftBracket, KeyCode::Period] {
        assert_eq!(effects(code, OnOff::Off), None, "{code:?} ends the word");
    }
}
