//! What each key does on screen to the word being typed.

use alloc::string::String;
use core::{iter, mem};

use super::capitalise::Sentence;
use super::spelling::Consonants;
use super::tone::ToneStyle;
use super::word::{Glyph, Keystroke, Rules, Word};
use super::{ComposerSettings, Edit, Effect, KeyPress, letter_char, telex, vni};
use crate::{CodeTable, InputMethod, KeyCode, Language, Letter, LetterCase, MacroTable, OnOff, units_in};

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

/// The keys that end a word with a character, with what each types without and with Shift.
const WORD_END_KEYS: [(KeyCode, char, char); 12] = [
    (KeyCode::Space, ' ', ' '),
    (KeyCode::Enter, '\n', '\n'),
    (KeyCode::Tab, '\t', '\t'),
    (KeyCode::Minus, '-', '_'),
    (KeyCode::Equal, '=', '+'),
    (KeyCode::Backslash, '\\', '|'),
    (KeyCode::Semicolon, ';', ':'),
    (KeyCode::Quote, '\'', '"'),
    (KeyCode::Comma, ',', '<'),
    (KeyCode::Period, '.', '>'),
    (KeyCode::Slash, '/', '?'),
    (KeyCode::Backquote, '`', '~'),
];

/// Turns keys into the word being typed, one key at a time.
#[derive(Debug)]
pub struct Composer {
    settings: ComposerSettings,
    word: Word,
    /// The word as the screen shows it, kept so `word` can lend it out.
    shown: String,
    /// The word a Space or punctuation just ended, as the screen shows it: a Backspace right after deletes that
    /// character and takes the word up again.
    ended: Option<(Word, String)>,
    /// What the screen shows before the word since the last key that ended a word other than a digit: Telex ends the
    /// word at a digit, yet a shortcut such as vn1 is looked up whole.
    run: String,
    sentence: Sentence,
    macros: MacroTable,
    /// The engine capitalised the first letter of the word being typed, which the user typed in lower case.
    raised: OnOff,
}

impl Composer {
    pub fn new(settings: ComposerSettings) -> Self {
        Self {
            settings,
            word: Word::default(),
            shown: String::new(),
            ended: None,
            run: String::new(),
            sentence: Sentence::default(),
            macros: MacroTable::default(),
            raised: OnOff::Off,
        }
    }

    /// What to do on screen with `key`.
    pub fn press(&mut self, key: KeyPress) -> Edit {
        let ended = self.ended.take();
        if key.mods().ctrl_or_alt().is_on() {
            return self.leave_word();
        }
        if self.settings.language() == Language::English {
            return self.press_in_english(key);
        }
        if key.code() == KeyCode::Backspace {
            return self.delete_back(ended);
        }
        match self.keystroke(key) {
            Some(keystroke) => self.type_into_word(keystroke, key),
            None if self.types_itself_after_the_word(key) => {
                self.sentence = text_of(key).map_or(self.sentence, |c| self.sentence.after(c));
                self.leave_word()
            }
            None => self.end_word(key),
        }
    }

    /// Forgets the word, so the next key starts a new one: after a click, a focus change, or a language switch.
    pub fn break_word(&mut self) {
        self.raised = OnOff::Off;
        self.word = Word::default();
        self.shown.clear();
        self.ended = None;
        self.run.clear();
        self.sentence = Sentence::default();
    }

    /// Takes new settings; the word being typed is dropped.
    pub fn set_settings(&mut self, settings: ComposerSettings) {
        self.settings = settings;
        self.break_word();
    }

    /// Takes the macros to expand; the word being typed goes on.
    pub fn set_macros(&mut self, macros: MacroTable) {
        self.macros = macros;
    }

    /// The word being typed, as the screen shows it.
    pub fn word(&self) -> &str {
        &self.shown
    }

    /// The word ends as the screen shows it, and the key goes through: a Ctrl or Alt chord is for the app, which acts on
    /// the text as the screen shows it.
    fn leave_word(&mut self) -> Edit {
        self.raised = OnOff::Off;
        self.word = Word::default();
        self.shown.clear();
        self.run.clear();
        Edit::PassThrough
    }

    /// Backspace deletes the last character of the word; with no word, the key goes through, and right after a Space or
    /// punctuation takes up the word it ended.
    fn delete_back(&mut self, ended: Option<(Word, String)>) -> Edit {
        if self.word.is_empty() {
            self.run.pop();
            if let Some((word, shown)) = ended {
                self.word = word;
                self.shown = shown;
            }
            return Edit::PassThrough;
        }
        self.word.delete_last(self.rules());
        let before = mem::replace(&mut self.shown, self.word.render());
        backspace_edit(&before, &self.shown, self.settings.typing().code_table())
    }

    /// The key as the word sees it, or `None` when it ends the word.
    fn keystroke(&self, key: KeyPress) -> Option<Keystroke> {
        let effects = self.effects_of(key)?;
        let typed = typed_by(key)?;
        let starts_word = self.word.is_empty();
        let keeps_w = starts_word && self.keeps_w(key.code());
        let capital = starts_word && self.starts_sentence();
        Some(Keystroke {
            code: key.code(),
            case: if capital {
                LetterCase::Upper
            } else {
                key.mods().letter_case()
            },
            effects: if keeps_w { &[] } else { effects },
            typed: if capital { typed.in_capitals() } else { typed },
        })
    }

    fn type_into_word(&mut self, keystroke: Keystroke, key: KeyPress) -> Edit {
        if self.word.is_empty() {
            let typed = typed_by(key).map(Glyph::to_char);
            self.raised = if typed.is_some_and(|c| c != keystroke.typed.to_char()) {
                OnOff::On
            } else {
                OnOff::Off
            };
        }
        self.word.type_key(keystroke, self.rules());
        self.sentence = self.sentence.after(keystroke.typed.to_char());
        let before = mem::replace(&mut self.shown, self.word.render());
        let typed = typed_by(key).map_or_else(|| keystroke.typed.to_char(), Glyph::to_char);
        edit_between(&before, &self.shown, typed)
    }

    /// A key that types no letter ends the word: the word may change once more, then the key follows. A digit carries
    /// what the screen shows on into the shortcut being typed.
    fn end_word(&mut self, key: KeyPress) -> Edit {
        let word = mem::take(&mut self.word);
        let raised = mem::replace(&mut self.raised, OnOff::Off);
        let shown = mem::take(&mut self.shown);
        let text = text_of(key);
        if let Some(c) = text {
            self.sentence = self.sentence.after(c);
        }
        let digit = text.filter(char::is_ascii_digit);
        if digit.is_none() {
            let run = mem::take(&mut self.run);
            let typed_keys = word.typed_keys();
            let forms = MacroForms {
                shown: &[run.as_str(), &shown].concat(),
                typed_keys: &[run.as_str(), &typed_keys].concat(),
                raised: if run.is_empty() { raised } else { OnOff::Off },
            };
            if let Some(edit) = self.macro_edit(&forms, key) {
                return edit;
            }
        }
        let ending = word.ending(self.rules());
        let edit = ending.as_ref().map_or(Edit::PassThrough, |text| Edit::Replace {
            delete: shown.chars().count(),
            insert: text.clone(),
            resend_key: true,
        });
        if let Some(digit) = digit {
            self.run.push_str(ending.as_deref().unwrap_or(&shown));
            self.run.push(digit);
        }
        if text.is_some_and(|c| !c.is_control() && digit.is_none()) {
            self.ended = Some(match ending {
                Some(text) => (Word::as_typed(&text), text),
                None => (word, shown),
            });
        }
        edit
    }

    /// English passes every key through. Only when macros work in English does the composer follow the typed word, to
    /// look it up where it ends.
    fn press_in_english(&mut self, key: KeyPress) -> Edit {
        if !self.macros_apply() {
            self.break_word();
            return Edit::PassThrough;
        }
        if key.code() == KeyCode::Backspace {
            self.shown.pop();
            return Edit::PassThrough;
        }
        if let Some(glyph) = typed_by(key) {
            self.shown.push(glyph.to_char());
            return Edit::PassThrough;
        }
        let shown = mem::take(&mut self.shown);
        let forms = MacroForms {
            shown: &shown,
            typed_keys: &shown,
            raised: OnOff::Off,
        };
        self.macro_edit(&forms, key).unwrap_or(Edit::PassThrough)
    }

    /// Whether macros expand in the language being typed.
    fn macros_apply(&self) -> bool {
        let macros = self.settings.macros();
        macros.use_macros().is_on()
            && (self.settings.language() == Language::Vietnamese || macros.macros_in_english().is_on())
    }

    /// The edit that swaps the word on screen for its macro's expansion, found by the word as shown or else by the keys
    /// as typed, followed by the key that ended the word. The key goes into the edit when it types text, and is sent
    /// after it when it types none. Tab moves on, often out of the text box, and leaves the word as it is.
    fn macro_edit(&self, forms: &MacroForms<'_>, key: KeyPress) -> Option<Edit> {
        if !self.macros_apply() || key.code() == KeyCode::Tab {
            return None;
        }
        let match_case = self.settings.macros().match_case();
        let mut insert = self
            .macros
            .lookup(forms.shown, match_case)
            .or_else(|| self.macros.lookup(forms.typed_keys, match_case))
            .or_else(|| {
                forms
                    .raised
                    .is_on()
                    .then(|| {
                        self.macros
                            .lookup_raised(forms.shown)
                            .or_else(|| self.macros.lookup_raised(forms.typed_keys))
                    })
                    .flatten()
            })?;
        let typed = text_of(key).filter(|c| !c.is_control());
        insert.extend(typed);
        Some(Edit::Replace {
            delete: forms.shown.chars().count(),
            insert,
            resend_key: typed.is_none(),
        })
    }

    fn effects_of(&self, key: KeyPress) -> Option<&'static [Effect]> {
        match self.settings.typing().input_method() {
            InputMethod::Telex => telex::effects(key.code(), telex::Style::Full),
            InputMethod::SimpleTelex => telex::effects(key.code(), telex::Style::Simple),
            InputMethod::Vni => vni::effects(key.code(), key.mods().shift()),
        }
    }

    /// Whether `key` types itself after the word and leaves the word as it shows: a bracket in Simple Telex.
    fn types_itself_after_the_word(&self, key: KeyPress) -> bool {
        self.settings.typing().input_method() == InputMethod::SimpleTelex
            && matches!(key.code(), KeyCode::LeftBracket | KeyCode::RightBracket)
    }

    /// Telex types a `w` that starts a word as w, when the setting asks for it.
    fn keeps_w(&self, code: KeyCode) -> bool {
        code == KeyCode::W
            && self.settings.keep_w().is_on()
            && self.settings.typing().input_method() == InputMethod::Telex
    }

    fn starts_sentence(&self) -> bool {
        self.settings.typing().capitalize_sentences().is_on() && self.sentence.starts_next()
    }

    fn rules(&self) -> Rules {
        let typing = self.settings.typing();
        Rules {
            style: match typing.new_tone_style() {
                OnOff::On => ToneStyle::New,
                OnOff::Off => ToneStyle::Old,
            },
            spell_check: typing.spell_check(),
            restore: typing.restore_invalid_word(),
            quick_telex: typing.quick_telex(),
            consonants: Consonants {
                zwjf_first: typing.allow_zwjf_consonants(),
                quick_first: typing.quick_first_consonant(),
                quick_last: typing.quick_last_consonant(),
            },
        }
    }
}

/// The ways a typed shortcut can match a macro.
#[derive(Debug)]
struct MacroForms<'a> {
    /// As the screen shows it.
    shown: &'a str,
    /// As its keys were typed.
    typed_keys: &'a str,
    /// The engine raised its first letter, which the user typed in lower case.
    raised: OnOff,
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
    shifted_char(&SYMBOL_KEYS, key).map(Glyph::Other)
}

/// The character a key that ends the word types, if any: a digit or symbol key ends a word where the input method
/// gives it nothing to do.
fn text_of(key: KeyPress) -> Option<char> {
    shifted_char(&WORD_END_KEYS, key).or_else(|| shifted_char(&SYMBOL_KEYS, key))
}

fn shifted_char(keys: &[(KeyCode, char, char)], key: KeyPress) -> Option<char> {
    let shifted = key.mods().shift().is_on();
    keys.iter()
        .find(|(code, ..)| *code == key.code())
        .map(|(_, plain, with_shift)| if shifted { *with_shift } else { *plain })
}

/// The edit that turns `before` on screen into `after`; `PassThrough` when the key typing `typed` does that itself.
fn edit_between(before: &str, after: &str, typed: char) -> Edit {
    let (delete, insert) = difference(before, after);
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

/// The edit Backspace makes from `before` to `after`: `PassThrough` when the key deletes the one code unit that goes.
fn backspace_edit(before: &str, after: &str, table: CodeTable) -> Edit {
    let mut kept = before.chars();
    let last = kept.next_back();
    if kept.as_str() == after && last.is_some_and(|c| units_in(c, table) == 1) {
        return Edit::PassThrough;
    }
    let (delete, insert) = difference(before, after);
    Edit::Replace {
        delete,
        insert,
        resend_key: false,
    }
}

/// How many characters of `before` to delete, and what to type after, to show `after`.
fn difference(before: &str, after: &str) -> (usize, String) {
    let kept = before
        .chars()
        .zip(after.chars())
        .take_while(|(old, new)| old == new)
        .count();
    let delete = before.chars().count().saturating_sub(kept);
    (delete, after.chars().skip(kept).collect())
}

#[cfg(test)]
mod tests;
