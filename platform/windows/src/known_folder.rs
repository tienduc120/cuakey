//! The folders Windows keeps for each user.

use std::ffi::c_void;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;

use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::UI::Shell::{
    FOLDERID_Desktop, FOLDERID_Fonts, FOLDERID_LocalAppData, FOLDERID_RoamingAppData, KF_FLAG_DEFAULT,
    SHGetKnownFolderPath,
};
use windows::core::{GUID, PWSTR};

/// Why a folder could not be found.
#[derive(Debug, thiserror::Error)]
#[error("could not find a known folder")]
pub struct KnownFolderError(#[source] windows::core::Error);

/// The user's roaming application data folder, where `CuaKey` keeps its files.
pub fn roaming_app_data() -> Result<PathBuf, KnownFolderError> {
    known_folder(&FOLDERID_RoamingAppData)
}

/// The folder Windows keeps its fonts in, where the windows find one that covers Vietnamese.
pub fn system_fonts() -> Result<PathBuf, KnownFolderError> {
    known_folder(&FOLDERID_Fonts)
}

/// The user's local application data folder, which holds the execution aliases of the packages they installed.
pub(crate) fn local_app_data() -> Result<PathBuf, KnownFolderError> {
    known_folder(&FOLDERID_LocalAppData)
}

/// The user's desktop folder.
pub(crate) fn desktop() -> Result<PathBuf, KnownFolderError> {
    known_folder(&FOLDERID_Desktop)
}

fn known_folder(id: &GUID) -> Result<PathBuf, KnownFolderError> {
    // SAFETY: `id` is a valid folder id, no token means the current user, and the call returns a string that the
    // guard below frees.
    let path = unsafe { SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None) }.map_err(KnownFolderError)?;
    let path = TaskMemory(path);
    // SAFETY: the system returned a terminated UTF-16 string, which stays valid until the guard frees it.
    let units = unsafe { path.0.as_wide() };
    Ok(PathBuf::from(std::ffi::OsString::from_wide(units)))
}

/// A string the system allocated, freed on drop.
struct TaskMemory(PWSTR);

impl Drop for TaskMemory {
    fn drop(&mut self) {
        // SAFETY: the memory was allocated by the system for this caller to free, and is freed once, here.
        unsafe { CoTaskMemFree(Some(self.0.0.cast::<c_void>().cast_const())) };
    }
}

#[cfg(test)]
mod tests;
