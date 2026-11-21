//! Random-sequence differences recorded as accepted, in `accepted-differences.toml`: no person would type them, so a
//! random run that finds only these passes. The corpus has no such list; it must match exactly.

use serde::Deserialize;

/// One accepted difference: the keys in the corpus notation, the settings label the report prints, and why.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Accepted {
    pub sequence: String,
    pub settings: String,
    pub reason: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct AcceptedFile {
    #[serde(default)]
    accepted: Vec<Accepted>,
}

pub fn parse(text: &str) -> Result<Vec<Accepted>, String> {
    let file: AcceptedFile = toml::from_str(text).map_err(|e| format!("accepted-differences.toml: {e}"))?;
    if let Some(entry) = file.accepted.iter().find(|entry| entry.reason.trim().is_empty()) {
        return Err(format!(
            "accepted-differences.toml: `{}` under `{}` gives no reason",
            entry.sequence, entry.settings
        ));
    }
    Ok(file.accepted)
}

/// The entry that accepts a difference in `sequence` under `settings`, if any.
pub fn find<'a>(accepted: &'a [Accepted], sequence: &str, settings: &str) -> Option<&'a Accepted> {
    accepted
        .iter()
        .find(|entry| entry.sequence == sequence && entry.settings == settings)
}

#[cfg(test)]
mod tests;
