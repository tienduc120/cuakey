use cuakey_domain::{CodeTable, ConvertCase, OnOff, encode};

use super::{ConvertCombo, cuakey_output, input, openkey_output, parse_text};
use crate::oracle::OpenKey;

fn unicode_to(to: CodeTable, case: ConvertCase) -> ConvertCombo {
    ConvertCombo {
        from: CodeTable::Unicode,
        to,
        case,
        remove_accents: OnOff::Off,
    }
}

#[test]
fn every_pair_of_tables_meets_every_case_with_and_without_accents() {
    let combos: Vec<ConvertCombo> = ConvertCombo::all().collect();

    assert_eq!(combos.len(), 5 * 5 * 5 * 2);
    assert_eq!(
        unicode_to(CodeTable::Tcvn3, ConvertCase::SentenceStart).to_string(),
        "from-unicode to-tcvn3 sentence-start"
    );
    assert_eq!(
        unicode_to(CodeTable::Tcvn3, ConvertCase::Keep).tokens(),
        ["from-unicode", "to-tcvn3", "keep", "!remove-accents"]
    );
}

#[test]
fn texts_write_line_breaks_and_invisible_characters_as_escapes() {
    assert_eq!(parse_text(r"a\nb"), Ok("a\nb".to_owned()));
    assert_eq!(parse_text(r"tie\u{302}\u{301}ng"), Ok("tie\u{302}\u{301}ng".to_owned()));
    assert_eq!(parse_text(r"a\\b"), Ok("a\\b".to_owned()));
    assert_eq!(
        parse_text(r"a\x"),
        Err(r"`\x` is no escape; write \n, \\, or \u{...}".to_owned())
    );
}

#[test]
fn both_engines_read_the_text_as_the_units_of_the_from_table() {
    assert_eq!(
        input("tiếng", CodeTable::VniWindows)
            .chars()
            .map(u32::from)
            .collect::<Vec<_>>(),
        encode("tiếng", CodeTable::VniWindows)
            .into_iter()
            .map(u32::from)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        input("a😀", CodeTable::Tcvn3),
        "a😀",
        "an emoji is no unit of any table and stays itself"
    );
}

#[test]
fn cuakey_converts_unicode_to_vni_windows() {
    let combo = unicode_to(CodeTable::VniWindows, ConvertCase::Keep);

    let output = cuakey_output(&input("Tiếng", CodeTable::Unicode), combo);

    assert_eq!(output, encode("Tiếng", CodeTable::VniWindows));
}

#[test]
fn openkey_raises_vietnamese_letters_when_converting_to_upper_case() {
    let mut engine = OpenKey::take().unwrap();
    let combo = unicode_to(CodeTable::Unicode, ConvertCase::Upper);

    let output = openkey_output(&mut engine, &input("việt nam", CodeTable::Unicode), combo);

    assert_eq!(output, Some(encode("VIỆT NAM", CodeTable::Unicode)));
}

#[test]
fn openkey_gives_nothing_when_its_conversion_stops_with_an_error() {
    let mut engine = OpenKey::take().unwrap();
    let combo = unicode_to(CodeTable::VniWindows, ConvertCase::Upper);

    let output = openkey_output(&mut engine, &input("nhé", CodeTable::Unicode), combo);

    assert_eq!(
        output, None,
        "OpenKey's own conversion of this text stops with an error"
    );
}
