//! Hand-written fakes of this crate's ports, for this service's tests (feature `test-support`).

use std::collections::BTreeMap;
use std::future::ready;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};

use cuakey_domain::{MacroTable, Preferences};

use crate::macros::{macro_table, macro_views};
use crate::{
    Clipboard, ClipboardError, ImportMode, InvalidMacro, MacroStore, MacroView, PreferencesStore, PreferencesView,
    StoreError, UpdateError,
};

/// Whether a fake does its work or fails the way a full disk or a clipboard another app holds would.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Health {
    Working,
    Broken,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn storage_failure(what: &str) -> StoreError {
    StoreError::Storage(format!("disk full writing {what}").into())
}

/// Holds preferences in memory and records every save it is asked for.
#[derive(Debug)]
pub struct FakePreferencesStore {
    stored: Mutex<Option<Preferences>>,
    saved: Mutex<Vec<PreferencesView>>,
    health: Health,
}

impl FakePreferencesStore {
    /// A store nothing was saved to yet.
    pub const fn new(health: Health) -> Self {
        Self {
            stored: Mutex::new(None),
            saved: Mutex::new(Vec::new()),
            health,
        }
    }

    /// A working store that holds the preferences `prefs` describes.
    pub fn holding(prefs: &PreferencesView) -> Result<Self, UpdateError> {
        Ok(Self {
            stored: Mutex::new(Some(prefs.to_preferences()?)),
            ..Self::new(Health::Working)
        })
    }

    /// Every value it was asked to save, in order.
    pub fn saved(&self) -> Vec<PreferencesView> {
        lock(&self.saved).clone()
    }
}

impl PreferencesStore for FakePreferencesStore {
    fn load(&self) -> impl Future<Output = Result<Option<Preferences>, StoreError>> + Send {
        ready(match self.health {
            Health::Working => Ok(lock(&self.stored).clone()),
            Health::Broken => Err(storage_failure("preferences")),
        })
    }

    fn save(&self, preferences: &Preferences) -> impl Future<Output = Result<(), StoreError>> + Send {
        lock(&self.saved).push(PreferencesView::of(preferences));
        ready(match self.health {
            Health::Working => {
                *lock(&self.stored) = Some(preferences.clone());
                Ok(())
            }
            Health::Broken => Err(storage_failure("preferences")),
        })
    }
}

/// Holds macros and macro files in memory.
#[derive(Debug)]
pub struct FakeMacroStore {
    stored: Mutex<MacroTable>,
    files: Mutex<BTreeMap<PathBuf, MacroTable>>,
    health: Health,
}

impl FakeMacroStore {
    /// A store with no macros and no files.
    pub fn new(health: Health) -> Self {
        Self {
            stored: Mutex::new(MacroTable::default()),
            files: Mutex::new(BTreeMap::new()),
            health,
        }
    }

    /// A working store that holds `macros`.
    pub fn holding(macros: &[MacroView]) -> Result<Self, InvalidMacro> {
        Ok(Self {
            stored: Mutex::new(macro_table(macros)?),
            ..Self::new(Health::Working)
        })
    }

    /// The store with a macro file at `path` that holds `macros`.
    pub fn with_file(self, path: &Path, macros: &[MacroView]) -> Result<Self, InvalidMacro> {
        lock(&self.files).insert(path.to_path_buf(), macro_table(macros)?);
        Ok(self)
    }

    pub fn stored(&self) -> Vec<MacroView> {
        macro_views(&lock(&self.stored))
    }

    /// The macros in the file at `path`, if there is one.
    pub fn file(&self, path: &Path) -> Option<Vec<MacroView>> {
        lock(&self.files).get(path).map(macro_views)
    }

    fn import_now(&self, path: &Path, mode: ImportMode) -> Result<MacroTable, StoreError> {
        let file = lock(&self.files)
            .get(path)
            .cloned()
            .ok_or_else(|| StoreError::Storage(format!("no macro file at {}", path.display()).into()))?;
        let mut stored = lock(&self.stored);
        match mode {
            ImportMode::Append => file.entries().for_each(|entry| stored.add(entry.clone())),
            ImportMode::Replace => *stored = file,
        }
        Ok(stored.clone())
    }
}

impl MacroStore for FakeMacroStore {
    fn load(&self) -> impl Future<Output = Result<MacroTable, StoreError>> + Send {
        ready(match self.health {
            Health::Working => Ok(lock(&self.stored).clone()),
            Health::Broken => Err(storage_failure("macros")),
        })
    }

    fn save(&self, macros: &MacroTable) -> impl Future<Output = Result<(), StoreError>> + Send {
        ready(match self.health {
            Health::Working => {
                *lock(&self.stored) = macros.clone();
                Ok(())
            }
            Health::Broken => Err(storage_failure("macros")),
        })
    }

    fn import(&self, path: &Path, mode: ImportMode) -> impl Future<Output = Result<MacroTable, StoreError>> + Send {
        ready(match self.health {
            Health::Working => self.import_now(path, mode),
            Health::Broken => Err(storage_failure("macros")),
        })
    }

    fn export(&self, path: &Path) -> impl Future<Output = Result<(), StoreError>> + Send {
        ready(match self.health {
            Health::Working => {
                let stored = lock(&self.stored).clone();
                lock(&self.files).insert(path.to_path_buf(), stored);
                Ok(())
            }
            Health::Broken => Err(storage_failure(&path.display().to_string())),
        })
    }
}

/// A clipboard in memory.
#[derive(Debug)]
pub struct FakeClipboard {
    text: Mutex<Option<String>>,
    health: Health,
}

impl FakeClipboard {
    /// A clipboard that holds no text.
    pub const fn new(health: Health) -> Self {
        Self {
            text: Mutex::new(None),
            health,
        }
    }

    /// A working clipboard that holds `text`.
    pub fn holding(text: &str) -> Self {
        Self {
            text: Mutex::new(Some(text.to_owned())),
            health: Health::Working,
        }
    }

    pub fn text(&self) -> Option<String> {
        lock(&self.text).clone()
    }

    fn unavailable() -> ClipboardError {
        ClipboardError::Unavailable("another app holds the clipboard open".into())
    }
}

impl Clipboard for FakeClipboard {
    fn read_text(&self) -> impl Future<Output = Result<Option<String>, ClipboardError>> + Send {
        ready(match self.health {
            Health::Working => Ok(lock(&self.text).clone()),
            Health::Broken => Err(Self::unavailable()),
        })
    }

    fn write_text(&self, text: &str) -> impl Future<Output = Result<(), ClipboardError>> + Send {
        ready(match self.health {
            Health::Working => {
                *lock(&self.text) = Some(text.to_owned());
                Ok(())
            }
            Health::Broken => Err(Self::unavailable()),
        })
    }
}
