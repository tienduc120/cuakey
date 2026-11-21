use cuakey_domain::{CodeTable, InputMethod, MacroEntry, MacroTable, encode};

use super::CuaKeyScreen;
use crate::combo::Combo;
use crate::corpus::parse;

fn screen_for(table: CodeTable) -> CuaKeyScreen {
    let combo = Combo {
        table,
        ..Combo::default_for(InputMethod::Telex)
    };
    CuaKeyScreen::new(combo.cuakey_settings(), &MacroTable::default())
}

fn type_line(screen: &mut CuaKeyScreen, line: &str) {
    for stroke in parse(line).unwrap() {
        screen.stroke(stroke);
    }
}

#[test]
fn cuakey_aa_gives_a_circumflex() {
    let mut screen = screen_for(CodeTable::Unicode);

    type_line(&mut screen, "aa");

    assert_eq!(
        screen.units(),
        encode("â", CodeTable::Unicode),
        "Telex aa through CuaKey"
    );
}

#[test]
fn screen_applies_vni_two_unit_backspaces() {
    let mut screen = screen_for(CodeTable::VniWindows);
    type_line(&mut screen, "as");
    assert_eq!(screen.units(), encode("á", CodeTable::VniWindows));
    assert_eq!(screen.units().len(), 2, "á takes two units in VNI Windows");

    // The second s takes the tone back: CuaKey deletes the one character á, which is two backspaces here.
    type_line(&mut screen, "s");

    assert_eq!(screen.units(), encode("as", CodeTable::VniWindows));
}

#[test]
fn cuakey_expands_a_macro() {
    let mut macros = MacroTable::default();
    macros.add(MacroEntry::new("btw", "by the way").unwrap());
    let mut screen = CuaKeyScreen::new(Combo::default_for(InputMethod::Telex).cuakey_settings(), &macros);

    type_line(&mut screen, "btw ");

    assert_eq!(screen.units(), encode("by the way ", CodeTable::Unicode));
}
