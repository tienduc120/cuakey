//! Converting text from one code table to another, with the case and accents changed on the way.
//!
//! Text is held as one `char` per code unit of its table, as `encode` yields the units. A character that does not fit
//! in one unit (an emoji) is not a code unit of any table here and passes through unchanged.

use alloc::string::String;
use alloc::vec::Vec;

use crate::{BaseLetter, CodeTable, Letter, OnOff, decode, encode};

/// What a conversion does to the case of the letters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvertCase {
    Keep,
    Upper,
    Lower,
    /// Raises the first letter of each sentence; the other letters stay as they are.
    SentenceStart,
    /// Raises the first letter of each word; the other letters stay as they are.
    EachWord,
}

/// How to convert a text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConvertOptions {
    from: CodeTable,
    to: CodeTable,
    case: ConvertCase,
    remove_accents: OnOff,
}

impl ConvertOptions {
    /// Converts from one table to another and changes nothing else.
    pub const fn new(from: CodeTable, to: CodeTable) -> Self {
        Self {
            from,
            to,
            case: ConvertCase::Keep,
            remove_accents: OnOff::Off,
        }
    }

    #[must_use]
    pub const fn with_case(self, case: ConvertCase) -> Self {
        Self { case, ..self }
    }

    /// Writes every Vietnamese letter as its plain Latin letter: `Tiếng Việt` becomes `Tieng Viet`.
    #[must_use]
    pub const fn with_remove_accents(self, remove_accents: OnOff) -> Self {
        Self { remove_accents, ..self }
    }

    pub const fn from(self) -> CodeTable {
        self.from
    }

    pub const fn to(self) -> CodeTable {
        self.to
    }

    pub const fn case(self) -> ConvertCase {
        self.case
    }

    pub const fn remove_accents(self) -> OnOff {
        self.remove_accents
    }
}

/// `text`, written in the `from` table, converted as `options` say; the result is written in the `to` table.
pub fn convert(text: &str, options: ConvertOptions) -> String {
    let unicode = decode_text(text, options.from);
    let plain = match options.remove_accents {
        OnOff::On => without_accents(&unicode),
        OnOff::Off => unicode,
    };
    let cased = change_case(&plain, options.case);
    encode_text(&cased, options.to)
}

/// Reads `text` as the code units of `table`, leaving the characters that are no unit as they are.
fn decode_text(text: &str, table: CodeTable) -> String {
    let mut out = String::with_capacity(text.len());
    let mut run: Vec<u16> = Vec::new();
    for c in text.chars() {
        if let Ok(unit) = u16::try_from(u32::from(c)) {
            run.push(unit);
        } else {
            out.push_str(&decode(&run, table));
            run.clear();
            out.push(c);
        }
    }
    out.push_str(&decode(&run, table));
    out
}

fn encode_text(text: &str, table: CodeTable) -> String {
    let mut out = String::with_capacity(text.len());
    let mut buffer = [0; 4];
    for c in text.chars() {
        if u16::try_from(u32::from(c)).is_err() {
            out.push(c);
            continue;
        }
        let units = encode(c.encode_utf8(&mut buffer), table);
        out.extend(units.into_iter().filter_map(|unit| char::from_u32(u32::from(unit))));
    }
    out
}

/// The combining marks Unicode compound writes tones with, and the other combining diacritics.
const COMBINING_MARKS: core::ops::RangeInclusive<char> = '\u{300}'..='\u{36f}';

fn without_accents(text: &str) -> String {
    text.chars()
        .filter(|c| !COMBINING_MARKS.contains(c))
        .map(plain_letter)
        .collect()
}

/// `c` as the plain Latin letter when it is a Vietnamese letter.
fn plain_letter(c: char) -> char {
    Letter::from_char(c)
        .and_then(|letter| Letter::from_parts(plain_base(letter.base()), crate::Tone::Ngang, letter.case()))
        .map_or(c, Letter::to_char)
}

const fn plain_base(base: BaseLetter) -> BaseLetter {
    match base {
        BaseLetter::ABreve | BaseLetter::ACircumflex => BaseLetter::A,
        BaseLetter::ECircumflex => BaseLetter::E,
        BaseLetter::OCircumflex | BaseLetter::OHorn => BaseLetter::O,
        BaseLetter::UHorn => BaseLetter::U,
        BaseLetter::DStroke => BaseLetter::D,
        BaseLetter::A
        | BaseLetter::E
        | BaseLetter::I
        | BaseLetter::O
        | BaseLetter::U
        | BaseLetter::Y
        | BaseLetter::D
        | BaseLetter::B
        | BaseLetter::C
        | BaseLetter::F
        | BaseLetter::G
        | BaseLetter::H
        | BaseLetter::J
        | BaseLetter::K
        | BaseLetter::L
        | BaseLetter::M
        | BaseLetter::N
        | BaseLetter::P
        | BaseLetter::Q
        | BaseLetter::R
        | BaseLetter::S
        | BaseLetter::T
        | BaseLetter::V
        | BaseLetter::W
        | BaseLetter::X
        | BaseLetter::Z => base,
    }
}

fn change_case(text: &str, case: ConvertCase) -> String {
    match case {
        ConvertCase::Keep => text.into(),
        ConvertCase::Upper => text.to_uppercase(),
        ConvertCase::Lower => text.to_lowercase(),
        ConvertCase::SentenceStart => raise_sentence_starts(text),
        ConvertCase::EachWord => raise_word_starts(text),
    }
}

/// Where a conversion stands in the sentence whose start it raises.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sentence {
    /// Inside a sentence.
    Going,
    /// Right after a `.`, `!`, or `?`, which ends the sentence only once a space follows: not in www.example.com.
    Stopped,
    /// The next letter starts a sentence.
    Opens,
}

/// Raises the first letter of the text and of each sentence: the first letter after a line break, or after a `.`, `!`,
/// or `?` and the space after it. Quotes and brackets in between do not start the sentence, but a digit does, so
/// `3 apples` stays.
fn raise_sentence_starts(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut sentence = Sentence::Opens;
    for c in text.chars() {
        if c.is_alphabetic() && sentence == Sentence::Opens {
            out.extend(c.to_uppercase());
        } else {
            out.push(c);
        }
        sentence = match (c, sentence) {
            ('\n', _) => Sentence::Opens,
            (_, Sentence::Stopped) if c.is_whitespace() => Sentence::Opens,
            ('.' | '!' | '?', _) => Sentence::Stopped,
            _ if c.is_alphanumeric() => Sentence::Going,
            _ => sentence,
        };
    }
    out
}

/// Raises the first letter of each word, a word being a run of letters and digits.
fn raise_word_starts(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_word = false;
    for c in text.chars() {
        if c.is_alphabetic() && !in_word {
            out.extend(c.to_uppercase());
        } else {
            out.push(c);
        }
        in_word = c.is_alphanumeric();
    }
    out
}

#[cfg(test)]
mod tests;
