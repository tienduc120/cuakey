use alloc::string::String;

use crate::PreferencesError;

/// Names the one profile the preferences belong to.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProfileId(String);

impl ProfileId {
    /// # Errors
    ///
    /// Refuses an empty or blank id.
    pub fn new(raw: &str) -> Result<Self, PreferencesError> {
        let raw = raw.trim();
        if raw.is_empty() {
            return Err(PreferencesError::EmptyId);
        }
        Ok(Self(String::from(raw)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
