//! The system clipboard, as Unicode text.

use std::ffi::c_void;

use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock};
use windows::Win32::System::Ole::CF_UNICODETEXT;

/// Why the clipboard could not be read or written.
#[derive(Debug, thiserror::Error)]
pub enum ClipboardError {
    /// Another program holds the clipboard open, or the system refused to open it.
    #[error("could not open the clipboard")]
    Open(#[source] windows::core::Error),
    #[error("could not read the clipboard")]
    Read(#[source] windows::core::Error),
    #[error("could not write the clipboard")]
    Write(#[source] windows::core::Error),
}

/// The text on the clipboard, or `None` when it holds no text.
pub fn read_clipboard_text() -> Result<Option<String>, ClipboardError> {
    let _open = OpenClipboardGuard::open()?;
    // SAFETY: the clipboard is open on this thread, and the call only asks whether a format is present.
    if unsafe { IsClipboardFormatAvailable(UNICODE_TEXT) }.is_err() {
        return Ok(None);
    }
    // SAFETY: the clipboard is open on this thread, as the call requires.
    let handle = unsafe { GetClipboardData(UNICODE_TEXT) }.map_err(ClipboardError::Read)?;
    // The clipboard owns this memory and keeps it valid while it stays open, so it is only locked and copied, never freed.
    let lock = LockedMemory::lock(HGLOBAL(handle.0)).map_err(ClipboardError::Read)?;
    let units = lock.units();
    let text = units.split(|unit| *unit == 0).next().unwrap_or_default();
    Ok(Some(String::from_utf16_lossy(text)))
}

/// Replaces the clipboard's contents with `text`.
pub fn write_clipboard_text(text: &str) -> Result<(), ClipboardError> {
    let units: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let _open = OpenClipboardGuard::open()?;
    let memory = GlobalMemory::alloc(units.len().saturating_mul(size_of::<u16>())).map_err(ClipboardError::Write)?;
    memory.fill(&units).map_err(ClipboardError::Write)?;
    // SAFETY: the clipboard is open on this thread, as the call requires.
    unsafe { EmptyClipboard() }.map_err(ClipboardError::Write)?;
    // SAFETY: the clipboard is open, the memory is moveable, unlocked, and holds a terminated UTF-16 string, as the
    // format requires.
    unsafe { SetClipboardData(UNICODE_TEXT, Some(HANDLE(memory.handle().0))) }.map_err(ClipboardError::Write)?;
    memory.hand_over_to_system();
    Ok(())
}

/// The clipboard format of UTF-16 text, as the API takes it.
const UNICODE_TEXT: u32 = CF_UNICODETEXT.0 as u32;

/// The clipboard, open for this thread until the guard drops.
struct OpenClipboardGuard;

impl OpenClipboardGuard {
    fn open() -> Result<Self, ClipboardError> {
        // SAFETY: no owner window is allowed; the clipboard then belongs to this thread until it is closed.
        unsafe { OpenClipboard(None) }.map_err(ClipboardError::Open)?;
        Ok(Self)
    }
}

impl Drop for OpenClipboardGuard {
    fn drop(&mut self) {
        // SAFETY: a guard exists only once `OpenClipboard` succeeded on this thread, and it closes the clipboard once.
        let closed = unsafe { CloseClipboard() };
        // Nothing can be done about a failed close, and the next open reports a clipboard that stays held.
        drop(closed);
    }
}

/// Global memory that stays locked, so its address is valid, until the guard drops.
struct LockedMemory {
    handle: HGLOBAL,
    address: *mut c_void,
}

impl LockedMemory {
    fn lock(handle: HGLOBAL) -> Result<Self, windows::core::Error> {
        // SAFETY: the handle comes from the system and names global memory that is still allocated.
        let address = unsafe { GlobalLock(handle) };
        if address.is_null() {
            return Err(windows::core::Error::from_thread());
        }
        Ok(Self { handle, address })
    }

    /// Everything the block can hold, read as UTF-16 code units.
    fn units(&self) -> &[u16] {
        // SAFETY: the handle is locked.
        let bytes = unsafe { GlobalSize(self.handle) };
        // SAFETY: the block is locked, so `address` is valid for `bytes` bytes and at least 8-byte aligned (global
        // allocations are), and nothing writes it while this shared borrow of the guard lives.
        unsafe { std::slice::from_raw_parts(self.address.cast::<u16>(), bytes / size_of::<u16>()) }
    }

    fn write_units(&self, units: &[u16]) {
        // SAFETY: the caller allocated the block for `units.len()` code units, the two do not overlap, and `address`
        // is a locked, aligned block.
        unsafe { std::ptr::copy_nonoverlapping(units.as_ptr(), self.address.cast::<u16>(), units.len()) }
    }
}

impl Drop for LockedMemory {
    fn drop(&mut self) {
        // SAFETY: the guard holds one lock on the handle, taken in `lock`, and releases it once.
        let unlocked = unsafe { GlobalUnlock(self.handle) };
        // Releasing the last lock reports failure with no error code, which is success, so the result says nothing.
        drop(unlocked);
    }
}

/// Moveable global memory that is freed on drop unless it was handed to the clipboard.
struct GlobalMemory {
    handle: Option<HGLOBAL>,
}

impl GlobalMemory {
    fn alloc(bytes: usize) -> Result<Self, windows::core::Error> {
        // SAFETY: the flags ask for plain moveable memory of a size that fits in the address space.
        let handle = unsafe { GlobalAlloc(GMEM_MOVEABLE, bytes) }?;
        Ok(Self { handle: Some(handle) })
    }

    fn handle(&self) -> HGLOBAL {
        self.handle.unwrap_or_default()
    }

    /// Copies `units` in; the block was allocated for exactly these.
    fn fill(&self, units: &[u16]) -> Result<(), windows::core::Error> {
        let lock = LockedMemory::lock(self.handle())?;
        lock.write_units(units);
        Ok(())
    }

    /// The clipboard owns the memory from here on and frees it itself.
    fn hand_over_to_system(mut self) {
        self.handle = None;
    }
}

impl Drop for GlobalMemory {
    fn drop(&mut self) {
        let Some(handle) = self.handle else {
            return;
        };
        // SAFETY: the memory was allocated here, is not locked, and never reached the clipboard, so it is freed once.
        let freed = unsafe { GlobalFree(Some(handle)) };
        // A failed free leaves nothing more to do.
        drop(freed);
    }
}

#[cfg(test)]
mod tests;
