//! `OpenKey`'s answers applied to a text box the way its Windows host applies them.
//!
//! In VNI Windows and Unicode compound a letter can take two code units, and a text box's Backspace deletes one. Seen
//! from the text box, `OpenKey` deletes the whole of a two-unit letter it typed since the last click or key that
//! ended a word; a letter typed before that loses one unit per Backspace.

use cuakey_domain::{CodeTable, KeyPress};

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
}

impl<'engine> OpenKeyScreen<'engine> {
    /// An empty text box, with the engine (already configured for `table`) starting afresh.
    pub fn start(engine: &'engine mut OpenKey, table: CodeTable) -> Self {
        engine.start_line();
        let two_unit_letters = matches!(table, CodeTable::VniWindows | CodeTable::UnicodeCompound);
        Self {
            engine,
            screen: Screen::default(),
            lengths: two_unit_letters.then(Vec::new),
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
        let reply = self.engine.press(key);
        match reply.action {
            Action::PassThrough => self.pass_through(key, reply.after),
            Action::Replace | Action::ReplaceThenKey | Action::ReplaceThenKeyThenNewWord => self.replace(key, &reply),
            Action::Unmodelled(number) => return Err(Unmodelled(number)),
        }
        Ok(())
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
