//! The typing engine: it turns keys into a Vietnamese word as the word is typed, in Telex, Simple Telex, or VNI.

mod composer;
mod telex;
mod tone;
mod vni;
mod word;

use alloc::string::String;

pub use composer::Composer;

use crate::{BaseLetter, KeyCode, Language, LetterCase, MacroSettings, OnOff, Settings, Tone, TypingSettings};

/// The modifiers a key was pressed with, and whether Caps Lock was on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyMods {
    shift: OnOff,
    caps_lock: OnOff,
    ctrl_or_alt: OnOff,
}

impl KeyMods {
    /// No modifier held, Caps Lock off.
    pub const fn none() -> Self {
        Self {
            shift: OnOff::Off,
            caps_lock: OnOff::Off,
            ctrl_or_alt: OnOff::Off,
        }
    }

    #[must_use]
    pub const fn with_shift(self, shift: OnOff) -> Self {
        Self { shift, ..self }
    }

    #[must_use]
    pub const fn with_caps_lock(self, caps_lock: OnOff) -> Self {
        Self { caps_lock, ..self }
    }

    /// Ctrl or Alt held: the key is a shortcut for the app, never text.
    #[must_use]
    pub const fn with_ctrl_or_alt(self, ctrl_or_alt: OnOff) -> Self {
        Self { ctrl_or_alt, ..self }
    }

    pub const fn shift(self) -> OnOff {
        self.shift
    }

    pub const fn caps_lock(self) -> OnOff {
        self.caps_lock
    }

    pub const fn ctrl_or_alt(self) -> OnOff {
        self.ctrl_or_alt
    }

    /// The case a letter key types in: Shift turns Caps Lock around.
    const fn letter_case(self) -> LetterCase {
        if self.shift.is_on() == self.caps_lock.is_on() {
            LetterCase::Lower
        } else {
            LetterCase::Upper
        }
    }
}

/// One key pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyPress {
    code: KeyCode,
    mods: KeyMods,
}

impl KeyPress {
    pub const fn new(code: KeyCode, mods: KeyMods) -> Self {
        Self { code, mods }
    }

    pub const fn code(self) -> KeyCode {
        self.code
    }

    pub const fn mods(self) -> KeyMods {
        self.mods
    }
}

/// What to do on screen with one key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    /// Let the key through untouched.
    PassThrough,
    /// Swallow the key: delete `delete` characters of the word on screen, then type `insert`. With `resend_key`, send
    /// the key itself afterwards, for a key that types no text.
    Replace {
        delete: usize,
        insert: String,
        resend_key: bool,
    },
}

/// What the typing engine reads of the settings, and the language being typed right now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposerSettings {
    language: Language,
    typing: TypingSettings,
    macros: MacroSettings,
    keep_w: OnOff,
}

impl ComposerSettings {
    pub fn from_settings(settings: &Settings, language: Language) -> Self {
        Self {
            language,
            typing: settings.typing().clone(),
            macros: settings.macros().clone(),
            keep_w: settings.behaviours().keep_w(),
        }
    }

    pub const fn language(&self) -> Language {
        self.language
    }

    pub const fn typing(&self) -> &TypingSettings {
        &self.typing
    }

    pub const fn macros(&self) -> &MacroSettings {
        &self.macros
    }

    pub const fn keep_w(&self) -> OnOff {
        self.keep_w
    }
}

/// A vowel mark a key types: each plain vowel it goes on, paired with the vowel it makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Mark(&'static [(BaseLetter, BaseLetter)]);

impl Mark {
    /// The vowel `plain` becomes under this mark, or `None` when the mark does not go on it.
    fn on(self, plain: BaseLetter) -> Option<BaseLetter> {
        self.0.iter().find(|(from, _)| *from == plain).map(|(_, to)| *to)
    }
}

/// What a key asks of the word being typed. When nothing it asks can be done, the key types its own character.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Effect {
    /// Put this mark on a vowel of the word.
    Mark(Mark),
    /// Give the word this tone; `Tone::Ngang` takes the tone off.
    Tone(Tone),
    /// Turn the d that starts the word into đ.
    Stroke,
    /// Type this vowel: the key stands for it on its own.
    Vowel(BaseLetter),
}

/// The letter keys and the letter each types in lower case.
const LETTER_KEYS: [(KeyCode, char); 26] = [
    (KeyCode::A, 'a'),
    (KeyCode::B, 'b'),
    (KeyCode::C, 'c'),
    (KeyCode::D, 'd'),
    (KeyCode::E, 'e'),
    (KeyCode::F, 'f'),
    (KeyCode::G, 'g'),
    (KeyCode::H, 'h'),
    (KeyCode::I, 'i'),
    (KeyCode::J, 'j'),
    (KeyCode::K, 'k'),
    (KeyCode::L, 'l'),
    (KeyCode::M, 'm'),
    (KeyCode::N, 'n'),
    (KeyCode::O, 'o'),
    (KeyCode::P, 'p'),
    (KeyCode::Q, 'q'),
    (KeyCode::R, 'r'),
    (KeyCode::S, 's'),
    (KeyCode::T, 't'),
    (KeyCode::U, 'u'),
    (KeyCode::V, 'v'),
    (KeyCode::W, 'w'),
    (KeyCode::X, 'x'),
    (KeyCode::Y, 'y'),
    (KeyCode::Z, 'z'),
];

/// The keys an input method reads, each with what it asks of the word.
type KeyTable = [(KeyCode, &'static [Effect])];

/// The lower-case letter `code` types, or `None` when it is not a letter key.
fn letter_char(code: KeyCode) -> Option<char> {
    LETTER_KEYS.iter().find(|(key, _)| *key == code).map(|(_, c)| *c)
}

/// What `table` says `code` asks of the word.
fn look_up(table: &KeyTable, code: KeyCode) -> Option<&'static [Effect]> {
    table.iter().find(|(key, _)| *key == code).map(|(_, effects)| *effects)
}

/// A letter key asks nothing of the word but to be typed; any other key is not part of it.
fn plain_letter(code: KeyCode) -> Option<&'static [Effect]> {
    letter_char(code).map(|_| -> &'static [Effect] { &[] })
}

#[cfg(test)]
mod tests;
