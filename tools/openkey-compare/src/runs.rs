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
use crate::openkey_screen::{Family, OpenKeyScreens, Unmodelled};
use crate::oracle::OpenKey;
use crate::random::{self, Rng};
use crate::report::{CorpusTally, Divergence, HostDefect, HostDefects, HostExample, RandomFinding, RunResult};
use crate::screen::{exact, show};

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
        let mut accept = |divergence: &Divergence| {
            acceptance.accept(&Found {
                source: file.name,
                sequence: &line.text,
                tokens: &tokens,
                step: divergence.step + 1,
                expected: &exact(&divergence.expected, combo.table),
                cuakey: &exact(&divergence.actual, combo.table),
            })
        };
        let result = compare_line(engine, combo, (&settings, &file.macros.table), strokes, &mut accept);
        tally.record(at, combo, result);
    }
}

/// What a random run found: the sequences with a divergence no entry accepts, how many had only accepted ones, and the
/// host defects.
#[derive(Debug, Default)]
pub struct RandomOutcome {
    pub findings: Vec<RandomFinding>,
    pub accepted: usize,
    pub host_defects: HostDefects,
}

impl RandomOutcome {
    /// Every engine divergence is accepted and no entry the run could judge has gone stale; host defects never fail.
    pub fn passed(&self, acceptance: &Acceptance) -> bool {
        self.findings.is_empty() && acceptance.stale().next().is_none()
    }
}

/// `count` random sequences from `seed`, each under random settings and with no macros.
pub fn compare_random(engine: &mut OpenKey, count: usize, seed: u64, acceptance: &mut Acceptance) -> RandomOutcome {
    let mut rng = Rng::new(seed);
    let sequences: Vec<(Combo, Vec<Stroke>)> = (0..count)
        .map(|_| {
            let combo = random::combo(&mut rng);
            (combo, random::sequence(&mut rng, combo.method))
        })
        .collect();
    compare_sequences(engine, sequences, acceptance)
}

/// Each sequence typed under its settings, with no macros.
pub fn compare_sequences(
    engine: &mut OpenKey,
    sequences: Vec<(Combo, Vec<Stroke>)>,
    acceptance: &mut Acceptance,
) -> RandomOutcome {
    let no_macros = MacroTable::default();
    engine.set_macros(&[]);
    let mut outcome = RandomOutcome::default();
    for (combo, strokes) in sequences {
        let sequence = notation(&strokes);
        let tokens = combo.tokens();
        acceptance.judge_sequence(RANDOM, &sequence, &tokens);
        engine.configure(combo);
        let settings = combo.cuakey_settings();
        let mut accept = |divergence: &Divergence| {
            acceptance.accept(&Found {
                source: RANDOM,
                sequence: &sequence,
                tokens: &tokens,
                step: divergence.step + 1,
                expected: &exact(&divergence.expected, combo.table),
                cuakey: &exact(&divergence.actual, combo.table),
            })
        };
        let result = compare_line(engine, combo, (&settings, &no_macros), &strokes, &mut accept);
        let key_at = |step: usize| strokes.get(step).map_or_else(String::new, |s| Notation(*s).to_string());
        match (result.open, result.accepted.is_empty(), result.host_defect) {
            (Some(divergence), ..) => outcome.findings.push(RandomFinding {
                step_key: key_at(divergence.step),
                sequence,
                combo,
                divergence,
            }),
            (None, false, _) => outcome.accepted += 1,
            (None, true, Some(defect)) => outcome.host_defects.add(defect.family, || HostExample {
                sequence: sequence.clone(),
                settings: combo.tokens().join(" "),
                after: format!("key {} `{}`", defect.step + 1, key_at(defect.step)),
                faithful: show(&defect.faithful, combo.table),
                corrected: show(&defect.corrected, combo.table),
            }),
            (None, true, None) => {}
        }
    }
    outcome
}

/// What the check of a run says to do after a stroke.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flow {
    Next,
    /// An entry accepts the divergence: the corrected host's text box takes `CuaKey`'s text, and the run goes on.
    Resync,
    Stop,
}

/// The three text boxes after a stroke: `CuaKey`'s, the corrected host's, and the faithful host's.
#[derive(Debug, Clone, Copy)]
pub struct Texts<'a> {
    pub cuakey: &'a [u16],
    pub corrected: &'a [u16],
    pub faithful: &'a [u16],
}

/// Follows one run stroke by stroke: `CuaKey` against the corrected host, and the faithful host against the
/// corrected one for a host defect.
#[derive(Debug, Default)]
pub struct RunCheck {
    result: RunResult,
    /// The faithful host's text has left the corrected host's, or `CuaKey`'s has: no later host defect is looked for.
    host_checked: bool,
}

impl RunCheck {
    /// The texts after the stroke at `step`, and the last host defect the faithful host showed; `accept` names the
    /// entry that accepts a divergence, if one does.
    pub fn step(
        &mut self,
        step: usize,
        texts: Texts<'_>,
        defect: Option<Family>,
        accept: &mut dyn FnMut(&Divergence) -> Option<usize>,
    ) -> Flow {
        let Texts {
            cuakey,
            corrected,
            faithful,
        } = texts;
        if !self.host_checked && faithful != corrected {
            self.host_checked = true;
            if let (Some(family), true) = (defect, corrected == cuakey) {
                self.result.host_defect = Some(HostDefect {
                    family,
                    step,
                    faithful: faithful.to_vec(),
                    corrected: corrected.to_vec(),
                });
            }
        }
        if corrected == cuakey {
            return Flow::Next;
        }
        self.host_checked = true;
        let divergence = Divergence {
            step,
            expected: corrected.to_vec(),
            actual: cuakey.to_vec(),
            unmodelled: None,
        };
        if let Some(at) = accept(&divergence) {
            self.result.accepted.push((divergence, at));
            Flow::Resync
        } else {
            self.result.open = Some(divergence);
            Flow::Stop
        }
    }

    /// `OpenKey` gave a reply the tool cannot apply at `step`: a finding that ends the run.
    fn unmodelled(&mut self, step: usize, number: i32, corrected: &[u16], cuakey: &[u16]) {
        self.result.open = Some(Divergence {
            step,
            expected: corrected.to_vec(),
            actual: cuakey.to_vec(),
            unmodelled: Some(number),
        });
    }

    pub fn finish(self) -> RunResult {
        self.result
    }
}

/// Types `strokes` into both engines, from empty text boxes, and compares the text after every stroke. `cuakey` is
/// `CuaKey`'s side: its settings and the macros it expands. A divergence `accept` accepts does not end the run: the
/// corrected host's text box takes `CuaKey`'s text, and the comparison goes on.
pub fn compare_line(
    engine: &mut OpenKey,
    combo: Combo,
    cuakey: (&ComposerSettings, &MacroTable),
    strokes: &[Stroke],
    accept: &mut dyn FnMut(&Divergence) -> Option<usize>,
) -> RunResult {
    let mut openkey = OpenKeyScreens::start(engine, combo);
    let mut cuakey = CuaKeyScreen::new(cuakey.0.clone(), cuakey.1);
    let mut check = RunCheck::default();
    for (step, stroke) in strokes.iter().enumerate() {
        let unmodelled = openkey.stroke(*stroke).err();
        cuakey.stroke(*stroke);
        if let Some(Unmodelled(number)) = unmodelled {
            check.unmodelled(step, number, openkey.corrected(), cuakey.units());
            break;
        }
        let texts = Texts {
            cuakey: cuakey.units(),
            corrected: openkey.corrected(),
            faithful: openkey.faithful(),
        };
        let flow = check.step(step, texts, openkey.defect(), accept);
        match flow {
            Flow::Next => {}
            Flow::Resync => openkey.resync(cuakey.units()),
            Flow::Stop => break,
        }
    }
    check.finish()
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

#[cfg(test)]
mod tests;
