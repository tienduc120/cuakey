use super::{CompareArgs, checkout_is_current, fetch_commands_for_sha, fetch_failure, tool_command};

fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(|a| (*a).to_owned()).collect()
}

#[test]
fn no_arguments_compare_the_whole_corpus() {
    let args = CompareArgs::parse(&[]).unwrap();

    assert_eq!(args, CompareArgs::default());
    assert!(args.tool_args().is_empty(), "nothing given, nothing forwarded");
}

#[test]
fn random_seed_and_filter_are_read_and_forwarded() {
    let given = strings(&["--random", "500", "--seed", "42"]);

    let args = CompareArgs::parse(&given).unwrap();

    assert_eq!(args.tool_args(), given);
    let filtered = CompareArgs::parse(&strings(&["--filter", "vni"])).unwrap();
    assert_eq!(filtered.tool_args(), strings(&["--filter", "vni"]));
}

#[test]
fn an_unknown_argument_is_refused_by_name() {
    let error = CompareArgs::parse(&strings(&["--fast"])).unwrap_err();

    assert_eq!(
        error.to_string(),
        "unknown argument `--fast`; expected --random N, --seed S, or --filter <corpus>"
    );
}

#[test]
fn a_flag_without_its_value_is_refused() {
    let error = CompareArgs::parse(&strings(&["--random"])).unwrap_err();

    assert_eq!(error.to_string(), "--random needs a value");
}

#[test]
fn counts_and_seeds_must_be_numbers() {
    let count = CompareArgs::parse(&strings(&["--random", "many"])).unwrap_err();
    let seed = CompareArgs::parse(&strings(&["--random", "5", "--seed", "-1"])).unwrap_err();

    assert_eq!(count.to_string(), "--random takes a whole number, not `many`");
    assert_eq!(seed.to_string(), "--seed takes a whole number, not `-1`");
}

#[test]
fn a_flag_given_twice_is_refused() {
    let error = CompareArgs::parse(&strings(&["--filter", "vni", "--filter", "telex"])).unwrap_err();

    assert_eq!(error.to_string(), "--filter is given twice");
}

#[test]
fn openkey_is_fetched_with_these_exact_git_invocations() {
    let sha = "89c2fd3bf258562f2349f89b49d81e2f140c3fc3";

    let commands = fetch_commands_for_sha(sha);

    assert_eq!(
        commands,
        vec![
            strings(&["git", "init"]),
            strings(&[
                "git",
                "fetch",
                "--depth",
                "1",
                "https://github.com/tuyenvm/OpenKey",
                sha
            ]),
            strings(&["git", "checkout", "FETCH_HEAD"]),
        ]
    );
}

#[test]
fn the_tool_runs_in_release_from_its_own_manifest_with_the_arguments_after_a_separator() {
    let args = CompareArgs::parse(&strings(&["--random", "3"])).unwrap();

    let command = tool_command(&args);

    assert_eq!(
        command,
        strings(&[
            "cargo",
            "run",
            "--release",
            "--manifest-path",
            "tools/openkey-compare/Cargo.toml",
            "--",
            "--random",
            "3",
        ])
    );
}

#[test]
fn a_failed_fetch_names_the_command_that_fetches_again() {
    let command = strings(&["git", "checkout", "FETCH_HEAD"]);

    let error = fetch_failure(&command);

    assert_eq!(
        error.to_string(),
        "fetching OpenKey failed at `git checkout FETCH_HEAD`; run `cargo xtask compare-openkey` again to fetch it"
    );
}

#[test]
fn a_checkout_at_another_commit_is_fetched_again() {
    assert!(checkout_is_current(Some("89c2fd3bf258562f2349f89b49d81e2f140c3fc3\n")));
    assert!(
        !checkout_is_current(Some("0123456789abcdef0123456789abcdef01234567\n")),
        "a checkout of another commit is stale"
    );
    assert!(!checkout_is_current(None), "a directory git cannot read is fetched");
}
