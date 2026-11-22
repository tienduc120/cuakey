//! The system's default beep.

use windows::Win32::System::Diagnostics::Debug::MessageBeep;
use windows::Win32::UI::WindowsAndMessaging::MB_OK;

/// Plays the default system sound. A failure is only logged: a missing beep is not worth interrupting anything.
pub fn beep() {
    // SAFETY: the call takes a sound identifier and plays it, touching no memory of ours.
    if let Err(error) = unsafe { MessageBeep(MB_OK) } {
        tracing::warn!(error = ?error, "could not play the beep");
    }
}
