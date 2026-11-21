//! `OpenKey`'s answers applied to a text box the way its Windows host applies them.
//!
//! In VNI Windows and Unicode compound a letter can take two code units, and a text box's Backspace deletes one. Seen
//! from the text box, `OpenKey` deletes the whole of a two-unit letter it typed since the last click or key that
//! ended a word; a letter typed before that loses one unit per Backspace.
//!
//! While typing English, the host asks the engine about a key only when macros are on in English, and then sends an
//! expansion without holding the key back: the key that ended the shortcut reaches the text box first.

use cuakey_domain::{CodeTable, KeyPress, Language};

use crate::combo::{Combo, Toggle};
use crate::corpus::Stroke;
use crate::oracle::{Action, After, OpenKey, Reply, Send};
use crate::screen::Screen;

/// The longest reply the host applies; it ignores a longer one.
const MAX_REPLY: usize = 32;

/// A reply from `OpenKey` the tool does not know how to apply, by the shim's number for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unmodelled(pub i32);

/// A text box `OpenKey` types into.
#[derive(Debug)]
pub struct OpenKeyScreen<'engine> {
    engine: &'engine mut OpenKey,
    screen: Screen,
    /// The units each character took, last typed last; kept only in the tables that need it.
    lengths: Option<Vec<usize>>,
    english: English,
}

/// What the host does with a key while typing English.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum English {
    /// Typing Vietnamese: every key goes to the engine.
    No,
    /// Every key goes to the text box untouched.
    PassThrough,
    /// The engine looks for a shortcut to expand.
    Macros,
}

impl<'engine> OpenKeyScreen<'engine> {
    /// An empty text box, with the engine (already configured for `combo`) starting afresh.
    pub fn start(engine: &'engine mut OpenKey, combo: Combo) -> Self {
        engine.start_line();
        let table = combo.table;
        let two_unit_letters = matches!(table, CodeTable::VniWindows | CodeTable::UnicodeCompound);
        let english = match combo.language {
            Language::Vietnamese => English::No,
            Language::English if combo.is_on(Toggle::Macros) && combo.is_on(Toggle::MacrosInEnglish) => English::Macros,
            Language::English => English::PassThrough,
        };
        Self {
            engine,
            screen: Screen::default(),
            lengths: two_unit_letters.then(Vec::new),
            english,
        }
    }

    pub fn stroke(&mut self, stroke: Stroke) -> Result<(), Unmodelled> {
        let Stroke::Key(key) = stroke else {
            // The host tells the engine of the button going down and of it coming up; neither changes the text.
            for _ in 0..2 {
                self.engine.mouse_button();
                self.forget_lengths();
            }
            return Ok(());
        };
        match self.english {
            English::No => self.vietnamese_key(key),
            English::PassThrough => {
                self.screen.key(key);
                Ok(())
            }
            English::Macros => self.english_key(key),
        }
    }

    fn vietnamese_key(&mut self, key: KeyPress) -> Result<(), Unmodelled> {
        let reply = self.engine.press(key);
        match reply.action {
            Action::PassThrough => self.pass_through(key, reply.after),
            Action::Replace | Action::ReplaceThenKey | Action::ReplaceThenKeyThenNewWord => self.replace(key, &reply),
            Action::Macro => self.expand(key, &reply),
            Action::Unmodelled(number) => return Err(Unmodelled(number)),
        }
        Ok(())
    }

    /// A key while typing English with macros on in English: the key goes to the text box either way, and an
    /// expansion follows it.
    fn english_key(&mut self, key: KeyPress) -> Result<(), Unmodelled> {
        let reply = self.engine.press_in_english(key);
        match reply.action {
            Action::PassThrough => self.screen.key(key),
            Action::Macro => {
                self.screen.key(key);
                self.expand(key, &reply);
            }
            Action::Replace | Action::ReplaceThenKey | Action::ReplaceThenKeyThenNewWord => {
                return Err(Unmodelled(1));
            }
            Action::Unmodelled(number) => return Err(Unmodelled(number)),
        }
        Ok(())
    }

    /// A shortcut expands: the host deletes it, types the expansion, then the key that ended it, in upper case only
    /// while Shift is held.
    fn expand(&mut self, key: KeyPress, reply: &Reply) {
        for _ in 0..reply.backspaces {
            self.backspace();
        }
        for send in &reply.sends {
            self.send(*send, key);
        }
        let send = self.engine.key_send(key.code(), key.mods().shift().is_on());
        self.send(send, key);
    }

    pub fn units(&self) -> &[u16] {
        self.screen.units()
    }

    /// The key goes on to the text box, and the host notes what it did.
    fn pass_through(&mut self, key: KeyPress, after: After) {
        // The text box deletes one unit; the host deletes the other half of a two-unit letter.
        if self.note(after) {
            self.screen.backspace();
        }
        self.screen.key(key);
    }

    /// Notes a key that passed through; `true` when it deletes a two-unit letter.
    fn note(&mut self, after: After) -> bool {
        let Some(lengths) = &mut self.lengths else {
            return false;
        };
        match after {
            After::WordBreak => lengths.clear(),
            After::Delete => return lengths.pop().is_some_and(|length| length > 1),
            After::Typed => lengths.push(1),
            After::Other => {}
        }
        false
    }

    /// The key is swallowed: the host deletes, types the reply, and for a restored word types the key after it.
    fn replace(&mut self, key: KeyPress, reply: &Reply) {
        if (1..MAX_REPLY).contains(&reply.backspaces) {
            for _ in 0..reply.backspaces {
                self.backspace();
            }
        }
        if (1..=MAX_REPLY).contains(&reply.sends.len()) {
            for send in &reply.sends {
                self.send(*send, key);
            }
        }
        if matches!(reply.action, Action::ReplaceThenKey | Action::ReplaceThenKeyThenNewWord) {
            let upper = key.mods().shift().is_on() || key.mods().caps_lock().is_on();
            let send = self.engine.key_send(key.code(), upper);
            self.send(send, key);
        }
        if reply.action == Action::ReplaceThenKeyThenNewWord {
            self.engine.new_word();
        }
    }

    /// One Backspace from the host: twice over a two-unit letter. A character it has no record of counts as one.
    fn backspace(&mut self) {
        self.screen.backspace();
        if let Some(lengths) = &mut self.lengths
            && lengths.pop().is_some_and(|length| length > 1)
        {
            self.screen.backspace();
        }
    }

    /// One character from the host; a key it sends as a key is typed with the modifiers the person is holding.
    fn send(&mut self, send: Send, key: KeyPress) {
        let length = match send {
            Send::Units { .. } => {
                self.screen.push(send.units());
                send.units().len()
            }
            Send::Key(code) => {
                self.screen.key(KeyPress::new(code, key.mods()));
                1
            }
        };
        if let Some(lengths) = &mut self.lengths {
            lengths.push(length);
        }
    }

    fn forget_lengths(&mut self) {
        if let Some(lengths) = &mut self.lengths {
            lengths.clear();
        }
    }
}

#[cfg(test)]
mod tests;
