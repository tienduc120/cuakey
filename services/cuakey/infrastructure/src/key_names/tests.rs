use cuakey_domain::KeyCode;

use super::{NAMED, key_code, key_name};

#[test]
fn every_key_but_other_round_trips_through_its_name() {
    for code in NAMED {
        let name = key_name(*code).unwrap();

        assert_eq!(key_code(name), Some(*code), "{name} reads back as the key it names");
    }
}

#[test]
fn names_are_the_ones_a_person_would_write() {
    assert_eq!(key_name(KeyCode::F5), Some("F5"));
    assert_eq!(key_name(KeyCode::A), Some("A"));
    assert_eq!(key_name(KeyCode::Digit0), Some("Digit0"));
}

#[test]
fn no_two_keys_share_a_name() {
    let mut names: Vec<_> = NAMED.iter().filter_map(|code| key_name(*code)).collect();
    names.sort_unstable();
    let before = names.len();
    names.dedup();

    assert_eq!(names.len(), before);
    assert_eq!(before, NAMED.len(), "every listed key has a name");
}

#[test]
fn other_and_unknown_names_are_no_key() {
    assert_eq!(key_name(KeyCode::Other), None);
    assert_eq!(key_code("Other"), None);
    assert_eq!(key_code("Banana"), None);
    assert_eq!(key_code("f5"), None, "names are case-sensitive");
}
