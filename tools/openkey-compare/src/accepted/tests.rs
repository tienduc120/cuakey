use cuakey_domain::{CodeTable, InputMethod, OnOff};

use super::{Acceptance, Found, parse};
use crate::combo::{Combo, Toggle};

const SOURCES: [&str; 2] = ["telex", "random"];

const FILE: &str = r#"
[[accepted]]
source = "telex"
sequence = "ass"
settings = "cp1258 !modern-tones"
step = 3
expected = "0061 0061 0073"
cuakey = "0061 0073"
reason = "the host deletes one unit of a two-unit letter"
"#;

fn cp1258(modern: OnOff) -> Combo {
    let defaults = Combo::default_for(InputMethod::Telex);
    Combo {
        table: CodeTable::Cp1258,
        toggles: defaults.toggles.with(Toggle::ModernTones, modern),
        ..defaults
    }
}

fn found<'a>(sequence: &'a str, tokens: &'a [String], step: usize, expected: &'a str) -> Found<'a> {
    Found {
        source: "telex",
        sequence,
        tokens,
        step,
        expected,
        cuakey: "0061 0073",
    }
}

fn acceptance(text: &str) -> Acceptance {
    Acceptance::new(parse(text, &SOURCES, &Combo::known_tokens()).unwrap())
}

#[test]
fn an_entry_accepts_only_its_own_divergence_under_settings_its_filter_allows() {
    let mut acceptance = acceptance(FILE);
    let old_tones = cp1258(OnOff::Off).tokens();
    let modern_tones = cp1258(OnOff::On).tokens();
    let unicode = Combo::default_for(InputMethod::Telex).tokens();

    assert_eq!(
        acceptance.accept(&found("ass", &old_tones, 3, "0061 0061 0073")),
        Some(0)
    );
    assert_eq!(
        acceptance.accept(&found("ass", &modern_tones, 3, "0061 0061 0073")),
        None
    );
    assert_eq!(acceptance.accept(&found("ass", &unicode, 3, "0061 0061 0073")), None);
    assert_eq!(acceptance.accept(&found("ass", &old_tones, 2, "0061 0061 0073")), None);
    assert_eq!(acceptance.accept(&found("ass", &old_tones, 3, "0061 0073")), None);
    assert_eq!(acceptance.accept(&found("as", &old_tones, 3, "0061 0061 0073")), None);
}

#[test]
fn an_entry_that_no_longer_occurs_is_stale_once_its_source_has_run() {
    let mut not_run = acceptance(FILE);
    let mut run = acceptance(FILE);
    let mut matched = acceptance(FILE);

    run.judge_source("telex");
    matched.judge_source("telex");
    matched.accept(&found("ass", &cp1258(OnOff::Off).tokens(), 3, "0061 0061 0073"));

    assert_eq!(not_run.stale().count(), 0, "a source that did not run judges nothing");
    assert_eq!(
        run.stale().map(|entry| entry.sequence.as_str()).collect::<Vec<_>>(),
        ["ass"]
    );
    assert_eq!(matched.stale().count(), 0);
    not_run.judge_source("random");
    assert_eq!(
        not_run.stale().count(),
        0,
        "running another source judges nothing of this one"
    );
}

#[test]
fn a_random_entry_is_judged_only_when_its_sequence_is_typed_under_its_settings() {
    let text = FILE.replace("source = \"telex\"", "source = \"random\"");
    let mut acceptance = acceptance(&text);

    acceptance.judge_sequence("random", "ass", &Combo::default_for(InputMethod::Telex).tokens());
    acceptance.judge_sequence("random", "as", &cp1258(OnOff::Off).tokens());
    assert_eq!(
        acceptance.stale().count(),
        0,
        "not typed under settings its filter allows"
    );

    acceptance.judge_sequence("random", "ass", &cp1258(OnOff::Off).tokens());
    assert_eq!(acceptance.stale().count(), 1);
}

#[test]
fn an_empty_file_accepts_nothing() {
    assert_eq!(parse("", &SOURCES, &Combo::known_tokens()).unwrap(), []);
}

#[test]
fn entries_that_cannot_match_are_refused() {
    let refused = |from: &str, to: &str| parse(&FILE.replace(from, to), &SOURCES, &Combo::known_tokens()).unwrap_err();

    assert_eq!(
        refused(
            "reason = \"the host deletes one unit of a two-unit letter\"",
            "reason = \" \""
        ),
        "accepted-differences.toml: `ass` in telex gives no reason"
    );
    assert_eq!(
        refused("cp1258 !modern-tones", "cp1258 modern"),
        "accepted-differences.toml: `ass` in telex names `modern`, which is no setting"
    );
    assert_eq!(
        refused("source = \"telex\"", "source = \"latin\""),
        "accepted-differences.toml: `ass` names `latin`, which is no corpus file, not convert, and not random"
    );
    assert_eq!(
        refused("step = 3", "step = 0"),
        "accepted-differences.toml: `ass` in telex gives step 0; the first key is step 1"
    );
}
