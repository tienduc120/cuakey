//! VNI: digits typed after a vowel stand for its marks and the tone (6 â ê ô, 7 ơ ư, 8 ă, 9 đ; 1 to 5 the tones, 0
//! none).

use super::{Effect, Mark, look_up, plain_letter};
use crate::{BaseLetter, KeyCode, OnOff, Tone};

const CIRCUMFLEX: Mark = Mark(&[
    (BaseLetter::A, BaseLetter::ACircumflex),
    (BaseLetter::E, BaseLetter::ECircumflex),
    (BaseLetter::O, BaseLetter::OCircumflex),
]);
const HORN: Mark = Mark(&[(BaseLetter::O, BaseLetter::OHorn), (BaseLetter::U, BaseLetter::UHorn)]);
const BREVE: Mark = Mark(&[(BaseLetter::A, BaseLetter::ABreve)]);

const DIGIT_KEYS: [(KeyCode, &[Effect]); 10] = [
    (KeyCode::Digit1, &[Effect::Tone(Tone::Sac)]),
    (KeyCode::Digit2, &[Effect::Tone(Tone::Huyen)]),
    (KeyCode::Digit3, &[Effect::Tone(Tone::Hoi)]),
    (KeyCode::Digit4, &[Effect::Tone(Tone::Nga)]),
    (KeyCode::Digit5, &[Effect::Tone(Tone::Nang)]),
    (KeyCode::Digit0, &[Effect::Tone(Tone::Ngang)]),
    (KeyCode::Digit6, &[Effect::Mark(CIRCUMFLEX)]),
    (KeyCode::Digit7, &[Effect::Mark(HORN)]),
    (KeyCode::Digit8, &[Effect::Mark(BREVE)]),
    (KeyCode::Digit9, &[Effect::Stroke]),
];

/// What `code` asks of the word in VNI, or `None` when the key ends the word. A digit with Shift is a symbol.
pub(super) fn effects(code: KeyCode, shift: OnOff) -> Option<&'static [Effect]> {
    let digit = match shift {
        OnOff::Off => look_up(&DIGIT_KEYS, code),
        OnOff::On => None,
    };
    digit.or_else(|| plain_letter(code))
}

#[cfg(test)]
mod tests;
