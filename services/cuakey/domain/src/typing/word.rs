//! The word being typed: its characters, the keys that typed them, and what each key's effects do to them.

use alloc::string::String;
use alloc::vec::Vec;

use super::quick;
use super::spelling::{self, Consonants, Spelling};
use super::tone::{Ending, MarkTarget, Syllable, ToneStyle, has_mark, is_vowel};
use super::{Effect, Mark};
use crate::{BaseLetter, KeyCode, Letter, LetterCase, OnOff, Tone};

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
    /// A letter with its tone.
    Letter(Letter),
    /// A character that is no letter, such as a digit, kept as typed.
    Other(char),
}

impl Glyph {
    fn base(self) -> Option<BaseLetter> {
        self.letter().map(Letter::base)
    }

    const fn letter(self) -> Option<Letter> {
        match self {
            Self::Letter(letter) => Some(letter),
            Self::Other(_) => None,
        }
    }

    pub(super) const fn to_char(self) -> char {
        match self {
            Self::Letter(letter) => letter.to_char(),
            Self::Other(c) => c,
        }
    }

    /// The glyph as a capital, when it is a letter.
    #[must_use]
    pub(super) fn in_capitals(self) -> Self {
        match self {
            Self::Letter(letter) => {
                Letter::from_parts(letter.base(), letter.tone(), LetterCase::Upper).map_or(self, Self::Letter)
            }
            Self::Other(_) => self,
        }
    }

    fn from_char(c: char) -> Self {
        Letter::from_char(c).map_or(Self::Other(c), Self::Letter)
    }
}

/// What the word reads of the settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Rules {
    pub(super) style: ToneStyle,
    pub(super) spell_check: OnOff,
    pub(super) restore: OnOff,
    pub(super) quick_telex: OnOff,
    pub(super) consonants: Consonants,
}

/// A key the word keeps, to type it again when the word is restored.
#[derive(Debug, Clone, Copy)]
struct TypedKey {
    code: KeyCode,
    /// What the key types by itself.
    typed: Glyph,
    /// The place of the character the key typed or changed.
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
    /// Changed the character at this place.
    Done(usize),
    /// Turned the d at this place into đ, which alone does not make the word one to restore.
    Stroked(usize),
    /// Typed a vowel by itself at this place.
    TypedVowel(usize),
    /// The key asked for what the word already had, so the change is taken back.
    TakenBack,
    /// The word gives the effect nothing to work on.
    NotHere,
}

/// The word being typed: its characters, each letter with its tone, and the keys that typed them.
#[derive(Debug, Default)]
pub(super) struct Word {
    glyphs: Vec<Glyph>,
    keys: Vec<TypedKey>,
    state: State,
    /// A mark, tone, or vowel key changed the word rather than only typing itself, so it may show other than the keys
    /// typed.
    changed: bool,
}

impl Word {
    /// A word showing `text` as it was typed, which later keys only add to.
    pub(super) fn as_typed(text: &str) -> Self {
        let glyphs: Vec<Glyph> = text.chars().map(Glyph::from_char).collect();
        let keys = glyphs
            .iter()
            .enumerate()
            .map(|(at, typed)| TypedKey {
                code: KeyCode::Other,
                typed: *typed,
                at,
            })
            .collect();
        Self {
            glyphs,
            keys,
            state: State::AsTyped { from: 0 },
            changed: false,
        }
    }

    /// The characters of the keys that typed the word, as typed.
    pub(super) fn typed_keys(&self) -> String {
        self.keys.iter().map(|key| key.typed.to_char()).collect()
    }

    pub(super) fn is_empty(&self) -> bool {
        self.glyphs.is_empty()
    }

    pub(super) fn type_key(&mut self, key: Keystroke, rules: Rules) {
        self.keys.push(TypedKey {
            code: key.code,
            typed: key.typed,
            at: self.glyphs.len(),
        });
        match self.state {
            State::AsTyped { .. } => self.glyphs.push(key.typed),
            State::TypedVowel(code) if code == key.code => self.take_back_vowel(key),
            State::TypedVowel(_) | State::Composing => self.compose(key, rules),
        }
    }

    fn compose(&mut self, key: Keystroke, rules: Rules) {
        self.state = State::Composing;
        if rules.quick_telex.is_on() && self.type_doubled_consonant(key) {
            return;
        }
        let outcome = if self.takes_changes(rules) {
            key.effects
                .iter()
                .map(|effect| self.apply(*effect, key.case, rules))
                .find(|outcome| *outcome != Outcome::NotHere)
        } else {
            None
        };
        match outcome {
            Some(Outcome::TakenBack) => {
                self.forget_key_taken_back(key.code);
                self.type_as_typed(key.typed);
                return;
            }
            Some(Outcome::TypedVowel(at)) => {
                self.state = State::TypedVowel(key.code);
                self.changed_at(at);
            }
            Some(Outcome::Done(at)) => self.changed_at(at),
            Some(Outcome::Stroked(at)) => self.typed_at(at),
            Some(Outcome::NotHere) | None => self.glyphs.push(key.typed),
        }
        self.join_horns();
        self.place_tone(rules.style);
    }

    /// Quick Telex: a consonant typed right after itself types the second letter of the pair it starts.
    fn type_doubled_consonant(&mut self, key: Keystroke) -> bool {
        let Some(letter) = key.typed.letter() else {
            return false;
        };
        let after_itself = self.glyphs.last().and_then(|glyph| glyph.base()) == Some(letter.base());
        let second = quick::second_of_doubled(letter.base())
            .filter(|_| after_itself)
            .and_then(|second| Letter::from_parts(second, Tone::Ngang, letter.case()));
        let Some(second) = second else {
            return false;
        };
        self.glyphs.push(Glyph::Letter(second));
        true
    }

    /// Whether keys may still change the word: always without spell check, and with it while the word can still be
    /// Vietnamese.
    fn takes_changes(&self, rules: Rules) -> bool {
        rules.spell_check == OnOff::Off || self.spelling(rules) != Spelling::Foreign
    }

    fn spelling(&self, rules: Rules) -> Spelling {
        spelling::spelling(letters_of(&self.glyphs).as_deref(), rules.consonants)
    }

    fn apply(&mut self, effect: Effect, case: LetterCase, rules: Rules) -> Outcome {
        let Some(letters) = letters_of(&self.glyphs) else {
            return Outcome::NotHere;
        };
        let syllable = Syllable::last(&letters);
        let reaches_vowels = spelling::may_end_syllable(syllable.last_consonants(), rules.consonants);
        match effect {
            Effect::Mark(mark) if reaches_vowels => self.put_mark(&syllable, mark),
            Effect::Tone(tone) if reaches_vowels && syllable.takes_tone() => {
                self.put_tone(&syllable, tone, rules.style)
            }
            Effect::Stroke => self.stroke(&syllable, rules),
            Effect::Vowel(vowel)
                if spelling::may_precede_typed_vowel(syllable.first_consonants(), rules.consonants.zwjf_first) =>
            {
                self.type_vowel(&syllable, vowel, case)
            }
            Effect::Mark(_) | Effect::Tone(_) | Effect::Vowel(_) => Outcome::NotHere,
        }
    }

    fn put_mark(&mut self, syllable: &Syllable<'_>, mark: Mark) -> Outcome {
        match mark_target(syllable, mark) {
            None => Outcome::NotHere,
            Some(MarkTarget::One { at, plain, marked }) => {
                let had_it = self.base_at(at) == Some(marked);
                self.set_base(at, if had_it { plain } else { marked });
                if had_it { Outcome::TakenBack } else { Outcome::Done(at) }
            }
            Some(MarkTarget::HornPair { u, o }) => {
                let had_it = self.base_at(u) == Some(BaseLetter::UHorn) && self.base_at(o) == Some(BaseLetter::OHorn);
                let (new_u, new_o, outcome) = if had_it {
                    (BaseLetter::U, BaseLetter::O, Outcome::TakenBack)
                } else {
                    (BaseLetter::UHorn, BaseLetter::OHorn, Outcome::Done(o))
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
        let current = self.tone_in(syllable);
        if let Some((place, _)) = current {
            self.set_tone(place, Tone::Ngang);
        }
        match current {
            // With no tone to take off, the key types itself.
            None if tone == Tone::Ngang => Outcome::NotHere,
            Some((place, _)) if tone == Tone::Ngang => Outcome::Done(place),
            Some((_, had)) if had == tone => Outcome::TakenBack,
            _ => {
                self.set_tone(at, tone);
                Outcome::Done(at)
            }
        }
    }

    /// Turns the d that starts the syllable into đ, or back. Without spell check, the d typed last takes the stroke too:
    /// add shows ađ.
    fn stroke(&mut self, syllable: &Syllable<'_>, rules: Rules) -> Outcome {
        let first = syllable.start();
        let last = self.glyphs.len().saturating_sub(1);
        let at = match self.base_at(first) {
            Some(BaseLetter::D | BaseLetter::DStroke) => first,
            _ if rules.spell_check == OnOff::Off && self.base_at(last) == Some(BaseLetter::D) => last,
            _ => return Outcome::NotHere,
        };
        if self.base_at(at) == Some(BaseLetter::D) {
            self.set_base(at, BaseLetter::DStroke);
            Outcome::Stroked(at)
        } else {
            self.set_base(at, BaseLetter::D);
            Outcome::TakenBack
        }
    }

    /// Types `vowel` at the end of the word, when the syllable has no vowel yet.
    fn type_vowel(&mut self, syllable: &Syllable<'_>, vowel: BaseLetter, case: LetterCase) -> Outcome {
        let has_vowel = self
            .glyphs
            .iter()
            .skip(syllable.start())
            .any(|glyph| glyph.base().is_some_and(is_vowel));
        match Letter::from_parts(vowel, Tone::Ngang, case) {
            Some(letter) if !has_vowel => {
                self.glyphs.push(Glyph::Letter(letter));
                Outcome::TypedVowel(self.glyphs.len() - 1)
            }
            _ => Outcome::NotHere,
        }
    }

    /// The key that typed the last vowel by itself, typed again: the vowel goes, and the key types itself, once.
    fn take_back_vowel(&mut self, key: Keystroke) {
        self.pop_glyph();
        self.keys.push(TypedKey {
            code: key.code,
            typed: key.typed,
            at: self.glyphs.len(),
        });
        self.type_as_typed(key.typed);
    }

    /// Types the key that took a change back as it is, and the rest of the word after it.
    fn type_as_typed(&mut self, typed: Glyph) {
        self.state = State::AsTyped {
            from: self.glyphs.len(),
        };
        self.glyphs.push(typed);
    }

    /// Forgets the key before the last one that `code` typed: the last key took its change back, so typing the word
    /// again types that key once.
    fn forget_key_taken_back(&mut self, code: KeyCode) {
        let before_last = self.keys.len().saturating_sub(1);
        if let Some(at) = self.keys.iter().take(before_last).rposition(|key| key.code == code) {
            self.keys.remove(at);
        }
    }

    fn changed_at(&mut self, at: usize) {
        self.changed = true;
        self.typed_at(at);
    }

    /// Records that the last key changed the character at `at`.
    fn typed_at(&mut self, at: usize) {
        if let Some(last) = self.keys.last_mut() {
            last.at = at;
        }
    }

    /// Backspace: the last character goes, with the keys that typed it, and the tone moves onto the vowel that carries
    /// it now. With spell check, deleting the key that took a change back lets the word change again; without it, the
    /// word stays as typed until it ends.
    pub(super) fn delete_last(&mut self, rules: Rules) {
        self.pop_glyph();
        if self.glyphs.is_empty() {
            *self = Self::default();
            return;
        }
        let len = self.glyphs.len();
        match self.state {
            State::TypedVowel(_) => self.state = State::Composing,
            State::AsTyped { from } if len <= from && rules.spell_check.is_on() => self.state = State::Composing,
            State::AsTyped { .. } | State::Composing => {}
        }
        if self.state == State::Composing {
            self.place_tone(rules.style);
        }
    }

    fn pop_glyph(&mut self) {
        self.glyphs.pop();
        let len = self.glyphs.len();
        self.keys.retain(|key| key.at < len);
    }

    /// Puts the horn on the u of uơ once a Vietnamese last consonant follows: uơ ends a syllable, ươ goes before a
    /// consonant. A quick last consonant does not count until it is written out.
    fn join_horns(&mut self) {
        let as_written = Consonants {
            zwjf_first: OnOff::Off,
            quick_first: OnOff::Off,
            quick_last: OnOff::Off,
        };
        let Some(letters) = letters_of(&self.glyphs) else {
            return;
        };
        let syllable = Syllable::last(&letters);
        let horn_on_o_alone = syllable.vowels() == [BaseLetter::U, BaseLetter::OHorn];
        let closed =
            syllable.ending() == Ending::Closed && spelling::may_end_syllable(syllable.last_consonants(), as_written);
        if horn_on_o_alone && closed {
            self.set_base(syllable.vowels_start(), BaseLetter::UHorn);
        }
    }

    /// Moves the tone of the last syllable onto the vowel that carries it now.
    fn place_tone(&mut self, style: ToneStyle) {
        let Some(letters) = letters_of(&self.glyphs) else {
            return;
        };
        let syllable = Syllable::last(&letters);
        let (Some((from, tone)), Some(to)) = (self.tone_in(&syllable), syllable.tone_position(style)) else {
            return;
        };
        if from != to {
            self.set_tone(from, Tone::Ngang);
            self.set_tone(to, tone);
        }
    }

    /// Where the tone of `syllable` is drawn, and which tone it is.
    fn tone_in(&self, syllable: &Syllable<'_>) -> Option<(usize, Tone)> {
        self.glyphs
            .iter()
            .enumerate()
            .skip(syllable.start())
            .find_map(|(at, glyph)| {
                glyph
                    .letter()
                    .map(|l| (at, l.tone()))
                    .filter(|(_, t)| *t != Tone::Ngang)
            })
    }

    fn base_at(&self, at: usize) -> Option<BaseLetter> {
        self.glyphs.get(at).and_then(|glyph| glyph.base())
    }

    /// Writes the letter at `at` on `base` instead, keeping its tone and case.
    fn set_base(&mut self, at: usize, base: BaseLetter) {
        if let Some(Glyph::Letter(letter)) = self.glyphs.get_mut(at)
            && let Some(changed) = Letter::from_parts(base, letter.tone(), letter.case())
        {
            *letter = changed;
        }
    }

    fn set_tone(&mut self, at: usize, tone: Tone) {
        if let Some(Glyph::Letter(letter)) = self.glyphs.get_mut(at)
            && let Some(toned) = letter.with_tone(tone)
        {
            *letter = toned;
        }
    }

    pub(super) fn render(&self) -> String {
        render(&self.glyphs)
    }

    /// What the word shows once it ends, when that differs from what it shows now: the keys as they were typed, when
    /// restoring is on and keys changed a word that is not Vietnamese (without spell check, only once a key took its
    /// change back and while a mark still shows); otherwise the quick consonants written out, which spell check allows
    /// only when they make a Vietnamese word. A word with no vowel stays as it is.
    pub(super) fn ending(&self, rules: Rules) -> Option<String> {
        let has_vowel = self.glyphs.iter().any(|glyph| glyph.base().is_some_and(is_vowel));
        if !has_vowel {
            return None;
        }
        let taken_back = matches!(self.state, State::AsTyped { .. });
        let checked = rules.spell_check.is_on() || (taken_back && self.shows_a_mark());
        if rules.restore.is_on() && self.changed && checked && self.spelling(rules) != Spelling::Syllable {
            let typed = self.typed_keys();
            return (typed != self.render()).then_some(typed);
        }
        let quick = self.with_quick_consonants(rules.consonants)?;
        let allowed = rules.spell_check == OnOff::Off
            || spelling::spelling(letters_of(&quick).as_deref(), rules.consonants) == Spelling::Syllable;
        allowed.then(|| render(&quick))
    }

    /// Whether a letter of the word carries a tone or a vowel mark.
    fn shows_a_mark(&self) -> bool {
        self.glyphs
            .iter()
            .filter_map(|glyph| glyph.letter())
            .any(|letter| letter.tone() != Tone::Ngang || has_mark(letter.base()))
    }

    /// The word with an f, j, or w that starts it, and a g, h, or k that ends it after a vowel, written as the pair
    /// each stands for, when the settings ask for it and the word has one.
    fn with_quick_consonants(&self, consonants: Consonants) -> Option<Vec<Glyph>> {
        let mut glyphs = self.glyphs.clone();
        let mut changed = false;
        if consonants.quick_last.is_on() {
            changed |= write_last_pair(&mut glyphs);
        }
        if consonants.quick_first.is_on() {
            changed |= write_first_pair(&mut glyphs);
        }
        changed.then_some(glyphs)
    }
}

/// Writes out a g, h, or k that ends `glyphs` after a vowel.
fn write_last_pair(glyphs: &mut Vec<Glyph>) -> bool {
    let (Some(last), Some(before)) = (
        glyphs.last().and_then(|g| g.letter()),
        glyphs.iter().rev().nth(1).and_then(|g| g.base()),
    ) else {
        return false;
    };
    let Some(pair) = quick::last_pair(last.base()).filter(|_| is_vowel(before)) else {
        return false;
    };
    glyphs.pop();
    glyphs.extend(pair.iter().filter_map(|base| letter_glyph(*base, last.case())));
    true
}

/// Writes out an f, j, or w that starts `glyphs`, the pair's second letter in capitals only when the word goes on in
/// capitals.
fn write_first_pair(glyphs: &mut Vec<Glyph>) -> bool {
    let Some(first) = glyphs.first().and_then(|g| g.letter()) else {
        return false;
    };
    let Some([one, two]) = quick::first_pair(first.base()) else {
        return false;
    };
    let next_case = glyphs.get(1).and_then(|g| g.letter()).map(Letter::case);
    let second_case = match (first.case(), next_case) {
        (LetterCase::Upper, Some(LetterCase::Upper)) => LetterCase::Upper,
        _ => LetterCase::Lower,
    };
    let pair = [letter_glyph(one, first.case()), letter_glyph(two, second_case)];
    glyphs.splice(0..1, pair.into_iter().flatten());
    true
}

fn letter_glyph(base: BaseLetter, case: LetterCase) -> Option<Glyph> {
    Letter::from_parts(base, Tone::Ngang, case).map(Glyph::Letter)
}

/// The letters of `glyphs` without tone or case, or `None` when one is no letter.
fn letters_of(glyphs: &[Glyph]) -> Option<Vec<BaseLetter>> {
    glyphs.iter().map(|glyph| glyph.base()).collect()
}

fn render(glyphs: &[Glyph]) -> String {
    glyphs.iter().map(|glyph| glyph.to_char()).collect()
}

/// Where `mark` goes in `syllable`. After th, a mark on an open uo goes on the o alone: thuở. A mark on a vowel before
/// another goes there only when Vietnamese writes the vowels it makes: êu, but no êo.
fn mark_target(syllable: &Syllable<'_>, mark: Mark) -> Option<MarkTarget> {
    let target = syllable.mark_target(mark)?;
    let open_after_th =
        syllable.ending() == Ending::Open && syllable.first_consonants() == [BaseLetter::T, BaseLetter::H];
    match target {
        MarkTarget::HornPair { o, .. } if open_after_th => Some(MarkTarget::One {
            at: o,
            plain: BaseLetter::O,
            marked: BaseLetter::OHorn,
        }),
        MarkTarget::One { at, marked, .. } if !fits_before_the_rest(syllable, at, marked) => None,
        MarkTarget::HornPair { .. } | MarkTarget::One { .. } => Some(target),
    }
}

/// Whether the syllable's vowels, with the vowel at `at` written as `marked`, are a Vietnamese vowel group, when
/// another vowel follows that one.
fn fits_before_the_rest(syllable: &Syllable<'_>, at: usize, marked: BaseLetter) -> bool {
    let vowels = syllable.vowels();
    let Some(place) = at.checked_sub(syllable.vowels_start()) else {
        return true;
    };
    if place + 1 >= vowels.len() {
        return true;
    }
    let mut written = vowels.to_vec();
    if let Some(vowel) = written.get_mut(place) {
        *vowel = marked;
    }
    spelling::is_vowel_group(&written)
}
