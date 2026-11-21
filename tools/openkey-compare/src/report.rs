//! The report: what differed, grouped so that one cause reads as one entry, with counts at the top. It describes
//! behaviour only: settings, keys, and the text each engine leaves.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::time::Duration;

use cuakey_domain::{InputMethod, decode};

use crate::combo::{Combo, Toggle, method_name, table_name};
use crate::corpus::{CorpusFile, Line, Notation};
use crate::screen::show;

/// The first key after which the two text boxes differ, and what each shows then.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Divergence {
    pub step: usize,
    pub expected: Vec<u16>,
    pub actual: Vec<u16>,
    /// The number of a reply from `OpenKey` the tool cannot apply, when that is what stopped the run.
    pub unmodelled: Option<i32>,
}

/// How many runs there were, and how many of them differed.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Count {
    pub runs: usize,
    pub differ: usize,
}

impl Count {
    fn add(&mut self, differs: bool) {
        self.runs += 1;
        self.differ += usize::from(differs);
    }

    /// "12 (3.4%)".
    fn differ_with_share(self) -> String {
        let tenths = (self.differ * 1000).checked_div(self.runs).unwrap_or(0);
        format!("{} ({}.{}%)", self.differ, tenths / 10, tenths % 10)
    }
}

/// The runs of one line that differed after the same key with the same texts: one entry of the report.
#[derive(Debug)]
struct Group {
    unmodelled: Option<i32>,
    combos: Vec<Combo>,
    /// The combination the entry shows: the one closest to the defaults.
    shown: Combo,
    expected: Vec<u16>,
    actual: Vec<u16>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct GroupKey {
    line: usize,
    step: usize,
    unmodelled: Option<i32>,
    expected: String,
    actual: String,
}

#[derive(Debug)]
struct FileTally {
    file: CorpusFile,
    lines: Vec<Line>,
    count: Count,
    groups: HashMap<GroupKey, Group>,
}

/// Everything a corpus run found.
#[derive(Debug)]
pub struct CorpusTally {
    files: Vec<FileTally>,
    settings: Vec<(String, Count)>,
    defaults: Count,
}

impl Default for CorpusTally {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            settings: Combo::possible_differences()
                .into_iter()
                .map(|name| (name, Count::default()))
                .collect(),
            defaults: Count::default(),
        }
    }
}

impl CorpusTally {
    /// Starts counting the runs of `file`, whose lines are `lines`.
    pub fn begin_file(&mut self, file: CorpusFile, lines: Vec<Line>) {
        self.files.push(FileTally {
            file,
            lines,
            count: Count::default(),
            groups: HashMap::new(),
        });
    }

    /// One run of line `line` of the current file under `combo`, and where it first differed, if it did.
    pub fn record(&mut self, line: usize, combo: Combo, divergence: Option<Divergence>) {
        let differs = divergence.is_some();
        let differences = combo.differences();
        if differences.is_empty() || differences == [method_name(combo.method)] {
            self.defaults.add(differs);
        }
        for difference in &differences {
            if let Some((_, count)) = self.settings.iter_mut().find(|(name, _)| name == difference) {
                count.add(differs);
            }
        }
        let Some(file) = self.files.last_mut() else {
            return;
        };
        file.count.add(differs);
        if let Some(divergence) = divergence {
            add_to_group(&mut file.groups, line, combo, divergence);
        }
    }

    pub fn total(&self) -> Count {
        self.files.iter().fold(Count::default(), |sum, file| Count {
            runs: sum.runs + file.count.runs,
            differ: sum.differ + file.count.differ,
        })
    }

    pub fn render(&self, elapsed: Duration) -> String {
        let total = self.total();
        let mut out = String::new();
        out.push_str("# OpenKey comparison: corpus\n\n");
        out.push_str(
            "Each line of a corpus file is typed into an empty text box, followed by a Space that ends its last word, \
             under every combination of settings for its input methods: each code table, and spell check, restoring \
             an invalid word, modern tones, quick Telex, quick first consonant, quick last consonant, and capitals \
             each on and off. After every key the text OpenKey's Windows host would leave (expected) is compared \
             with the text CuaKey's edits leave. A run stops at its first difference.\n\n",
        );
        let _ = writeln!(
            out,
            "{} runs in {:.1} s; {} differ.\n",
            total.runs,
            elapsed.as_secs_f64(),
            total.differ_with_share()
        );
        self.render_summary(&mut out);
        out.push_str(NOT_COMPARED);
        out.push_str(HOW_TO_READ);
        for file in &self.files {
            render_file(&mut out, file);
        }
        out
    }

    fn render_summary(&self, out: &mut String) {
        out.push_str("## Summary\n\n");
        out.push_str("| Corpus file | Lines | Runs | Runs that differ | Lines that differ | Entries |\n");
        out.push_str("| --- | ---: | ---: | ---: | ---: | ---: |\n");
        for file in &self.files {
            let mut lines: Vec<usize> = file.groups.keys().map(|key| key.line).collect();
            lines.sort_unstable();
            lines.dedup();
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} | {} | {} |",
                file.file.name,
                file.lines.len(),
                file.count.runs,
                file.count.differ_with_share(),
                lines.len(),
                file.groups.len()
            );
        }
        out.push_str("\n| Setting that differs from the default | Runs | Runs that differ |\n");
        out.push_str("| --- | ---: | ---: |\n");
        let _ = writeln!(
            out,
            "| none (every setting at its default) | {} | {} |",
            self.defaults.runs,
            self.defaults.differ_with_share()
        );
        for (name, count) in &self.settings {
            let _ = writeln!(out, "| {name} | {} | {} |", count.runs, count.differ_with_share());
        }
        out.push('\n');
    }
}

const NOT_COMPARED: &str = "## Not compared yet\n\n\
    - Macros: CuaKey has no macros yet, so the comparison runs with macros off.\n\
    - The text conversion tool: CuaKey has no conversion tool yet.\n\n";

const HOW_TO_READ: &str = "## Differences\n\n\
    Keys are written in the corpus notation (see `tools/openkey-compare/README.md`). Each entry gives the line, the \
    key after which the texts first differ, the expected text and CuaKey's, then one combination it happens under \
    (the one closest to the defaults) and how many of the line's combinations end the same way, with the settings \
    they all share. Text is shown as is for Unicode, and as code units in hex, then the text they read as, for every \
    other table.\n\n";

fn add_to_group(groups: &mut HashMap<GroupKey, Group>, line: usize, combo: Combo, divergence: Divergence) {
    let key = GroupKey {
        line,
        step: divergence.step,
        unmodelled: divergence.unmodelled,
        expected: decode(&divergence.expected, combo.table),
        actual: decode(&divergence.actual, combo.table),
    };
    let group = groups.entry(key).or_insert_with(|| Group {
        unmodelled: divergence.unmodelled,
        combos: Vec::new(),
        shown: combo,
        expected: divergence.expected.clone(),
        actual: divergence.actual.clone(),
    });
    group.combos.push(combo);
    if combo.differences().len() < group.shown.differences().len() {
        group.shown = combo;
        group.expected = divergence.expected;
        group.actual = divergence.actual;
    }
}

fn render_file(out: &mut String, file: &FileTally) {
    let _ = writeln!(out, "### {}\n", file.file.name);
    if file.groups.is_empty() {
        out.push_str("No differences.\n\n");
        return;
    }
    let mut groups: Vec<(&GroupKey, &Group)> = file.groups.iter().collect();
    groups.sort_by(|(a, ga), (b, gb)| {
        (a.line, a.step, gb.combos.len(), &a.expected, &a.actual).cmp(&(
            b.line,
            b.step,
            ga.combos.len(),
            &b.expected,
            &b.actual,
        ))
    });
    let per_line = Combo::PER_METHOD * file.file.methods.len();
    for (key, group) in groups {
        let Some(line) = file.lines.get(key.line) else {
            continue;
        };
        let after = line.strokes.get(key.step).map_or_else(
            || "the Space that ends it".to_owned(),
            |stroke| format!("key {} `{}`", key.step + 1, Notation(*stroke)),
        );
        let _ = writeln!(
            out,
            "- `{}`, after {after}: expected {}, CuaKey {}.{} Under `{}`{}.",
            line.text,
            show(&group.expected, group.shown.table),
            show(&group.actual, group.shown.table),
            unmodelled_note(group.unmodelled),
            group.shown,
            spread(group, per_line, file.file.methods)
        );
    }
    out.push('\n');
}

/// Says when a reply the tool cannot apply stopped the run, so it reads as a finding about the tool.
fn unmodelled_note(unmodelled: Option<i32>) -> String {
    unmodelled.map_or_else(String::new, |number| {
        format!(" OpenKey gave a reply this tool does not model ({number}), so the run stopped there.")
    })
}

/// How many combinations end the same way, and the settings they all share.
fn spread(group: &Group, per_line: usize, methods: &[InputMethod]) -> String {
    let more = group.combos.len().saturating_sub(1);
    if group.combos.len() >= per_line {
        return format!(" and every other of the {per_line} combinations");
    }
    let shared = shared_settings(&group.combos, methods);
    let shared = if shared.is_empty() {
        String::new()
    } else {
        format!(", all with {}", shared.join(", "))
    };
    format!(" and {more} more of {per_line} combinations{shared}")
}

/// The settings every one of `combos` has; the input method only when the file is typed in more than one.
fn shared_settings(combos: &[Combo], methods: &[InputMethod]) -> Vec<String> {
    let Some(first) = combos.first() else {
        return Vec::new();
    };
    let mut shared = Vec::new();
    if methods.len() > 1 && combos.iter().all(|c| c.method == first.method) {
        shared.push(method_name(first.method).to_owned());
    }
    if combos.iter().all(|c| c.table == first.table) {
        shared.push(table_name(first.table).to_owned());
    }
    for toggle in Toggle::ALL {
        let on = first.is_on(toggle);
        if combos.iter().all(|c| c.is_on(toggle) == on) {
            shared.push(toggle.state(on));
        }
    }
    shared
}

/// One random sequence that differed, and the entry that accepts it, if one does.
#[derive(Debug, Clone)]
pub struct RandomFinding {
    pub sequence: String,
    pub combo: Combo,
    pub step_key: String,
    pub divergence: Divergence,
    pub accepted: Option<String>,
}

/// The report of a random run of `count` sequences from `seed`.
pub fn render_random(seed: u64, count: usize, findings: &[RandomFinding], elapsed: Duration) -> String {
    let accepted = findings.iter().filter(|f| f.accepted.is_some()).count();
    let mut out = String::new();
    out.push_str("# OpenKey comparison: random sequences\n\n");
    let _ = writeln!(
        out,
        "{count} sequences from seed {seed}, each under random settings, in {:.1} s: {} differ, {accepted} of them \
         accepted in `accepted-differences.toml`, {} not.\n",
        elapsed.as_secs_f64(),
        findings.len(),
        findings.len() - accepted
    );
    out.push_str(NOT_COMPARED);
    out.push_str(
        "## Differences\n\nKeys are written in the corpus notation (see `tools/openkey-compare/README.md`).\n\n",
    );
    for finding in findings {
        let table = finding.combo.table;
        let _ = write!(
            out,
            "- `{}`, after key {} `{}`: expected {}, CuaKey {}.{} Under `{}`.",
            finding.sequence,
            finding.divergence.step + 1,
            finding.step_key,
            show(&finding.divergence.expected, table),
            show(&finding.divergence.actual, table),
            unmodelled_note(finding.divergence.unmodelled),
            finding.combo
        );
        match &finding.accepted {
            Some(reason) => {
                let _ = writeln!(out, " Accepted: {reason}");
            }
            None => out.push('\n'),
        }
    }
    out
}

#[cfg(test)]
mod tests;
