use alloc::vec::Vec;

use super::{Consonants, Spelling, spelling};
use crate::{BaseLetter, Letter, OnOff};

const VIETNAMESE: Consonants = Consonants {
    zwjf_first: OnOff::Off,
    quick_first: OnOff::Off,
    quick_last: OnOff::Off,
};

fn letters(word: &str) -> Vec<BaseLetter> {
    word.chars().map(|c| Letter::from_char(c).unwrap().base()).collect()
}

fn spelled(word: &str, consonants: Consonants) -> Spelling {
    spelling(Some(&letters(word)), consonants)
}

#[test]
fn vietnamese_syllables_are_syllables() {
    for word in [
        "a",
        "tiêng",
        "việt",
        "nghiêng",
        "người",
        "khuya",
        "quốc",
        "giường",
        "gì",
        "oăn",
        "thuở",
        "khuỷu",
        "chuyên",
    ] {
        assert_eq!(spelled(word, VIETNAMESE), Spelling::Syllable, "{word}");
    }
}

#[test]
fn the_start_of_a_syllable_can_become_one() {
    for word in ["ngh", "q", "ae", "kh", "p"] {
        assert_eq!(spelled(word, VIETNAMESE), Spelling::Start, "{word}");
    }
}

#[test]
fn foreign_words_can_never_become_one() {
    for word in ["class", "hello", "toana", "tt", "java", "hag", "fan", "wa", "ze"] {
        assert_eq!(spelled(word, VIETNAMESE), Spelling::Foreign, "{word}");
    }
    assert_eq!(spelling(None, VIETNAMESE), Spelling::Foreign, "a word with a digit");
}

#[test]
fn the_vowels_must_be_a_vietnamese_vowel_group() {
    assert_eq!(spelled("hai", VIETNAMESE), Spelling::Syllable, "ai");
    assert_eq!(spelled("tiên", VIETNAMESE), Spelling::Syllable, "iê");
    assert_eq!(
        spelled("ă", VIETNAMESE),
        Spelling::Syllable,
        "ă on its own still reads as one"
    );
    assert_eq!(spelled("tưo", VIETNAMESE), Spelling::Start, "ưo is no vowel group");
    assert_eq!(spelled("ngôoc", VIETNAMESE), Spelling::Start, "nor is ôo");
}

#[test]
fn the_settings_allow_more_consonants() {
    let zwjf = Consonants {
        zwjf_first: OnOff::On,
        ..VIETNAMESE
    };
    let quick = Consonants {
        quick_first: OnOff::On,
        quick_last: OnOff::On,
        ..VIETNAMESE
    };

    assert_eq!(spelled("za", zwjf), Spelling::Syllable, "z first");
    assert_eq!(spelled("za", quick), Spelling::Foreign, "z stands for nothing");
    assert_eq!(spelled("fanh", quick), Spelling::Syllable, "f for ph");
    assert_eq!(spelled("hag", quick), Spelling::Syllable, "g for ng");
}

#[test]
fn uo_with_a_horn_on_the_o_alone_ends_a_syllable() {
    assert_eq!(spelled("thuơ", VIETNAMESE), Spelling::Syllable, "thuơ");
    assert_eq!(
        spelled("thuơng", VIETNAMESE),
        Spelling::Start,
        "before a consonant it is ươ"
    );
}

#[test]
fn ch_and_nh_follow_only_a_e_circumflex_and_i_and_the_groups_built_on_them() {
    for word in [
        "sách", "bếch", "thích", "huých", "khuếch", "oách", "anh", "lênh", "xinh", "huỳnh", "khuếnh", "oanh",
    ] {
        assert_eq!(spelled(word, VIETNAMESE), Spelling::Syllable, "{word}");
    }
    for word in ["âch", "ânh", "unh", "ônh", "ơch", "ech", "uch"] {
        assert_ne!(
            spelled(word, VIETNAMESE),
            Spelling::Syllable,
            "{word}: no Vietnamese syllable ends so"
        );
    }
}
