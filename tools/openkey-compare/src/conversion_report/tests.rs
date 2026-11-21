use cuakey_domain::{CodeTable, ConvertCase, OnOff, encode};

use super::{ConversionTally, Expected};
use crate::conversion::{ConvertCombo, Text};

fn combo(from: CodeTable, case: ConvertCase) -> ConvertCombo {
    ConvertCombo {
        from,
        to: CodeTable::Unicode,
        case,
        remove_accents: OnOff::Off,
    }
}

fn tally() -> ConversionTally {
    let texts = vec![Text {
        line: "Việt Nam".to_owned(),
        text: "Việt Nam".to_owned(),
    }];
    let mut tally = ConversionTally::new(texts);
    let (expected, actual) = (
        encode("việt nam", CodeTable::Unicode),
        encode("Việt Nam", CodeTable::Unicode),
    );
    tally.record(
        0,
        combo(CodeTable::Unicode, ConvertCase::Keep),
        Some((Expected::Text(expected.clone()), actual.clone())),
        None,
    );
    tally.record(
        0,
        combo(CodeTable::Tcvn3, ConvertCase::Keep),
        Some((Expected::Text(expected), actual)),
        None,
    );
    tally.record(0, combo(CodeTable::Unicode, ConvertCase::Upper), None, None);
    tally
}

#[test]
fn conversions_that_end_the_same_way_read_as_one_entry_with_the_settings_they_share() {
    let mut out = String::new();

    tally().render_differences(&mut out);

    assert!(
        out.contains(
            "- `Việt Nam`: expected `việt nam`, CuaKey `Việt Nam`. Settings `from-unicode to-unicode keep` and 1 more \
             of 250 settings, all matching `to-unicode keep !remove-accents`."
        ),
        "{out}"
    );
}

#[test]
fn the_summary_counts_runs_per_setting() {
    let mut out = String::new();

    tally().render_summary(&mut out);

    assert!(out.contains("| convert | 1 | 3 | 2 (66.6%) | 0 | 1 | 1 |"), "{out}");
    assert!(out.contains("| keep | 2 | 2 (100.0%) | 2 |"), "{out}");
    assert!(out.contains("| from-tcvn3 | 1 | 1 (100.0%) | 1 |"), "{out}");
}

#[test]
fn a_conversion_openkey_cannot_finish_is_a_finding() {
    let texts = vec![Text {
        line: "nhé".to_owned(),
        text: "nhé".to_owned(),
    }];
    let mut tally = ConversionTally::new(texts);
    let combo = ConvertCombo {
        to: CodeTable::VniWindows,
        ..combo(CodeTable::Unicode, ConvertCase::Upper)
    };
    tally.record(
        0,
        combo,
        Some((Expected::Failed, encode("NHÉ", CodeTable::VniWindows))),
        None,
    );
    let mut out = String::new();

    tally.render_differences(&mut out);

    assert!(
        out.contains(
            "- `nhé`: expected OpenKey's conversion to stop with an error, CuaKey 004e 0048 0045 00d9 (`NHÉ`)."
        ),
        "{out}"
    );
}
