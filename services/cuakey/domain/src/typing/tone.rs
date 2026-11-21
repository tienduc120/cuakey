//! Where the marks of a syllable go: which vowel carries the tone, and which vowel takes a key's vowel mark.

use core::ops::Range;

use super::Mark;
use crate::BaseLetter;

/// Where an open syllable of oa, oe, or uy carries its tone: old style on the first vowel (hòa), new style on the
/// second (hoà).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ToneStyle {
    Old,
    New,
}

/// Whether a syllable ends in a consonant, which moves the tone onto its last vowel.
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

/// A word read as one syllable: first consonant, vowels, final consonant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Syllable<'a> {
    letters: &'a [BaseLetter],
    vowels: Range<usize>,
    ending: Ending,
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

impl<'a> Syllable<'a> {
    /// `letters` read as a syllable, or `None` when a vowel follows the final consonant.
    pub(super) fn read(letters: &'a [BaseLetter]) -> Option<Self> {
        let first_vowel = letters.iter().position(|l| is_vowel(*l)).unwrap_or(letters.len());
        let start = first_vowel + usize::from(glide_belongs_to_first_consonant(letters, first_vowel));
        let end = start + letters.iter().skip(start).take_while(|l| is_vowel(**l)).count();
        if letters.iter().skip(end).any(|l| is_vowel(*l)) {
            return None;
        }
        let ending = if end < letters.len() {
            Ending::Closed
        } else {
            Ending::Open
        };
        Some(Self {
            letters,
            vowels: start..end,
            ending,
        })
    }

    pub(super) const fn ending(&self) -> Ending {
        self.ending
    }

    /// The place in the word of the vowel that carries the tone, or `None` when there is no vowel.
    pub(super) fn tone_position(&self, style: ToneStyle) -> Option<usize> {
        let vowels = self.letters.get(self.vowels.clone())?;
        tone_vowel(vowels, self.ending, style).map(|at| self.vowels.start + at)
    }

    /// The vowels `mark` goes on, or `None` when no vowel takes it.
    pub(super) fn mark_target(&self, mark: Mark) -> Option<MarkTarget> {
        let vowels = self.letters.get(self.vowels.clone())?;
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

fn is_vowel(letter: BaseLetter) -> bool {
    plain_vowel(letter).is_some()
}

/// ă â ê ô ơ ư: a vowel with a mark of its own, which draws the tone to it.
fn has_mark(letter: BaseLetter) -> bool {
    plain_vowel(letter).is_some_and(|plain| plain != letter)
}

/// Whether the i of gi or the u of qu belongs to the first consonant, as it does when another vowel follows: gia, qua.
fn glide_belongs_to_first_consonant(letters: &[BaseLetter], first_vowel: usize) -> bool {
    let consonant_and_glide = (letters.get(..first_vowel), letters.get(first_vowel));
    let vowel_follows = letters.get(first_vowel + 1).is_some_and(|l| is_vowel(*l));
    vowel_follows
        && matches!(
            consonant_and_glide,
            (Some([BaseLetter::G]), Some(BaseLetter::I)) | (Some([BaseLetter::Q]), Some(BaseLetter::U))
        )
}

/// Which of `vowels` carries the tone.
fn tone_vowel(vowels: &[BaseLetter], ending: Ending, style: ToneStyle) -> Option<usize> {
    if let Some(marked) = vowels.iter().rposition(|v| has_mark(*v)) {
        return Some(marked);
    }
    match (vowels, ending) {
        ([], _) => None,
        (_, Ending::Closed) => vowels.len().checked_sub(1),
        ([_], Ending::Open) => Some(0),
        ([first, second], Ending::Open) => Some(usize::from(style == ToneStyle::New && is_glide_pair(*first, *second))),
        (_, Ending::Open) => Some(1),
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
