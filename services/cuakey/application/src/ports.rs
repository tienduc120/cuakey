use std::error::Error;
use std::path::Path;

use cuakey_domain::{MacroTable, Preferences};

use crate::WindowPlaces;

/// Where the preferences are kept between runs. The futures are `Send` so a use case can run on any runtime thread.
pub trait PreferencesStore: Send + Sync {
    /// The stored preferences, or `None` when none were saved yet.
    fn load(&self) -> impl Future<Output = Result<Option<Preferences>, StoreError>> + Send;

    fn save(&self, preferences: &Preferences) -> impl Future<Output = Result<(), StoreError>> + Send;
}

/// Where the macros are kept, and the macro files a user exchanges.
pub trait MacroStore: Send + Sync {
    /// The stored macros; none saved yet is an empty table.
    fn load(&self) -> impl Future<Output = Result<MacroTable, StoreError>> + Send;

    fn save(&self, macros: &MacroTable) -> impl Future<Output = Result<(), StoreError>> + Send;

    /// Reads the macro file at `path` into the stored macros and returns the macros stored now.
    fn import(&self, path: &Path, mode: ImportMode) -> impl Future<Output = Result<MacroTable, StoreError>> + Send;

    /// Writes the stored macros to a macro file at `path`.
    fn export(&self, path: &Path) -> impl Future<Output = Result<(), StoreError>> + Send;
}

/// Where each window's last place on screen is kept between runs.
pub trait WindowPlacesStore: Send + Sync {
    /// The saved places; none saved yet is an empty set.
    fn load(&self) -> impl Future<Output = Result<WindowPlaces, StoreError>> + Send;

    fn save(&self, places: &WindowPlaces) -> impl Future<Output = Result<(), StoreError>> + Send;
}

/// What an imported macro file does to the macros already stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportMode {
    /// Keeps the stored macros; a shortcut in the file takes the file's expansion.
    Append,
    /// The file's macros take the place of the stored ones.
    Replace,
}

/// The system clipboard, which quick convert reads from and writes back to.
pub trait Clipboard: Send + Sync {
    /// The text on the clipboard, or `None` when it holds no text.
    fn read_text(&self) -> impl Future<Output = Result<Option<String>, ClipboardError>> + Send;

    fn write_text(&self, text: &str) -> impl Future<Output = Result<(), ClipboardError>> + Send;
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("storage failed")]
    Storage(#[source] Box<dyn Error + Send + Sync>),
}

#[derive(Debug, thiserror::Error)]
pub enum ClipboardError {
    /// Another app holds the clipboard open, or the system refused.
    #[error("the clipboard is unavailable")]
    Unavailable(#[source] Box<dyn Error + Send + Sync>),
}
