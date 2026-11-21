//! `cargo xtask hook pre-edit|pre-bash`: Claude Code runs this before every edit and shell command
//! (.claude/settings.json). Exit 2 blocks the call and the reason on stderr goes to the agent; a change to a protected
//! rule file instead prints an "ask" decision, so the user approves it in the permission prompt (ADR-0012). Any other
//! failure exits 1, which lets the call through, so a broken guard never bricks a session (ADR-0005).

use std::io::{self, Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use serde::Deserialize;

use crate::gates::guard::{APPROVAL_ENV, Repo, check_command, check_edit, repo_relative};

const BLOCK: u8 = 2;

#[derive(Default, Deserialize)]
struct Call {
    #[serde(default)]
    cwd: String,
    /// Absent in some calls; an empty mode is treated as unknown, which blocks rather than asks.
    #[serde(default)]
    permission_mode: String,
    #[serde(default)]
    tool_input: ToolInput,
}

#[derive(Default, Deserialize)]
struct ToolInput {
    #[serde(default)]
    file_path: String,
    #[serde(default)]
    notebook_path: String,
    #[serde(default)]
    command: String,
}

/// `built_root` is where this binary was built; the hook prefers the checkout it is run in (settings.json `cd`s
/// there), so a second checkout or worktree guards itself.
pub(crate) fn run(built_root: &Path, event: &str) -> ExitCode {
    // No size cap: a large Write payload that failed to parse would be let through.
    let mut input = String::new();
    let call: Call = match io::stdin()
        .read_to_string(&mut input)
        .map(|_| serde_json::from_str(&input))
    {
        Ok(Ok(call)) => call,
        Ok(Err(_)) | Err(_) => return ExitCode::FAILURE,
    };
    let root = std::env::current_dir()
        .ok()
        .filter(|dir| dir.join("architecture.toml").is_file())
        .unwrap_or_else(|| built_root.to_path_buf());
    #[expect(
        clippy::disallowed_methods,
        reason = "the approval lives in the environment that starts the session (ADR-0005)"
    )]
    let approved = std::env::var(APPROVAL_ENV).is_ok_and(|v| v == "1");
    let verdict = if event == "pre-edit" {
        edit(&root, &call, approved)
    } else {
        let root = slashed(&root);
        let cwd = if call.cwd.is_empty() {
            root.clone()
        } else {
            call.cwd.replace('\\', "/")
        };
        let repo = Repo { root: &root, cwd: &cwd };
        command_verdict(&call.tool_input.command, &repo, &call.permission_mode, approved)
    };
    match verdict {
        Verdict::Allow => ExitCode::SUCCESS,
        // An ask that cannot be written must not turn into a silent allow.
        Verdict::Ask(reason) if writeln!(io::stdout(), "{}", ask_json(&reason)).is_ok() => ExitCode::SUCCESS,
        Verdict::Ask(reason) | Verdict::Block(reason) => {
            // The reason is what the agent reads; if stderr is gone there is no one to tell.
            writeln!(io::stderr(), "{reason}").unwrap_or_default();
            ExitCode::from(BLOCK)
        }
    }
}

fn edit(root: &Path, call: &Call, approved: bool) -> Verdict {
    let input = &call.tool_input;
    let target = if input.file_path.is_empty() {
        &input.notebook_path
    } else {
        &input.file_path
    };
    if target.is_empty() {
        return Verdict::Allow;
    }
    let cwd = if call.cwd.is_empty() {
        root.to_path_buf()
    } else {
        PathBuf::from(&call.cwd)
    };
    let Some(rel) = relative(root, &cwd.join(target)) else {
        return Verdict::Allow;
    };
    // "Protected" means committed: a rule in HEAD was accepted; one still being written this session was not.
    let committed = |rel: &str| {
        Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["ls-tree", "-r", "--name-only", "HEAD"])
            .output()
            .is_ok_and(|o| o.status.success() && listed(&String::from_utf8_lossy(&o.stdout), rel))
    };
    edit_verdict(&rel, &call.permission_mode, approved, committed)
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    Allow,
    /// The user decides in Claude Code's permission prompt, which shows this reason.
    Ask(String),
    /// Refused outright; the reason goes to the agent.
    Block(String),
}

/// Modes in which Claude Code documents that a hook's "ask" reaches the user. The docs are silent for
/// `acceptEdits` and `bypassPermissions`, `dontAsk` denies every prompt, and an absent mode is unknown: in those a
/// protected change stays blocked rather than risk a prompt that is approved without the user seeing it.
const PROMPTING_MODES: [&str; 3] = ["default", "plan", "auto"];

pub(crate) fn command_verdict(cmd: &str, repo: &Repo<'_>, mode: &str, approved: bool) -> Verdict {
    // Approval lifts only the protected-file rule, so whatever is refused even with it is a hook bypass or a hang.
    if let Some(reason) = check_command(cmd, true, repo) {
        return Verdict::Block(reason);
    }
    if approved {
        return Verdict::Allow;
    }
    check_command(cmd, false, repo).map_or(Verdict::Allow, |reason| protected(&reason, mode))
}

pub(crate) fn edit_verdict(rel: &str, mode: &str, approved: bool, committed: impl Fn(&str) -> bool) -> Verdict {
    check_edit(rel, approved, committed).map_or(Verdict::Allow, |reason| protected(&reason, mode))
}

fn protected(reason: &str, mode: &str) -> Verdict {
    if PROMPTING_MODES.contains(&mode) {
        let reason = reason.trim_start_matches("BLOCKED: ");
        return Verdict::Ask(format!(
            "Protected rule file: approve only if you want the rule itself changed. {reason}"
        ));
    }
    Verdict::Block(format!(
        "{reason} (In permission mode `{mode}` a prompt might be approved without the user seeing it, so this stays \
         blocked.)"
    ))
}

/// The decision Claude Code reads from a `PreToolUse` hook's stdout to show its permission prompt.
pub(crate) fn ask_json(reason: &str) -> String {
    serde_json::json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "ask",
            "permissionDecisionReason": reason,
        }
    })
    .to_string()
}

/// A file inside the repository as a path relative to its root; `None` outside it, or for the root itself.
pub(crate) fn relative(root: &Path, path: &Path) -> Option<String> {
    let root = slashed(root);
    repo_relative(&root, &root, &slashed(path)).filter(|rel| !rel.is_empty())
}

/// Whether `head` (the output of `git ls-tree -r --name-only HEAD`) lists `rel`, ignoring case as Windows does.
pub(crate) fn listed(head: &str, rel: &str) -> bool {
    head.lines().any(|line| line.eq_ignore_ascii_case(rel))
}

fn slashed(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests;
