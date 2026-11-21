//! The word being typed: its characters, its tone, and what each key's effects do to them.

use alloc::string::String;
use alloc::vec::Vec;

use super::tone::{Ending, MarkTarget, Syllable, ToneStyle};
use super::{Effect, Mark};
use crate::{BaseLetter, KeyCode, Letter, LetterCase, Tone};

/// A key that types into the word, as the word sees it.
#[derive(Debug, Clone, Copy)]
pub(super) struct Keystroke {
    pub(super) code: KeyCode,
    /// The case of a letter the key types.
    pub(super) case: LetterCase,
    pub(super) effects: &'static [Effect],
    /// What the key types when nothing it asks of the word can be done.
    pub(super) typed: Glyph,
}

/// One character of the word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Glyph {
    /// A letter without its tone: the word draws its tone on one of its vowels.
    Letter(Letter),
    /// A character that is no letter, such as a digit, kept as typed.
    Other(char),
}

impl Glyph {
    fn base(self) -> Option<BaseLetter> {
        match self {
            Self::Letter(letter) => Some(letter.base()),
            Self::Other(_) => None,
        }
    }

    pub(super) const fn to_char(self) -> char {
        match self {
            Self::Letter(letter) => letter.to_char(),
            Self::Other(c) => c,
        }
    }
}

/// The word's tone, and the place of the vowel it is drawn on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PlacedTone {
    tone: Tone,
    at: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum State {
    #[default]
    Composing,
    /// The last key typed a vowel by itself; the same key again takes the vowel back.
    TypedVowel(KeyCode),
    /// A key typed again took its change back, so the rest of the word is typed as it is, until Backspace deletes
    /// back to the `from` characters there were before that key.
    AsTyped { from: usize },
}

/// What one of a key's effects did to the word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Done,
    TypedVowel,
    /// The key asked for what the word already had, so the change is taken back.
    TakenBack,
    /// The word gives the effect nothing to work on.
    NotHere,
}

/// The word being typed: its characters, and its tone kept apart so it can move as the word grows.
#[derive(Debug, Default)]
pub(super) struct Word {
    glyphs: Vec<Glyph>,
    tone: Option<PlacedTone>,
    state: State,
}

impl Word {
    pub(super) fn type_key(&mut self, key: Keystroke, style: ToneStyle) {
        match self.state {
            State::AsTyped { .. } => self.glyphs.push(key.typed),
            State::TypedVowel(code) if code == key.code => self.take_back_vowel(key.typed),
            State::TypedVowel(_) | State::Composing => self.compose(key, style),
        }
    }

    fn compose(&mut self, key: Keystroke, style: ToneStyle) {
        self.state = State::Composing;
        let outcome = key
            .effects
            .iter()
            .map(|effect| self.apply(*effect, key.case, style))
            .find(|outcome| *outcome != Outcome::NotHere);
        match outcome {
            Some(Outcome::TakenBack) => {
                self.type_as_typed(key.typed);
                return;
            }
            Some(Outcome::TypedVowel) => self.state = State::TypedVowel(key.code),
            Some(Outcome::Done) => {}
            Some(Outcome::NotHere) | None => self.glyphs.push(key.typed),
        }
        self.place_tone(style);
    }

    fn apply(&mut self, effect: Effect, case: LetterCase, style: ToneStyle) -> Outcome {
        let Some(letters) = self.letters() else {
            return Outcome::NotHere;
        };
        let Some(syllable) = Syllable::read(&letters) else {
            return Outcome::NotHere;
        };
        match effect {
            Effect::Mark(mark) => self.put_mark(&syllable, mark),
            Effect::Tone(tone) => self.put_tone(&syllable, tone, style),
            Effect::Stroke => self.stroke(),
            Effect::Vowel(vowel) => self.type_vowel(&syllable, vowel, case),
        }
    }

    fn put_mark(&mut self, syllable: &Syllable<'_>, mark: Mark) -> Outcome {
        match syllable.mark_target(mark) {
            None => Outcome::NotHere,
            Some(MarkTarget::One { at, plain, marked }) => {
                let had_it = self.base_at(at) == Some(marked);
                self.set_base(at, if had_it { plain } else { marked });
                if had_it { Outcome::TakenBack } else { Outcome::Done }
            }
            Some(MarkTarget::HornPair { u, o }) => {
                let had_it = self.base_at(u) == Some(BaseLetter::UHorn) && self.base_at(o) == Some(BaseLetter::OHorn);
                let (new_u, new_o, outcome) = if had_it {
                    (BaseLetter::U, BaseLetter::O, Outcome::TakenBack)
                } else {
                    (BaseLetter::UHorn, BaseLetter::OHorn, Outcome::Done)
                };
                self.set_base(u, new_u);
                self.set_base(o, new_o);
                outcome
            }
        }
    }

    fn put_tone(&mut self, syllable: &Syllable<'_>, tone: Tone, style: ToneStyle) -> Outcome {
        let Some(at) = syllable.tone_position(style) else {
            return Outcome::NotHere;
        };
        if tone == Tone::Ngang {
            // With no tone to take off, the key types itself.
            return self.tone.take().map_or(Outcome::NotHere, |_| Outcome::Done);
        }
        if self.tone.is_some_and(|placed| placed.tone == tone) {
            self.tone = None;
            return Outcome::TakenBack;
        }
        self.tone = Some(PlacedTone { tone, at });
        Outcome::Done
    }

    fn stroke(&mut self) -> Outcome {
        let first = self.base_at(0);
        if first == Some(BaseLetter::D) {
            self.set_base(0, BaseLetter::DStroke);
            Outcome::Done
        } else if first == Some(BaseLetter::DStroke) {
            self.set_base(0, BaseLetter::D);
            Outcome::TakenBack
        } else {
            Outcome::NotHere
        }
    }

    /// Types `vowel` at the end of the word, unless a final consonant is already there.
    fn type_vowel(&mut self, syllable: &Syllable<'_>, vowel: BaseLetter, case: LetterCase) -> Outcome {
        let letter = Letter::from_parts(vowel, Tone::Ngang, case);
        match (syllable.ending(), letter) {
            (Ending::Open, Some(letter)) => {
                self.glyphs.push(Glyph::Letter(letter));
                Outcome::TypedVowel
            }
            (Ending::Open, None) | (Ending::Closed, _) => Outcome::NotHere,
        }
    }

    /// The key that typed the last vowel by itself, typed again: the vowel goes, and the key types itself.
    fn take_back_vowel(&mut self, typed: Glyph) {
        self.delete_last();
        self.type_as_typed(typed);
    }

    /// Types the key that took a change back as it is, and the rest of the word after it.
    fn type_as_typed(&mut self, typed: Glyph) {
        self.state = State::AsTyped {
            from: self.glyphs.len(),
        };
        self.glyphs.push(typed);
    }

    /// Backspace: the last character goes, and the tone with it when it was drawn there. Nothing else moves, so the
    /// word stays what the screen shows. Deleting the key that took a change back lets the word change again.
    pub(super) fn delete_last(&mut self) {
        self.glyphs.pop();
        let len = self.glyphs.len();
        if self.tone.is_some_and(|placed| placed.at >= len) {
            self.tone = None;
        }
        if self.glyphs.is_empty() {
            *self = Self::default();
            return;
        }
        match self.state {
            State::TypedVowel(_) => self.state = State::Composing,
            State::AsTyped { from } if len <= from => self.state = State::Composing,
            State::AsTyped { .. } | State::Composing => {}
        }
    }

    /// Moves the tone onto the vowel that carries it now. A word that is no syllable keeps it where it is.
    fn place_tone(&mut self, style: ToneStyle) {
        let Some(placed) = self.tone else {
            return;
        };
        let at = self
            .letters()
            .and_then(|letters| Syllable::read(&letters)?.tone_position(style));
        if let Some(at) = at {
            self.tone = Some(PlacedTone { at, ..placed });
        }
    }

    /// The word's letters without their case, or `None` when it holds something that is no letter.
    fn letters(&self) -> Option<Vec<BaseLetter>> {
        self.glyphs.iter().map(|glyph| glyph.base()).collect()
    }

    fn base_at(&self, at: usize) -> Option<BaseLetter> {
        self.glyphs.get(at).and_then(|glyph| glyph.base())
    }

    /// Writes the letter at `at` on `base` instead, keeping its case.
    fn set_base(&mut self, at: usize, base: BaseLetter) {
        if let Some(Glyph::Letter(letter)) = self.glyphs.get_mut(at)
            && let Some(changed) = Letter::from_parts(base, letter.tone(), letter.case())
        {
            *letter = changed;
        }
    }

    pub(super) fn render(&self) -> String {
        self.glyphs
            .iter()
            .enumerate()
            .map(|(at, glyph)| self.shown_at(at, *glyph))
            .collect()
    }

    /// The character `glyph` shows as, carrying the tone when it is drawn there.
    fn shown_at(&self, at: usize, glyph: Glyph) -> char {
        let toned = match (glyph, self.tone) {
            (Glyph::Letter(letter), Some(placed)) if placed.at == at => letter.with_tone(placed.tone),
            _ => None,
        };
        toned.map_or_else(|| glyph.to_char(), Letter::to_char)
    }
}
