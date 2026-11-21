//! Seeded random key sequences over random settings: the keys a person types, and the slips they make.

use cuakey_domain::{CodeTable, InputMethod, KeyCode, KeyMods, KeyPress, Language, OnOff};

use crate::combo::{CODE_TABLES, Combo, Toggle, Toggles};
use crate::corpus::Stroke;

/// The longest sequence generated.
const MAX_LENGTH: usize = 12;

/// `SplitMix64`: small, fast, and the same numbers for the same seed on every machine.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    pub const fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// A number in `0..bound`; `bound` is never 0 here.
    fn below(&mut self, bound: usize) -> usize {
        let bound = u64::try_from(bound).unwrap_or(u64::MAX).max(1);
        usize::try_from(self.next() % bound).unwrap_or(0)
    }

    fn pick<T: Copy>(&mut self, items: &[T]) -> Option<T> {
        items.get(self.below(items.len())).copied()
    }

    fn chance(&mut self, percent: usize) -> bool {
        self.below(100) < percent
    }
}

/// One random combination of settings.
pub fn combo(rng: &mut Rng) -> Combo {
    let method = rng
        .pick(&[InputMethod::Telex, InputMethod::Vni, InputMethod::SimpleTelex])
        .unwrap_or(InputMethod::Telex);
    let table = rng.pick(&CODE_TABLES).unwrap_or(CodeTable::Unicode);
    let toggles = Toggle::TYPING.into_iter().fold(Toggles::defaults(), |toggles, toggle| {
        toggles.with(toggle, if rng.chance(50) { OnOff::On } else { OnOff::Off })
    });
    Combo {
        method,
        table,
        language: Language::Vietnamese,
        toggles,
    }
}

/// One random sequence for `method`: 1 to 12 strokes, then a Space.
pub fn sequence(rng: &mut Rng, method: InputMethod) -> Vec<Stroke> {
    let length = 1 + rng.below(MAX_LENGTH);
    let caps = if rng.chance(5) { OnOff::On } else { OnOff::Off };
    // A Space ends the last word, as it ends each corpus line: restoring and quick consonants act at a word end.
    let space = Stroke::Key(KeyPress::new(KeyCode::Space, KeyMods::none().with_caps_lock(caps)));
    (0..length).map(|_| stroke(rng, method, caps)).chain([space]).collect()
}

const VOWELS: [KeyCode; 6] = [KeyCode::A, KeyCode::E, KeyCode::I, KeyCode::O, KeyCode::U, KeyCode::Y];
const CONSONANTS: [KeyCode; 16] = [
    KeyCode::B,
    KeyCode::C,
    KeyCode::D,
    KeyCode::G,
    KeyCode::H,
    KeyCode::K,
    KeyCode::L,
    KeyCode::M,
    KeyCode::N,
    KeyCode::P,
    KeyCode::Q,
    KeyCode::R,
    KeyCode::S,
    KeyCode::T,
    KeyCode::V,
    KeyCode::X,
];
/// The keys Telex reads as marks and tones.
const TELEX_MARKS: [KeyCode; 13] = [
    KeyCode::S,
    KeyCode::F,
    KeyCode::R,
    KeyCode::X,
    KeyCode::J,
    KeyCode::Z,
    KeyCode::W,
    KeyCode::A,
    KeyCode::E,
    KeyCode::O,
    KeyCode::D,
    KeyCode::LeftBracket,
    KeyCode::RightBracket,
];
const DIGITS: [KeyCode; 10] = [
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
const PUNCTUATION: [KeyCode; 6] = [
    KeyCode::Period,
    KeyCode::Comma,
    KeyCode::Semicolon,
    KeyCode::Quote,
    KeyCode::Slash,
    KeyCode::Minus,
];
const SHORTCUTS: [KeyCode; 2] = [KeyCode::C, KeyCode::S];

fn stroke(rng: &mut Rng, method: InputMethod, caps: OnOff) -> Stroke {
    let marks: &[KeyCode] = match method {
        InputMethod::Vni => &DIGITS,
        InputMethod::Telex | InputMethod::SimpleTelex => &TELEX_MARKS,
    };
    let roll = rng.below(100);
    let (keys, shift, ctrl): (&[KeyCode], OnOff, OnOff) = match roll {
        0..=24 => (&VOWELS, upper(rng), OnOff::Off),
        25..=49 => (&CONSONANTS, upper(rng), OnOff::Off),
        50..=64 => (marks, OnOff::Off, OnOff::Off),
        65..=74 => (&[KeyCode::Space], OnOff::Off, OnOff::Off),
        75..=84 => (&[KeyCode::Backspace], OnOff::Off, OnOff::Off),
        85..=90 => (&PUNCTUATION, OnOff::Off, OnOff::Off),
        91..=95 => return Stroke::Click,
        _ => (&SHORTCUTS, OnOff::Off, OnOff::On),
    };
    let code = rng.pick(keys).unwrap_or(KeyCode::A);
    let mods = KeyMods::none()
        .with_shift(shift)
        .with_caps_lock(caps)
        .with_ctrl_or_alt(ctrl);
    Stroke::Key(KeyPress::new(code, mods))
}

/// Shift on a letter, now and then.
fn upper(rng: &mut Rng) -> OnOff {
    if rng.chance(8) { OnOff::On } else { OnOff::Off }
}

#[cfg(test)]
mod tests;
