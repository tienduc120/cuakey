//! Typing into the app in front: synthetic keys, tagged so the app's own hook lets them through.

use crate::RawKey;

/// One step of a replacement, in the order it is typed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendOp {
    Backspace,
    /// A UTF-16 code unit typed as text, whatever the keyboard layout.
    Unit(u16),
    /// Shift+Left, which selects the character before the caret so the next key replaces it.
    SelectPreviousChar,
    /// Text put on the clipboard and pasted with Shift+Insert, as UTF-16 code units.
    Paste(Vec<u16>),
    /// The key that came in, pressed again; the user's own key-up follows as usual.
    ResendKey(RawKey),
}

/// The runs of `ops` that go out in one batch each. A paste starts a new run, so everything before it is typed before
/// the clipboard changes for it.
#[cfg(any(windows, test))]
fn segments(ops: &[SendOp]) -> impl Iterator<Item = &[SendOp]> {
    let mut rest = ops;
    std::iter::from_fn(move || {
        if rest.is_empty() {
            return None;
        }
        let end = rest
            .iter()
            .skip(1)
            .position(|op| matches!(op, SendOp::Paste(_)))
            .map_or(rest.len(), |at| at + 1);
        let (segment, tail) = rest.split_at_checked(end)?;
        rest = tail;
        Some(segment)
    })
}

/// How many `WM_CHAR` backspaces a Microsoft Store app gets on top of the keys for `ops`: two for each backspace.
#[cfg(any(windows, test))]
fn store_app_backspace_messages(ops: &[SendOp]) -> usize {
    const PER_BACKSPACE: usize = 2;
    ops.iter()
        .filter(|op| matches!(op, SendOp::Backspace))
        .count()
        .saturating_mul(PER_BACKSPACE)
}

#[cfg(windows)]
pub use self::inject::{SendError, send};

#[cfg(windows)]
mod inject {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
        KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, SendInput, VIRTUAL_KEY, VK_BACK, VK_INSERT, VK_LEFT, VK_SHIFT,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, PostMessageW, WM_CHAR};

    use super::{SendOp, segments, store_app_backspace_messages};
    use crate::{ClipboardError, INJECTED_TAG, RawKey, write_clipboard_text};

    /// Why a replacement did not reach the app in front.
    #[derive(Debug, thiserror::Error)]
    pub enum SendError {
        /// Windows took fewer events than asked, which is what it does when the app in front runs with higher rights.
        #[error("the system took {sent} of {asked} key events, so the app in front may run as administrator")]
        Blocked { sent: u32, asked: u32 },
        #[error("could not put the text to paste on the clipboard")]
        Clipboard(#[source] ClipboardError),
        #[error("could not post a backspace to the store app in front")]
        StoreAppBackspace(#[source] windows::core::Error),
    }

    /// Types `ops` into the app in front, every event tagged with [`INJECTED_TAG`]. A paste writes the clipboard after
    /// the keys before it are sent and before its Shift+Insert. With `store_app`, each backspace is also posted to the
    /// window in front as a character, which Microsoft Store apps need.
    pub fn send(ops: &[SendOp], store_app: bool) -> Result<(), SendError> {
        // Read once, before any of these keys go out, so a Shift sent for one op is never taken for the user's.
        let shift_held = !ops.is_empty() && shift_is_held();
        for segment in segments(ops) {
            if let Some(SendOp::Paste(units)) = segment.first() {
                write_clipboard_text(&String::from_utf16_lossy(units)).map_err(SendError::Clipboard)?;
            }
            send_inputs(&build_inputs(segment, shift_held))?;
            if store_app {
                post_backspace_chars(store_app_backspace_messages(segment))?;
            }
        }
        Ok(())
    }

    /// The keyboard events for `ops`, in order; `shift_held` says whether the user is holding Shift.
    pub(super) fn build_inputs(ops: &[SendOp], shift_held: bool) -> Vec<INPUT> {
        let mut inputs = Vec::with_capacity(ops.len().saturating_mul(4));
        for op in ops {
            match op {
                SendOp::Backspace => tap(&mut inputs, VK_BACK, PRESS),
                SendOp::Unit(unit) => {
                    inputs.push(unicode(*unit, PRESS));
                    inputs.push(unicode(*unit, KEYEVENTF_KEYUP));
                }
                SendOp::SelectPreviousChar => shifted_tap(&mut inputs, VK_LEFT, shift_held),
                SendOp::Paste(_) => shifted_tap(&mut inputs, VK_INSERT, shift_held),
                SendOp::ResendKey(key) => inputs.push(resent(key)),
            }
        }
        inputs
    }

    const PRESS: KEYBD_EVENT_FLAGS = KEYBD_EVENT_FLAGS(0);

    fn tap(inputs: &mut Vec<INPUT>, vk: VIRTUAL_KEY, flags: KEYBD_EVENT_FLAGS) {
        inputs.push(key(vk, 0, flags));
        inputs.push(key(vk, 0, flags | KEYEVENTF_KEYUP));
    }

    /// An extended key, the arrow or navigation one rather than its number pad twin, with Shift held around it. When
    /// the user already holds Shift, that Shift applies and is left alone: releasing it would turn the next letters
    /// lowercase while the user still holds it.
    fn shifted_tap(inputs: &mut Vec<INPUT>, vk: VIRTUAL_KEY, shift_held: bool) {
        if shift_held {
            tap(inputs, vk, KEYEVENTF_EXTENDEDKEY);
            return;
        }
        inputs.push(key(VK_SHIFT, 0, PRESS));
        tap(inputs, vk, KEYEVENTF_EXTENDEDKEY);
        inputs.push(key(VK_SHIFT, 0, KEYEVENTF_KEYUP));
    }

    /// Whether the user is physically holding Shift, read the way the hook reads it.
    fn shift_is_held() -> bool {
        // SAFETY: the call takes a key code and only reads the system's key state.
        let state = unsafe { GetAsyncKeyState(i32::from(VK_SHIFT.0)) };
        // The high bit, which makes the value negative, says the key is down.
        state < 0
    }

    fn resent(typed: &RawKey) -> INPUT {
        let flags = if typed.extended { KEYEVENTF_EXTENDEDKEY } else { PRESS };
        key(VIRTUAL_KEY(typed.vk), typed.scan, flags)
    }

    /// A code unit carried in the scan code, which the keyboard layout cannot change.
    fn unicode(unit: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
        key(VIRTUAL_KEY(0), unit, KEYEVENTF_UNICODE | flags)
    }

    fn key(vk: VIRTUAL_KEY, scan: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: scan,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: INJECTED_TAG,
                },
            },
        }
    }

    fn send_inputs(inputs: &[INPUT]) -> Result<(), SendError> {
        if inputs.is_empty() {
            return Ok(());
        }
        let asked = u32::try_from(inputs.len()).unwrap_or(u32::MAX);
        // An INPUT is a few dozen bytes. Were its size ever too large to pass, SendInput would refuse a size of zero
        // and the batch would be reported as blocked.
        let size = i32::try_from(size_of::<INPUT>()).unwrap_or(0);
        // SAFETY: the slice is valid for the call, which only reads it, and `size` is the size of one of its elements.
        let sent = unsafe { SendInput(inputs, size) };
        if sent < asked {
            return Err(SendError::Blocked { sent, asked });
        }
        Ok(())
    }

    fn post_backspace_chars(count: usize) -> Result<(), SendError> {
        if count == 0 {
            return Ok(());
        }
        // SAFETY: the call takes nothing and returns a handle, null when no window is in front.
        let window = unsafe { GetForegroundWindow() };
        // Posting to a null window would reach this thread's own queue instead.
        if window.is_invalid() {
            return Ok(());
        }
        let backspace = WPARAM(usize::from(VK_BACK.0));
        for _ in 0..count {
            // SAFETY: the message carries no pointers, so any window may receive it; a stale handle makes the call
            // fail rather than fault.
            unsafe { PostMessageW(Some(window), WM_CHAR, backspace, LPARAM(1)) }
                .map_err(SendError::StoreAppBackspace)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
