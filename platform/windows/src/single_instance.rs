//! Keeping `CuaKey` to one instance, with a named mutex.

use windows::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE};
use windows::Win32::System::Threading::CreateMutexW;
use windows::core::HSTRING;

/// Why the guard could not be made at all, which is not the same as another instance holding it.
#[derive(Debug, thiserror::Error)]
#[error("could not create the single-instance mutex")]
pub struct SingleInstanceError(#[source] windows::core::Error);

/// Holds the single-instance name until dropped.
#[derive(Debug)]
#[must_use = "dropping the guard lets another instance start"]
pub struct Guard(HANDLE);

impl Guard {
    /// Takes the mutex `name`. `Ok(None)` means another instance already holds it; an error means the mutex could not
    /// be made, which says nothing about other instances.
    pub fn acquire(name: &str) -> Result<Option<Self>, SingleInstanceError> {
        // SAFETY: the name is a terminated string that outlives the call, and no security attributes are given.
        let handle = unsafe { CreateMutexW(None, false, &HSTRING::from(name)) }.map_err(SingleInstanceError)?;
        let guard = Self(handle);
        // SAFETY: the call only reads this thread's last error, set by `CreateMutexW` just above.
        let existed = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
        // Dropping the guard of an existing mutex closes only this process's handle to it.
        Ok((!existed).then_some(guard))
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        // SAFETY: the handle came from `CreateMutexW` and is closed once, here.
        if let Err(error) = unsafe { CloseHandle(self.0) } {
            tracing::warn!(error = ?error, "could not close the single-instance mutex");
        }
    }
}

#[cfg(test)]
mod tests;
