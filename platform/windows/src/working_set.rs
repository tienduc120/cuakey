//! Handing memory pages back to Windows.

use windows::Win32::System::Threading::{GetCurrentProcess, SetProcessWorkingSetSize};

/// Why the working set could not be trimmed.
#[derive(Debug, thiserror::Error)]
pub enum TrimError {
    #[error("could not trim the process's working set")]
    Trim(#[source] windows::core::Error),
}

/// Asks Windows to take every page it can out of this process's working set. Pages still in use come back on the next
/// touch, so this changes no behaviour, only what the process holds in memory afterwards.
pub fn trim_working_set() -> Result<(), TrimError> {
    // SAFETY: the call takes nothing and returns a pseudo handle for this process.
    let process = unsafe { GetCurrentProcess() };
    // Both sizes at the maximum value are how the call is told to empty the set rather than resize it.
    // SAFETY: the pseudo handle of the current process is always valid and has the access this call needs.
    unsafe { SetProcessWorkingSetSize(process, usize::MAX, usize::MAX) }.map_err(TrimError::Trim)
}

#[cfg(test)]
mod tests;
