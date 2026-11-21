//! `cargo xtask compare-openkey`: compares the typing engine with `OpenKey` as a black box, through a tool in `tools/`
//! that is its own Cargo project outside the workspace (ADR-0014). `OpenKey`'s sources are fetched into
//! `target/openkey/` at a pinned commit, so they are never committed.

use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context as _, Result, bail};

/// The `OpenKey` commit the comparison is made against: moving it changes what the tool calls expected.
const OPENKEY_SHA: &str = "89c2fd3bf258562f2349f89b49d81e2f140c3fc3";
const OPENKEY_REPOSITORY: &str = "https://github.com/tuyenvm/OpenKey";
const OPENKEY_DIR: &str = "target/openkey";
/// A file only a finished checkout has, so an interrupted fetch is fetched again.
const OPENKEY_MARKER: &str = "Sources/OpenKey/engine/Engine.cpp";
const TOOL_MANIFEST: &str = "tools/openkey-compare/Cargo.toml";

/// What the command was asked to compare: the corpus (optionally one file of it), or seeded random sequences.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct CompareArgs {
    random: Option<u32>,
    seed: Option<u64>,
    filter: Option<String>,
}

impl CompareArgs {
    pub(crate) fn parse(args: &[String]) -> Result<Self> {
        let mut parsed = Self::default();
        let mut rest = args.iter();
        while let Some(flag) = rest.next() {
            let mut value = || rest.next().with_context(|| format!("{flag} needs a value"));
            match flag.as_str() {
                "--random" => set_once(&mut parsed.random, flag, number(flag, value()?)?)?,
                "--seed" => set_once(&mut parsed.seed, flag, number(flag, value()?)?)?,
                "--filter" => set_once(&mut parsed.filter, flag, value()?.clone())?,
                _ => bail!("unknown argument `{flag}`; expected --random N, --seed S, or --filter <corpus>"),
            }
        }
        Ok(parsed)
    }

    /// The arguments the comparison tool is run with, in the order the command reads them.
    pub(crate) fn tool_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        if let Some(count) = self.random {
            args.extend(["--random".to_owned(), count.to_string()]);
        }
        if let Some(seed) = self.seed {
            args.extend(["--seed".to_owned(), seed.to_string()]);
        }
        if let Some(filter) = &self.filter {
            args.extend(["--filter".to_owned(), filter.clone()]);
        }
        args
    }
}

fn set_once<T>(slot: &mut Option<T>, flag: &str, value: T) -> Result<()> {
    if slot.replace(value).is_some() {
        bail!("{flag} is given twice");
    }
    Ok(())
}

fn number<T: std::str::FromStr>(flag: &str, value: &str) -> Result<T> {
    value
        .parse()
        .ok()
        .with_context(|| format!("{flag} takes a whole number, not `{value}`"))
}

/// The git invocations that fetch `OpenKey` at `sha` into an empty directory, without its history.
pub(crate) fn fetch_commands_for_sha(sha: &str) -> Vec<Vec<String>> {
    [
        vec!["git", "init"],
        vec!["git", "fetch", "--depth", "1", OPENKEY_REPOSITORY, sha],
        vec!["git", "checkout", "FETCH_HEAD"],
    ]
    .into_iter()
    .map(|command| command.into_iter().map(str::to_owned).collect())
    .collect()
}

/// The command that builds and runs the comparison tool, from the repository root.
pub(crate) fn tool_command(args: &CompareArgs) -> Vec<String> {
    let mut command: Vec<String> = ["cargo", "run", "--release", "--manifest-path", TOOL_MANIFEST, "--"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    command.extend(args.tool_args());
    command
}

/// `true` when the tool found no difference it was not told to accept; the report says what differed.
pub(crate) fn run(root: &Path, args: &[String]) -> Result<bool> {
    let args = CompareArgs::parse(args)?;
    fetch_openkey(root)?;
    run_command(root, &tool_command(&args))
}

fn fetch_openkey(root: &Path) -> Result<()> {
    let dir = root.join(OPENKEY_DIR);
    if dir.join(OPENKEY_MARKER).is_file() {
        return Ok(());
    }
    fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    for command in fetch_commands_for_sha(OPENKEY_SHA) {
        if !run_command(&dir, &command)? {
            bail!("fetching OpenKey failed at `{}`", command.join(" "));
        }
    }
    Ok(())
}

/// Runs `command` in `dir` with this terminal's output, so a long build shows its progress.
fn run_command(dir: &Path, command: &[String]) -> Result<bool> {
    let Some((program, args)) = command.split_first() else {
        bail!("an empty command");
    };
    let status = Command::new(program)
        .args(args)
        .current_dir(dir)
        .status()
        .with_context(|| format!("starting `{}`", command.join(" ")))?;
    Ok(status.success())
}

#[cfg(test)]
mod tests;
