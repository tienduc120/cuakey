//! Whether this copy of `CuaKey` runs from its Microsoft Store package, and the path that starts it either way.

#[cfg(any(windows, test))]
use std::path::{Path, PathBuf};

/// The name the package gives `CuaKey` on the command line, as its manifest declares it. Windows keeps the alias in the
/// user's `WindowsApps` folder, where it outlives updates and starts the program inside its package.
pub const EXECUTION_ALIAS: &str = "cuakey.exe";

/// How this copy of `CuaKey` was installed, which decides how Windows starts it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Install {
    /// Unpacked from the zip: started by its own path, and with Windows through the `Run` key.
    Loose,
    /// Installed from the Microsoft Store: started through its execution alias, and with Windows through the package's
    /// startup task.
    Packaged,
}

/// The execution alias in the user's local application data folder `local_app_data`.
#[cfg(any(windows, test))]
fn alias_path(local_app_data: &Path) -> PathBuf {
    local_app_data
        .join("Microsoft")
        .join("WindowsApps")
        .join(EXECUTION_ALIAS)
}

#[cfg(windows)]
pub use self::system::{ProgramPathError, program_path};

#[cfg(windows)]
mod system {
    use std::io;
    use std::path::PathBuf;

    use windows::Win32::Foundation::{APPMODEL_ERROR_NO_PACKAGE, WIN32_ERROR};
    use windows::Win32::Storage::Packaging::Appx::GetCurrentPackageFullName;

    use super::{Install, alias_path};
    use crate::known_folder::{self, KnownFolderError};

    impl Install {
        /// How the running process was installed.
        #[must_use]
        pub fn current() -> Self {
            let mut length = 0_u32;
            // SAFETY: `length` is a local the call fills, and no buffer means the call only reports the length it
            // needs.
            let answer = unsafe { GetCurrentPackageFullName(&raw mut length, None) };
            install_from(answer)
        }
    }

    /// Why the path that starts `CuaKey` could not be found.
    #[derive(Debug, thiserror::Error)]
    pub enum ProgramPathError {
        #[error("could not find the program's own path")]
        CurrentExe(#[source] io::Error),
        #[error("could not find the folder that holds the execution alias")]
        LocalAppData(#[source] KnownFolderError),
    }

    /// The path that starts `CuaKey`: its own path, or from a package its execution alias, since the folder a package
    /// is installed in changes with every update and a program started from there by another runs outside the package.
    pub fn program_path() -> Result<PathBuf, ProgramPathError> {
        match Install::current() {
            Install::Loose => std::env::current_exe().map_err(ProgramPathError::CurrentExe),
            Install::Packaged => {
                let local = known_folder::local_app_data().map_err(ProgramPathError::LocalAppData)?;
                Ok(alias_path(&local))
            }
        }
    }

    /// What asking for the package name, with no room for it, says of the install: a process without a package is told
    /// so, and one with a package is told the name needs room.
    pub(super) fn install_from(answer: WIN32_ERROR) -> Install {
        if answer == APPMODEL_ERROR_NO_PACKAGE {
            Install::Loose
        } else {
            Install::Packaged
        }
    }
}

#[cfg(test)]
mod tests;
