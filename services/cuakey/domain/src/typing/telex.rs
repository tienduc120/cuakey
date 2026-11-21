//! Telex: letters typed after a vowel stand for its marks and the tone (aa â, aw ă, ow ơ, uw ư, dd đ; s f r x j the
//! tones, z none).

use super::{Effect, KeyTable, Mark, look_up, plain_letter};
use crate::{BaseLetter, KeyCode, Tone};

/// Telex as most people type it, or Simple Telex, where a lone `w` and the brackets type no vowel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Style {
    Full,
    Simple,
}

const CIRCUMFLEX_A: Mark = Mark(&[(BaseLetter::A, BaseLetter::ACircumflex)]);
const CIRCUMFLEX_E: Mark = Mark(&[(BaseLetter::E, BaseLetter::ECircumflex)]);
const CIRCUMFLEX_O: Mark = Mark(&[(BaseLetter::O, BaseLetter::OCircumflex)]);

/// The mark `w` types: a breve on a, a horn on o and u.
const BREVE_OR_HORN: Mark = Mark(&[
    (BaseLetter::A, BaseLetter::ABreve),
    (BaseLetter::O, BaseLetter::OHorn),
    (BaseLetter::U, BaseLetter::UHorn),
]);

/// The keys both styles read alike.
const SHARED_KEYS: [(KeyCode, &[Effect]); 10] = [
    (KeyCode::A, &[Effect::Mark(CIRCUMFLEX_A)]),
    (KeyCode::E, &[Effect::Mark(CIRCUMFLEX_E)]),
    (KeyCode::O, &[Effect::Mark(CIRCUMFLEX_O)]),
    (KeyCode::D, &[Effect::Stroke]),
    (KeyCode::S, &[Effect::Tone(Tone::Sac)]),
    (KeyCode::F, &[Effect::Tone(Tone::Huyen)]),
    (KeyCode::R, &[Effect::Tone(Tone::Hoi)]),
    (KeyCode::X, &[Effect::Tone(Tone::Nga)]),
    (KeyCode::J, &[Effect::Tone(Tone::Nang)]),
    (KeyCode::Z, &[Effect::Tone(Tone::Ngang)]),
];

/// In full Telex a `w` with nothing to mark types ư, and the brackets type ơ and ư.
const FULL_KEYS: [(KeyCode, &[Effect]); 3] = [
    (
        KeyCode::W,
        &[Effect::Mark(BREVE_OR_HORN), Effect::Vowel(BaseLetter::UHorn)],
    ),
    (KeyCode::LeftBracket, &[Effect::Vowel(BaseLetter::OHorn)]),
    (KeyCode::RightBracket, &[Effect::Vowel(BaseLetter::UHorn)]),
];

const SIMPLE_KEYS: [(KeyCode, &[Effect]); 1] = [(KeyCode::W, &[Effect::Mark(BREVE_OR_HORN)])];

/// What `code` asks of the word in Telex, or `None` when the key ends the word.
pub(super) fn effects(code: KeyCode, style: Style) -> Option<&'static [Effect]> {
    let style_keys: &KeyTable = match style {
        Style::Full => &FULL_KEYS,
        Style::Simple => &SIMPLE_KEYS,
    };
    look_up(style_keys, code)
        .or_else(|| look_up(&SHARED_KEYS, code))
        .or_else(|| plain_letter(code))
}

#[cfg(test)]
mod tests;
