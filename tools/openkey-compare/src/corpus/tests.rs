use cuakey_domain::{KeyCode, KeyMods, KeyPress, OnOff};

use super::{Stroke, notation, parse};

fn key(code: KeyCode, mods: KeyMods) -> Stroke {
    Stroke::Key(KeyPress::new(code, mods))
}

#[test]
fn characters_are_the_keys_that_type_them_and_names_the_other_keys() {
    let strokes = parse("Aa1!<bs><click><ctrl-c>").unwrap();

    let none = KeyMods::none();
    let shift = none.with_shift(OnOff::On);
    assert_eq!(
        strokes,
        [
            key(KeyCode::A, shift),
            key(KeyCode::A, none),
            key(KeyCode::Digit1, none),
            key(KeyCode::Digit1, shift),
            key(KeyCode::Backspace, none),
            Stroke::Click,
            key(KeyCode::C, none.with_ctrl_or_alt(OnOff::On)),
        ]
    );
}

#[test]
fn caps_lock_is_a_state_of_the_keys_after_it() {
    let strokes = parse("<caps>v<caps>v").unwrap();

    let caps = KeyMods::none().with_caps_lock(OnOff::On);
    assert_eq!(strokes, [key(KeyCode::V, caps), key(KeyCode::V, KeyMods::none())]);
}

#[test]
fn the_notation_reads_back_as_the_same_strokes() {
    let line = "<caps>TOO<caps>ij <lt>x>{ss}<bs><enter><tab><esc>";

    let strokes = parse(line).unwrap();

    assert_eq!(notation(&strokes), line);
}

#[test]
fn an_unknown_key_name_or_character_is_refused() {
    assert_eq!(
        parse("a<shift>").unwrap_err(),
        "`<shift>` is no key this notation knows"
    );
    assert_eq!(parse("ă").unwrap_err(), "`ă` is no key on a US layout");
    assert_eq!(parse("a<bs").unwrap_err(), "`<bs` opens a key name it never closes");
}

#[test]
fn the_comparison_ends_every_line_with_a_space() {
    let line = super::Line {
        text: "hag".to_owned(),
        strokes: parse("hag").unwrap(),
    };

    assert_eq!(line.ended(), parse("hag ").unwrap());
}

#[test]
fn the_macro_table_splits_each_line_at_its_first_colon_and_skips_comments() {
    let table = super::parse_macro_table("; the table\nbtw:by the way\n\ntg:thời gian: 8:30\n").unwrap();

    assert_eq!(
        table,
        [
            ("btw".to_owned(), "by the way".to_owned()),
            ("tg".to_owned(), "thời gian: 8:30".to_owned())
        ]
    );
    assert_eq!(
        super::parse_macro_table("btw by the way").unwrap_err(),
        "`btw by the way` has no colon between the shortcut and its expansion"
    );
}
