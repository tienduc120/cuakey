//! Vietnamese letters and the code tables text is written in.
//!
//! Text is written as UTF-16 code units, as the keyboard sends it; a single-byte table's byte `b` is the unit `b`
//! (U+00xx). A character a table cannot represent is written as Unicode writes it.
//!
//! The tables are written from these public charts:
//! - Windows code page 1258: Unicode's mapping, <https://www.unicode.org/Public/MAPPINGS/VENDORS/MICSFT/WINDOWS/CP1258.TXT>
//! - TCVN3 (TCVN 5712:1993 VN3): the VSCII chart, <https://en.wikipedia.org/wiki/VSCII>
//! - VNI Windows: the Vietnamese Unicode FAQs chart, <https://vietunicode.sourceforge.net/charset/>

mod cp1258;
mod letter;
mod tcvn3;
mod units;
mod vni;

use alloc::string::String;
use alloc::vec::Vec;

pub use letter::{BaseLetter, Letter, LetterCase, Tone};
use units::{Units, single_unit, utf16_chars};

use crate::CodeTable;

/// The combining marks Unicode compound and the decoders write the tones with.
const TONE_MARKS: [(Tone, char); 5] = [
    (Tone::Sac, '\u{301}'),
    (Tone::Huyen, '\u{300}'),
    (Tone::Hoi, '\u{309}'),
    (Tone::Nga, '\u{303}'),
    (Tone::Nang, '\u{323}'),
];

/// `text` written in `table`.
pub fn encode(text: &str, table: CodeTable) -> Vec<u16> {
    let mut units = Vec::with_capacity(text.len());
    for c in text.chars() {
        units_for(c, table).push_onto(&mut units);
    }
    units
}

/// Reads text written in `table` back to Unicode, every letter precomposed.
pub fn decode(units: &[u16], table: CodeTable) -> String {
    match table {
        CodeTable::Unicode => utf16_chars(units).collect(),
        CodeTable::UnicodeCompound => compose(utf16_chars(units)),
        CodeTable::Cp1258 => cp1258::decode(units),
        CodeTable::Tcvn3 => tcvn3::TABLE.decode(units),
        CodeTable::VniWindows => vni::TABLE.decode(units),
    }
}

/// How many code units `c` takes in `table`: the backspaces that delete it.
pub fn units_in(c: char, table: CodeTable) -> usize {
    units_for(c, table).len()
}

fn units_for(c: char, table: CodeTable) -> Units {
    let in_table = match table {
        CodeTable::Unicode => None,
        CodeTable::UnicodeCompound => compound_units(c),
        CodeTable::Cp1258 => cp1258::units_of(c),
        CodeTable::Tcvn3 => tcvn3::TABLE.units_of(c),
        CodeTable::VniWindows => vni::TABLE.units_of(c),
    };
    in_table.unwrap_or_else(|| Units::of_char(c))
}

/// Unicode compound writes a toned letter as the letter without its tone, then the tone's combining mark.
fn compound_units(c: char) -> Option<Units> {
    let letter = Letter::from_char(c)?;
    let mark = combining_mark(letter.tone())?;
    let base = letter.with_tone(Tone::Ngang)?.to_char();
    Some(Units::Two(single_unit(base)?, single_unit(mark)?))
}

fn combining_mark(tone: Tone) -> Option<char> {
    TONE_MARKS.iter().find(|(t, _)| *t == tone).map(|(_, mark)| *mark)
}

/// Joins each vowel and the combining tone mark after it into one precomposed letter. A mark after anything else
/// stays as it is.
fn compose(chars: impl Iterator<Item = char>) -> String {
    let mut text = String::new();
    for c in chars {
        let toned = text.chars().next_back().and_then(|before| with_mark(before, c));
        if let Some(letter) = toned {
            text.pop();
            text.push(letter);
        } else {
            text.push(c);
        }
    }
    text
}

/// `vowel` carrying the tone `mark` writes, when `vowel` is a vowel with no tone yet and `mark` a tone mark.
fn with_mark(vowel: char, mark: char) -> Option<char> {
    let tone = TONE_MARKS.iter().find(|(_, m)| *m == mark).map(|(tone, _)| *tone)?;
    let letter = Letter::from_char(vowel).filter(|letter| letter.tone() == Tone::Ngang)?;
    letter.with_tone(tone).map(Letter::to_char)
}

#[cfg(test)]
mod tests;
