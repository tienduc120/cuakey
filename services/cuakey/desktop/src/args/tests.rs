use std::path::PathBuf;

use cuakey_interfaces::WindowKind;

use super::{ArgsError, Mode, parse, window_args, window_name};

fn args(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| (*word).to_owned()).collect()
}

#[test]
fn args_parse_modes() {
    assert_eq!(parse(&[]).unwrap(), Mode::Background { focus_log: None });
    let Ok(Mode::Ui(window)) = parse(&args(&["--ui", "settings"])) else {
        panic!("--ui starts the window process");
    };
    assert_eq!(window, WindowKind::Settings);
    let Ok(Mode::Background { focus_log }) = parse(&args(&["--focus-log", r"C:\logs\focus.log"])) else {
        panic!("--focus-log starts the background process");
    };
    assert_eq!(focus_log, Some(PathBuf::from(r"C:\logs\focus.log")));
    assert!(
        matches!(parse(&args(&["--frobnicate"])), Err(ArgsError::Unexpected(arg)) if arg == "--frobnicate"),
        "an unknown argument is refused"
    );
}

#[test]
fn every_window_is_asked_for_by_its_name() {
    for kind in [
        WindowKind::Settings,
        WindowKind::Macros,
        WindowKind::Convert,
        WindowKind::AlwaysOn,
        WindowKind::About,
    ] {
        assert_eq!(window_args(kind), ["--ui", window_name(kind)]);
        assert_eq!(parse(&args(&window_args(kind))).unwrap(), Mode::Ui(kind));
    }
}

#[test]
fn a_flag_without_its_value_is_refused() {
    for flag in ["--ui", "--focus-log"] {
        assert!(
            matches!(parse(&args(&[flag])), Err(ArgsError::MissingValue(named)) if named == flag),
            "{flag} needs a value"
        );
    }
}

#[test]
fn an_unknown_window_is_refused() {
    assert!(matches!(
        parse(&args(&["--ui", "games"])),
        Err(ArgsError::UnknownWindow(name)) if name == "games"
    ));
}

#[test]
fn a_window_and_a_focus_log_do_not_mix() {
    assert!(matches!(
        parse(&args(&["--ui", "settings", "--focus-log", "focus.log"])),
        Err(ArgsError::Unexpected(arg)) if arg == "--focus-log"
    ));
    assert!(matches!(
        parse(&args(&["--focus-log", "focus.log", "--ui", "settings"])),
        Err(ArgsError::Unexpected(arg)) if arg == "--ui"
    ));
}
