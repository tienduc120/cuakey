//! The quick keys: a consonant typed twice for the pair it starts (quick Telex), and the single consonants that stand
//! for a Vietnamese pair at the start or the end of a word.

use crate::BaseLetter;

/// Quick Telex: each consonant typed twice, and the letter its second press types: cc ch, gg gi, kk kh, nn ng, pp ph,
/// qq qu, tt th.
const DOUBLED: [(BaseLetter, BaseLetter); 7] = [
    (BaseLetter::C, BaseLetter::H),
    (BaseLetter::G, BaseLetter::I),
    (BaseLetter::K, BaseLetter::H),
    (BaseLetter::N, BaseLetter::G),
    (BaseLetter::P, BaseLetter::H),
    (BaseLetter::Q, BaseLetter::U),
    (BaseLetter::T, BaseLetter::H),
];

/// Quick first consonant: f, j, w at the start of a word stand for ph, gi, qu.
const FIRST: [(BaseLetter, [BaseLetter; 2]); 3] = [
    (BaseLetter::F, [BaseLetter::P, BaseLetter::H]),
    (BaseLetter::J, [BaseLetter::G, BaseLetter::I]),
    (BaseLetter::W, [BaseLetter::Q, BaseLetter::U]),
];

/// Quick last consonant: g, h, k at the end of a word stand for ng, nh, ch.
const LAST: [(BaseLetter, [BaseLetter; 2]); 3] = [
    (BaseLetter::G, [BaseLetter::N, BaseLetter::G]),
    (BaseLetter::H, [BaseLetter::N, BaseLetter::H]),
    (BaseLetter::K, [BaseLetter::C, BaseLetter::H]),
];

/// The letter a second press of `consonant` types in quick Telex.
pub(super) fn second_of_doubled(consonant: BaseLetter) -> Option<BaseLetter> {
    DOUBLED
        .iter()
        .find(|(first, _)| *first == consonant)
        .map(|(_, second)| *second)
}

/// The pair `consonant` stands for at the start of a word.
pub(super) fn first_pair(consonant: BaseLetter) -> Option<[BaseLetter; 2]> {
    FIRST
        .iter()
        .find(|(single, _)| *single == consonant)
        .map(|(_, pair)| *pair)
}

/// The pair `consonant` stands for at the end of a word.
pub(super) fn last_pair(consonant: BaseLetter) -> Option<[BaseLetter; 2]> {
    LAST.iter()
        .find(|(single, _)| *single == consonant)
        .map(|(_, pair)| *pair)
}

#[cfg(test)]
mod tests;
