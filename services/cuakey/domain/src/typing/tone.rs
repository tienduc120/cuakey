//! Where the marks of a syllable go: which vowel carries the tone, and which vowel takes a key's vowel mark.

use core::ops::Range;

use super::Mark;
use crate::BaseLetter;

/// Where an open syllable of oa, oe, or uy carries its tone: old style on the first vowel (hòa), new style on the
/// second (hoà). Vowels Vietnamese does not group together carry it on the first old style, on the second new style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ToneStyle {
    Old,
    New,
}

/// Whether a syllable ends in a consonant, which moves the tone of oa, oe, and uy onto their second vowel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Ending {
    Open,
    Closed,
}

/// The vowels a vowel mark goes on, by their place in the word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MarkTarget {
    /// One vowel, which is `plain` without the mark and `marked` with it.
    One {
        at: usize,
        plain: BaseLetter,
        marked: BaseLetter,
    },
    /// The u and o of uo, which take the horn together: ươ.
    HornPair { u: usize, o: usize },
}

/// One syllable of a word: first consonants, vowels, last consonants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Syllable<'a> {
    letters: &'a [BaseLetter],
    /// Where the syllable starts in the word.
    start: usize,
    vowels: Range<usize>,
}

/// Every vowel with the plain vowel it is written on.
const VOWELS: [(BaseLetter, BaseLetter); 12] = [
    (BaseLetter::A, BaseLetter::A),
    (BaseLetter::ABreve, BaseLetter::A),
    (BaseLetter::ACircumflex, BaseLetter::A),
    (BaseLetter::E, BaseLetter::E),
    (BaseLetter::ECircumflex, BaseLetter::E),
    (BaseLetter::I, BaseLetter::I),
    (BaseLetter::O, BaseLetter::O),
    (BaseLetter::OCircumflex, BaseLetter::O),
    (BaseLetter::OHorn, BaseLetter::O),
    (BaseLetter::U, BaseLetter::U),
    (BaseLetter::UHorn, BaseLetter::U),
    (BaseLetter::Y, BaseLetter::Y),
];

/// Pairs of vowels that carry the tone on their first vowel in either style: ai, ưa, âu, and the like.
const FIRST_VOWEL_PAIRS: [(BaseLetter, BaseLetter); 18] = [
    (BaseLetter::A, BaseLetter::I),
    (BaseLetter::A, BaseLetter::O),
    (BaseLetter::A, BaseLetter::U),
    (BaseLetter::A, BaseLetter::Y),
    (BaseLetter::ACircumflex, BaseLetter::U),
    (BaseLetter::ACircumflex, BaseLetter::Y),
    (BaseLetter::E, BaseLetter::O),
    (BaseLetter::ECircumflex, BaseLetter::U),
    (BaseLetter::I, BaseLetter::A),
    (BaseLetter::I, BaseLetter::U),
    (BaseLetter::O, BaseLetter::I),
    (BaseLetter::OCircumflex, BaseLetter::I),
    (BaseLetter::OHorn, BaseLetter::I),
    (BaseLetter::U, BaseLetter::A),
    (BaseLetter::U, BaseLetter::I),
    (BaseLetter::UHorn, BaseLetter::A),
    (BaseLetter::UHorn, BaseLetter::I),
    (BaseLetter::UHorn, BaseLetter::U),
];

/// Pairs Vietnamese lacks that carry the tone on their first vowel in an open syllable, new style too.
const OPEN_FIRST_VOWEL_PAIRS: [(BaseLetter, BaseLetter); 4] = [
    (BaseLetter::I, BaseLetter::E),
    (BaseLetter::I, BaseLetter::I),
    (BaseLetter::O, BaseLetter::U),
    (BaseLetter::U, BaseLetter::U),
];

/// The groups of three vowels Vietnamese writes, which carry the tone on their middle vowel.
const VIETNAMESE_TRIPLES: [&[BaseLetter]; 11] = [
    &[BaseLetter::O, BaseLetter::A, BaseLetter::I],
    &[BaseLetter::O, BaseLetter::A, BaseLetter::Y],
    &[BaseLetter::O, BaseLetter::E, BaseLetter::O],
    &[BaseLetter::U, BaseLetter::ACircumflex, BaseLetter::Y],
    &[BaseLetter::U, BaseLetter::OCircumflex, BaseLetter::I],
    &[BaseLetter::UHorn, BaseLetter::OHorn, BaseLetter::I],
    &[BaseLetter::UHorn, BaseLetter::OHorn, BaseLetter::U],
    &[BaseLetter::I, BaseLetter::ECircumflex, BaseLetter::U],
    &[BaseLetter::Y, BaseLetter::ECircumflex, BaseLetter::U],
    &[BaseLetter::U, BaseLetter::Y, BaseLetter::A],
    &[BaseLetter::U, BaseLetter::Y, BaseLetter::U],
];

impl<'a> Syllable<'a> {
    /// The last syllable of `letters`: their last vowels, the consonants after them, and the consonants before them
    /// back to the vowel before. With no vowel, every letter is a first consonant.
    pub(super) fn last(letters: &'a [BaseLetter]) -> Self {
        let Some(last_vowel) = letters.iter().rposition(|l| is_vowel(*l)) else {
            return Self {
                letters,
                start: 0,
                vowels: letters.len()..letters.len(),
            };
        };
        let before = letters.get(..last_vowel).unwrap_or_default();
        let first_vowel = before.iter().rposition(|l| !is_vowel(*l)).map_or(0, |at| at + 1);
        let start = before
            .get(..first_vowel)
            .unwrap_or_default()
            .iter()
            .rposition(|l| is_vowel(*l))
            .map_or(0, |at| at + 1);
        let first_consonants = letters.get(start..first_vowel).unwrap_or_default();
        let vowels = letters.get(first_vowel..=last_vowel).unwrap_or_default();
        let glide = usize::from(glide_belongs_to_first_consonants(first_consonants, vowels));
        Self {
            letters,
            start,
            vowels: first_vowel + glide..last_vowel + 1,
        }
    }

    /// Whether the syllable is the whole word.
    pub(super) const fn starts_word(&self) -> bool {
        self.start == 0
    }

    /// Where the syllable starts in the word.
    pub(super) const fn start(&self) -> usize {
        self.start
    }

    /// The consonants before the vowels, with the glide of qu or gi.
    pub(super) fn first_consonants(&self) -> &'a [BaseLetter] {
        self.letters.get(self.start..self.vowels.start).unwrap_or_default()
    }

    /// Where the vowels start in the word.
    pub(super) const fn vowels_start(&self) -> usize {
        self.vowels.start
    }

    pub(super) fn vowels(&self) -> &'a [BaseLetter] {
        self.letters.get(self.vowels.clone()).unwrap_or_default()
    }

    pub(super) fn last_consonants(&self) -> &'a [BaseLetter] {
        self.letters.get(self.vowels.end..).unwrap_or_default()
    }

    pub(super) fn ending(&self) -> Ending {
        if self.vowels.is_empty() || self.last_consonants().is_empty() {
            Ending::Open
        } else {
            Ending::Closed
        }
    }

    /// The place in the word of the vowel that carries the tone, or `None` when there is no vowel.
    pub(super) fn tone_position(&self, style: ToneStyle) -> Option<usize> {
        tone_vowel(self.vowels(), self.ending(), style).map(|at| self.vowels.start + at)
    }

    /// Whether a tone key may mark the syllable. A y that is the whole vowel group never comes before a last consonant
    /// (the y of quỳnh follows the glide of qu, as uy), and no vowel is written twice but oo (moóc) and uu (as ưu), so
    /// those take no tone.
    pub(super) fn takes_tone(&self) -> bool {
        let after_qu = self.first_consonants().ends_with(&[BaseLetter::Q, BaseLetter::U]);
        let lone_y_closed = self.vowels() == [BaseLetter::Y] && self.ending() == Ending::Closed && !after_qu;
        let doubled = match self.vowels() {
            [first, second] => {
                let plain = plain_vowel(*first);
                plain == plain_vowel(*second) && !matches!(plain, Some(BaseLetter::O | BaseLetter::U))
            }
            _ => false,
        };
        !lone_y_closed && !doubled
    }

    /// The vowels `mark` goes on, or `None` when no vowel takes it.
    pub(super) fn mark_target(&self, mark: Mark) -> Option<MarkTarget> {
        let vowels = self.vowels();
        let start = self.vowels.start;
        let pair = horn_pair(vowels, mark).map(|u| MarkTarget::HornPair {
            u: start + u,
            o: start + u + 1,
        });
        pair.or_else(|| {
            vowels
                .iter()
                .enumerate()
                .filter(|(at, _)| !is_medial_o(vowels, *at))
                .find_map(|(at, vowel)| {
                    let plain = plain_vowel(*vowel)?;
                    let marked = mark.on(plain)?;
                    Some(MarkTarget::One {
                        at: start + at,
                        plain,
                        marked,
                    })
                })
        })
    }
}

/// The plain vowel `letter` is written on, or `None` for a consonant.
fn plain_vowel(letter: BaseLetter) -> Option<BaseLetter> {
    VOWELS
        .iter()
        .find(|(vowel, _)| *vowel == letter)
        .map(|(_, plain)| *plain)
}

pub(super) fn is_vowel(letter: BaseLetter) -> bool {
    plain_vowel(letter).is_some()
}

/// ă â ê ô ơ ư: a vowel with a mark of its own, which draws the tone to it.
pub(super) fn has_mark(letter: BaseLetter) -> bool {
    plain_vowel(letter).is_some_and(|plain| plain != letter)
}

/// Whether the first of `vowels` belongs to the `first_consonants` before them: the u after q always (qua, and qu
/// itself), the i of gi when another vowel but i follows (gia).
fn glide_belongs_to_first_consonants(first_consonants: &[BaseLetter], vowels: &[BaseLetter]) -> bool {
    match (first_consonants, vowels) {
        ([.., BaseLetter::Q], [BaseLetter::U, ..]) => true,
        ([BaseLetter::G], [BaseLetter::I, next, ..]) => *next != BaseLetter::I,
        _ => false,
    }
}

/// Which of `vowels` carries the tone.
fn tone_vowel(vowels: &[BaseLetter], ending: Ending, style: ToneStyle) -> Option<usize> {
    match vowels {
        [] => None,
        [_] => Some(0),
        [first, second] => Some(pair_tone_vowel(*first, *second, ending, style)),
        [_, _, ..] => Some(triple_tone_vowel(vowels, style)),
    }
}

/// Which vowel of a pair carries the tone: the second of oa, oe, uy before a consonant or in the new style, a vowel
/// with a mark of its own after another, the second of oo (moóc), and the second of uo written without its marks. The
/// other Vietnamese pairs end a syllable and carry it on their first vowel, which keeps it before a consonant only
/// when it has a mark of its own (ưi, but uac). Pairs Vietnamese lacks carry it on the first vowel old style, and new
/// style on the second, but for the open ie, ii, ou, and uu.
fn pair_tone_vowel(first: BaseLetter, second: BaseLetter, ending: Ending, style: ToneStyle) -> usize {
    let second_in_new_style = usize::from(style == ToneStyle::New);
    if is_glide_pair(first, second) {
        return if ending == Ending::Closed {
            1
        } else {
            second_in_new_style
        };
    }
    if has_mark(second) || matches!((first, second), (BaseLetter::U | BaseLetter::O, BaseLetter::O)) {
        return 1;
    }
    let open = ending == Ending::Open;
    if FIRST_VOWEL_PAIRS.contains(&(first, second)) {
        return usize::from(!open && !has_mark(first));
    }
    if open && OPEN_FIRST_VOWEL_PAIRS.contains(&(first, second)) {
        0
    } else {
        second_in_new_style
    }
}

/// Which of three vowels or more carries the tone: the middle one of a Vietnamese group, or the ê of uyê. A group
/// Vietnamese lacks carries it on the middle vowel too, but new style on the vowel after a glide pair (uye), and on a
/// first vowel with a mark of its own (ưii).
fn triple_tone_vowel(vowels: &[BaseLetter], style: ToneStyle) -> usize {
    let first_three = vowels.get(..3);
    if first_three == Some(&[BaseLetter::U, BaseLetter::Y, BaseLetter::ECircumflex]) {
        return 2;
    }
    let vietnamese = VIETNAMESE_TRIPLES.iter().any(|triple| first_three == Some(*triple));
    match (style, vowels) {
        (ToneStyle::New, [first, second, ..]) if !vietnamese && is_glide_pair(*first, *second) => 2,
        (ToneStyle::New, [first, ..]) if !vietnamese && has_mark(*first) => 0,
        _ => 1,
    }
}

/// oa, oe, uy: a glide then the main vowel.
fn is_glide_pair(first: BaseLetter, second: BaseLetter) -> bool {
    matches!(
        (first, second),
        (BaseLetter::O, BaseLetter::A | BaseLetter::E) | (BaseLetter::U, BaseLetter::Y)
    )
}

/// Where a u stands just before an o, when `mark` puts a horn on both.
fn horn_pair(vowels: &[BaseLetter], mark: Mark) -> Option<usize> {
    let horns_both =
        mark.on(BaseLetter::U) == Some(BaseLetter::UHorn) && mark.on(BaseLetter::O) == Some(BaseLetter::OHorn);
    if !horns_both {
        return None;
    }
    vowels.windows(2).position(|pair| {
        matches!(pair, [u, o] if plain_vowel(*u) == Some(BaseLetter::U) && plain_vowel(*o) == Some(BaseLetter::O))
    })
}

/// Whether the vowel at `at` is the o of oa or oe, a glide the mark passes over to the vowel after it.
fn is_medial_o(vowels: &[BaseLetter], at: usize) -> bool {
    let plain_at = |at: usize| vowels.get(at).and_then(|v| plain_vowel(*v));
    plain_at(at) == Some(BaseLetter::O) && matches!(plain_at(at + 1), Some(BaseLetter::A | BaseLetter::E))
}

#[cfg(test)]
mod tests;
