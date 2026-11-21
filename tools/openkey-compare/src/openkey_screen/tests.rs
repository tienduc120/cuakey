use cuakey_domain::{CodeTable, InputMethod, Language, OnOff, encode};

use super::{Family, OpenKeyScreens};
use crate::combo::{Combo, Toggle};
use crate::corpus::parse;
use crate::oracle::OpenKey;

/// What the two hosts leave after `OpenKey` types `line` under `combo` with `macros`: the corrected host's text, the
/// faithful host's, and the last host defect the faithful host showed.
fn typed_with_macros(combo: Combo, line: &str, macros: &[(String, String)]) -> (Vec<u16>, Vec<u16>, Option<Family>) {
    let mut engine = OpenKey::take().unwrap();
    engine.set_macros(macros);
    engine.configure(combo);
    let mut screens = OpenKeyScreens::start(&mut engine, combo);
    for stroke in parse(line).unwrap() {
        screens.stroke(stroke).unwrap();
    }
    (
        screens.corrected().to_vec(),
        screens.faithful().to_vec(),
        screens.defect(),
    )
}

fn typed(combo: Combo, line: &str) -> (Vec<u16>, Vec<u16>, Option<Family>) {
    typed_with_macros(combo, line, &[])
}

fn telex_in(table: CodeTable) -> Combo {
    Combo {
        table,
        ..Combo::default_for(InputMethod::Telex)
    }
}

#[test]
fn openkey_aa_gives_a_circumflex() {
    let (corrected, faithful, defect) = typed(Combo::default_for(InputMethod::Telex), "aa");

    assert_eq!(corrected, encode("â", CodeTable::Unicode), "Telex aa through OpenKey");
    assert_eq!(faithful, corrected);
    assert_eq!(defect, None);
}

#[test]
fn both_hosts_delete_both_units_of_a_two_unit_letter_with_one_backspace_in_vni_windows() {
    let (corrected, faithful, _) = typed(telex_in(CodeTable::VniWindows), "tas<bs>");

    assert_eq!(
        corrected,
        encode("t", CodeTable::VniWindows),
        "the host deletes the whole of á"
    );
    assert_eq!(faithful, corrected);
}

#[test]
fn the_corrected_host_deletes_the_whole_of_a_cp1258_letter_and_the_faithful_host_one_unit() {
    let (corrected, faithful, defect) = typed(telex_in(CodeTable::Cp1258), "ass");

    assert_eq!(corrected, encode("as", CodeTable::Cp1258));
    assert_eq!(
        faithful,
        [0x61, 0x61, 0x73],
        "OpenKey's host leaves the first unit of á"
    );
    assert_eq!(defect, Some(Family::WholeLetter));
}

#[test]
fn a_space_right_after_a_backspace_deletes_half_a_letter_only_under_the_faithful_host() {
    let (corrected, faithful, defect) = typed(telex_in(CodeTable::VniWindows), "vieetj <bs><bs> ");

    assert_eq!(corrected, encode("việ ", CodeTable::VniWindows));
    assert_eq!(
        faithful,
        [0x76, 0x69, 0x65, 0x20],
        "OpenKey's host deletes the second unit of ệ"
    );
    assert_eq!(defect, Some(Family::SpaceAfterBackspace));
}

fn btw() -> Vec<(String, String)> {
    vec![("btw".to_owned(), "by the way".to_owned())]
}

fn english(macros_in_english: OnOff) -> Combo {
    let defaults = Combo::default_for(InputMethod::Telex);
    Combo {
        language: Language::English,
        toggles: defaults.toggles.with(Toggle::MacrosInEnglish, macros_in_english),
        ..defaults
    }
}

#[test]
fn openkey_expands_a_macro_and_types_the_key_that_ended_it() {
    let (corrected, faithful, _) = typed_with_macros(Combo::default_for(InputMethod::Telex), "btw ", &btw());

    assert_eq!(corrected, encode("by the way ", CodeTable::Unicode));
    assert_eq!(faithful, corrected);
}

#[test]
fn in_english_mode_only_the_faithful_host_lets_the_ending_key_through_before_the_expansion() {
    let (corrected, faithful, defect) = typed_with_macros(english(OnOff::On), "btw ", &btw());

    assert_eq!(corrected, encode("by the way ", CodeTable::Unicode));
    assert_eq!(
        faithful,
        encode("bby the way ", CodeTable::Unicode),
        "the Space reaches the text box, then three backspaces delete it and `tw`, then the expansion and a Space"
    );
    assert_eq!(defect, Some(Family::EnglishMacroKey));
}

#[test]
fn in_english_mode_without_macros_in_english_every_key_passes_through() {
    let (corrected, faithful, _) = typed_with_macros(english(OnOff::Off), "btw ", &btw());

    assert_eq!(corrected, encode("btw ", CodeTable::Unicode));
    assert_eq!(faithful, corrected);
}
