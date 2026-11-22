//! The preferences as `settings.json` in a folder.

mod dto;
mod hotkeys;

use std::fs;
use std::future::ready;
use std::path::{Path, PathBuf};

use cuakey_application::{PreferencesStore, StoreError};
use cuakey_domain::Preferences;
use platform_clock::Clock;

use self::dto::PreferencesFile;
use crate::files::{FileError, read_if_present, write_replacing};

const FILE_NAME: &str = "settings.json";

/// Keeps the preferences in `settings.json` in a folder. A file that cannot be read as preferences is renamed aside
/// with the time from the clock, so the user's data is never overwritten.
#[derive(Debug)]
pub struct JsonPreferencesStore<C> {
    dir: PathBuf,
    clock: C,
}

impl<C: Clock> JsonPreferencesStore<C> {
    pub const fn new(dir: PathBuf, clock: C) -> Self {
        Self { dir, clock }
    }

    fn path(&self) -> PathBuf {
        self.dir.join(FILE_NAME)
    }

    fn load_blocking(&self) -> Result<Option<Preferences>, FileError> {
        let path = self.path();
        let Some(bytes) = read_if_present(&path)? else {
            return Ok(None);
        };
        match serde_json::from_slice::<PreferencesFile>(&bytes) {
            Ok(file) => file.into_preferences().map(Some).map_err(FileError::Stored),
            Err(error) => {
                let kept = self.set_aside(&path)?;
                tracing::warn!(%error, kept = %kept.display(), "the preferences file is corrupt and was kept aside");
                Ok(None)
            }
        }
    }

    /// Renames the file to `settings.corrupt-<unix seconds>.json` beside it, and returns where it went.
    fn set_aside(&self, path: &Path) -> Result<PathBuf, FileError> {
        let seconds = self.clock.now().as_second();
        let kept = path.with_file_name(format!("settings.corrupt-{seconds}.json"));
        fs::rename(path, &kept).map_err(|source| FileError::SetAside {
            path: path.to_path_buf(),
            source,
        })?;
        Ok(kept)
    }

    fn save_blocking(&self, preferences: &Preferences) -> Result<(), FileError> {
        let json = serde_json::to_vec_pretty(&PreferencesFile::from(preferences)).map_err(FileError::Encode)?;
        write_replacing(&self.path(), &json)
    }
}

// The file is small, and the calls have nothing to await, so each answer is ready at once.
impl<C: Clock> PreferencesStore for JsonPreferencesStore<C> {
    fn load(&self) -> impl Future<Output = Result<Option<Preferences>, StoreError>> + Send {
        ready(self.load_blocking().map_err(StoreError::from))
    }

    fn save(&self, preferences: &Preferences) -> impl Future<Output = Result<(), StoreError>> + Send {
        ready(self.save_blocking(preferences).map_err(StoreError::from))
    }
}

#[cfg(test)]
mod tests;
