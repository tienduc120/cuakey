//! Keys as the hook reports them, in the names the typing session knows, and the session's answer as keys to send.

use cuakey_application::KeyName;
#[cfg(windows)]
use cuakey_application::{FocusValue, KeyCommand, KeyFlags, LayoutFlags, OutputAction};
#[cfg(windows)]
use platform_windows::{FocusClass, RawKey, SendOp};

/// The key a Windows virtual-key code stands for. Both sides of a modifier are that modifier, and a key the typing
/// engine does not know is `Other`.
pub(crate) fn to_key_name(vk: u16) -> KeyName {
    match vk {
        0x41..=0x5a => letter(vk - 0x41),
        0x30..=0x39 => digit(vk - 0x30),
        0x70..=0x7b => function_key(vk - 0x70),
        0x20 => KeyName::Space,
        0x08 => KeyName::Backspace,
        0x0d => KeyName::Enter,
        0x09 => KeyName::Tab,
        0x1b => KeyName::Escape,
        // The OEM keys, as a US layout labels them.
        0xbd => KeyName::Minus,
        0xbb => KeyName::Equal,
        0xdb => KeyName::LeftBracket,
        0xdd => KeyName::RightBracket,
        0xdc => KeyName::Backslash,
        0xba => KeyName::Semicolon,
        0xde => KeyName::Quote,
        0xbc => KeyName::Comma,
        0xbe => KeyName::Period,
        0xbf => KeyName::Slash,
        0xc0 => KeyName::Backquote,
        0x10 | 0xa0 | 0xa1 => KeyName::Shift,
        0x11 | 0xa2 | 0xa3 => KeyName::Ctrl,
        0x12 | 0xa4 | 0xa5 => KeyName::Alt,
        0x5b | 0x5c => KeyName::Win,
        _ => KeyName::Other,
    }
}

/// Whether `vk` is Shift, Ctrl, Alt or Win, either side.
pub(crate) fn is_modifier(vk: u16) -> bool {
    matches!(
        to_key_name(vk),
        KeyName::Shift | KeyName::Ctrl | KeyName::Alt | KeyName::Win
    )
}

const LETTERS: [KeyName; 26] = [
    KeyName::A,
    KeyName::B,
    KeyName::C,
    KeyName::D,
    KeyName::E,
    KeyName::F,
    KeyName::G,
    KeyName::H,
    KeyName::I,
    KeyName::J,
    KeyName::K,
    KeyName::L,
    KeyName::M,
    KeyName::N,
    KeyName::O,
    KeyName::P,
    KeyName::Q,
    KeyName::R,
    KeyName::S,
    KeyName::T,
    KeyName::U,
    KeyName::V,
    KeyName::W,
    KeyName::X,
    KeyName::Y,
    KeyName::Z,
];

const DIGITS: [KeyName; 10] = [
    KeyName::Digit0,
    KeyName::Digit1,
    KeyName::Digit2,
    KeyName::Digit3,
    KeyName::Digit4,
    KeyName::Digit5,
    KeyName::Digit6,
    KeyName::Digit7,
    KeyName::Digit8,
    KeyName::Digit9,
];

const FUNCTION_KEYS: [KeyName; 12] = [
    KeyName::F1,
    KeyName::F2,
    KeyName::F3,
    KeyName::F4,
    KeyName::F5,
    KeyName::F6,
    KeyName::F7,
    KeyName::F8,
    KeyName::F9,
    KeyName::F10,
    KeyName::F11,
    KeyName::F12,
];

/// The key at `offset` from the first of a run of consecutive virtual keys.
fn nth(keys: &[KeyName], offset: u16) -> KeyName {
    keys.get(usize::from(offset)).copied().unwrap_or(KeyName::Other)
}

fn letter(offset: u16) -> KeyName {
    nth(&LETTERS, offset)
}

fn digit(offset: u16) -> KeyName {
    nth(&DIGITS, offset)
}

fn function_key(offset: u16) -> KeyName {
    nth(&FUNCTION_KEYS, offset)
}

/// The session's command for one key the hook saw, in `app`, with the focus and layout as they stand.
#[cfg(windows)]
pub(crate) fn key_command(key: RawKey, focus: FocusValue, app: &str, layout: LayoutFlags) -> KeyCommand<'_> {
    KeyCommand {
        key: to_key_name(key.vk),
        down: key.down,
        flags: KeyFlags {
            shift: key.mods.shift,
            ctrl_alt_win: key.mods.ctrl_alt_win,
            caps_lock: key.mods.caps_lock,
        },
        focus,
        app,
        layout,
    }
}

#[cfg(windows)]
pub(crate) const fn focus_value(class: FocusClass) -> FocusValue {
    match class {
        FocusClass::Pending => FocusValue::Pending,
        FocusClass::Text => FocusValue::Text,
        FocusClass::Password => FocusValue::Password,
        FocusClass::NotText => FocusValue::NotText,
        FocusClass::Unknown => FocusValue::Unknown,
    }
}

/// The keys that type the session's answer to `key`.
#[cfg(windows)]
pub(crate) fn send_ops(actions: Vec<OutputAction>, key: RawKey) -> Vec<SendOp> {
    actions
        .into_iter()
        .map(|action| match action {
            OutputAction::Backspace => SendOp::Backspace,
            OutputAction::Unit(unit) => SendOp::Unit(unit),
            OutputAction::SelectPreviousChar => SendOp::SelectPreviousChar,
            OutputAction::Paste(units) => SendOp::Paste(units),
            OutputAction::ResendKey => SendOp::ResendKey(key),
        })
        .collect()
}

#[cfg(test)]
mod tests;
