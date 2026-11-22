//! The use cases on the files in the folder `CuaKey` keeps them in, each run to completion on a runtime of this
//! process's own. Every adapter answers at once, so a call blocks only for the file it reads or writes.

use std::io;
use std::path::PathBuf;

use cuakey_application::{
    ExportMacros, ExportMacrosCommand, ExportMacrosError, ImportMacros, ImportMacrosCommand, ImportMacrosError,
    ImportMode, LoadMacros, LoadMacrosError, LoadPreferences, LoadPreferencesError, MacroView, PreferencesView,
    QuickConvert, QuickConvertError, QuickConvertResult, ResetPreferences, ResetPreferencesError, SaveMacros,
    SaveMacrosError, SavePreferences, SavePreferencesError,
};
use cuakey_infrastructure::{JsonPreferencesStore, TextMacroStore, WindowsClipboard};
use platform_clock::SystemClock;
use platform_windows::{KnownFolderError, roaming_app_data};
use tokio::runtime::Runtime;

/// The folder under the user's roaming application data.
const APP_FOLDER: &str = "CuaKey";

/// Why the stores could not be opened.
#[derive(Debug, thiserror::Error)]
pub(crate) enum OpenStoresError {
    #[error("could not find the folder the app keeps its files in")]
    Folder(#[source] KnownFolderError),
    #[error("could not start the runtime the use cases run on")]
    Runtime(#[source] io::Error),
}

/// The preferences, the macros and the clipboard, through their use cases.
#[derive(Debug)]
pub(crate) struct Stores {
    dir: PathBuf,
    runtime: Runtime,
}

impl Stores {
    pub(crate) fn open() -> Result<Self, OpenStoresError> {
        let dir = roaming_app_data().map_err(OpenStoresError::Folder)?.join(APP_FOLDER);
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .map_err(OpenStoresError::Runtime)?;
        Ok(Self { dir, runtime })
    }

    fn preferences(&self) -> JsonPreferencesStore<SystemClock> {
        JsonPreferencesStore::new(self.dir.clone(), SystemClock)
    }

    fn macros(&self) -> TextMacroStore {
        TextMacroStore::new(self.dir.clone())
    }

    pub(crate) fn load_preferences(&self) -> Result<PreferencesView, LoadPreferencesError> {
        self.runtime
            .block_on(LoadPreferences::new(self.preferences()).execute())
    }

    pub(crate) fn save_preferences(&self, prefs: &PreferencesView) -> Result<(), SavePreferencesError> {
        self.runtime
            .block_on(SavePreferences::new(self.preferences()).execute(prefs))
    }

    pub(crate) fn reset_preferences(
        &self,
        current: &PreferencesView,
    ) -> Result<PreferencesView, ResetPreferencesError> {
        self.runtime
            .block_on(ResetPreferences::new(self.preferences()).execute(current))
    }

    pub(crate) fn load_macros(&self) -> Result<Vec<MacroView>, LoadMacrosError> {
        self.runtime.block_on(LoadMacros::new(self.macros()).execute())
    }

    pub(crate) fn save_macros(&self, macros: &[MacroView]) -> Result<(), SaveMacrosError> {
        self.runtime.block_on(SaveMacros::new(self.macros()).execute(macros))
    }

    pub(crate) fn import_macros(&self, path: PathBuf, mode: ImportMode) -> Result<Vec<MacroView>, ImportMacrosError> {
        let command = ImportMacrosCommand { path, mode };
        self.runtime.block_on(ImportMacros::new(self.macros()).execute(command))
    }

    pub(crate) fn export_macros(&self, path: PathBuf) -> Result<(), ExportMacrosError> {
        let command = ExportMacrosCommand { path };
        self.runtime.block_on(ExportMacros::new(self.macros()).execute(command))
    }

    /// Converts the text on the clipboard in place, with the options last stored.
    pub(crate) fn quick_convert(&self) -> Result<QuickConvertResult, QuickConvertError> {
        self.runtime
            .block_on(QuickConvert::new(WindowsClipboard, self.preferences()).execute())
    }
}
