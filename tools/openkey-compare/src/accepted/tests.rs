use super::{find, parse};

const FILE: &str = r#"
[[accepted]]
sequence = "a<bs><bs>s"
settings = "telex unicode spell-check restore"
reason = "a Backspace on an empty box"
"#;

#[test]
fn an_entry_accepts_only_its_own_sequence_under_its_own_settings() {
    let accepted = parse(FILE).unwrap();

    assert!(find(&accepted, "a<bs><bs>s", "telex unicode spell-check restore").is_some());
    assert!(find(&accepted, "a<bs><bs>s", "telex unicode spell-check").is_none());
    assert!(find(&accepted, "a<bs>s", "telex unicode spell-check restore").is_none());
}

#[test]
fn an_empty_file_accepts_nothing() {
    assert_eq!(parse("").unwrap(), []);
}

#[test]
fn an_entry_without_a_reason_is_refused() {
    let error = parse("[[accepted]]\nsequence = \"x\"\nsettings = \"vni cp1258\"\nreason = \" \"\n").unwrap_err();

    assert_eq!(
        error,
        "accepted-differences.toml: `x` under `vni cp1258` gives no reason"
    );
}
