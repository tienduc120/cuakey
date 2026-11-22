use std::future::ready;

use cuakey_application::{Clipboard, ClipboardError};
use platform_windows::{read_clipboard_text, write_clipboard_text};

/// The Windows clipboard. Each call opens it, uses it, and closes it, so it is never held between calls.
#[derive(Debug, Default, Clone, Copy)]
pub struct WindowsClipboard;

impl Clipboard for WindowsClipboard {
    // The Win32 calls block for a moment at most and have nothing to await, so the answer is ready at once.
    fn read_text(&self) -> impl Future<Output = Result<Option<String>, ClipboardError>> + Send {
        ready(read_clipboard_text().map_err(unavailable))
    }

    fn write_text(&self, text: &str) -> impl Future<Output = Result<(), ClipboardError>> + Send {
        ready(write_clipboard_text(text).map_err(unavailable))
    }
}

fn unavailable(source: platform_windows::ClipboardError) -> ClipboardError {
    ClipboardError::Unavailable(Box::new(source))
}
