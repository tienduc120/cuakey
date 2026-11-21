//! Vietnamese spelling: whether a word is a Vietnamese syllable, could still become one, or never can. Spell check
//! reads it to leave other words as they were typed.

use super::tone::Syllable;
use crate::BaseLetter::{
    self, A, ABreve, ACircumflex, B, C, D, DStroke, E, ECircumflex, F, G, H, I, J, K, L, M, N, O, OCircumflex, OHorn,
    P, Q, R, S, T, U, UHorn, V, W, X, Y, Z,
};
use crate::OnOff;

/// How a word reads as Vietnamese.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Spelling {
    /// A whole Vietnamese syllable.
    Syllable,
    /// Not a syllable yet, but more letters can make it one.
    Start,
    /// No letters typed after it can make it Vietnamese.
    Foreign,
}

/// The consonants the settings let a word start or end with besides the Vietnamese ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Consonants {
    /// z, w, j, f may start a word.
    pub(super) zwjf_first: OnOff,
    /// f, j, w may start a word: they stand for ph, gi, qu.
    pub(super) quick_first: OnOff,
    /// g, h, k may end a word: they stand for ng, nh, ch.
    pub(super) quick_last: OnOff,
}

/// The consonants a Vietnamese syllable starts with. A lone p only starts one: ph.
const FIRST_CONSONANTS: [&[BaseLetter]; 26] = [
    &[B],
    &[C],
    &[C, H],
    &[D],
    &[DStroke],
    &[G],
    &[G, H],
    &[G, I],
    &[H],
    &[K],
    &[K, H],
    &[L],
    &[M],
    &[N],
    &[N, G],
    &[N, G, H],
    &[N, H],
    &[P, H],
    &[Q, U],
    &[R],
    &[S],
    &[T],
    &[T, H],
    &[T, R],
    &[V],
    &[X],
];

const ZWJF_FIRST: [&[BaseLetter]; 4] = [&[Z], &[W], &[J], &[F]];
const QUICK_FIRST: [&[BaseLetter]; 3] = [&[F], &[J], &[W]];

/// The consonants a Vietnamese syllable ends with.
const LAST_CONSONANTS: [&[BaseLetter]; 8] = [&[C], &[C, H], &[M], &[N], &[N, G], &[N, H], &[P], &[T]];

const QUICK_LAST: [&[BaseLetter]; 3] = [&[G], &[H], &[K]];

/// The vowel groups of Vietnamese syllables.
const VOWEL_GROUPS: [&[BaseLetter]; 55] = [
    &[A],
    &[ABreve],
    &[ACircumflex],
    &[E],
    &[ECircumflex],
    &[I],
    &[O],
    &[OCircumflex],
    &[OHorn],
    &[U],
    &[UHorn],
    &[Y],
    &[A, I],
    &[A, O],
    &[A, U],
    &[A, Y],
    &[ACircumflex, U],
    &[ACircumflex, Y],
    &[E, O],
    &[ECircumflex, U],
    &[I, A],
    &[I, U],
    &[I, ECircumflex],
    &[O, A],
    &[O, ABreve],
    &[O, E],
    &[O, I],
    &[O, O],
    &[OCircumflex, I],
    &[OHorn, I],
    &[U, A],
    &[U, ACircumflex],
    &[U, ECircumflex],
    &[U, I],
    &[U, OCircumflex],
    &[U, O],
    &[U, OHorn],
    &[U, Y],
    &[UHorn, A],
    &[UHorn, I],
    &[UHorn, U],
    &[UHorn, OHorn],
    &[Y, ECircumflex],
    &[O, A, I],
    &[O, A, Y],
    &[O, E, O],
    &[U, ACircumflex, Y],
    &[U, OCircumflex, I],
    &[UHorn, OHorn, I],
    &[U, Y, ECircumflex],
    &[I, ECircumflex, U],
    &[UHorn, OHorn, U],
    &[Y, ECircumflex, U],
    &[U, Y, A],
    &[U, Y, U],
];

/// How `letters` read as Vietnamese, with `consonants` allowed besides the Vietnamese ones. `None` stands for a
/// character that is no letter.
pub(super) fn spelling(letters: Option<&[BaseLetter]>, consonants: Consonants) -> Spelling {
    let Some(letters) = letters else {
        return Spelling::Foreign;
    };
    let syllable = Syllable::last(letters);
    if !syllable.starts_word() {
        return Spelling::Foreign;
    }
    let first = syllable.first_consonants();
    let last = syllable.last_consonants();
    let first_ok = first.is_empty() || first_consonants(consonants).any(|c| c == first);
    let last_ok = last.is_empty() || last_consonants(consonants).any(|c| c == last);
    if first_ok && last_ok && is_vowel_group(syllable.vowels(), last.is_empty()) {
        return Spelling::Syllable;
    }
    let first_starts = first_consonants(consonants).any(|c| c.starts_with(first));
    let last_starts = last_consonants(consonants).any(|c| c.starts_with(last));
    if first_starts && last_starts {
        Spelling::Start
    } else {
        Spelling::Foreign
    }
}

fn first_consonants(consonants: Consonants) -> impl Iterator<Item = &'static [BaseLetter]> {
    let zwjf = ZWJF_FIRST.iter().filter(move |_| consonants.zwjf_first.is_on());
    let quick = QUICK_FIRST.iter().filter(move |_| consonants.quick_first.is_on());
    FIRST_CONSONANTS.iter().chain(zwjf).chain(quick).copied()
}

fn last_consonants(consonants: Consonants) -> impl Iterator<Item = &'static [BaseLetter]> {
    let quick = QUICK_LAST.iter().filter(move |_| consonants.quick_last.is_on());
    LAST_CONSONANTS.iter().chain(quick).copied()
}

/// Whether `last` letters may follow a syllable's vowels as its last consonants, or start them, so marks and tones
/// still reach the vowels before them.
pub(super) fn may_end_syllable(last: &[BaseLetter], consonants: Consonants) -> bool {
    last.is_empty() || last_consonants(consonants).any(|c| c.starts_with(last))
}

/// Whether `vowels` are a Vietnamese vowel group, in a syllable that is `open` or not: uơ only ends one.
fn is_vowel_group(vowels: &[BaseLetter], open: bool) -> bool {
    VOWEL_GROUPS.contains(&vowels) && (open || vowels != [U, OHorn])
}

#[cfg(test)]
mod tests;
