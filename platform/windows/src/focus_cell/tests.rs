use super::{FocusCell, FocusClass, focus_moving_key};

const TAB: u16 = 0x09;
const ENTER: u16 = 0x0d;
const ESCAPE: u16 = 0x1b;
const F6: u16 = 0x75;
const LETTER_A: u16 = 0x41;

#[test]
fn focus_moving_keys_invalidate() {
    let moving = [TAB, ENTER, ESCAPE, F6].map(|vk| focus_moving_key(vk, false));
    let letter = focus_moving_key(LETTER_A, false);
    let shortcut = focus_moving_key(LETTER_A, true);

    assert_eq!(moving, [true; 4], "Tab, Enter, Esc and F6 move the focus on their own");
    assert!(!letter, "a plain letter stays in the control it is typed into");
    assert!(shortcut, "a key with Ctrl, Alt or Win may switch windows or tabs");
}

#[test]
fn focus_cell_ignores_stale_set() {
    let cell = FocusCell::new();
    let first = cell.invalidate();
    let second = cell.invalidate();

    let stale_recorded = cell.set(FocusClass::Text, first);
    let after_stale = cell.get();
    let latest_recorded = cell.set(FocusClass::Password, second);
    let after_latest = cell.get();

    assert_eq!(
        (stale_recorded, latest_recorded),
        (false, true),
        "the cell says which answer it kept"
    );
    assert_eq!(
        after_stale,
        FocusClass::Pending,
        "a class worked out for the control the focus left is dropped"
    );
    assert_eq!(
        after_latest,
        FocusClass::Password,
        "the class for the latest focus is kept"
    );
}

#[test]
fn a_new_cell_waits_for_its_first_class() {
    let cell = FocusCell::new();

    assert_eq!(
        cell.get(),
        FocusClass::Pending,
        "nothing is known before the first answer"
    );
}

#[test]
fn every_class_reads_back() {
    let cell = FocusCell::new();
    let classes = [
        FocusClass::Text,
        FocusClass::Password,
        FocusClass::NotText,
        FocusClass::Unknown,
        FocusClass::Pending,
    ];

    let read = classes.map(|class| {
        let generation = cell.invalidate();
        cell.set(class, generation);
        cell.get()
    });

    assert_eq!(read, classes, "the cell holds whichever class it is given");
}
