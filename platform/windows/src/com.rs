//! COM, initialised for the length of one call.

use windows::Win32::Foundation::RPC_E_CHANGED_MODE;
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize};

/// COM initialised for this thread until the scope drops. A thread that already initialised COM, in either apartment
/// kind, is left as it was. Every interface taken in the scope must be released before it drops.
#[derive(Debug)]
pub(crate) struct ComScope {
    /// Whether this scope's call succeeded and so has to be balanced with an uninitialise.
    initialised: bool,
}

impl ComScope {
    pub(crate) fn enter() -> windows::core::Result<Self> {
        // SAFETY: the call takes no pointers; a success, including "already initialised", is balanced in `drop`.
        let result = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        if result == RPC_E_CHANGED_MODE {
            // The thread already runs COM in another apartment kind, which serves this use as well.
            return Ok(Self { initialised: false });
        }
        result.ok()?;
        Ok(Self { initialised: true })
    }
}

impl Drop for ComScope {
    fn drop(&mut self) {
        if self.initialised {
            // SAFETY: this scope's `CoInitializeEx` succeeded on this thread, so this balances it once.
            unsafe { CoUninitialize() };
        }
    }
}
