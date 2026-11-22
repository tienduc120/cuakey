//! The text conversion both engines offer: a text written in one code table converted to another, its case and
//! accents changed on the way. Both engines get the same input, the text written in the `from` table one character
//! per code unit, and their outputs are compared unit for unit.

use std::fmt;
use std::fs;
use std::path::Path;

use cuakey_domain::{CodeTable, ConvertCase, ConvertOptions, OnOff, convert, encode};

use crate::combo::{CODE_TABLES, table_token};
use crate::oracle::OpenKey;

/// The case options, in the order the report lists them.
pub const CASES: [ConvertCase; 5] = [
    ConvertCase::Keep,
    ConvertCase::Upper,
    ConvertCase::Lower,
    ConvertCase::SentenceStart,
    ConvertCase::EachWord,
];

/// One setting of the conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConvertCombo {
    pub from: CodeTable,
    pub to: CodeTable,
    pub case: ConvertCase,
    pub remove_accents: OnOff,
}

impl ConvertCombo {
    /// Every pair of tables with every case option, with and without accents.
    pub fn all() -> impl Iterator<Item = Self> {
        let (tables, cases) = (CODE_TABLES.len(), CASES.len());
        // Counts through the settings with the accents fastest, then the case, the table to, and the table from.
        (0..tables * tables * cases * 2).filter_map(move |n| {
            Some(Self {
                from: *CODE_TABLES.get(n / (tables * cases * 2))?,
                to: *CODE_TABLES.get(n / (cases * 2) % tables)?,
                case: *CASES.get(n / 2 % cases)?,
                remove_accents: if n % 2 == 1 { OnOff::On } else { OnOff::Off },
            })
        })
    }

    /// The setting as tokens an accepted entry's filter is matched against.
    pub fn tokens(self) -> Vec<String> {
        vec![
            format!("from-{}", table_token(self.from)),
            format!("to-{}", table_token(self.to)),
            case_token(self.case).to_owned(),
            if self.remove_accents.is_on() {
                "remove-accents"
            } else {
                "!remove-accents"
            }
            .to_owned(),
        ]
    }

    /// Every token a filter on a conversion may name.
    pub fn known_tokens() -> Vec<String> {
        let mut known: Vec<String> = CODE_TABLES
            .iter()
            .flat_map(|table| {
                [
                    format!("from-{}", table_token(*table)),
                    format!("to-{}", table_token(*table)),
                ]
            })
            .collect();
        known.extend(CASES.map(case_token).into_iter().map(str::to_owned));
        known.extend(["remove-accents".to_owned(), "!remove-accents".to_owned()]);
        known
    }

    pub const fn options(self) -> ConvertOptions {
        ConvertOptions::new(self.from, self.to)
            .with_case(self.case)
            .with_remove_accents(self.remove_accents)
    }
}

/// "from-unicode to-tcvn3 upper remove-accents": the tokens, with `remove-accents` only when it is on.
impl fmt::Display for ConvertCombo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "from-{} to-{} {}",
            table_token(self.from),
            table_token(self.to),
            case_token(self.case)
        )?;
        if self.remove_accents.is_on() {
            f.write_str(" remove-accents")?;
        }
        Ok(())
    }
}

pub const fn case_token(case: ConvertCase) -> &'static str {
    match case {
        ConvertCase::Keep => "keep",
        ConvertCase::Upper => "upper",
        ConvertCase::Lower => "lower",
        ConvertCase::SentenceStart => "sentence-start",
        ConvertCase::EachWord => "each-word",
    }
}

/// One text of `convert.txt`: the line as the file writes it, and the text it stands for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text {
    pub line: String,
    pub text: String,
}

/// The texts of `dir/convert.txt`; a line starting with `#` is a comment.
pub fn read_texts(dir: &Path) -> Result<Vec<Text>, String> {
    let path = dir.join("convert.txt");
    let file = fs::read_to_string(&path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    file.lines()
        .enumerate()
        .filter(|(_, line)| !line.is_empty() && !line.starts_with('#'))
        .map(|(at, line)| {
            let text = parse_text(line).map_err(|e| format!("{}:{}: {e}", path.display(), at + 1))?;
            Ok(Text {
                line: line.to_owned(),
                text,
            })
        })
        .collect()
}

/// A text as `convert.txt` writes it: `\n` a line break, `\\` a backslash, `\u{301}` the character U+0301, so that
/// line breaks and combining marks can be written on one visible line.
pub fn parse_text(line: &str) -> Result<String, String> {
    let mut text = String::new();
    let mut rest = line.chars();
    while let Some(c) = rest.next() {
        if c != '\\' {
            text.push(c);
            continue;
        }
        match rest.next() {
            Some('n') => text.push('\n'),
            Some('\\') => text.push('\\'),
            Some('u') => text.push(unicode_escape(&mut rest)?),
            other => {
                let shown: String = other.into_iter().collect();
                return Err(format!(r"`\{shown}` is no escape; write \n, \\, or \u{{...}}"));
            }
        }
    }
    Ok(text)
}

/// The character of a `\u{...}` escape, the `\u` already read.
fn unicode_escape(rest: &mut std::str::Chars<'_>) -> Result<char, String> {
    let malformed = || r"a \u escape is written \u{301}".to_owned();
    if rest.next() != Some('{') {
        return Err(malformed());
    }
    let digits: String = rest.by_ref().take_while(|c| *c != '}').collect();
    u32::from_str_radix(&digits, 16)
        .ok()
        .and_then(char::from_u32)
        .ok_or_else(malformed)
}

/// `text` written in `from`, one character per code unit, as both engines read it; `None` when `from` is a
/// single-byte table that cannot write all of it, since no text in that table holds a unit above a byte.
pub fn input(text: &str, from: CodeTable) -> Option<String> {
    let written = written(text, from);
    let single_byte = matches!(from, CodeTable::Tcvn3 | CodeTable::VniWindows | CodeTable::Cp1258);
    (!single_byte || written.chars().all(|c| u32::from(c) <= 0xff)).then_some(written)
}

/// `text` written in `from`; a character the table cannot write stays as Unicode writes it.
fn written(text: &str, from: CodeTable) -> String {
    let mut buffer = [0; 4];
    let mut written = String::with_capacity(text.len());
    for c in text.chars() {
        if c.len_utf16() > 1 {
            written.push(c);
            continue;
        }
        let units = encode(c.encode_utf8(&mut buffer), from);
        written.extend(units.into_iter().filter_map(|unit| char::from_u32(u32::from(unit))));
    }
    written
}

/// What `CuaKey` makes of `input`, in UTF-16 code units.
pub fn cuakey_output(input: &str, combo: ConvertCombo) -> Vec<u16> {
    convert(input, combo.options()).encode_utf16().collect()
}

/// What `OpenKey` makes of `input`, in UTF-16 code units, or `None` when its conversion stops with an error.
pub fn openkey_output(engine: &mut OpenKey, input: &str, combo: ConvertCombo) -> Option<Vec<u16>> {
    let units: Vec<u16> = input.encode_utf16().collect();
    engine.convert(&units, combo.options())
}

#[cfg(test)]
mod tests;
