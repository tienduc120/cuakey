use alloc::string::String;

use crate::PreferencesError;

/// An executable's file name in lower case, so `C:\Games\Game.EXE` and `game.exe` are the same app.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AppName(String);

impl AppName {
    /// # Errors
    ///
    /// Refuses a name with no file name in it.
    pub fn new(raw: &str) -> Result<Self, PreferencesError> {
        let file_name = raw.rsplit(['\\', '/']).next().unwrap_or_default().trim();
        if file_name.is_empty() {
            return Err(PreferencesError::EmptyAppName);
        }
        Ok(Self(file_name.to_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
