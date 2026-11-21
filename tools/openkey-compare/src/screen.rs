//! The text box both engines type into: UTF-16 code units, the caret always at the end. A single-byte code table's
//! byte `b` arrives as the unit `b`, as a keyboard sends it.

use std::fmt::Write as _;

use cuakey_domain::{CodeTable, KeyCode, KeyPress, decode};

use crate::corpus::char_typed;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Screen {
    units: Vec<u16>,
}

impl Screen {
    pub fn units(&self) -> &[u16] {
        &self.units
    }

    /// One Backspace deletes one code unit, whatever character it was part of.
    pub fn backspace(&mut self) {
        self.units.pop();
    }

    pub fn push(&mut self, units: &[u16]) {
        self.units.extend_from_slice(units);
    }

    /// What the text box does with `key` when it reaches it: Backspace deletes, a key that types a character types it,
    /// Ctrl or Alt makes it a shortcut that types nothing, and any other key types nothing either.
    pub fn key(&mut self, key: KeyPress) {
        if key.code() == KeyCode::Backspace {
            self.backspace();
        } else if let Some(c) = text_of(key) {
            let mut buffer = [0; 2];
            self.push(c.encode_utf16(&mut buffer));
        }
    }
}

/// The character `key` types in a text box, if any.
pub fn text_of(key: KeyPress) -> Option<char> {
    if key.mods().ctrl_or_alt().is_on() {
        return None;
    }
    match key.code() {
        KeyCode::Enter => return Some('\n'),
        KeyCode::Tab => return Some('\t'),
        _ => {}
    }
    let c = char_typed(key.code(), key.mods().shift())?;
    // Caps Lock turns the case of a letter around, and leaves every other key alone.
    Some(if key.mods().caps_lock().is_on() && c.is_ascii_alphabetic() {
        if c.is_ascii_lowercase() {
            c.to_ascii_uppercase()
        } else {
            c.to_ascii_lowercase()
        }
    } else {
        c
    })
}

/// `units` as the report shows them: the text for Unicode, and for any other table the units in hex followed by the
/// text they read as in that table.
pub fn show(units: &[u16], table: CodeTable) -> String {
    let text = decode(units, table);
    if table == CodeTable::Unicode {
        return format!("`{}`", escape(&text));
    }
    let mut hex = String::new();
    for unit in units {
        // Writing to a String cannot fail.
        write!(hex, "{}{unit:04x}", if hex.is_empty() { "" } else { " " }).unwrap_or_default();
    }
    if hex.is_empty() {
        hex.push_str("(empty)");
    }
    format!("{hex} (`{}`)", escape(&text))
}

fn escape(text: &str) -> String {
    text.replace('\n', "\\n").replace('\t', "\\t").replace('`', "\\`")
}

#[cfg(test)]
mod tests;
