use alloc::vec::Vec;

use super::{Ending, MarkTarget, Syllable, ToneStyle};
use crate::typing::Mark;
use crate::{BaseLetter, Letter};

/// Telex w.
const W: Mark = Mark(&[
    (BaseLetter::A, BaseLetter::ABreve),
    (BaseLetter::O, BaseLetter::OHorn),
    (BaseLetter::U, BaseLetter::UHorn),
]);

/// VNI 7.
const HORN: Mark = Mark(&[(BaseLetter::O, BaseLetter::OHorn), (BaseLetter::U, BaseLetter::UHorn)]);

/// VNI 6.
const CIRCUMFLEX: Mark = Mark(&[
    (BaseLetter::A, BaseLetter::ACircumflex),
    (BaseLetter::E, BaseLetter::ECircumflex),
    (BaseLetter::O, BaseLetter::OCircumflex),
]);

/// Telex o.
const CIRCUMFLEX_O: Mark = Mark(&[(BaseLetter::O, BaseLetter::OCircumflex)]);

fn letters(word: &str) -> Vec<BaseLetter> {
    word.chars().map(|c| Letter::from_char(c).unwrap().base()).collect()
}

fn tone_on(word: &str, style: ToneStyle) -> Option<usize> {
    let letters = letters(word);
    Syllable::read(&letters).and_then(|syllable| syllable.tone_position(style))
}

fn mark_on(word: &str, mark: Mark) -> Option<MarkTarget> {
    let letters = letters(word);
    Syllable::read(&letters).and_then(|syllable| syllable.mark_target(mark))
}

fn one(at: usize, plain: BaseLetter, marked: BaseLetter) -> MarkTarget {
    MarkTarget::One { at, plain, marked }
}

#[test]
fn a_vowel_with_a_mark_carries_the_tone() {
    for style in [ToneStyle::Old, ToneStyle::New] {
        assert_eq!(tone_on("tiêng", style), Some(2), "ê of tiêng");
        assert_eq!(tone_on("ngươi", style), Some(3), "ơ of ươi");
        assert_eq!(tone_on("cưu", style), Some(1), "ư of ưu");
        assert_eq!(tone_on("thuê", style), Some(3), "ê of uê");
    }
}

#[test]
fn a_closed_syllable_puts_the_tone_on_its_last_vowel() {
    for style in [ToneStyle::Old, ToneStyle::New] {
        assert_eq!(tone_on("toan", style), Some(2), "a of toan");
        assert_eq!(tone_on("huynh", style), Some(2), "y of huynh");
    }
}

#[test]
fn an_open_pair_puts_it_on_the_first_vowel() {
    for style in [ToneStyle::Old, ToneStyle::New] {
        assert_eq!(tone_on("mua", style), Some(1), "u of mua");
        assert_eq!(tone_on("mia", style), Some(1), "i of mia");
        assert_eq!(tone_on("cao", style), Some(1), "a of cao");
        assert_eq!(tone_on("mai", style), Some(1), "a of mai");
    }
}

#[test]
fn oa_oe_uy_put_it_on_the_first_vowel_old_style_and_the_second_new_style() {
    assert_eq!(tone_on("hoa", ToneStyle::Old), Some(1), "hòa");
    assert_eq!(tone_on("hoa", ToneStyle::New), Some(2), "hoà");
    assert_eq!(tone_on("hoe", ToneStyle::Old), Some(1), "hòe");
    assert_eq!(tone_on("hoe", ToneStyle::New), Some(2), "hoè");
    assert_eq!(tone_on("thuy", ToneStyle::Old), Some(2), "thủy");
    assert_eq!(tone_on("thuy", ToneStyle::New), Some(3), "thuỷ");
}

#[test]
fn three_open_vowels_put_it_on_the_middle_one() {
    assert_eq!(tone_on("ngoai", ToneStyle::Old), Some(3), "a of oai");
    assert_eq!(tone_on("khuyu", ToneStyle::Old), Some(3), "y of uyu");
}

#[test]
fn gi_and_qu_belong_to_the_first_consonant_before_another_vowel() {
    assert_eq!(tone_on("gia", ToneStyle::Old), Some(2), "a of gia");
    assert_eq!(tone_on("gi", ToneStyle::Old), Some(1), "i of gi");
    assert_eq!(tone_on("qua", ToneStyle::Old), Some(2), "a of qua");
    assert_eq!(tone_on("quy", ToneStyle::Old), Some(2), "y of quy");
}

#[test]
fn a_final_consonant_closes_the_syllable() {
    let open = letters("toa");
    let closed = letters("toan");

    assert_eq!(Syllable::read(&open).map(|s| s.ending()), Some(Ending::Open), "toa");
    assert_eq!(
        Syllable::read(&closed).map(|s| s.ending()),
        Some(Ending::Closed),
        "toan"
    );
}

#[test]
fn a_vowel_after_the_final_consonant_is_not_a_syllable() {
    let syllable = letters("hell");
    let more = letters("hello");

    assert!(Syllable::read(&syllable).is_some(), "hell is one syllable");
    assert_eq!(Syllable::read(&more), None, "hello is not");
}

#[test]
fn consonants_alone_are_a_syllable_with_no_vowel_to_mark() {
    let consonants = letters("nh");

    assert_eq!(
        Syllable::read(&consonants).map(|s| s.ending()),
        Some(Ending::Open),
        "a vowel may still follow"
    );
    assert_eq!(tone_on("nh", ToneStyle::Old), None, "no vowel for the tone");
    assert_eq!(mark_on("nh", W), None, "no vowel for the mark");
}

#[test]
fn w_marks_the_a_after_a_medial_o_and_the_u_before_a() {
    assert_eq!(
        mark_on("hoa", W),
        Some(one(2, BaseLetter::A, BaseLetter::ABreve)),
        "hoă"
    );
    assert_eq!(mark_on("mua", W), Some(one(1, BaseLetter::U, BaseLetter::UHorn)), "mưa");
    assert_eq!(
        mark_on("qua", W),
        Some(one(2, BaseLetter::A, BaseLetter::ABreve)),
        "quă"
    );
}

#[test]
fn u_and_o_take_the_horn_together() {
    assert_eq!(
        mark_on("duo", W),
        Some(MarkTarget::HornPair { u: 1, o: 2 }),
        "dươ with w"
    );
    assert_eq!(
        mark_on("thuong", HORN),
        Some(MarkTarget::HornPair { u: 2, o: 3 }),
        "thương with 7"
    );
    assert_eq!(
        mark_on("dưo", HORN),
        Some(MarkTarget::HornPair { u: 1, o: 2 }),
        "o joins ư"
    );
}

#[test]
fn a_circumflex_goes_on_the_vowel_it_fits() {
    assert_eq!(
        mark_on("tie", CIRCUMFLEX),
        Some(one(2, BaseLetter::E, BaseLetter::ECircumflex)),
        "tiê"
    );
    assert_eq!(
        mark_on("cuo", CIRCUMFLEX),
        Some(one(2, BaseLetter::O, BaseLetter::OCircumflex)),
        "cuô"
    );
    assert_eq!(mark_on("ngoe", CIRCUMFLEX_O), None, "the o of oe is a glide");
}

#[test]
fn a_marked_vowel_is_found_under_its_plain_one() {
    assert_eq!(
        mark_on("â", W),
        Some(one(0, BaseLetter::A, BaseLetter::ABreve)),
        "w turns â into ă"
    );
    assert_eq!(
        mark_on("ơ", CIRCUMFLEX_O),
        Some(one(0, BaseLetter::O, BaseLetter::OCircumflex)),
        "o turns ơ into ô"
    );
}

#[test]
fn the_last_syllable_of_a_longer_word_starts_after_the_vowel_before() {
    let word = letters("hello");
    let last = Syllable::last(&word);

    assert_eq!(last.start(), 2, "the l after e starts it");
    assert_eq!(last.first_consonants(), [BaseLetter::L, BaseLetter::L], "llo");
    assert_eq!(last.vowels(), [BaseLetter::O], "o");
}
