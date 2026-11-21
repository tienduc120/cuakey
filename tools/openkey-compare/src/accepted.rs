//! Differences recorded as accepted, in `accepted-differences.toml`: each entry pins one divergence (where it was
//! typed, the keys, the settings it occurs under, the key after which the texts differ, and both texts) and says why
//! it is accepted. A run passes when every difference it finds is accepted and no entry it could judge has gone stale.

use serde::Deserialize;

/// The source a random sequence is accepted under.
pub const RANDOM: &str = "random";

/// One accepted difference.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Accepted {
    /// The corpus file, or `random`.
    pub source: String,
    /// The line as the corpus file writes it, or the random sequence in the corpus notation.
    pub sequence: String,
    /// Setting tokens that must all hold: a method, a table, a toggle that is on, or `!` and a toggle that is off.
    pub settings: String,
    /// The key after which the texts differ, counting from 1.
    pub step: usize,
    /// The texts as the report writes them: the text for Unicode, the code units in hex for any other table.
    pub expected: String,
    pub cuakey: String,
    pub reason: String,
}

impl Accepted {
    fn allows(&self, tokens: &[String]) -> bool {
        self.settings
            .split_whitespace()
            .all(|wanted| tokens.iter().any(|token| token == wanted))
    }

    fn pins(&self, found: &Found<'_>) -> bool {
        self.source == found.source
            && self.sequence == found.sequence
            && self.step == found.step
            && self.expected == found.expected
            && self.cuakey == found.cuakey
            && self.allows(found.tokens)
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct AcceptedFile {
    #[serde(default)]
    accepted: Vec<Accepted>,
}

/// A divergence as an entry names it: `tokens` are the settings it occurred under.
#[derive(Debug, Clone, Copy)]
pub struct Found<'a> {
    pub source: &'a str,
    pub sequence: &'a str,
    pub tokens: &'a [String],
    pub step: usize,
    pub expected: &'a str,
    pub cuakey: &'a str,
}

/// What a run found of one entry.
#[derive(Debug, Clone, Copy, Default)]
struct Seen {
    /// The run typed what the entry names, so it can tell whether the entry still occurs.
    judged: bool,
    matches: usize,
}

/// The entries, with what this run found of each.
#[derive(Debug)]
pub struct Acceptance {
    entries: Vec<(Accepted, Seen)>,
}

impl Acceptance {
    pub fn new(entries: Vec<Accepted>) -> Self {
        Self {
            entries: entries.into_iter().map(|entry| (entry, Seen::default())).collect(),
        }
    }

    /// The number of the entry that accepts `found`, if one does.
    pub fn accept(&mut self, found: &Found<'_>) -> Option<usize> {
        let (at, (_, seen)) = self
            .entries
            .iter_mut()
            .enumerate()
            .find(|(_, (entry, _))| entry.pins(found))?;
        seen.matches += 1;
        Some(at)
    }

    /// The whole of a corpus file was typed: each of its entries either occurred or is stale.
    pub fn judge_source(&mut self, source: &str) {
        for (entry, seen) in &mut self.entries {
            seen.judged |= entry.source == source;
        }
    }

    /// `sequence` was typed under `tokens`: an entry for it under those settings either occurred or is stale.
    pub fn judge_sequence(&mut self, source: &str, sequence: &str, tokens: &[String]) {
        for (entry, seen) in &mut self.entries {
            seen.judged |= entry.source == source && entry.sequence == sequence && entry.allows(tokens);
        }
    }

    pub fn entry(&self, at: usize) -> Option<&Accepted> {
        self.entries.get(at).map(|(entry, _)| entry)
    }

    /// The entries the run could judge that matched nothing.
    pub fn stale(&self) -> impl Iterator<Item = &Accepted> {
        self.entries
            .iter()
            .filter(|(_, seen)| seen.judged && seen.matches == 0)
            .map(|(entry, _)| entry)
    }
}

/// The entries of `text`, each checked to name a source in `sources` and settings in `tokens`, so that a typo in the
/// file is an error rather than an entry that never matches.
pub fn parse(text: &str, sources: &[&str], tokens: &[String]) -> Result<Vec<Accepted>, String> {
    let file: AcceptedFile = toml::from_str(text).map_err(|e| format!("accepted-differences.toml: {e}"))?;
    for entry in &file.accepted {
        check(entry, sources, tokens).map_err(|e| format!("accepted-differences.toml: {e}"))?;
    }
    Ok(file.accepted)
}

fn check(entry: &Accepted, sources: &[&str], tokens: &[String]) -> Result<(), String> {
    let name = format!("`{}` in {}", entry.sequence, entry.source);
    if !sources.contains(&entry.source.as_str()) {
        return Err(format!(
            "`{}` names `{}`, which is no corpus file and not random",
            entry.sequence, entry.source
        ));
    }
    if let Some(unknown) = entry
        .settings
        .split_whitespace()
        .find(|t| !tokens.iter().any(|known| known == t))
    {
        return Err(format!("{name} names `{unknown}`, which is no setting"));
    }
    if entry.step == 0 {
        return Err(format!("{name} gives step 0; the first key is step 1"));
    }
    if entry.reason.trim().is_empty() {
        return Err(format!("{name} gives no reason"));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
