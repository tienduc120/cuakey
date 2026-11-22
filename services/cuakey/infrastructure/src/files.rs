//! Reading and writing the small files the adapters keep, and what can go wrong with them.

use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

use cuakey_application::StoreError;

/// Why a file the adapters keep could not be read or written.
#[derive(Debug, thiserror::Error)]
pub(crate) enum FileError {
    #[error("could not read {}", path.display())]
    Read { path: PathBuf, source: io::Error },
    #[error("could not write {}", path.display())]
    Write { path: PathBuf, source: io::Error },
    #[error("could not set aside the corrupt file {}", path.display())]
    SetAside { path: PathBuf, source: io::Error },
    #[error("could not encode the preferences")]
    Encode(#[source] serde_json::Error),
    #[error("the stored preferences are invalid")]
    Stored(#[source] cuakey_domain::PreferencesError),
}

impl From<FileError> for StoreError {
    fn from(error: FileError) -> Self {
        Self::Storage(Box::new(error))
    }
}

/// The file's bytes, or `None` when there is no such file.
pub(crate) fn read_if_present(path: &Path) -> Result<Option<Vec<u8>>, FileError> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(FileError::Read {
            path: path.to_path_buf(),
            source,
        }),
    }
}

/// Writes `bytes` to a file beside `path` and renames it over `path`, so a crash leaves the old file or the new one,
/// never half of one. The folder is made if it is missing.
pub(crate) fn write_replacing(path: &Path, bytes: &[u8]) -> Result<(), FileError> {
    let fail = |source| FileError::Write {
        path: path.to_path_buf(),
        source,
    };
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(fail)?;
    }
    let mut temporary_name: OsString = path.file_name().unwrap_or_default().to_owned();
    temporary_name.push(".tmp");
    let temporary = path.with_file_name(temporary_name);
    let mut file = File::create(&temporary).map_err(fail)?;
    file.write_all(bytes).map_err(fail)?;
    file.sync_all().map_err(fail)?;
    drop(file);
    fs::rename(&temporary, path).map_err(fail)
}
