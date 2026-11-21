//! Capitalising sentences: the first letter after a full stop, a question mark, or an exclamation mark and the space
//! after it is typed as a capital.

/// Where the text being typed stands in its sentence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum Sentence {
    /// Inside a sentence.
    #[default]
    Going,
    /// Right after the mark that ends a sentence; a letter here still belongs to it, as in 3.5 or a.m.
    Ended,
    /// After the end of a sentence and the space that follows it: the next letter starts a new one.
    Next,
}

impl Sentence {
    /// The sentence once `c` is typed.
    #[must_use]
    pub(super) const fn after(self, c: char) -> Self {
        match (c, self) {
            ('.' | '?' | '!', _) => Self::Ended,
            (' ' | '\t' | '\n', Self::Ended | Self::Next) => Self::Next,
            _ => Self::Going,
        }
    }

    /// Whether the next letter starts a sentence.
    pub(super) const fn starts_next(self) -> bool {
        matches!(self, Self::Next)
    }
}

#[cfg(test)]
mod tests;
