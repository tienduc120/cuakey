use std::time::Duration;

use cuakey_domain::{CodeTable, InputMethod, OnOff, encode};

use super::{CorpusTally, Count, Divergence, HostDefect, RunResult};
use crate::accepted::{Acceptance, Accepted, Found};
use crate::combo::{Combo, Toggle};
use crate::corpus::{CORPUS_FILES, Line, parse};
use crate::openkey_screen::Family;

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
        unmodelled: None,
    }
}

/// A run whose first divergence no entry accepts.
fn open(divergence: Divergence) -> RunResult {
    RunResult {
        open: Some(divergence),
        ..RunResult::default()
    }
}

fn no_entries() -> Acceptance {
    Acceptance::new(Vec::new())
}

/// An entry for `ajj` in the telex file under CP1258.
fn cp1258_entry() -> Acceptance {
    Acceptance::new(vec![Accepted {
        source: "telex".to_owned(),
        sequence: "ajj".to_owned(),
        settings: "cp1258".to_owned(),
        step: 3,
        expected: "0061 0061 006a".to_owned(),
        cuakey: "0061 006a".to_owned(),
        reason: "an engine difference this test accepts".to_owned(),
    }])
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
    tally.record(0, vni_windows, open(differs(2, "as", "á", CodeTable::VniWindows)));
    tally.record(0, defaults, open(differs(2, "as", "á", CodeTable::Unicode)));
    tally.record(0, no_spelling, RunResult::default());
    tally.record(1, defaults, RunResult::default());
    tally
}

#[test]
fn runs_are_counted_per_file_and_per_setting_that_differs_from_the_default() {
    let report = tally().render(Duration::from_secs(3), &no_entries(), None);

    assert_eq!(
        tally().total(),
        Count {
            runs: 4,
            differ: 2,
            accepted: 0
        }
    );
    assert!(
        report.contains("| telex | 2 | 4 | 2 (50.0%) | 0 | 0 | 1 | 1 |"),
        "{report}"
    );
    assert!(
        report.contains("| none (every setting at its default) | 2 | 1 (50.0%) | 1 |"),
        "{report}"
    );
    assert!(report.contains("| VNI Windows | 1 | 1 (100.0%) | 1 |"), "{report}");
    assert!(report.contains("| spell-check off | 1 | 0 (0.0%) | 0 |"), "{report}");
}

#[test]
fn one_entry_stands_for_every_combination_that_ends_the_same_way_and_gives_the_settings_they_share() {
    let report = tally().render(Duration::from_secs(3), &no_entries(), None);

    assert!(
        report.contains(
            "- `ass`, after key 3 `s`: expected `as`, CuaKey `á`. Settings `telex unicode spell-check restore \
             macros` and 1 more of 640 combinations, all matching `spell-check restore !modern-tones !quick-telex \
             !quick-first !quick-last !capitals`."
        ),
        "{report}"
    );
}

#[test]
fn a_difference_at_the_closing_space_says_so() {
    let mut tally = CorpusTally::default();
    tally.begin_file(CORPUS_FILES[0], vec![line("hag")]);

    tally.record(
        0,
        Combo::default_for(InputMethod::Telex),
        open(differs(3, "hang ", "hag ", CodeTable::Unicode)),
    );

    let report = tally.render(Duration::from_secs(1), &no_entries(), None);
    assert!(
        report.contains("- `hag`, after key 4, the Space that ends it: expected `hang `, CuaKey `hag `."),
        "{report}"
    );
}

#[test]
fn a_reply_the_tool_does_not_model_is_a_finding_and_not_the_end_of_the_run() {
    let mut tally = CorpusTally::default();
    tally.begin_file(CORPUS_FILES[0], vec![line("ab")]);

    tally.record(
        0,
        Combo::default_for(InputMethod::Telex),
        open(Divergence {
            unmodelled: Some(-1),
            ..differs(1, "a", "ab", CodeTable::Unicode)
        }),
    );

    let report = tally.render(Duration::from_secs(1), &no_entries(), None);
    assert!(
        report.contains("CuaKey `ab`. OpenKey gave a reply this tool does not model (-1), so the run stopped there."),
        "{report}"
    );
}

/// The telex file's `ajj` differing under CP1258, accepted by the entry when `accepted` says so.
fn cp1258() -> Combo {
    Combo {
        table: CodeTable::Cp1258,
        ..Combo::default_for(InputMethod::Telex)
    }
}

fn cp1258_tally(accepted: Option<usize>) -> CorpusTally {
    let combo = cp1258();
    let divergence = Divergence {
        step: 2,
        expected: vec![0x61, 0x61, 0x6a],
        actual: vec![0x61, 0x6a],
        unmodelled: None,
    };
    let mut tally = CorpusTally::default();
    tally.begin_file(CORPUS_FILES[0], vec![line("ajj")]);
    let result = match accepted {
        Some(at) => RunResult {
            accepted: vec![(divergence, at)],
            ..RunResult::default()
        },
        None => open(divergence),
    };
    tally.record(0, combo, result);
    tally
}

#[test]
fn an_accepted_difference_is_listed_apart_with_its_reason_and_the_run_passes() {
    let mut acceptance = cp1258_entry();
    acceptance.judge_source("telex");
    let tokens = cp1258().tokens();
    let accepted = acceptance.accept(&Found {
        source: "telex",
        sequence: "ajj",
        tokens: &tokens,
        step: 3,
        expected: "0061 0061 006a",
        cuakey: "0061 006a",
    });
    let tally = cp1258_tally(accepted);

    let report = tally.render(Duration::from_secs(1), &acceptance, None);

    assert!(
        report.contains("| telex | 1 | 1 | 1 (100.0%) | 1 | 0 | 0 | 0 |"),
        "{report}"
    );
    assert!(
        report.contains(
            "- `ajj`, after key 3 `j`: expected 0061 0061 006a (`aaj`), CuaKey 0061 006a (`aj`), under 1 \
             combination. Accepted: an engine difference this test accepts."
        ),
        "{report}"
    );
    assert!(tally.passed(&acceptance));
}

#[test]
fn a_difference_no_entry_accepts_fails_the_run() {
    let tally = cp1258_tally(None);

    assert!(!tally.passed(&no_entries()));
}

#[test]
fn an_entry_that_matched_nothing_is_reported_stale_and_fails_the_run() {
    let tally = CorpusTally::default();
    let mut acceptance = cp1258_entry();
    acceptance.judge_source("telex");

    let report = tally.render(Duration::from_secs(1), &acceptance, None);

    assert!(
        report.contains(
            "- `ajj` in telex, step 3, settings `cp1258`: expected `0061 0061 006a`, CuaKey `0061 006a`. Reason \
             given: an engine difference this test accepts."
        ),
        "{report}"
    );
    assert!(!tally.passed(&acceptance));
}

#[test]
fn host_defects_are_counted_apart_per_family_with_examples_and_never_fail_the_run() {
    let mut tally = CorpusTally::default();
    tally.begin_file(CORPUS_FILES[0], vec![line("ajj")]);
    let defect = HostDefect {
        family: Family::WholeLetter,
        step: 2,
        faithful: vec![0x61, 0x61, 0x6a],
        corrected: vec![0x61, 0x6a],
    };

    tally.record(
        0,
        cp1258(),
        RunResult {
            host_defect: Some(defect),
            ..RunResult::default()
        },
    );

    let report = tally.render(Duration::from_secs(1), &no_entries(), None);
    assert!(
        report.contains("| telex | 1 | 1 | 0 (0.0%) | 0 | 1 | 0 | 0 |"),
        "{report}"
    );
    assert!(report.contains("## OpenKey host defects (not reproduced)"), "{report}");
    assert!(report.contains("| telex | 1 | 0 | 0 |"), "{report}");
    assert!(
        report.contains(
            "- `ajj`, after key 3 `j`, settings `telex cp1258 spell-check restore macros`: OpenKey's host leaves \
             0061 0061 006a (`aaj`), a corrected host 0061 006a (`aj`)."
        ),
        "{report}"
    );
    assert!(tally.passed(&no_entries()));
}
