//! Types the same keys into `CuaKey`'s typing engine and into `OpenKey`'s, and reports where the text on screen differs.
//! `cargo xtask compare-openkey` fetches `OpenKey`'s sources and runs this; README.md says how to read the report.

mod accepted;
mod combo;
mod conversion;
mod conversion_report;
mod corpus;
mod cuakey_screen;
mod openkey_screen;
mod oracle;
mod random;
mod report;
mod runs;
mod screen;

use std::fs;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use crate::accepted::{Acceptance, CONVERT, RANDOM};
use crate::combo::Combo;
use crate::conversion::ConvertCombo;
use crate::conversion_report::ConversionTally;
use crate::corpus::{CORPUS_FILES, CorpusFile};
use crate::oracle::OpenKey;

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
/// In the repository's `target/`, which is never committed.
const REPORT: &str = "target/openkey-compare/report.md";
const ACCEPTED: &str = "accepted-differences.toml";
const DEFAULT_SEED: u64 = 1;

/// What to compare: the corpus, or one part of it; or random sequences.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Corpus(Option<Only>),
    Random { count: usize, seed: u64 },
}

/// One part of the corpus: a file of typed lines, or the texts the conversion is compared on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Only {
    File(CorpusFile),
    Convert,
}

/// What a run leaves behind: the report, whether it passed, and a line for the terminal.
#[derive(Debug)]
struct Outcome {
    report: String,
    passed: bool,
    summary: String,
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match parse_mode(&args).and_then(run) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("openkey-compare: {e}");
            ExitCode::from(2)
        }
    }
}

fn parse_mode(args: &[String]) -> Result<Mode, String> {
    let (mut random, mut seed, mut filter) = (None, None, None);
    let mut rest = args.iter();
    while let Some(flag) = rest.next() {
        if !matches!(flag.as_str(), "--random" | "--seed" | "--filter") {
            return Err(format!("unknown argument `{flag}`"));
        }
        let value = rest.next().ok_or_else(|| format!("{flag} needs a value"))?;
        let number = || format!("{flag} takes a whole number, not `{value}`");
        match flag.as_str() {
            "--random" => random = Some(value.parse::<usize>().map_err(|_| number())?),
            "--seed" => seed = Some(value.parse::<u64>().map_err(|_| number())?),
            _ => filter = Some(only(value)?),
        }
    }
    match (random, seed, filter) {
        (Some(0), ..) => {
            Err("--random needs at least one sequence; with none it would pass without comparing anything".to_owned())
        }
        (Some(count), seed, None) => Ok(Mode::Random {
            count,
            seed: seed.unwrap_or(DEFAULT_SEED),
        }),
        (None, None, filter) => Ok(Mode::Corpus(filter)),
        (Some(_), _, Some(_)) => {
            Err("--filter chooses corpus files; --random types random sequences instead".to_owned())
        }
        (None, Some(_), _) => Err("--seed only applies with --random".to_owned()),
    }
}

fn only(name: &str) -> Result<Only, String> {
    if name == CONVERT {
        return Ok(Only::Convert);
    }
    let file = CORPUS_FILES.into_iter().find(|file| file.name == name).ok_or_else(|| {
        let names: Vec<&str> = CORPUS_FILES.iter().map(|file| file.name).chain([CONVERT]).collect();
        format!("no corpus file is called `{name}`; there are {}", names.join(", "))
    })?;
    Ok(Only::File(file))
}

fn run(mode: Mode) -> Result<bool, String> {
    let mut engine = OpenKey::take().ok_or("the OpenKey engine is already in use")?;
    let started = Instant::now();
    let outcome = match mode {
        Mode::Corpus(filter) => compare_corpus(&mut engine, filter, started)?,
        Mode::Random { count, seed } => compare_random(&mut engine, count, seed, started)?,
    };
    // This manifest is two levels below the repository root.
    let root = Path::new(MANIFEST_DIR)
        .ancestors()
        .nth(2)
        .unwrap_or(Path::new(MANIFEST_DIR));
    let path = root.join(REPORT);
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
    }
    fs::write(&path, outcome.report).map_err(|e| format!("writing {}: {e}", path.display()))?;
    println!("{}\nThe report is in {}", outcome.summary, path.display());
    Ok(outcome.passed)
}

fn compare_corpus(engine: &mut OpenKey, only: Option<Only>, started: Instant) -> Result<Outcome, String> {
    let dir = Path::new(MANIFEST_DIR).join("corpus");
    let mut acceptance = load_acceptance()?;
    let files: Vec<CorpusFile> = match only {
        None => CORPUS_FILES.to_vec(),
        Some(Only::File(file)) => vec![file],
        Some(Only::Convert) => Vec::new(),
    };
    let tally = runs::compare_files(engine, &files, &dir, &mut acceptance, started)?;
    let conversion = match only {
        None | Some(Only::Convert) => Some(runs::compare_conversions(engine, &dir, &mut acceptance)?),
        Some(Only::File(_)) => None,
    };
    let (typed, converted) = (
        tally.total(),
        conversion.as_ref().map(ConversionTally::total).unwrap_or_default(),
    );
    let stale = acceptance.stale().count();
    Ok(Outcome {
        report: tally.render(started.elapsed(), &acceptance, conversion.as_ref()),
        passed: tally.passed(&acceptance) && converted.not_accepted() == 0,
        summary: format!(
            "{} of {} typing runs and {} of {} conversions differ, {} of them not accepted; {stale} accepted entries \
             are stale.",
            typed.differ,
            typed.runs,
            converted.differ,
            converted.runs,
            typed.not_accepted() + converted.not_accepted()
        ),
    })
}

/// The entries of `accepted-differences.toml`.
fn load_acceptance() -> Result<Acceptance, String> {
    let path = Path::new(MANIFEST_DIR).join(ACCEPTED);
    let text = fs::read_to_string(&path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    let sources: Vec<&str> = CORPUS_FILES
        .iter()
        .map(|file| file.name)
        .chain([CONVERT, RANDOM])
        .collect();
    let mut tokens = Combo::known_tokens();
    tokens.extend(ConvertCombo::known_tokens());
    Ok(Acceptance::new(accepted::parse(&text, &sources, &tokens)?))
}

fn compare_random(engine: &mut OpenKey, count: usize, seed: u64, started: Instant) -> Result<Outcome, String> {
    let mut acceptance = load_acceptance()?;
    let findings = runs::compare_random(engine, count, seed, &mut acceptance);
    let unaccepted = findings.iter().filter(|f| f.accepted.is_none()).count();
    let stale = acceptance.stale().count();
    Ok(Outcome {
        report: report::render_random(seed, count, &findings, started.elapsed(), &acceptance),
        passed: unaccepted == 0 && stale == 0,
        summary: format!(
            "{} of {count} random sequences differ, {unaccepted} of them not accepted; {stale} accepted entries \
             are stale.",
            findings.len()
        ),
    })
}

#[cfg(test)]
mod tests;
