//! The `CuaKey` shortcut on the desktop.

use std::io;
use std::path::Path;

use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance, IPersistFile};
use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
use windows::core::{HSTRING, Interface};

use crate::com::ComScope;
use crate::known_folder::{self, KnownFolderError};

/// The file name of the shortcut on the desktop.
const FILE_NAME: &str = "CuaKey.lnk";

/// Why the desktop shortcut could not be changed.
#[derive(Debug, thiserror::Error)]
pub enum ShortcutError {
    #[error("could not find the desktop folder")]
    Desktop(#[source] KnownFolderError),
    #[error("could not prepare to create a shortcut")]
    Com(#[source] windows::core::Error),
    #[error("could not create the shortcut")]
    Create(#[source] windows::core::Error),
    #[error("could not remove the shortcut")]
    Remove(#[source] io::Error),
}

/// Puts the `CuaKey` shortcut on the desktop, pointing at `exe`, or takes it off.
pub fn set_desktop_shortcut(on: bool, exe: &Path) -> Result<(), ShortcutError> {
    let shortcut = known_folder::desktop().map_err(ShortcutError::Desktop)?.join(FILE_NAME);
    if on {
        create_shortcut_at(&shortcut, exe)
    } else {
        remove_shortcut_at(&shortcut)
    }
}

/// Writes a shortcut to `exe` at `path`, replacing one already there.
fn create_shortcut_at(path: &Path, exe: &Path) -> Result<(), ShortcutError> {
    let _com = ComScope::enter().map_err(ShortcutError::Com)?;
    save_link(path, exe).map_err(ShortcutError::Create)
}

fn save_link(path: &Path, exe: &Path) -> windows::core::Result<()> {
    // SAFETY: COM is initialised on this thread for the life of the scope that called this, and the class and
    // interface match.
    let link: IShellLinkW = unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER) }?;
    // SAFETY: the string is terminated and outlives the call, which copies it.
    unsafe { link.SetPath(&HSTRING::from(exe.as_os_str())) }?;
    if let Some(folder) = exe.parent() {
        // SAFETY: the string is terminated and outlives the call, which copies it.
        unsafe { link.SetWorkingDirectory(&HSTRING::from(folder.as_os_str())) }?;
    }
    let file: IPersistFile = link.cast()?;
    // SAFETY: the string is terminated and outlives the call, which copies it.
    unsafe { file.Save(&HSTRING::from(path.as_os_str()), true) }
}

/// Deletes the shortcut at `path`; one that is not there is already as wanted.
fn remove_shortcut_at(path: &Path) -> Result<(), ShortcutError> {
    match std::fs::remove_file(path) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => Err(ShortcutError::Remove(error)),
        Ok(()) | Err(_) => Ok(()),
    }
}

#[cfg(test)]
mod tests;
