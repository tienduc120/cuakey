//! Types the same keys into `CuaKey`'s typing engine and into `OpenKey`'s, and reports where the text on screen differs.
//! `cargo xtask compare-openkey` fetches `OpenKey`'s sources and runs this; README.md says how to read the report.

mod accepted;
mod combo;
mod corpus;
mod cuakey_screen;
mod openkey_screen;
mod oracle;
mod random;
mod report;
mod screen;

use std::fs;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use cuakey_domain::ComposerSettings;

use crate::combo::Combo;
use crate::corpus::{CORPUS_FILES, CorpusFile, Line, Notation, Stroke, notation};
use crate::cuakey_screen::CuaKeyScreen;
use crate::openkey_screen::OpenKeyScreen;
use crate::oracle::OpenKey;
use crate::random::Rng;
use crate::report::{CorpusTally, Divergence, RandomFinding};

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
/// In the repository's `target/`, which is never committed.
const REPORT: &str = "target/openkey-compare/report.md";
const ACCEPTED: &str = "accepted-differences.toml";
const DEFAULT_SEED: u64 = 1;

/// What to compare: the corpus, or one file of it; or random sequences.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Corpus(Option<CorpusFile>),
    Random { count: usize, seed: u64 },
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
            _ => filter = Some(corpus_file(value)?),
        }
    }
    match (random, seed, filter) {
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

fn corpus_file(name: &str) -> Result<CorpusFile, String> {
    CORPUS_FILES.into_iter().find(|file| file.name == name).ok_or_else(|| {
        let names: Vec<&str> = CORPUS_FILES.iter().map(|file| file.name).collect();
        format!("no corpus file is called `{name}`; there are {}", names.join(", "))
    })
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

fn compare_corpus(engine: &mut OpenKey, filter: Option<CorpusFile>, started: Instant) -> Result<Outcome, String> {
    let dir = Path::new(MANIFEST_DIR).join("corpus");
    let mut tally = CorpusTally::default();
    for file in CORPUS_FILES
        .into_iter()
        .filter(|file| filter.is_none_or(|only| only == *file))
    {
        let lines = corpus::read_file(&dir, file)?;
        let typed: Vec<Vec<Stroke>> = lines.iter().map(Line::ended).collect();
        tally.begin_file(file, lines);
        for &method in file.methods {
            for combo in Combo::all_for(method) {
                compare_lines(engine, combo, &typed, &mut tally)?;
            }
        }
        println!("{}: done after {:.1} s", file.name, started.elapsed().as_secs_f64());
    }
    let total = tally.total();
    Ok(Outcome {
        report: tally.render(started.elapsed()),
        passed: total.differ == 0,
        summary: format!("{} of {} corpus runs differ.", total.differ, total.runs),
    })
}

fn compare_lines(
    engine: &mut OpenKey,
    combo: Combo,
    lines: &[Vec<Stroke>],
    tally: &mut CorpusTally,
) -> Result<(), String> {
    engine.configure(combo);
    let settings = combo.cuakey_settings();
    for (at, strokes) in lines.iter().enumerate() {
        let divergence = first_divergence(engine, combo, &settings, strokes)?;
        tally.record(at, combo, divergence);
    }
    Ok(())
}

fn compare_random(engine: &mut OpenKey, count: usize, seed: u64, started: Instant) -> Result<Outcome, String> {
    let path = Path::new(MANIFEST_DIR).join(ACCEPTED);
    let text = fs::read_to_string(&path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    let accepted = accepted::parse(&text)?;
    let mut rng = Rng::new(seed);
    let mut findings = Vec::new();
    for _ in 0..count {
        let combo = random::combo(&mut rng);
        let strokes = random::sequence(&mut rng, combo.method);
        engine.configure(combo);
        let Some(divergence) = first_divergence(engine, combo, &combo.cuakey_settings(), &strokes)? else {
            continue;
        };
        let sequence = notation(&strokes);
        let reason = accepted::find(&accepted, &sequence, &combo.to_string()).map(|entry| entry.reason.clone());
        findings.push(RandomFinding {
            step_key: strokes
                .get(divergence.step)
                .map_or_else(String::new, |s| Notation(*s).to_string()),
            sequence,
            combo,
            divergence,
            accepted: reason,
        });
    }
    let unaccepted = findings.iter().filter(|f| f.accepted.is_none()).count();
    Ok(Outcome {
        report: report::render_random(seed, count, &findings, started.elapsed()),
        passed: unaccepted == 0,
        summary: format!(
            "{} of {count} random sequences differ; {unaccepted} not accepted.",
            findings.len()
        ),
    })
}

/// Types `strokes` into both engines, from an empty text box, and compares the text after every stroke.
fn first_divergence(
    engine: &mut OpenKey,
    combo: Combo,
    settings: &ComposerSettings,
    strokes: &[Stroke],
) -> Result<Option<Divergence>, String> {
    let mut openkey = OpenKeyScreen::start(engine, combo.table);
    let mut cuakey = CuaKeyScreen::new(settings.clone());
    for (step, stroke) in strokes.iter().enumerate() {
        openkey.stroke(*stroke)?;
        cuakey.stroke(*stroke);
        if openkey.units() != cuakey.units() {
            return Ok(Some(Divergence {
                step,
                expected: openkey.units().to_vec(),
                actual: cuakey.units().to_vec(),
            }));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests;
