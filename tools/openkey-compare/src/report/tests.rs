use std::time::Duration;

use cuakey_domain::{CodeTable, InputMethod, OnOff, encode};

use super::{CorpusTally, Count, Divergence};
use crate::combo::{Combo, Toggle};
use crate::corpus::{CORPUS_FILES, Line, parse};

fn line(text: &str) -> Line {
    Line {
        text: text.to_owned(),
        strokes: parse(text).unwrap(),
    }
}

fn differs(step: usize, expected: &str, actual: &str, table: CodeTable) -> Divergence {
    Divergence {
        step,
        expected: encode(expected, table),
        actual: encode(actual, table),
    }
}

/// A tally of the telex file with two lines: `ass` differs under two combinations, `aa` never does.
fn tally() -> CorpusTally {
    let telex = CORPUS_FILES[0];
    let defaults = Combo::default_for(InputMethod::Telex);
    let vni_windows = Combo {
        table: CodeTable::VniWindows,
        ..defaults
    };
    let no_spelling = Combo {
        toggles: defaults.toggles.with(Toggle::SpellCheck, OnOff::Off),
        ..defaults
    };
    let mut tally = CorpusTally::default();
    tally.begin_file(telex, vec![line("ass"), line("aa")]);
    tally.record(0, vni_windows, Some(differs(2, "as", "á", CodeTable::VniWindows)));
    tally.record(0, defaults, Some(differs(2, "as", "á", CodeTable::Unicode)));
    tally.record(0, no_spelling, None);
    tally.record(1, defaults, None);
    tally
}

#[test]
fn runs_are_counted_per_file_and_per_setting_that_differs_from_the_default() {
    let report = tally().render(Duration::from_secs(3));

    assert_eq!(tally().total(), Count { runs: 4, differ: 2 });
    assert!(report.contains("| telex | 2 | 4 | 2 (50.0%) | 1 | 1 |"), "{report}");
    assert!(
        report.contains("| none (every setting at its default) | 2 | 1 (50.0%) |"),
        "{report}"
    );
    assert!(report.contains("| VNI Windows | 1 | 1 (100.0%) |"), "{report}");
    assert!(report.contains("| spell-check off | 1 | 0 (0.0%) |"), "{report}");
}

#[test]
fn one_entry_stands_for_every_combination_that_ends_the_same_way_and_shows_the_closest_to_the_defaults() {
    let report = tally().render(Duration::from_secs(3));

    assert!(
        report.contains(
            "- `ass`, after key 3 `s`: expected `as`, CuaKey `á`. Under `telex unicode spell-check restore` and 1 \
             more of 640 combinations, all with spell-check on, restore on, modern-tones off, quick-telex off, \
             quick-first off, quick-last off, capitals off."
        ),
        "{report}"
    );
    assert!(
        report.contains("## Not compared yet"),
        "macros and the conversion tool are named"
    );
}

#[test]
fn a_difference_at_the_closing_space_says_so() {
    let mut tally = CorpusTally::default();
    tally.begin_file(CORPUS_FILES[0], vec![line("hag")]);

    tally.record(
        0,
        Combo::default_for(InputMethod::Telex),
        Some(differs(3, "hang ", "hag ", CodeTable::Unicode)),
    );

    let report = tally.render(Duration::from_secs(1));
    assert!(
        report.contains("- `hag`, after the Space that ends it: expected `hang `, CuaKey `hag `."),
        "{report}"
    );
}
