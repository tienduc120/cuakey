//! The conversion part of the report: what each engine made of each text, grouped so that one cause reads as one
//! entry, with counts per setting.

use std::collections::HashMap;
use std::fmt::Write as _;

use cuakey_domain::CodeTable;

use crate::accepted::Acceptance;
use crate::conversion::{CASES, ConvertCombo, Text};
use crate::report::{Count, RunEnd};
use crate::screen::{exact, show};

/// What `OpenKey` made of a text: its output, or nothing when its conversion stopped with an error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expected {
    Text(Vec<u16>),
    Failed,
}

/// The conversions of one text that ended with the same outputs: one entry of the report.
#[derive(Debug)]
struct Group {
    combos: Vec<ConvertCombo>,
    /// The setting the entry shows: the one closest to converting Unicode to Unicode and changing nothing.
    shown: ConvertCombo,
    expected: Expected,
    actual: Vec<u16>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct GroupKey {
    text: usize,
    accepted: Option<usize>,
    expected: String,
    actual: String,
}

/// Everything the conversion comparison found.
#[derive(Debug)]
pub struct ConversionTally {
    texts: Vec<Text>,
    count: Count,
    groups: HashMap<GroupKey, Group>,
    settings: Vec<(String, Count)>,
}

impl ConversionTally {
    pub fn new(texts: Vec<Text>) -> Self {
        let settings = ConvertCombo::known_tokens()
            .into_iter()
            .filter(|token| !token.starts_with('!'))
            .map(|token| (token, Count::default()))
            .collect();
        Self {
            texts,
            count: Count::default(),
            groups: HashMap::new(),
            settings,
        }
    }

    /// One conversion of text `text` under `combo`: both outputs when they differ, and the entry that accepts that.
    pub fn record(
        &mut self,
        text: usize,
        combo: ConvertCombo,
        differs: Option<(Expected, Vec<u16>)>,
        accepted: Option<usize>,
    ) {
        let end = RunEnd::of(differs.as_ref(), accepted);
        self.count.add(end);
        for token in combo.tokens() {
            if let Some((_, count)) = self.settings.iter_mut().find(|(name, _)| *name == token) {
                count.add(end);
            }
        }
        let Some((expected, actual)) = differs else {
            return;
        };
        let key = GroupKey {
            text,
            accepted,
            expected: exact_expected(&expected, combo.to),
            actual: exact(&actual, combo.to),
        };
        let group = self.groups.entry(key).or_insert_with(|| Group {
            combos: Vec::new(),
            shown: combo,
            expected: expected.clone(),
            actual: actual.clone(),
        });
        group.combos.push(combo);
        if distance(combo) < distance(group.shown) {
            group.shown = combo;
            group.expected = expected;
            group.actual = actual;
        }
    }

    pub const fn total(&self) -> Count {
        self.count
    }

    /// The groups, accepted or not, in the order the report lists them.
    fn sorted(&self, accepted: bool) -> Vec<(&GroupKey, &Group)> {
        let mut groups: Vec<(&GroupKey, &Group)> = self
            .groups
            .iter()
            .filter(|(key, _)| key.accepted.is_some() == accepted)
            .collect();
        groups.sort_by(|(a, ga), (b, gb)| {
            (a.text, gb.combos.len(), &a.expected).cmp(&(b.text, ga.combos.len(), &b.expected))
        });
        groups
    }

    pub fn render_summary(&self, out: &mut String) {
        let open = self.sorted(false);
        let mut texts: Vec<usize> = open.iter().map(|(key, _)| key.text).collect();
        texts.dedup();
        out.push_str(
            "| Conversion texts | Texts | Runs | Runs that differ | Accepted | Texts that differ | Entries |\n",
        );
        out.push_str("| --- | ---: | ---: | ---: | ---: | ---: | ---: |\n");
        let _ = writeln!(
            out,
            "| convert | {} | {} | {} | {} | {} | {} |",
            self.texts.len(),
            self.count.runs,
            self.count.differ_with_share(),
            self.count.accepted,
            texts.len(),
            open.len()
        );
        out.push_str("\n| Conversion setting | Runs | Runs that differ | Not accepted |\n");
        out.push_str("| --- | ---: | ---: | ---: |\n");
        for (name, count) in &self.settings {
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

    pub fn render_differences(&self, out: &mut String) {
        out.push_str("### convert\n\n");
        let groups = self.sorted(false);
        if groups.is_empty() {
            out.push_str("No differences that are not accepted.\n\n");
            return;
        }
        let per_text = ConvertCombo::all().count();
        for (key, group) in groups {
            let Some(text) = self.texts.get(key.text) else {
                continue;
            };
            let more = group.combos.len().saturating_sub(1);
            let shared = shared_tokens(&group.combos);
            let spread = if group.combos.len() >= per_text {
                format!(" and every other of the {per_text} settings")
            } else if shared.is_empty() {
                format!(" and {more} more of {per_text} settings")
            } else {
                format!(
                    " and {more} more of {per_text} settings, all matching `{}`",
                    shared.join(" ")
                )
            };
            let _ = writeln!(
                out,
                "- `{}`: expected {}, CuaKey {}. Settings `{}`{spread}.",
                text.line,
                show_expected(&group.expected, group.shown.to),
                show(&group.actual, group.shown.to),
                group.shown
            );
        }
        out.push('\n');
    }

    pub fn render_accepted(&self, out: &mut String, acceptance: &Acceptance) {
        for (key, group) in self.sorted(true) {
            let (Some(text), Some(entry)) = (
                self.texts.get(key.text),
                key.accepted.and_then(|at| acceptance.entry(at)),
            ) else {
                continue;
            };
            let count = group.combos.len();
            let _ = writeln!(
                out,
                "- `{}` converted: expected {}, CuaKey {}, under {count} setting{}. Accepted: {}.",
                text.line,
                show_expected(&group.expected, group.shown.to),
                show(&group.actual, group.shown.to),
                if count == 1 { "" } else { "s" },
                entry.reason.trim_end_matches('.')
            );
        }
    }
}

/// `expected` as an accepted entry writes it: as `exact` writes units, or `(error)` when the conversion stopped.
pub fn exact_expected(expected: &Expected, table: CodeTable) -> String {
    match expected {
        Expected::Text(units) => exact(units, table),
        Expected::Failed => "(error)".to_owned(),
    }
}

/// `expected` as the report shows it.
fn show_expected(expected: &Expected, table: CodeTable) -> String {
    match expected {
        Expected::Text(units) => show(units, table),
        Expected::Failed => "OpenKey's conversion to stop with an error".to_owned(),
    }
}

/// How far a setting is from converting Unicode to Unicode and changing nothing.
fn distance(combo: ConvertCombo) -> usize {
    [
        combo.from != cuakey_domain::CodeTable::Unicode,
        combo.to != cuakey_domain::CodeTable::Unicode,
        combo.case != CASES[0],
        combo.remove_accents.is_on(),
    ]
    .into_iter()
    .filter(|far| *far)
    .count()
}

/// The tokens every one of `combos` has.
fn shared_tokens(combos: &[ConvertCombo]) -> Vec<String> {
    let Some(first) = combos.first() else {
        return Vec::new();
    };
    first
        .tokens()
        .into_iter()
        .filter(|token| combos.iter().all(|combo| combo.tokens().contains(token)))
        .collect()
}

#[cfg(test)]
mod tests;
