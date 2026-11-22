//! The keyboard layout of the app in front, and whether an input method editor is composing in it.

/// Whether the layout handle `hkl` is for English, any variant.
pub const fn is_english(hkl: isize) -> bool {
    primary_language(hkl) == LANG_ENGLISH
}

/// Whether the layout handle `hkl` is for a language typed through an input method editor: Chinese, Japanese or
/// Korean.
pub const fn may_have_ime(hkl: isize) -> bool {
    matches!(primary_language(hkl), LANG_CHINESE | LANG_JAPANESE | LANG_KOREAN)
}

// Primary language identifiers, from the Windows language identifier constants.
const LANG_CHINESE: u16 = 0x04;
const LANG_ENGLISH: u16 = 0x09;
const LANG_JAPANESE: u16 = 0x11;
const LANG_KOREAN: u16 = 0x12;

/// The primary language of a layout handle: the low ten bits of its low word, which is the language identifier; the
/// rest of that word names the country, and the high word the layout.
const fn primary_language(hkl: isize) -> u16 {
    let [low, high, ..] = hkl.to_le_bytes();
    u16::from_le_bytes([low, high]) & 0x03ff
}

#[cfg(windows)]
pub use self::foreground::{foreground_layout, ime_open};

#[cfg(windows)]
mod foreground {
    use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
    use windows::Win32::UI::Input::Ime::ImmGetDefaultIMEWnd;
    use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyboardLayout;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowThreadProcessId, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_IME_CONTROL,
    };

    use super::may_have_ime;

    /// `IMC_GETOPENSTATUS` from `imm.h`: asks an IME window whether its input method is open.
    const IMC_GETOPENSTATUS: usize = 0x0005;
    /// How long the hook waits for an IME window to answer before it takes the IME as closed.
    const IME_TIMEOUT_MS: u32 = 20;

    /// The keyboard layout handle of the window in front.
    pub fn foreground_layout() -> isize {
        layout_of(foreground_window())
    }

    /// Whether an input method editor is open in the window in front. Only a layout that may have one is asked, and a
    /// window that does not answer within 20 ms counts as closed, so a hung app cannot stall typing.
    pub fn ime_open() -> bool {
        let window = foreground_window();
        if window.is_invalid() || !may_have_ime(layout_of(window)) {
            return false;
        }
        // SAFETY: the call takes any window handle; for a stale one it returns null, checked below.
        let ime_window = unsafe { ImmGetDefaultIMEWnd(window) };
        if ime_window.is_invalid() {
            return false;
        }
        let mut open = 0_usize;
        // SAFETY: the message carries no pointers; `open` is a local that outlives the call writing the answer to it;
        // the flags and the timeout bound the wait, and a stale handle makes the call fail rather than fault.
        let answered = unsafe {
            SendMessageTimeoutW(
                ime_window,
                WM_IME_CONTROL,
                WPARAM(IMC_GETOPENSTATUS),
                LPARAM(0),
                SMTO_ABORTIFHUNG,
                IME_TIMEOUT_MS,
                Some(&raw mut open),
            )
        };
        answered.0 != 0 && open != 0
    }

    fn foreground_window() -> HWND {
        // SAFETY: the call takes nothing and returns a handle, null when no window is in front.
        unsafe { GetForegroundWindow() }
    }

    fn layout_of(window: HWND) -> isize {
        // SAFETY: the call takes any window handle, returning 0 for a stale one, and no process id is asked for.
        let thread = unsafe { GetWindowThreadProcessId(window, None) };
        // SAFETY: the call takes any thread id; 0 names this thread.
        let layout = unsafe { GetKeyboardLayout(thread) };
        layout.0 as isize
    }
}

#[cfg(test)]
mod tests;
