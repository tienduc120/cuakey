use alloc::vec::Vec;

use core::iter::repeat_n;

use crate::{CodeTable, Edit, OnOff, encode, units_in};

/// A narrow no-break space typed first so an autocomplete suggestion is dismissed instead of eaten by the backspaces.
const DISMISS_SUGGESTION: u16 = 0x202f;

/// One step of sending a replacement to the focused app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputOp {
    Backspace,
    /// A UTF-16 code unit typed as text.
    Unit(u16),
    /// Shift+Left, which selects the character before the caret so the next key replaces it.
    SelectPreviousChar,
    /// Text pasted through the clipboard.
    Paste(Vec<u16>),
    /// The key that triggered the edit, sent again because it types no text.
    ResendKey,
}

/// How the focused app takes replacements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputContext {
    code_table: CodeTable,
    browser_fix: OnOff,
    chromium_fix: OnOff,
    chromium_app: OnOff,
    clipboard: OnOff,
}

impl OutputContext {
    pub const fn new(code_table: CodeTable) -> Self {
        Self {
            code_table,
            browser_fix: OnOff::Off,
            chromium_fix: OnOff::Off,
            chromium_app: OnOff::Off,
            clipboard: OnOff::Off,
        }
    }

    #[must_use]
    pub const fn with_browser_fix(mut self, on_off: OnOff) -> Self {
        self.browser_fix = on_off;
        self
    }

    #[must_use]
    pub const fn with_chromium_fix(mut self, on_off: OnOff) -> Self {
        self.chromium_fix = on_off;
        self
    }

    /// Whether the focused app is a Chromium browser.
    #[must_use]
    pub const fn with_chromium_app(mut self, on_off: OnOff) -> Self {
        self.chromium_app = on_off;
        self
    }

    #[must_use]
    pub const fn with_clipboard(mut self, on_off: OnOff) -> Self {
        self.clipboard = on_off;
        self
    }
}

/// The events that replace the end of `word_before` as `edit` says. A pass-through plans nothing: the key goes to the
/// app as it is.
pub fn plan_output(edit: &Edit, word_before: &str, ctx: OutputContext) -> Vec<OutputOp> {
    let Edit::Replace {
        delete,
        insert,
        resend_key,
    } = edit
    else {
        return Vec::new();
    };
    let mut ops = deletion(*delete, word_before, ctx);
    if ctx.clipboard.is_on() {
        if !insert.is_empty() {
            ops.push(OutputOp::Paste(encode(insert, ctx.code_table)));
        }
    } else {
        ops.extend(encode(insert, ctx.code_table).into_iter().map(OutputOp::Unit));
    }
    if *resend_key {
        ops.push(OutputOp::ResendKey);
    }
    ops
}

/// The events that delete the last `delete` characters of `word`, with the app-specific fixes.
fn deletion(delete: usize, word: &str, ctx: OutputContext) -> Vec<OutputOp> {
    let shown = word.chars().rev().take(delete);
    let missing = delete.saturating_sub(word.chars().count());
    let units: usize = shown.map(|c| units_in(c, ctx.code_table)).sum::<usize>() + missing;
    if units == 0 {
        return Vec::new();
    }
    let mut ops = Vec::with_capacity(units + 1);
    if ctx.chromium_fix.is_on() && ctx.chromium_app.is_on() {
        ops.push(OutputOp::SelectPreviousChar);
        ops.extend(repeat_n(OutputOp::Backspace, units - 1));
    } else if ctx.browser_fix.is_on() {
        ops.push(OutputOp::Unit(DISMISS_SUGGESTION));
        ops.extend(repeat_n(OutputOp::Backspace, units + 1));
    } else {
        ops.extend(repeat_n(OutputOp::Backspace, units));
    }
    ops
}

#[cfg(test)]
mod tests;
