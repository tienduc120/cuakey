//! Macros: short words that expand to longer text as they are typed.

use alloc::collections::BTreeMap;
use alloc::string::String;

use crate::OnOff;

/// Why a macro was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum MacroError {
    #[error("the shortcut must not be empty")]
    Empty,
    #[error("the shortcut must not contain whitespace")]
    Whitespace,
    #[error("the shortcut must not contain a colon")]
    Colon,
}

/// One shortcut and the text it expands to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroEntry {
    shortcut: String,
    expansion: String,
}

impl MacroEntry {
    /// A shortcut is a single word the macro file can write before its first colon, so it holds no whitespace and no
    /// colon. The expansion may hold any character.
    ///
    /// # Errors
    ///
    /// Returns a [`MacroError`] when the shortcut is empty or holds whitespace or a colon.
    pub fn new(shortcut: &str, expansion: &str) -> Result<Self, MacroError> {
        if shortcut.is_empty() {
            return Err(MacroError::Empty);
        }
        if shortcut.contains(char::is_whitespace) {
            return Err(MacroError::Whitespace);
        }
        if shortcut.contains(':') {
            return Err(MacroError::Colon);
        }
        Ok(Self {
            shortcut: shortcut.into(),
            expansion: expansion.into(),
        })
    }

    pub fn shortcut(&self) -> &str {
        &self.shortcut
    }

    pub fn expansion(&self) -> &str {
        &self.expansion
    }
}

/// The macros, ordered by shortcut.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MacroTable {
    entries: BTreeMap<String, MacroEntry>,
}

impl MacroTable {
    /// Adds the macro; a shortcut that is already there gets the new expansion.
    pub fn add(&mut self, entry: MacroEntry) {
        self.entries.insert(entry.shortcut.clone(), entry);
    }

    /// Takes the macro with this shortcut out, and gives it back.
    pub fn remove(&mut self, shortcut: &str) -> Option<MacroEntry> {
        self.entries.remove(shortcut)
    }

    pub fn entries(&self) -> impl Iterator<Item = &MacroEntry> {
        self.entries.values()
    }

    /// What `word` expands to. A shortcut matches exactly; with `match_caps` on, a Capitalised word or an ALL CAPS
    /// word also matches the shortcut in lower case, and the expansion takes the word's case.
    pub fn lookup(&self, word: &str, match_caps: OnOff) -> Option<String> {
        if let Some(entry) = self.entries.get(word) {
            return Some(entry.expansion.clone());
        }
        if match_caps == OnOff::Off {
            return None;
        }
        let case = WordCase::of(word)?;
        let entry = self.entries.get(word.to_lowercase().as_str())?;
        Some(case.apply(&entry.expansion))
    }
}

/// How a typed word is written in capitals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WordCase {
    /// Only the first letter is a capital.
    Capitalised,
    /// Two or more letters, all capitals.
    AllCaps,
}

impl WordCase {
    /// The case `word` is written in, or `None` when it is lower case or mixed.
    fn of(word: &str) -> Option<Self> {
        let mut chars = word.chars();
        let first = chars.next()?;
        if !first.is_uppercase() {
            return None;
        }
        let rest = chars.as_str();
        if !rest.chars().any(char::is_uppercase) {
            Some(Self::Capitalised)
        } else if rest.chars().all(|c| c.is_uppercase() || !c.is_lowercase()) {
            Some(Self::AllCaps)
        } else {
            None
        }
    }

    fn apply(self, expansion: &str) -> String {
        match self {
            Self::AllCaps => expansion.to_uppercase(),
            Self::Capitalised => {
                let mut chars = expansion.chars();
                chars
                    .next()
                    .map(|first| first.to_uppercase().chain(chars).collect())
                    .unwrap_or_default()
            }
        }
    }
}

#[cfg(test)]
mod tests;
