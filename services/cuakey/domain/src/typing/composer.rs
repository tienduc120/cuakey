//! What each key does on screen to the word being typed.

use alloc::string::String;
use core::{iter, mem};

use super::tone::ToneStyle;
use super::word::{Glyph, Keystroke, Word};
use super::{ComposerSettings, Edit, Effect, KeyPress, letter_char, telex, vni};
use crate::{InputMethod, KeyCode, Language, Letter, LetterCase, OnOff};

/// The keys besides letters that can type into a word, with what each types without and with Shift.
const SYMBOL_KEYS: [(KeyCode, char, char); 12] = [
    (KeyCode::Digit0, '0', ')'),
    (KeyCode::Digit1, '1', '!'),
    (KeyCode::Digit2, '2', '@'),
    (KeyCode::Digit3, '3', '#'),
    (KeyCode::Digit4, '4', '$'),
    (KeyCode::Digit5, '5', '%'),
    (KeyCode::Digit6, '6', '^'),
    (KeyCode::Digit7, '7', '&'),
    (KeyCode::Digit8, '8', '*'),
    (KeyCode::Digit9, '9', '('),
    (KeyCode::LeftBracket, '[', '{'),
    (KeyCode::RightBracket, ']', '}'),
];

/// Turns keys into the word being typed, one key at a time.
#[derive(Debug)]
pub struct Composer {
    settings: ComposerSettings,
    word: Word,
    /// The word as the screen shows it, kept so `word` can lend it out.
    shown: String,
}

impl Composer {
    pub fn new(settings: ComposerSettings) -> Self {
        Self {
            settings,
            word: Word::default(),
            shown: String::new(),
        }
    }

    /// What to do on screen with `key`.
    pub fn press(&mut self, key: KeyPress) -> Edit {
        if self.settings.language() == Language::English || key.mods().ctrl_or_alt().is_on() {
            self.break_word();
            return Edit::PassThrough;
        }
        if key.code() == KeyCode::Backspace {
            self.word.delete_last();
            self.shown = self.word.render();
            return Edit::PassThrough;
        }
        let Some((effects, typed)) = self.effects_of(key).zip(typed_by(key)) else {
            self.break_word();
            return Edit::PassThrough;
        };
        let keystroke = Keystroke {
            code: key.code(),
            case: key.mods().letter_case(),
            effects,
            typed,
        };
        self.word.type_key(keystroke, self.tone_style());
        let before = mem::replace(&mut self.shown, self.word.render());
        edit_between(&before, &self.shown, typed.to_char())
    }

    /// Forgets the word, so the next key starts a new one: after a click, a focus change, or a language switch.
    pub fn break_word(&mut self) {
        self.word = Word::default();
        self.shown.clear();
    }

    /// Takes new settings; the word being typed is dropped.
    pub fn set_settings(&mut self, settings: ComposerSettings) {
        self.settings = settings;
        self.break_word();
    }

    /// The word being typed, as the screen shows it.
    pub fn word(&self) -> &str {
        &self.shown
    }

    fn effects_of(&self, key: KeyPress) -> Option<&'static [Effect]> {
        match self.settings.typing().input_method() {
            InputMethod::Telex => telex::effects(key.code(), telex::Style::Full),
            InputMethod::SimpleTelex => telex::effects(key.code(), telex::Style::Simple),
            InputMethod::Vni => vni::effects(key.code(), key.mods().shift()),
        }
    }

    fn tone_style(&self) -> ToneStyle {
        match self.settings.typing().new_tone_style() {
            OnOff::On => ToneStyle::New,
            OnOff::Off => ToneStyle::Old,
        }
    }
}

/// What `key` types by itself, when it is a key that can type into a word.
fn typed_by(key: KeyPress) -> Option<Glyph> {
    if let Some(lower) = letter_char(key.code()) {
        let letter = match key.mods().letter_case() {
            LetterCase::Lower => lower,
            LetterCase::Upper => lower.to_ascii_uppercase(),
        };
        return Letter::from_char(letter).map(Glyph::Letter);
    }
    let shifted = key.mods().shift().is_on();
    SYMBOL_KEYS
        .iter()
        .find(|(code, ..)| *code == key.code())
        .map(|(_, plain, with_shift)| Glyph::Other(if shifted { *with_shift } else { *plain }))
}

/// The edit that turns `before` on screen into `after`; `PassThrough` when the key typing `typed` does that itself.
fn edit_between(before: &str, after: &str, typed: char) -> Edit {
    let kept = before
        .chars()
        .zip(after.chars())
        .take_while(|(old, new)| old == new)
        .count();
    let delete = before.chars().count().saturating_sub(kept);
    let insert: String = after.chars().skip(kept).collect();
    if delete == 0 && insert.chars().eq(iter::once(typed)) {
        Edit::PassThrough
    } else {
        Edit::Replace {
            delete,
            insert,
            resend_key: false,
        }
    }
}

#[cfg(test)]
mod tests;
