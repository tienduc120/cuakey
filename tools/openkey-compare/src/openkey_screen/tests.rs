use cuakey_domain::{CodeTable, InputMethod, encode};

use super::OpenKeyScreen;
use crate::combo::Combo;
use crate::corpus::parse;
use crate::oracle::OpenKey;

/// What the text box shows after `OpenKey` types `line` under `combo`.
fn typed(combo: Combo, line: &str) -> Vec<u16> {
    let mut engine = OpenKey::take().unwrap();
    engine.configure(combo);
    let mut screen = OpenKeyScreen::start(&mut engine, combo.table);
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
