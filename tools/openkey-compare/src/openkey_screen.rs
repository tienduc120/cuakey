//! `OpenKey`'s answers applied to text boxes by two hosts. The engine is asked once per key, and both hosts apply its
//! answer:
//!
//! - the corrected host, which `CuaKey` is held to: each deletion sends the backspaces the letter's code units need;
//! - the faithful host, which applies the answer the way `OpenKey`'s Windows host does, defects included.
//!
//! The two differ in three behaviours of `OpenKey`'s host, the [`Family`] values. In VNI Windows and Unicode compound a
//! letter can take two code units and a text box's Backspace deletes one; both hosts delete the whole of a two-unit
//! letter typed since the last click or key that ended a word, and a letter typed before that loses one unit per
//! Backspace. While typing English, the hosts ask the engine about a key only when macros are on in English.

use cuakey_domain::{CodeTable, KeyCode, KeyPress, Language};

use crate::combo::{Combo, Toggle};
use crate::corpus::Stroke;
use crate::oracle::{Action, After, OpenKey, Reply, Send};
use crate::screen::Screen;

/// The longest reply the host applies; it ignores a longer one.
const MAX_REPLY: usize = 32;

/// A reply from `OpenKey` the tool does not know how to apply, by the shim's number for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unmodelled(pub i32);

/// A behaviour of `OpenKey`'s Windows host that `CuaKey` does not reproduce.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Family {
    /// In CP1258 a toned letter is two code units, and the host deletes it with one Backspace.
    WholeLetter,
    /// In VNI Windows and Unicode compound, a Space typed right after a Backspace deletes half the last letter.
    SpaceAfterBackspace,
    /// In English with macros, the key that ends a shortcut reaches the text box before the expansion.
    EnglishMacroKey,
}

impl Family {
    pub const ALL: [Self; 3] = [Self::WholeLetter, Self::SpaceAfterBackspace, Self::EnglishMacroKey];

    pub const fn token(self) -> &'static str {
        match self {
            Self::WholeLetter => "cp1258-whole-letter",
            Self::SpaceAfterBackspace => "space-after-backspace",
            Self::EnglishMacroKey => "english-macro-key",
        }
    }

    /// What the host does, in terms of what the text box shows.
    pub const fn description(self) -> &'static str {
        match self {
            Self::WholeLetter => {
                "in CP1258 OpenKey writes a toned letter as two code units but deletes it with one Backspace, both \
                 when it replaces the letter and when the person types Backspace, so the letter's first unit stays on \
                 screen; a corrected host deletes the whole letter"
            }
            Self::SpaceAfterBackspace => {
                "in VNI Windows and Unicode compound, a Space typed right after a Backspace makes OpenKey's host \
                 delete the second unit of the last two-unit letter before the Space; a corrected host only types \
                 the Space"
            }
            Self::EnglishMacroKey => {
                "typing English with macros on in English, OpenKey's host lets the key that ends a shortcut reach \
                 the text box before it deletes the shortcut and types the expansion, so the shortcut's first letter \
                 stays; a corrected host holds the key back until the expansion is typed"
            }
        }
    }
}

/// What the hosts do with a key while typing English.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum English {
    /// Typing Vietnamese: every key goes to the engine.
    No,
    /// Every key goes to the text box untouched.
    PassThrough,
    /// The engine looks for a shortcut to expand.
    Macros,
}

/// The two text boxes `OpenKey` types into.
#[derive(Debug)]
pub struct OpenKeyScreens<'engine> {
    engine: &'engine mut OpenKey,
    english: English,
    corrected: Host,
    faithful: Host,
}

impl<'engine> OpenKeyScreens<'engine> {
    /// Two empty text boxes, with the engine (already configured for `combo`) starting afresh.
    pub fn start(engine: &'engine mut OpenKey, combo: Combo) -> Self {
        engine.start_line();
        let english = match combo.language {
            Language::Vietnamese => English::No,
            Language::English if combo.is_on(Toggle::Macros) && combo.is_on(Toggle::MacrosInEnglish) => English::Macros,
            Language::English => English::PassThrough,
        };
        Self {
            engine,
            english,
            corrected: Host::new(Mode::Corrected, combo.table),
            faithful: Host::new(Mode::Faithful, combo.table),
        }
    }

    pub fn stroke(&mut self, stroke: Stroke) -> Result<(), Unmodelled> {
        let Stroke::Key(key) = stroke else {
            // The host tells the engine of the button going down and of it coming up; neither changes the text.
            self.engine.mouse_button();
            self.engine.mouse_button();
            self.corrected.forget();
            self.faithful.forget();
            return Ok(());
        };
        match self.english {
            English::No => self.vietnamese_key(key),
            English::PassThrough => {
                self.corrected.screen.key(key);
                self.faithful.screen.key(key);
                Ok(())
            }
            English::Macros => self.english_key(key),
        }
    }

    /// The corrected host's text box: what `CuaKey` is held to.
    pub fn corrected(&self) -> &[u16] {
        self.corrected.screen.units()
    }

    /// The text box of the host that applies `OpenKey`'s answers the way its Windows host does.
    pub fn faithful(&self) -> &[u16] {
        self.faithful.screen.units()
    }

    /// The last host defect the faithful host showed.
    pub const fn defect(&self) -> Option<Family> {
        self.faithful.defect
    }

    /// The corrected host's text box takes `units`, after a divergence an entry accepts.
    pub fn resync(&mut self, units: &[u16]) {
        self.corrected.screen = Screen::default();
        self.corrected.screen.push(units);
    }

    fn vietnamese_key(&mut self, key: KeyPress) -> Result<(), Unmodelled> {
        let reply = self.engine.press(key);
        let shift = key.mods().shift().is_on();
        let key_send = match reply.action {
            // A restored word ends with the key, in upper case while Shift or Caps Lock is on.
            Action::ReplaceThenKey | Action::ReplaceThenKeyThenNewWord => Some(
                self.engine
                    .key_send(key.code(), shift || key.mods().caps_lock().is_on()),
            ),
            // An expansion ends with the key, in upper case only while Shift is held.
            Action::Macro => Some(self.engine.key_send(key.code(), shift)),
            Action::Unmodelled(number) => return Err(Unmodelled(number)),
            Action::PassThrough | Action::Replace => None,
        };
        self.corrected.vietnamese(key, &reply, key_send);
        self.faithful.vietnamese(key, &reply, key_send);
        if reply.action == Action::ReplaceThenKeyThenNewWord {
            self.engine.new_word();
        }
        Ok(())
    }

    fn english_key(&mut self, key: KeyPress) -> Result<(), Unmodelled> {
        let reply = self.engine.press_in_english(key);
        match reply.action {
            Action::PassThrough => {
                self.corrected.screen.key(key);
                self.faithful.screen.key(key);
            }
            Action::Macro => {
                let key_send = self.engine.key_send(key.code(), key.mods().shift().is_on());
                self.corrected.english_expansion(key, &reply, key_send);
                self.faithful.english_expansion(key, &reply, key_send);
            }
            Action::Replace | Action::ReplaceThenKey | Action::ReplaceThenKeyThenNewWord => {
                return Err(Unmodelled(1));
            }
            Action::Unmodelled(number) => return Err(Unmodelled(number)),
        }
        Ok(())
    }
}

/// Which host a [`Host`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Corrected,
    Faithful,
}

/// One host and its text box.
#[derive(Debug)]
struct Host {
    mode: Mode,
    screen: Screen,
    /// The units each character took, last typed last; kept in the tables where a letter can take two.
    lengths: Option<Vec<usize>>,
    /// A deletion takes every unit of the letter, as `OpenKey`'s host does in VNI Windows and Unicode compound.
    whole_letters: bool,
    /// The last host defect this host showed.
    defect: Option<Family>,
}

impl Host {
    fn new(mode: Mode, table: CodeTable) -> Self {
        let two_unit_letters = matches!(
            table,
            CodeTable::VniWindows | CodeTable::UnicodeCompound | CodeTable::Cp1258
        );
        Self {
            mode,
            screen: Screen::default(),
            lengths: two_unit_letters.then(Vec::new),
            whole_letters: mode == Mode::Corrected || table != CodeTable::Cp1258,
            defect: None,
        }
    }

    fn vietnamese(&mut self, key: KeyPress, reply: &Reply, key_send: Option<Send>) {
        match reply.action {
            Action::PassThrough => self.pass_through(key, reply.after),
            Action::Replace | Action::ReplaceThenKey | Action::ReplaceThenKeyThenNewWord => {
                self.replace(key, reply, key_send);
            }
            Action::Macro => self.expand(key, reply, key_send),
            Action::Unmodelled(_) => {}
        }
    }

    /// A shortcut typed in English expands; `OpenKey`'s host lets the key through first.
    fn english_expansion(&mut self, key: KeyPress, reply: &Reply, key_send: Send) {
        if self.mode == Mode::Faithful {
            self.screen.key(key);
            self.defect = Some(Family::EnglishMacroKey);
        }
        self.expand(key, reply, Some(key_send));
    }

    /// The key goes on to the text box, and the host notes what it did.
    fn pass_through(&mut self, key: KeyPress, after: After) {
        let after = if after == After::Delete && key.code() != KeyCode::Backspace {
            // The engine reports a key typed right after a Backspace as a deletion too. `OpenKey`'s host takes it as
            // one where it keeps a record; a corrected host knows the key types a character.
            if self.mode == Mode::Faithful && self.whole_letters {
                self.defect = Some(Family::SpaceAfterBackspace);
                After::Delete
            } else {
                After::Typed
            }
        } else {
            after
        };
        match after {
            After::WordBreak => self.forget(),
            // The text box deletes one unit; the host deletes the rest of a two-unit letter.
            After::Delete => self.delete_rest_of_letter(),
            After::Typed => self.note(1),
            After::Other => {}
        }
        self.screen.key(key);
    }

    /// The key is swallowed: the host deletes, types the reply, and for a restored word types the key after it.
    fn replace(&mut self, key: KeyPress, reply: &Reply, key_send: Option<Send>) {
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
        if let Some(send) = key_send {
            self.send(send, key);
        }
    }

    /// A shortcut expands: the host deletes it, types the expansion, then the key that ended it.
    fn expand(&mut self, key: KeyPress, reply: &Reply, key_send: Option<Send>) {
        for _ in 0..reply.backspaces {
            self.backspace();
        }
        for send in &reply.sends {
            self.send(*send, key);
        }
        if let Some(send) = key_send {
            self.send(send, key);
        }
    }

    /// One Backspace from the host, and the rest of the letter where it deletes whole letters. A character it has no
    /// record of counts as one unit.
    fn backspace(&mut self) {
        self.screen.backspace();
        self.delete_rest_of_letter();
    }

    fn delete_rest_of_letter(&mut self) {
        let Some(lengths) = &mut self.lengths else {
            return;
        };
        if lengths.pop().is_some_and(|length| length > 1) {
            if self.whole_letters {
                self.screen.backspace();
            } else {
                self.defect = Some(Family::WholeLetter);
            }
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
        self.note(length);
    }

    fn note(&mut self, length: usize) {
        if let Some(lengths) = &mut self.lengths {
            lengths.push(length);
        }
    }

    fn forget(&mut self) {
        if let Some(lengths) = &mut self.lengths {
            lengths.clear();
        }
    }
}

#[cfg(test)]
mod tests;
