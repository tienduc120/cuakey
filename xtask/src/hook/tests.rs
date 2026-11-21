use std::path::Path;

use super::{Verdict, ask_json, command_verdict, edit_verdict, relative};
use crate::gates::guard::Repo;

const REPO: Repo<'static> = Repo {
    root: "D:/repo",
    cwd: "D:/repo",
};

fn is_ask(v: &Verdict) -> bool {
    matches!(v, Verdict::Ask(_))
}

fn is_block(v: &Verdict) -> bool {
    matches!(v, Verdict::Block(_))
}

#[test]
fn a_hook_bypass_or_a_hanging_command_is_blocked_in_every_mode_even_when_approved() {
    for mode in ["default", "auto", "acceptEdits", "bypassPermissions"] {
        assert!(
            is_block(&command_verdict("git commit --no-verify -m x", &REPO, mode, false)),
            "{mode}"
        );
        assert!(is_block(&command_verdict("cargo test", &REPO, mode, true)), "{mode}");
    }
}

#[test]
fn a_shell_write_to_a_protected_file_asks_where_a_prompt_is_documented_to_reach_the_user() {
    for mode in ["default", "auto", "plan"] {
        assert!(is_ask(&command_verdict("rm clippy.toml", &REPO, mode, false)), "{mode}");
    }
}

#[test]
fn where_a_prompt_might_be_skipped_a_protected_write_stays_blocked() {
    for mode in ["acceptEdits", "bypassPermissions", "dontAsk", ""] {
        assert!(
            is_block(&command_verdict("rm clippy.toml", &REPO, mode, false)),
            "{mode:?}"
        );
    }
}

#[test]
fn the_session_approval_lets_a_protected_write_through_and_ordinary_commands_pass() {
    assert_eq!(
        command_verdict("rm clippy.toml", &REPO, "bypassPermissions", true),
        Verdict::Allow
    );
    assert_eq!(
        command_verdict("cargo nextest run -p xtask", &REPO, "default", false),
        Verdict::Allow
    );
}

#[test]
fn editing_a_committed_protected_file_asks_and_a_new_or_ordinary_file_does_not() {
    assert!(is_ask(&edit_verdict("clippy.toml", "auto", false, |_| true)));
    assert!(is_block(&edit_verdict("clippy.toml", "acceptEdits", false, |_| true)));
    assert_eq!(
        edit_verdict("clippy.toml", "auto", true, |_| true),
        Verdict::Allow,
        "approved"
    );
    assert_eq!(
        edit_verdict("clippy.toml", "auto", false, |_| false),
        Verdict::Allow,
        "not committed"
    );
    assert_eq!(
        edit_verdict("xtask/src/main.rs", "auto", false, |_| true),
        Verdict::Allow
    );
}

#[test]
fn an_ask_is_printed_as_the_json_claude_code_reads() {
    let json: serde_json::Value = serde_json::from_str(&ask_json("why it is protected")).unwrap();

    let out = &json["hookSpecificOutput"];
    assert_eq!(out["hookEventName"], "PreToolUse");
    assert_eq!(out["permissionDecision"], "ask");
    assert_eq!(out["permissionDecisionReason"], "why it is protected");
}

#[test]
fn a_path_inside_the_repository_becomes_relative_whatever_the_drive_letter_case_or_slashes() {
    let root = Path::new(r"D:\Duc\3_Project\rust-base");

    assert_eq!(
        relative(root, Path::new(r"d:\Duc\3_Project\rust-base\clippy.toml")).as_deref(),
        Some("clippy.toml")
    );
    assert_eq!(
        relative(root, Path::new("D:/Duc/3_Project/rust-base/xtask/src/gates/arch.rs")).as_deref(),
        Some("xtask/src/gates/arch.rs")
    );
}

#[test]
fn a_path_outside_the_repository_or_a_sibling_with_the_same_prefix_is_not_ours() {
    let root = Path::new(r"D:\Duc\3_Project\rust-base");

    assert_eq!(
        relative(root, Path::new(r"D:\Duc\3_Project\rust-base-old\clippy.toml")),
        None
    );
    assert_eq!(relative(root, Path::new(r"C:\elsewhere\clippy.toml")), None);
}

#[test]
fn dot_dot_is_resolved_before_the_path_is_judged() {
    let root = Path::new(r"D:\Duc\3_Project\rust-base");

    assert_eq!(
        relative(root, Path::new(r"D:\Duc\3_Project\rust-base\xtask\..\Cargo.toml")).as_deref(),
        Some("Cargo.toml")
    );
    assert_eq!(
        relative(root, Path::new(r"D:\Duc\3_Project\rust-base")),
        None,
        "the root is not a file"
    );
}

#[test]
fn committed_is_decided_against_heads_file_list_whatever_the_case() {
    let head = "Cargo.toml\nxtask/src/gates/arch.rs\n";

    assert!(super::listed(head, "cargo.TOML"));
    assert!(super::listed(head, "XTASK/src/gates/arch.rs"));
    assert!(!super::listed(head, "clippy.toml"));
}
