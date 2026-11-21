use super::run;

#[test]
fn the_command_says_the_tool_is_not_built_yet_and_fails() {
    let error = run(&["--random".to_owned(), "5".to_owned()]).unwrap_err();

    assert_eq!(error.to_string(), "the comparison tool is not built yet");
}
