use std::path::PathBuf;

use cuakey_domain::{MacroEntry, MacroError, MacroTable};

use crate::{ImportMode, MacroStore, StoreError};

/// One macro as plain values: the shortcut typed and the text it expands to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroView {
    pub shortcut: String,
    pub expansion: String,
}

/// Why a macro was refused, naming the shortcut so the user can find it.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidMacro {
    #[error("a shortcut must not be empty")]
    EmptyShortcut,
    #[error("the shortcut {0} must not contain whitespace")]
    Whitespace(String),
    #[error("the shortcut {0} must not contain a colon")]
    Colon(String),
}

/// The table holding `macros`; a later shortcut that repeats an earlier one wins.
pub(crate) fn macro_table(macros: &[MacroView]) -> Result<MacroTable, InvalidMacro> {
    let mut table = MacroTable::default();
    for view in macros {
        let entry = MacroEntry::new(&view.shortcut, &view.expansion).map_err(|e| invalid(e, &view.shortcut))?;
        table.add(entry);
    }
    Ok(table)
}

/// The macros in `table`, in shortcut order.
pub(crate) fn macro_views(table: &MacroTable) -> Vec<MacroView> {
    table
        .entries()
        .map(|entry| MacroView {
            shortcut: entry.shortcut().to_owned(),
            expansion: entry.expansion().to_owned(),
        })
        .collect()
}

fn invalid(error: MacroError, shortcut: &str) -> InvalidMacro {
    match error {
        MacroError::Empty => InvalidMacro::EmptyShortcut,
        MacroError::Whitespace => InvalidMacro::Whitespace(shortcut.to_owned()),
        MacroError::Colon => InvalidMacro::Colon(shortcut.to_owned()),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LoadMacrosError {
    #[error("could not load the macros")]
    Storage(#[source] StoreError),
}

#[derive(Debug, thiserror::Error)]
pub enum SaveMacrosError {
    #[error("invalid macro: {0}")]
    Invalid(#[from] InvalidMacro),
    #[error("could not save the macros")]
    Storage(#[source] StoreError),
}

#[derive(Debug, thiserror::Error)]
pub enum ImportMacrosError {
    #[error("could not import the macros")]
    Storage(#[source] StoreError),
}

#[derive(Debug, thiserror::Error)]
pub enum ExportMacrosError {
    #[error("could not export the macros")]
    Storage(#[source] StoreError),
}

/// Reads the stored macros.
#[derive(Debug)]
pub struct LoadMacros<S> {
    store: S,
}

impl<S: MacroStore> LoadMacros<S> {
    pub const fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn execute(&self) -> Result<Vec<MacroView>, LoadMacrosError> {
        let macros = self.store.load().await.map_err(LoadMacrosError::Storage)?;
        Ok(macro_views(&macros))
    }
}

/// Stores the macros the macro window shows, once every shortcut is valid.
#[derive(Debug)]
pub struct SaveMacros<S> {
    store: S,
}

impl<S: MacroStore> SaveMacros<S> {
    pub const fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn execute(&self, macros: &[MacroView]) -> Result<(), SaveMacrosError> {
        let table = macro_table(macros)?;
        self.store.save(&table).await.map_err(SaveMacrosError::Storage)
    }

    #[cfg(test)]
    const fn store(&self) -> &S {
        &self.store
    }
}

/// A macro file to read into the stored macros.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportMacrosCommand {
    pub path: PathBuf,
    pub mode: ImportMode,
}

/// Reads a macro file into the stored macros.
#[derive(Debug)]
pub struct ImportMacros<S> {
    store: S,
}

impl<S: MacroStore> ImportMacros<S> {
    pub const fn new(store: S) -> Self {
        Self { store }
    }

    /// The macros stored once the file is read in.
    pub async fn execute(&self, cmd: ImportMacrosCommand) -> Result<Vec<MacroView>, ImportMacrosError> {
        let macros = self
            .store
            .import(&cmd.path, cmd.mode)
            .await
            .map_err(ImportMacrosError::Storage)?;
        Ok(macro_views(&macros))
    }

    #[cfg(test)]
    const fn store(&self) -> &S {
        &self.store
    }
}

/// Where to write the stored macros.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportMacrosCommand {
    pub path: PathBuf,
}

/// Writes the stored macros to a macro file.
#[derive(Debug)]
pub struct ExportMacros<S> {
    store: S,
}

impl<S: MacroStore> ExportMacros<S> {
    pub const fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn execute(&self, cmd: ExportMacrosCommand) -> Result<(), ExportMacrosError> {
        self.store.export(&cmd.path).await.map_err(ExportMacrosError::Storage)
    }

    #[cfg(test)]
    const fn store(&self) -> &S {
        &self.store
    }
}

#[cfg(test)]
mod tests;
