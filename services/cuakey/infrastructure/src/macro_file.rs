//! The macros as a text file other typing programs read too: one `shortcut:expansion` per line.

use std::future::ready;
use std::io;
use std::path::{Path, PathBuf};

use cuakey_application::{ImportMode, MacroStore, StoreError};
use cuakey_domain::{MacroEntry, MacroTable};

use crate::files::{FileError, read_if_present, write_replacing};

const FILE_NAME: &str = "macros.txt";

/// The first line of every file written, which other programs look for.
const HEADER: &str = ";Compatible OpenKey Macro Data file for UniKey*** version=1 ***";

/// Keeps the macros in `macros.txt` in a folder.
#[derive(Debug)]
pub struct TextMacroStore {
    dir: PathBuf,
}

impl TextMacroStore {
    pub const fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    fn path(&self) -> PathBuf {
        self.dir.join(FILE_NAME)
    }

    fn read_stored(&self) -> Result<MacroTable, FileError> {
        read_macros(&self.path()).map(Option::unwrap_or_default)
    }

    fn save_blocking(&self, macros: &MacroTable) -> Result<(), FileError> {
        write_replacing(&self.path(), format_macros(macros).as_bytes())
    }

    fn import_blocking(&self, path: &Path, mode: ImportMode) -> Result<MacroTable, FileError> {
        let imported = read_macros(path)?.ok_or_else(|| FileError::Read {
            path: path.to_path_buf(),
            source: io::ErrorKind::NotFound.into(),
        })?;
        let stored = match mode {
            ImportMode::Append => {
                let mut stored = self.read_stored()?;
                for entry in imported.entries() {
                    stored.add(entry.clone());
                }
                stored
            }
            ImportMode::Replace => imported,
        };
        self.save_blocking(&stored)?;
        Ok(stored)
    }
}

// The files are small, and the calls have nothing to await, so each answer is ready at once.
impl MacroStore for TextMacroStore {
    fn load(&self) -> impl Future<Output = Result<MacroTable, StoreError>> + Send {
        ready(self.read_stored().map_err(StoreError::from))
    }

    fn save(&self, macros: &MacroTable) -> impl Future<Output = Result<(), StoreError>> + Send {
        ready(self.save_blocking(macros).map_err(StoreError::from))
    }

    fn import(&self, path: &Path, mode: ImportMode) -> impl Future<Output = Result<MacroTable, StoreError>> + Send {
        ready(self.import_blocking(path, mode).map_err(StoreError::from))
    }

    fn export(&self, path: &Path) -> impl Future<Output = Result<(), StoreError>> + Send {
        let exported = self
            .read_stored()
            .and_then(|stored| write_replacing(path, format_macros(&stored).as_bytes()));
        ready(exported.map_err(StoreError::from))
    }
}

/// The macros in the file, or `None` when there is no such file.
fn read_macros(path: &Path) -> Result<Option<MacroTable>, FileError> {
    let Some(bytes) = read_if_present(path)? else {
        return Ok(None);
    };
    let text = String::from_utf8(bytes).map_err(|error| FileError::Read {
        path: path.to_path_buf(),
        source: io::Error::new(io::ErrorKind::InvalidData, error),
    })?;
    Ok(Some(parse_macros(&text)))
}

/// Reads the lines of a macro file. A byte order mark and CRLF line ends are accepted; blank lines, `;` comments, and
/// lines that are no valid macro are skipped, and a later line wins over an earlier one with the same shortcut.
fn parse_macros(text: &str) -> MacroTable {
    let mut table = MacroTable::default();
    let mut skipped = 0_usize;
    for line in text.strip_prefix('\u{feff}').unwrap_or(text).lines() {
        if line.trim().is_empty() || line.starts_with(';') {
            continue;
        }
        let entry = line
            .split_once(':')
            .and_then(|(shortcut, expansion)| MacroEntry::new(shortcut, expansion).ok());
        match entry {
            Some(entry) => table.add(entry),
            None => skipped = skipped.saturating_add(1),
        }
    }
    if skipped > 0 {
        tracing::warn!(skipped, "lines of the macro file are no macro and were left out");
    }
    table
}

/// The file's text: the header, then a line per macro, with LF ends and no byte order mark.
fn format_macros(macros: &MacroTable) -> String {
    let mut text = String::from(HEADER);
    text.push('\n');
    for entry in macros.entries() {
        text.push_str(entry.shortcut());
        text.push(':');
        text.push_str(entry.expansion());
        text.push('\n');
    }
    text
}

#[cfg(test)]
mod tests;
