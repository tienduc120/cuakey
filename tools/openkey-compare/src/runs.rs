//! The typing runs: each corpus line, or each random sequence, typed into both engines from an empty text box.

use std::path::Path;
use std::time::Instant;

use cuakey_domain::{ComposerSettings, MacroEntry, MacroTable};

use crate::accepted::{Acceptance, CONVERT, Found, RANDOM};
use crate::combo::Combo;
use crate::conversion::{self, ConvertCombo};
use crate::conversion_report::{self, ConversionTally, Expected};
use crate::corpus::{self, CorpusFile, Line, Notation, Stroke, notation};
use crate::cuakey_screen::CuaKeyScreen;
use crate::openkey_screen::{OpenKeyScreen, Unmodelled};
use crate::oracle::OpenKey;
use crate::random::{self, Rng};
use crate::report::{CorpusTally, Divergence, RandomFinding};
use crate::screen::exact;

/// A macro table, as both engines take it.
#[derive(Debug, Default)]
pub struct Macros {
    pairs: Vec<(String, String)>,
    table: MacroTable,
}

impl Macros {
    pub fn new(pairs: Vec<(String, String)>) -> Result<Self, String> {
        let mut table = MacroTable::default();
        for (shortcut, expansion) in &pairs {
            let entry = MacroEntry::new(shortcut, expansion).map_err(|e| format!("macro `{shortcut}`: {e}"))?;
            table.add(entry);
        }
        Ok(Self { pairs, table })
    }
}

/// A corpus file's lines, the strokes the comparison types for each, and the macros they are typed with.
#[derive(Debug)]
struct TypedFile<'a> {
    name: &'static str,
    lines: &'a [Line],
    strokes: Vec<Vec<Stroke>>,
    macros: &'a Macros,
}

/// Every line of `files` under every combination of its settings.
pub fn compare_files(
    engine: &mut OpenKey,
    files: &[CorpusFile],
    dir: &Path,
    acceptance: &mut Acceptance,
    started: Instant,
) -> Result<CorpusTally, String> {
    let macro_table = Macros::new(corpus::read_macro_table(dir)?)?;
    let no_macros = Macros::default();
    let mut tally = CorpusTally::default();
    for file in files {
        let lines = corpus::read_file(dir, *file)?;
        let macros = if file.macros { &macro_table } else { &no_macros };
        engine.set_macros(&macros.pairs);
        let typed = TypedFile {
            name: file.name,
            lines: &lines,
            strokes: lines.iter().map(Line::ended).collect(),
            macros,
        };
        acceptance.judge_source(file.name);
        tally.begin_file(*file, lines.clone());
        for combo in file.combos() {
            compare_lines(engine, combo, &typed, &mut tally, acceptance);
        }
        println!("{}: done after {:.1} s", file.name, started.elapsed().as_secs_f64());
    }
    Ok(tally)
}

fn compare_lines(
    engine: &mut OpenKey,
    combo: Combo,
    file: &TypedFile<'_>,
    tally: &mut CorpusTally,
    acceptance: &mut Acceptance,
) {
    engine.configure(combo);
    let settings = combo.cuakey_settings();
    let tokens = combo.tokens();
    for (at, (line, strokes)) in file.lines.iter().zip(&file.strokes).enumerate() {
        let divergence = first_divergence(engine, combo, (&settings, &file.macros.table), strokes);
        let accepted = divergence.as_ref().and_then(|divergence| {
            let (expected, cuakey) = (
                exact(&divergence.expected, combo.table),
                exact(&divergence.actual, combo.table),
            );
            acceptance.accept(&Found {
                source: file.name,
                sequence: &line.text,
                tokens: &tokens,
                step: divergence.step + 1,
                expected: &expected,
                cuakey: &cuakey,
            })
        });
        tally.record(at, combo, divergence, accepted);
    }
}

/// `count` random sequences from `seed`, each under random settings and with no macros.
pub fn compare_random(
    engine: &mut OpenKey,
    count: usize,
    seed: u64,
    acceptance: &mut Acceptance,
) -> Vec<RandomFinding> {
    let mut rng = Rng::new(seed);
    let mut findings = Vec::new();
    let no_macros = MacroTable::default();
    engine.set_macros(&[]);
    for _ in 0..count {
        let combo = random::combo(&mut rng);
        let strokes = random::sequence(&mut rng, combo.method);
        let sequence = notation(&strokes);
        let tokens = combo.tokens();
        acceptance.judge_sequence(RANDOM, &sequence, &tokens);
        engine.configure(combo);
        let settings = combo.cuakey_settings();
        let Some(divergence) = first_divergence(engine, combo, (&settings, &no_macros), &strokes) else {
            continue;
        };
        let (expected, cuakey) = (
            exact(&divergence.expected, combo.table),
            exact(&divergence.actual, combo.table),
        );
        let reason = acceptance
            .accept(&Found {
                source: RANDOM,
                sequence: &sequence,
                tokens: &tokens,
                step: divergence.step + 1,
                expected: &expected,
                cuakey: &cuakey,
            })
            .and_then(|at| acceptance.entry(at))
            .map(|entry| entry.reason.clone());
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
    findings
}

/// Types `strokes` into both engines, from an empty text box, and compares the text after every stroke. `cuakey` is
/// `CuaKey`'s side: its settings and the macros it expands.
fn first_divergence(
    engine: &mut OpenKey,
    combo: Combo,
    cuakey: (&ComposerSettings, &MacroTable),
    strokes: &[Stroke],
) -> Option<Divergence> {
    let mut openkey = OpenKeyScreen::start(engine, combo);
    let mut cuakey = CuaKeyScreen::new(cuakey.0.clone(), cuakey.1);
    for (step, stroke) in strokes.iter().enumerate() {
        let unmodelled = openkey.stroke(*stroke).err().map(|Unmodelled(number)| number);
        cuakey.stroke(*stroke);
        if unmodelled.is_some() || openkey.units() != cuakey.units() {
            return Some(Divergence {
                step,
                expected: openkey.units().to_vec(),
                actual: cuakey.units().to_vec(),
                unmodelled,
            });
        }
    }
    None
}

/// Every text of `convert.txt` converted by both engines under every setting of the conversion.
pub fn compare_conversions(
    engine: &mut OpenKey,
    dir: &Path,
    acceptance: &mut Acceptance,
) -> Result<ConversionTally, String> {
    let texts = conversion::read_texts(dir)?;
    acceptance.judge_source(CONVERT);
    let mut tally = ConversionTally::new(texts.clone());
    for combo in ConvertCombo::all() {
        let tokens = combo.tokens();
        for (at, text) in texts.iter().enumerate() {
            let input = conversion::input(&text.text, combo.from);
            let expected = conversion::openkey_output(engine, &input, combo).map_or(Expected::Failed, Expected::Text);
            let actual = conversion::cuakey_output(&input, combo);
            if expected == Expected::Text(actual.clone()) {
                tally.record(at, combo, None, None);
                continue;
            }
            let accepted = acceptance.accept(&Found {
                source: CONVERT,
                sequence: &text.line,
                tokens: &tokens,
                step: 1,
                expected: &conversion_report::exact_expected(&expected, combo.to),
                cuakey: &exact(&actual, combo.to),
            });
            tally.record(at, combo, Some((expected, actual)), accepted);
        }
    }
    Ok(tally)
}
