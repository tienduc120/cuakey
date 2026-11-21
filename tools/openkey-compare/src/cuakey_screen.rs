//! `CuaKey`'s edits applied to a text box: each edit encoded in the code table, each deleted character taking as many
//! backspaces as it has code units there.

use cuakey_domain::{CodeTable, Composer, ComposerSettings, Edit, MacroTable, encode, units_in};

use crate::corpus::Stroke;
use crate::screen::Screen;

/// A text box `CuaKey` types into.
#[derive(Debug)]
pub struct CuaKeyScreen {
    composer: Composer,
    table: CodeTable,
    screen: Screen,
}

impl CuaKeyScreen {
    pub fn new(settings: ComposerSettings, macros: &MacroTable) -> Self {
        let table = settings.typing().code_table();
        let mut composer = Composer::new(settings);
        composer.set_macros(macros.clone());
        Self {
            composer,
            table,
            screen: Screen::default(),
        }
    }

    pub fn stroke(&mut self, stroke: Stroke) {
        let Stroke::Key(key) = stroke else {
            self.composer.break_word();
            return;
        };
        let shown = self.composer.word().to_owned();
        match self.composer.press(key) {
            Edit::PassThrough => self.screen.key(key),
            Edit::Replace {
                delete,
                insert,
                resend_key,
            } => {
                for _ in 0..self.backspaces(&shown, delete) {
                    self.screen.backspace();
                }
                self.screen.push(&encode(&insert, self.table));
                if resend_key {
                    self.screen.key(key);
                }
            }
        }
    }

    pub fn units(&self) -> &[u16] {
        self.screen.units()
    }

    /// The backspaces that delete the last `delete` characters of `shown`; one for each beyond what it holds.
    fn backspaces(&self, shown: &str, delete: usize) -> usize {
        let held = shown.chars().count();
        let units: usize = shown.chars().rev().take(delete).map(|c| units_in(c, self.table)).sum();
        units + delete.saturating_sub(held)
    }
}

#[cfg(test)]
mod tests;
