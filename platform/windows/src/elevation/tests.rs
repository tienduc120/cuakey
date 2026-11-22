use windows::Win32::Foundation::{ERROR_ACCESS_DENIED, ERROR_CANCELLED};

use super::{RelaunchError, command_line, relaunch_error};

#[test]
fn refused_prompt_is_cancelled() {
    let error = windows::core::Error::from(ERROR_CANCELLED.to_hresult());

    assert!(
        matches!(relaunch_error(error), RelaunchError::Cancelled),
        "a refused UAC prompt is its own case"
    );
}

#[test]
fn other_failure_is_not_cancelled() {
    let error = windows::core::Error::from(ERROR_ACCESS_DENIED.to_hresult());

    assert!(
        matches!(relaunch_error(error), RelaunchError::Launch(_)),
        "any other failure is a launch failure"
    );
}

#[test]
fn command_line_quotes_what_needs_it() {
    let args = ["--plain", "two words", "", r#"say "hi""#, r"ends\"].map(String::from);

    assert_eq!(command_line(&args), r#"--plain "two words" "" "say \"hi\"" ends\"#);
}

#[test]
fn trailing_backslash_is_doubled_before_the_closing_quote() {
    assert_eq!(command_line(&[r"a b\".to_owned()]), r#""a b\\""#);
}
