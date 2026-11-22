//! Vietnamese spelling: whether a word is a Vietnamese syllable, could still become one, or never can. Spell check
//! reads it to leave other words as they were typed.

use super::tone::Syllable;
use crate::BaseLetter::{
    self, A, ABreve, ACircumflex, B, C, D, DStroke, E, ECircumflex, F, G, H, I, J, K, L, M, N, O, OCircumflex, OHorn,
    P, Q, R, S, T, U, UHorn, V, W, X, Y, Z,
};
use crate::{OnOff, Tone};

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

/// The first consonants that go only before i, e, ê, and y.
const BEFORE_FRONT_VOWELS_ONLY: [&[BaseLetter]; 3] = [&[K], &[G, H], &[N, G, H]];
const QUICK_FIRST: [&[BaseLetter]; 3] = [&[F], &[J], &[W]];

/// The consonants a Vietnamese syllable ends with.
const LAST_CONSONANTS: [&[BaseLetter]; 8] = [&[C], &[C, H], &[M], &[N], &[N, G], &[N, H], &[P], &[T]];

const QUICK_LAST: [&[BaseLetter]; 3] = [&[G], &[H], &[K]];

/// The last consonants that stop the syllable short, which then takes only the acute or the dot below.
const STOPS: [&[BaseLetter]; 4] = [&[C], &[C, H], &[P], &[T]];

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
    if first_ok && last_ok && rhyme_fits(syllable.vowels(), last) {
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

/// The first letters of ph and qu, which a typist may write out as p and q alone before a vowel.
const STARTS_OF_PAIRS: [&[BaseLetter]; 2] = [&[P], &[Q]];

/// Whether a vowel a key types on its own, ư or ơ, may follow `first`: with no consonant before it, or after a first
/// consonant that goes before it, or the p or q that starts ph or qu. Quick first consonants do not count: f stands for
/// ph only once the word ends.
pub(super) fn may_precede_typed_vowel(first: &[BaseLetter], zwjf_first: OnOff) -> bool {
    let zwjf = ZWJF_FIRST.iter().filter(|_| zwjf_first.is_on());
    let mut known = FIRST_CONSONANTS.iter().chain(&STARTS_OF_PAIRS).chain(zwjf);
    first.is_empty() || (known.any(|c| *c == first) && !BEFORE_FRONT_VOWELS_ONLY.contains(&first))
}

/// Whether `last` letters may follow a syllable's vowels as its last consonants, or start them, so marks and tones
/// still reach the vowels before them.
pub(super) fn may_end_syllable(last: &[BaseLetter], consonants: Consonants) -> bool {
    last.is_empty() || last_consonants(consonants).any(|c| c.starts_with(last))
}

/// Whether a syllable ending in `last` consonants can carry `tone`: after c, ch, p, or t only the acute and the dot
/// below, as in mát, học, and hạp.
pub(super) fn tone_fits_last_consonants(tone: Tone, last: &[BaseLetter]) -> bool {
    !STOPS.contains(&last) || matches!(tone, Tone::Sac | Tone::Nang | Tone::Ngang)
}

/// Whether `vowels` are a Vietnamese vowel group.
pub(super) fn is_vowel_group(vowels: &[BaseLetter]) -> bool {
    VOWEL_GROUPS.contains(&vowels)
}

/// The vowels ch and nh follow: a, ê, and i, and the groups built on them (sách, bếch, thích, oách, khuếch, huých).
const BEFORE_CH_NH: [&[BaseLetter]; 7] = [&[A], &[ECircumflex], &[I], &[Y], &[O, A], &[U, ECircumflex], &[U, Y]];

/// The vowel groups that end a syllable themselves, in a semivowel (mai, kêu, tuổi) or as ia, ua, ưa, and uơ do: no
/// last consonant follows them.
const ENDING_GROUPS: [&[BaseLetter]; 30] = [
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
    &[O, I],
    &[OCircumflex, I],
    &[OHorn, I],
    &[U, A],
    &[U, I],
    &[UHorn, A],
    &[UHorn, I],
    &[UHorn, U],
    &[U, OHorn],
    &[O, A, I],
    &[O, A, Y],
    &[O, E, O],
    &[U, ACircumflex, Y],
    &[U, OCircumflex, I],
    &[UHorn, OHorn, I],
    &[UHorn, OHorn, U],
    &[I, ECircumflex, U],
    &[Y, ECircumflex, U],
    &[U, Y, A],
    &[U, Y, U],
];

/// Whether `vowels` are a Vietnamese vowel group that `last` consonants may follow: a group that ends a syllable takes
/// none, and ch and nh follow only the vowels that take them.
pub(super) fn rhyme_fits(vowels: &[BaseLetter], last: &[BaseLetter]) -> bool {
    let fits_last = match last {
        [] => true,
        _ if ENDING_GROUPS.contains(&vowels) => false,
        [C | N, H] => BEFORE_CH_NH.contains(&vowels),
        _ => true,
    };
    is_vowel_group(vowels) && fits_last
}

#[cfg(test)]
mod tests;
