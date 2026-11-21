use cuakey_domain::{CodeTable, InputMethod, Language, OnOff, encode};

use super::OpenKeyScreen;
use crate::combo::{Combo, Toggle};
use crate::corpus::parse;
use crate::oracle::OpenKey;

/// What the text box shows after `OpenKey` types `line` under `combo`.
fn typed(combo: Combo, line: &str) -> Vec<u16> {
    typed_with_macros(combo, line, &[])
}

/// The same, with `macros` as the macro table.
fn typed_with_macros(combo: Combo, line: &str, macros: &[(String, String)]) -> Vec<u16> {
    let mut engine = OpenKey::take().unwrap();
    engine.set_macros(macros);
    engine.configure(combo);
    let mut screen = OpenKeyScreen::start(&mut engine, combo);
    for stroke in parse(line).unwrap() {
        screen.stroke(stroke).unwrap();
    }
    screen.units().to_vec()
}

#[test]
fn openkey_aa_gives_a_circumflex() {
    let units = typed(Combo::default_for(InputMethod::Telex), "aa");

    assert_eq!(units, encode("â", CodeTable::Unicode), "Telex aa through OpenKey");
}

#[test]
fn openkey_deletes_both_units_of_a_two_unit_letter_with_one_backspace_in_vni_windows() {
    let combo = Combo {
        table: CodeTable::VniWindows,
        ..Combo::default_for(InputMethod::Telex)
    };

    let units = typed(combo, "tas<bs>");

    assert_eq!(
        units,
        encode("t", CodeTable::VniWindows),
        "the host deletes the whole of á"
    );
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
    let units = typed_with_macros(Combo::default_for(InputMethod::Telex), "btw ", &btw());

    assert_eq!(units, encode("by the way ", CodeTable::Unicode));
}

#[test]
fn in_english_mode_the_host_lets_the_ending_key_through_before_it_sends_the_expansion() {
    let units = typed_with_macros(english(OnOff::On), "btw ", &btw());

    assert_eq!(
        units,
        encode("bby the way ", CodeTable::Unicode),
        "the Space reaches the text box, then three backspaces delete it and `tw`, then the expansion and a Space"
    );
}

#[test]
fn in_english_mode_without_macros_in_english_every_key_passes_through() {
    let units = typed_with_macros(english(OnOff::Off), "btw ", &btw());

    assert_eq!(units, encode("btw ", CodeTable::Unicode));
}
