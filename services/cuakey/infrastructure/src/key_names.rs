//! Hotkey keys are stored by name, so the file does not depend on how the enum is declared.

use cuakey_domain::KeyCode;

/// Declares each key's stored name, which is the variant's own name, and the keys that have one. `Other` has none:
/// naming every key but it is what `key_name`'s exhaustive match checks.
macro_rules! key_names {
    ($($key:ident),* $(,)?) => {
        /// Every key a hotkey can end on.
        pub(crate) const NAMED: &[KeyCode] = &[$(KeyCode::$key),*];

        /// The name a key is stored under, or `None` for `Other`, which is no key.
        pub(crate) const fn key_name(code: KeyCode) -> Option<&'static str> {
            match code {
                $(KeyCode::$key => Some(stringify!($key)),)*
                KeyCode::Other => None,
            }
        }
    };
}

key_names!(
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Space,
    Backspace,
    Enter,
    Tab,
    Escape,
    Minus,
    Equal,
    LeftBracket,
    RightBracket,
    Backslash,
    Semicolon,
    Quote,
    Comma,
    Period,
    Slash,
    Backquote,
);

/// The key stored under `name`, or `None` when no key has that name.
pub(crate) fn key_code(name: &str) -> Option<KeyCode> {
    NAMED.iter().copied().find(|code| key_name(*code) == Some(name))
}

#[cfg(test)]
mod tests;
