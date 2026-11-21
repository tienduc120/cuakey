//! The report: what differed, grouped so that one cause reads as one entry, with counts at the top. It describes
//! behaviour only: settings, keys, and the text each engine leaves.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::time::Duration;

use cuakey_domain::{InputMethod, decode};

use crate::accepted::{Acceptance, Accepted};
use crate::combo::{Combo, Toggle, method_name, method_token, table_token};
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

/// How many runs there were, how many of them differed, and how many of those an entry accepts.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Count {
    pub runs: usize,
    pub differ: usize,
    pub accepted: usize,
}

/// How one run ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunEnd {
    Matched,
    Differs,
    /// It differed, and an entry accepts the difference.
    Accepted,
}

impl RunEnd {
    pub const fn of<T>(divergence: Option<&T>, accepted: Option<usize>) -> Self {
        match (divergence, accepted) {
            (None, _) => Self::Matched,
            (Some(_), None) => Self::Differs,
            (Some(_), Some(_)) => Self::Accepted,
        }
    }
}

impl Count {
    pub fn add(&mut self, end: RunEnd) {
        self.runs += 1;
        self.differ += usize::from(end != RunEnd::Matched);
        self.accepted += usize::from(end == RunEnd::Accepted);
    }

    pub const fn not_accepted(self) -> usize {
        self.differ - self.accepted
    }

    /// "12 (3.4%)".
    pub fn differ_with_share(self) -> String {
        let tenths = (self.differ * 1000).checked_div(self.runs).unwrap_or(0);
        format!("{} ({}.{}%)", self.differ, tenths / 10, tenths % 10)
    }

    fn plus(self, other: Self) -> Self {
        Self {
            runs: self.runs + other.runs,
            differ: self.differ + other.differ,
            accepted: self.accepted + other.accepted,
        }
    }
}

/// The runs of one line that differed after the same key with the same texts: one entry of the report.
#[derive(Debug)]
struct Group {
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
    /// The entry that accepts these runs.
    accepted: Option<usize>,
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

impl FileTally {
    /// The groups, accepted or not, in the order the report lists them.
    fn sorted(&self, accepted: bool) -> Vec<(&GroupKey, &Group)> {
        let mut groups: Vec<(&GroupKey, &Group)> = self
            .groups
            .iter()
            .filter(|(key, _)| key.accepted.is_some() == accepted)
            .collect();
        groups.sort_by(|(a, ga), (b, gb)| {
            (a.line, a.step, gb.combos.len(), &a.expected, &a.actual).cmp(&(
                b.line,
                b.step,
                ga.combos.len(),
                &b.expected,
                &b.actual,
            ))
        });
        groups
    }
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

    /// One run of line `line` of the current file under `combo`; where it first differed, if it did; and the entry
    /// that accepts that, if one does.
    pub fn record(&mut self, line: usize, combo: Combo, divergence: Option<Divergence>, accepted: Option<usize>) {
        let end = RunEnd::of(divergence.as_ref(), accepted);
        let differences = combo.differences();
        if differences.is_empty() || differences == [method_name(combo.method)] {
            self.defaults.add(end);
        }
        for difference in &differences {
            if let Some((_, count)) = self.settings.iter_mut().find(|(name, _)| name == difference) {
                count.add(end);
            }
        }
        let Some(file) = self.files.last_mut() else {
            return;
        };
        file.count.add(end);
        if let Some(divergence) = divergence {
            add_to_group(&mut file.groups, line, combo, divergence, accepted);
        }
    }

    pub fn total(&self) -> Count {
        self.files
            .iter()
            .fold(Count::default(), |sum, file| sum.plus(file.count))
    }

    /// Every difference is accepted, and no entry the run could judge has gone stale.
    pub fn passed(&self, acceptance: &Acceptance) -> bool {
        self.total().not_accepted() == 0 && acceptance.stale().next().is_none()
    }

    pub fn render(&self, elapsed: Duration, acceptance: &Acceptance) -> String {
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
            "{} runs in {:.1} s; {} differ, {} of them accepted in `accepted-differences.toml`.\n",
            total.runs,
            elapsed.as_secs_f64(),
            total.differ_with_share(),
            total.accepted
        );
        self.render_summary(&mut out);
        out.push_str(NOT_COMPARED);
        out.push_str(HOW_TO_READ);
        for file in &self.files {
            render_file(&mut out, file);
        }
        self.render_accepted(&mut out, acceptance);
        render_stale(&mut out, acceptance.stale());
        out
    }

    fn render_summary(&self, out: &mut String) {
        out.push_str("## Summary\n\n");
        out.push_str("| Corpus file | Lines | Runs | Runs that differ | Accepted | Lines that differ | Entries |\n");
        out.push_str("| --- | ---: | ---: | ---: | ---: | ---: | ---: |\n");
        for file in &self.files {
            let open = file.sorted(false);
            let mut lines: Vec<usize> = open.iter().map(|(key, _)| key.line).collect();
            lines.dedup();
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} | {} | {} | {} |",
                file.file.name,
                file.lines.len(),
                file.count.runs,
                file.count.differ_with_share(),
                file.count.accepted,
                lines.len(),
                open.len()
            );
        }
        out.push_str("\nLines that differ and entries count only the differences no entry accepts.\n");
        out.push_str("\n| Setting that differs from the default | Runs | Runs that differ | Not accepted |\n");
        out.push_str("| --- | ---: | ---: | ---: |\n");
        let rows = std::iter::once(("none (every setting at its default)", self.defaults))
            .chain(self.settings.iter().map(|(name, count)| (name.as_str(), *count)));
        for (name, count) in rows {
            let _ = writeln!(
                out,
                "| {name} | {} | {} | {} |",
                count.runs,
                count.differ_with_share(),
                count.not_accepted()
            );
        }
        out.push('\n');
    }

    fn render_accepted(&self, out: &mut String, acceptance: &Acceptance) {
        if self.total().accepted == 0 {
            return;
        }
        out.push_str("## Accepted differences\n\n");
        for file in &self.files {
            render_accepted_file(out, file, acceptance);
        }
        out.push('\n');
    }
}

fn render_accepted_file(out: &mut String, file: &FileTally, acceptance: &Acceptance) {
    for (key, group) in file.sorted(true) {
        let (Some(line), Some(entry)) = (
            file.lines.get(key.line),
            key.accepted.and_then(|at| acceptance.entry(at)),
        ) else {
            continue;
        };
        let count = group.combos.len();
        let _ = writeln!(
            out,
            "- `{}`, after {}: expected {}, CuaKey {}, under {count} combination{}. Accepted: {}.",
            line.text,
            after_key(line, key.step),
            show(&group.expected, group.shown.table),
            show(&group.actual, group.shown.table),
            if count == 1 { "" } else { "s" },
            entry.reason.trim_end_matches('.')
        );
    }
}

const NOT_COMPARED: &str = "## Not compared yet\n\n\
    - Macros: CuaKey has no macros yet, so the comparison runs with macros off.\n\
    - The text conversion tool: CuaKey has no conversion tool yet.\n\n";

const HOW_TO_READ: &str = "## Differences\n\n\
    Keys are written in the corpus notation (see `tools/openkey-compare/README.md`). Each entry gives the line, the \
    key after which the texts first differ, the expected text and CuaKey's, then one combination it happens under \
    (the one closest to the defaults), how many of the line's combinations end the same way, and the setting tokens \
    they all match. Text is shown as is for Unicode, and as code units in hex, then the text they read as, for every \
    other table. To accept an entry, copy these into `accepted-differences.toml`: the file, the line, the tokens, the \
    key number, and both texts (the hex alone for a table other than Unicode).\n\n";

fn add_to_group(
    groups: &mut HashMap<GroupKey, Group>,
    line: usize,
    combo: Combo,
    divergence: Divergence,
    accepted: Option<usize>,
) {
    let key = GroupKey {
        line,
        step: divergence.step,
        unmodelled: divergence.unmodelled,
        accepted,
        expected: decode(&divergence.expected, combo.table),
        actual: decode(&divergence.actual, combo.table),
    };
    let group = groups.entry(key).or_insert_with(|| Group {
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

/// "key 3 `s`", or "key 4, the Space that ends it" for the Space the comparison types after the line.
fn after_key(line: &Line, step: usize) -> String {
    line.strokes.get(step).map_or_else(
        || format!("key {}, the Space that ends it", step + 1),
        |stroke| format!("key {} `{}`", step + 1, Notation(*stroke)),
    )
}

fn render_file(out: &mut String, file: &FileTally) {
    let _ = writeln!(out, "### {}\n", file.file.name);
    let groups = file.sorted(false);
    if groups.is_empty() {
        out.push_str("No differences that are not accepted.\n\n");
        return;
    }
    let per_line = Combo::PER_METHOD * file.file.methods.len();
    for (key, group) in groups {
        let Some(line) = file.lines.get(key.line) else {
            continue;
        };
        let _ = writeln!(
            out,
            "- `{}`, after {}: expected {}, CuaKey {}.{} Settings `{}`{}.",
            line.text,
            after_key(line, key.step),
            show(&group.expected, group.shown.table),
            show(&group.actual, group.shown.table),
            unmodelled_note(key.unmodelled),
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

/// How many combinations end the same way, and the setting tokens they all match.
fn spread(group: &Group, per_line: usize, methods: &[InputMethod]) -> String {
    let more = group.combos.len().saturating_sub(1);
    if group.combos.len() >= per_line {
        return format!(" and every other of the {per_line} combinations");
    }
    let shared = shared_tokens(&group.combos, methods);
    let shared = if shared.is_empty() {
        String::new()
    } else {
        format!(", all matching `{}`", shared.join(" "))
    };
    format!(" and {more} more of {per_line} combinations{shared}")
}

/// The setting tokens every one of `combos` has; the input method only when the file is typed in more than one.
fn shared_tokens(combos: &[Combo], methods: &[InputMethod]) -> Vec<String> {
    let Some(first) = combos.first() else {
        return Vec::new();
    };
    let mut shared = Vec::new();
    if methods.len() > 1 && combos.iter().all(|c| c.method == first.method) {
        shared.push(method_token(first.method).to_owned());
    }
    if combos.iter().all(|c| c.table == first.table) {
        shared.push(table_token(first.table).to_owned());
    }
    for toggle in Toggle::ALL {
        let on = first.is_on(toggle);
        if combos.iter().all(|c| c.is_on(toggle) == on) {
            shared.push(if on {
                toggle.token().to_owned()
            } else {
                format!("!{}", toggle.token())
            });
        }
    }
    shared
}

/// Entries the run could judge that matched nothing.
fn render_stale<'a>(out: &mut String, stale: impl Iterator<Item = &'a Accepted>) {
    let mut stale = stale.peekable();
    if stale.peek().is_none() {
        return;
    }
    out.push_str(
        "## Stale accepted entries\n\nThese entries match no difference this run found: remove each one, or fix the \
         entry if what it names changed.\n\n",
    );
    for entry in stale {
        let _ = writeln!(
            out,
            "- `{}` in {}, step {}, settings `{}`: expected `{}`, CuaKey `{}`. Reason given: {}.",
            entry.sequence,
            entry.source,
            entry.step,
            entry.settings,
            entry.expected,
            entry.cuakey,
            entry.reason.trim_end_matches('.')
        );
    }
    out.push('\n');
}

/// One random sequence that differed, and the reason of the entry that accepts it, if one does.
#[derive(Debug, Clone)]
pub struct RandomFinding {
    pub sequence: String,
    pub combo: Combo,
    pub step_key: String,
    pub divergence: Divergence,
    pub accepted: Option<String>,
}

/// The report of a random run of `count` sequences from `seed`.
pub fn render_random(
    seed: u64,
    count: usize,
    findings: &[RandomFinding],
    elapsed: Duration,
    acceptance: &Acceptance,
) -> String {
    let accepted = findings.iter().filter(|f| f.accepted.is_some()).count();
    let mut out = String::new();
    out.push_str("# OpenKey comparison: random sequences\n\n");
    let _ = writeln!(
        out,
        "{count} sequences from seed {seed}, each under random settings and ended by a Space, in {:.1} s: {} differ, \
         {accepted} of them accepted in `accepted-differences.toml`, {} not.\n",
        elapsed.as_secs_f64(),
        findings.len(),
        findings.len() - accepted
    );
    out.push_str(NOT_COMPARED);
    out.push_str(
        "## Differences\n\nKeys are written in the corpus notation (see `tools/openkey-compare/README.md`); the \
         settings are written as the tokens an accepted entry names.\n\n",
    );
    for finding in findings {
        let table = finding.combo.table;
        let _ = write!(
            out,
            "- `{}`, after key {} `{}`: expected {}, CuaKey {}.{} Settings `{}`.",
            finding.sequence,
            finding.divergence.step + 1,
            finding.step_key,
            show(&finding.divergence.expected, table),
            show(&finding.divergence.actual, table),
            unmodelled_note(finding.divergence.unmodelled),
            finding.combo.tokens().join(" ")
        );
        match &finding.accepted {
            Some(reason) => {
                let _ = writeln!(out, " Accepted: {}.", reason.trim_end_matches('.'));
            }
            None => out.push('\n'),
        }
    }
    out.push('\n');
    render_stale(&mut out, acceptance.stale());
    out
}

#[cfg(test)]
mod tests;
