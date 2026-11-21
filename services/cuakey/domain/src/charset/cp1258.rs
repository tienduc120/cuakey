//! Windows code page 1258, from Unicode's CP1258.TXT mapping. A toned letter is written as Windows composes it: its
//! letter without the tone (ă â ê ô ơ ư đ have a byte of their own), then one of five combining tone bytes. The code
//! page also holds 16 toned letters whole; reading takes those too.

use alloc::string::String;

use super::units::{Units, single_unit, utf16_chars};
use super::{Letter, Tone, combining_mark, compose};

/// The Vietnamese letters the code page holds whole, with their bytes.
const WHOLE_LETTERS: [(char, u16); 30] = [
    ('À', 0xc0),
    ('Á', 0xc1),
    ('Â', 0xc2),
    ('Ă', 0xc3),
    ('È', 0xc8),
    ('É', 0xc9),
    ('Ê', 0xca),
    ('Í', 0xcd),
    ('Đ', 0xd0),
    ('Ó', 0xd3),
    ('Ô', 0xd4),
    ('Ơ', 0xd5),
    ('Ù', 0xd9),
    ('Ú', 0xda),
    ('Ư', 0xdd),
    ('à', 0xe0),
    ('á', 0xe1),
    ('â', 0xe2),
    ('ă', 0xe3),
    ('è', 0xe8),
    ('é', 0xe9),
    ('ê', 0xea),
    ('í', 0xed),
    ('đ', 0xf0),
    ('ó', 0xf3),
    ('ô', 0xf4),
    ('ơ', 0xf5),
    ('ù', 0xf9),
    ('ú', 0xfa),
    ('ư', 0xfd),
];

/// The combining tone bytes: U+0301, U+0300, U+0309, U+0303, U+0323.
const TONE_BYTES: [(Tone, u16); 5] = [
    (Tone::Sac, 0xec),
    (Tone::Huyen, 0xcc),
    (Tone::Hoi, 0xd2),
    (Tone::Nga, 0xde),
    (Tone::Nang, 0xf2),
];

pub(super) fn units_of(c: char) -> Option<Units> {
    let letter = Letter::from_char(c)?;
    let base = letter.with_tone(Tone::Ngang)?.to_char();
    let base_byte = whole_byte(base).or_else(|| single_unit(base))?;
    let tone_byte = TONE_BYTES
        .iter()
        .find(|(tone, _)| *tone == letter.tone())
        .map(|(_, byte)| *byte);
    Some(tone_byte.map_or(Units::One(base_byte), |tone_byte| Units::Two(base_byte, tone_byte)))
}

/// Reads code page bytes back to Unicode text, taking a letter written whole and one written with a tone byte alike.
pub(super) fn decode(units: &[u16]) -> String {
    compose(utf16_chars(units).map(from_code_page))
}

fn whole_byte(c: char) -> Option<u16> {
    WHOLE_LETTERS
        .iter()
        .find(|(letter, _)| *letter == c)
        .map(|(_, byte)| *byte)
}

/// The Unicode character that the byte `c` stands for (it arrives as U+00xx): a Vietnamese letter, a combining tone
/// mark, or `c` itself.
fn from_code_page(c: char) -> char {
    let unit = u32::from(c);
    let letter = WHOLE_LETTERS
        .iter()
        .find(|(_, byte)| u32::from(*byte) == unit)
        .map(|(letter, _)| *letter);
    let mark = || {
        TONE_BYTES
            .iter()
            .find(|(_, byte)| u32::from(*byte) == unit)
            .and_then(|(tone, _)| combining_mark(*tone))
    };
    letter.or_else(mark).unwrap_or(c)
}
