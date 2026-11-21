use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Reverse;

use super::letter::{LOWER_VOWELS, UPPER_VOWELS};

/// The UTF-16 code units one character is written as. No code table needs more than two.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Units {
    One(u16),
    Two(u16, u16),
}

impl Units {
    /// `c` as Unicode writes it: what a table writes for a character it cannot represent.
    pub(super) fn of_char(c: char) -> Self {
        let mut buffer = [0; 2];
        let len = c.encode_utf16(&mut buffer).len();
        let [first, second] = buffer;
        if len == 1 {
            Self::One(first)
        } else {
            Self::Two(first, second)
        }
    }

    pub(super) const fn len(self) -> usize {
        match self {
            Self::One(_) => 1,
            Self::Two(..) => 2,
        }
    }

    /// Whether `units` starts with these.
    fn is_prefix_of(self, units: &[u16]) -> bool {
        match self {
            Self::One(first) => units.starts_with(&[first]),
            Self::Two(first, second) => units.starts_with(&[first, second]),
        }
    }

    pub(super) fn push_onto(self, units: &mut Vec<u16>) {
        match self {
            Self::One(first) => units.push(first),
            Self::Two(first, second) => units.extend([first, second]),
        }
    }
}

/// A code table that writes every Vietnamese letter as a fixed run of units. Its grids line up with the letter grids:
/// rows a ă â e ê i o ô ơ u ư y, columns ngang, sắc, huyền, hỏi, ngã, nặng.
#[derive(Debug)]
pub(super) struct UnitTable {
    lower: &'static [[Units; 6]; 12],
    upper: &'static [[Units; 6]; 12],
    small_d_stroke: Units,
    capital_d_stroke: Units,
}

impl UnitTable {
    pub(super) const fn new(
        lower: &'static [[Units; 6]; 12],
        upper: &'static [[Units; 6]; 12],
        small_d_stroke: Units,
        capital_d_stroke: Units,
    ) -> Self {
        Self {
            lower,
            upper,
            small_d_stroke,
            capital_d_stroke,
        }
    }

    pub(super) fn units_of(&self, c: char) -> Option<Units> {
        self.entries().find(|(letter, _)| *letter == c).map(|(_, units)| units)
    }

    /// Reads `units` back to text: each run of units is the letter it stands for, the longest run first, and any
    /// other unit is the Unicode character it is.
    pub(super) fn decode(&self, units: &[u16]) -> String {
        let mut text = String::new();
        let mut rest = units;
        while !rest.is_empty() {
            let (c, used) = self.letter_at(rest).unwrap_or_else(|| char_at(rest));
            text.push(c);
            rest = rest.get(used..).unwrap_or_default();
        }
        text
    }

    /// The letter whose units `units` starts with, and how many units it takes. A table writing two letters alike
    /// (TCVN3 writes a toned capital as its lower-case letter) reads them back as the one listed first.
    fn letter_at(&self, units: &[u16]) -> Option<(char, usize)> {
        self.entries()
            .filter(|(_, letter_units)| letter_units.is_prefix_of(units))
            .min_by_key(|(_, letter_units)| Reverse(letter_units.len()))
            .map(|(letter, letter_units)| (letter, letter_units.len()))
    }

    /// Every letter in the table with its units, lower case first.
    fn entries(&self) -> impl Iterator<Item = (char, Units)> + '_ {
        let lower = LOWER_VOWELS.iter().flatten().zip(self.lower.iter().flatten());
        let upper = UPPER_VOWELS.iter().flatten().zip(self.upper.iter().flatten());
        lower
            .chain(upper)
            .map(|(letter, units)| (*letter, *units))
            .chain([('đ', self.small_d_stroke), ('Đ', self.capital_d_stroke)])
    }
}

/// The Unicode character `units` starts with, and how many units it takes; a broken surrogate reads as U+FFFD.
fn char_at(units: &[u16]) -> (char, usize) {
    match char::decode_utf16(units.iter().copied()).next() {
        Some(Ok(c)) => (c, c.len_utf16()),
        Some(Err(_)) | None => (char::REPLACEMENT_CHARACTER, 1),
    }
}

/// The one UTF-16 unit `c` is, when it fits in one.
pub(super) fn single_unit(c: char) -> Option<u16> {
    u16::try_from(u32::from(c)).ok()
}

/// `units` read as UTF-16, a broken surrogate as U+FFFD.
pub(super) fn utf16_chars(units: &[u16]) -> impl Iterator<Item = char> + '_ {
    char::decode_utf16(units.iter().copied()).map(|c| c.unwrap_or(char::REPLACEMENT_CHARACTER))
}
