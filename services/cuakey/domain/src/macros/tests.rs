use alloc::string::String;

use super::{MacroEntry, MacroError, MacroTable};
use crate::OnOff;

fn table(pairs: &[(&str, &str)]) -> MacroTable {
    let mut table = MacroTable::default();
    for (shortcut, expansion) in pairs {
        table.add(MacroEntry::new(shortcut, expansion).unwrap());
    }
    table
}

#[test]
fn lookup_exact() {
    let table = table(&[("btw", "by the way")]);

    assert_eq!(
        table.lookup("btw", OnOff::Off),
        Some("by the way".into()),
        "the shortcut"
    );
    assert_eq!(table.lookup("btx", OnOff::Off), None, "another word");
    assert_eq!(
        table.lookup("Btw", OnOff::Off),
        None,
        "a capital does not match with match caps off"
    );
}

#[test]
fn lookup_match_caps() {
    let table = table(&[("btw", "by the way")]);

    assert_eq!(table.lookup("btw", OnOff::On), Some("by the way".into()), "lower case");
    assert_eq!(table.lookup("Btw", OnOff::On), Some("By the way".into()), "Capitalised");
    assert_eq!(table.lookup("BTW", OnOff::On), Some("BY THE WAY".into()), "ALL CAPS");
    assert_eq!(table.lookup("bTw", OnOff::On), None, "mixed case is no match");
}

#[test]
fn a_shortcut_written_in_capitals_matches_exactly_first() {
    let table = table(&[("BTW", "back then we"), ("btw", "by the way")]);

    assert_eq!(
        table.lookup("BTW", OnOff::On),
        Some("back then we".into()),
        "the exact shortcut wins"
    );
}

#[test]
fn a_single_capital_letter_is_capitalised_not_all_caps() {
    let table = table(&[("k", "không")]);

    assert_eq!(table.lookup("K", OnOff::On), Some("Không".into()), "one letter");
}

#[test]
fn match_caps_works_on_vietnamese_shortcuts() {
    let table = table(&[("đc", "được")]);

    assert_eq!(table.lookup("ĐC", OnOff::On), Some("ĐƯỢC".into()), "ALL CAPS");
    assert_eq!(table.lookup("Đc", OnOff::On), Some("Được".into()), "Capitalised");
}

#[test]
fn invalid_shortcuts_refused() {
    assert_eq!(MacroEntry::new("", "x"), Err(MacroError::Empty), "empty");
    assert_eq!(MacroEntry::new("a b", "x"), Err(MacroError::Whitespace), "space");
    assert_eq!(MacroEntry::new("a\tb", "x"), Err(MacroError::Whitespace), "tab");
    assert_eq!(MacroEntry::new("a:b", "x"), Err(MacroError::Colon), "colon");
}

#[test]
fn an_expansion_may_hold_a_colon_and_spaces() {
    let entry = MacroEntry::new("t", "ratio 1:2 ok").unwrap();

    assert_eq!(entry.expansion(), "ratio 1:2 ok", "kept whole");
}

#[test]
fn adding_an_existing_shortcut_replaces_its_expansion() {
    let mut table = table(&[("ko", "không")]);

    table.add(MacroEntry::new("ko", "không có").unwrap());

    assert_eq!(table.entries().count(), 1, "one entry");
    assert_eq!(
        table.lookup("ko", OnOff::Off),
        Some("không có".into()),
        "the new expansion"
    );
}

#[test]
fn entries_come_ordered_by_shortcut() {
    let table = table(&[("zz", "1"), ("aa", "2"), ("mm", "3")]);

    let shortcuts: alloc::vec::Vec<&str> = table.entries().map(MacroEntry::shortcut).collect();

    assert_eq!(shortcuts, ["aa", "mm", "zz"], "sorted");
}

#[test]
fn remove_takes_a_macro_out() {
    let mut table = table(&[("ko", "không")]);

    let removed = table.remove("ko");

    assert_eq!(
        removed.map(|e| String::from(e.expansion())),
        Some("không".into()),
        "gives it back"
    );
    assert_eq!(table.lookup("ko", OnOff::Off), None, "gone");
    assert_eq!(table.remove("ko"), None, "nothing to remove twice");
}
