//! A notice, shown without holding the caller up or, when the program is about to end, until it is read.

use windows::Win32::UI::WindowsAndMessaging::{MB_ICONINFORMATION, MB_OK, MB_SETFOREGROUND, MB_TOPMOST, MessageBoxW};
use windows::core::HSTRING;

/// Shows a top-most message box with `title` and `text` and returns at once; the box lives on a thread of its own
/// until the user closes it. A failure to start that thread is logged, since a menu action has nothing to do about it.
/// The box goes with the process, so a program about to end uses [`show_and_wait`].
pub fn show(title: &str, text: &str) {
    let title = title.to_owned();
    let text = text.to_owned();
    let started = platform_task::spawn_thread("notice", move || show_and_wait(&title, &text));
    // The thread ends by itself when the box closes, so nothing waits on it.
    if let Err(error) = started {
        tracing::warn!(error = ?error, "could not start the thread that shows a notice");
    }
}

/// Shows a top-most message box with `title` and `text` on this thread and returns once the user closes it.
pub fn show_and_wait(title: &str, text: &str) {
    // SAFETY: both strings are terminated and outlive the call, and no owner window is given, so the box is a
    // top-level window of this thread.
    let shown = unsafe {
        MessageBoxW(
            None,
            &HSTRING::from(text),
            &HSTRING::from(title),
            MB_OK | MB_ICONINFORMATION | MB_TOPMOST | MB_SETFOREGROUND,
        )
    };
    tracing::debug!(button = shown.0, "the notice was closed");
}
