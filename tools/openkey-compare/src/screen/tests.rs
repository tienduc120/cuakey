use cuakey_domain::{CodeTable, KeyCode, KeyMods, KeyPress, OnOff};

use super::{Screen, show};

#[test]
fn caps_lock_turns_letters_around_and_shift_turns_them_back() {
    let mut screen = Screen::default();
    let caps = KeyMods::none().with_caps_lock(OnOff::On);

    screen.key(KeyPress::new(KeyCode::V, caps));
    screen.key(KeyPress::new(KeyCode::V, caps.with_shift(OnOff::On)));
    screen.key(KeyPress::new(KeyCode::Digit1, caps));
    screen.key(KeyPress::new(KeyCode::C, KeyMods::none().with_ctrl_or_alt(OnOff::On)));

    assert_eq!(String::from_utf16_lossy(screen.units()), "Vv1");
}

#[test]
fn a_backspace_deletes_one_unit() {
    let mut screen = Screen::default();
    screen.push(&[0x61, 0xf9]);

    screen.key(KeyPress::new(KeyCode::Backspace, KeyMods::none()));

    assert_eq!(screen.units(), [0x61]);
}

#[test]
fn other_tables_show_their_units_in_hex() {
    assert_eq!(show(&[0xe2], CodeTable::Unicode), "`â`");
    assert_eq!(show(&[0x61, 0xe2], CodeTable::VniWindows), "0061 00e2 (`â`)");
    assert_eq!(show(&[], CodeTable::Tcvn3), "(empty) (``)");
}
