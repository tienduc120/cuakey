use alloc::collections::{BTreeMap, BTreeSet};

use crate::{AppName, CodeTable, Language, OnOff, PreferencesError, ProfileId, SettingChange, Settings};

/// Programs that are not places a person types in with a language of their own: the desktop shell, which focus passes
/// through all the time, and the app's own settings window. They never count as a switch and are never remembered.
const IGNORED_APPS: [&str; 2] = ["explorer.exe", "cuakey.exe"];

/// The language and code table that were current in one app.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppState {
    language: Language,
    code_table: CodeTable,
}

impl AppState {
    pub const fn new(language: Language, code_table: CodeTable) -> Self {
        Self { language, code_table }
    }

    pub const fn language(self) -> Language {
        self.language
    }

    pub const fn code_table(self) -> CodeTable {
        self.code_table
    }
}

/// The aggregate root this service owns: the settings of the one profile, the language in use, the apps that are
/// always on, and what each app last used. agent:aggregate
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preferences {
    id: ProfileId,
    settings: Settings,
    language: Language,
    always_on: BTreeSet<AppName>,
    app_memory: BTreeMap<AppName, AppState>,
}

impl Preferences {
    /// Starts in English with no app remembered.
    ///
    /// # Errors
    ///
    /// Refuses settings in which two hotkeys are the same.
    pub fn new(id: ProfileId, settings: Settings) -> Result<Self, PreferencesError> {
        settings.check_hotkeys()?;
        Ok(Self {
            id,
            settings,
            language: Language::English,
            always_on: BTreeSet::new(),
            app_memory: BTreeMap::new(),
        })
    }

    /// Rebuilds a stored value for a repository. It was valid when saved, so nothing is checked again and no default
    /// is applied; only infrastructure calls this (`cargo xtask verify` enforces it).
    pub const fn restore(
        id: ProfileId,
        settings: Settings,
        language: Language,
        always_on: BTreeSet<AppName>,
        app_memory: BTreeMap<AppName, AppState>,
    ) -> Self {
        Self {
            id,
            settings,
            language,
            always_on,
            app_memory,
        }
    }

    pub const fn id(&self) -> &ProfileId {
        &self.id
    }

    pub const fn settings(&self) -> &Settings {
        &self.settings
    }

    pub const fn language(&self) -> Language {
        self.language
    }

    pub const fn always_on_apps(&self) -> &BTreeSet<AppName> {
        &self.always_on
    }

    pub const fn app_memory(&self) -> &BTreeMap<AppName, AppState> {
        &self.app_memory
    }

    /// Applies one change.
    ///
    /// # Errors
    ///
    /// Refuses a change that would make two hotkeys the same; the preferences stay as they were.
    pub fn update(&mut self, change: SettingChange) -> Result<(), PreferencesError> {
        let candidate = self.settings.clone().with(change);
        candidate.check_hotkeys()?;
        self.settings = candidate;
        Ok(())
    }

    pub const fn set_language(&mut self, language: Language) {
        self.language = language;
    }

    pub fn set_always_on(&mut self, app: AppName, on: OnOff) {
        if on.is_on() {
            self.always_on.insert(app);
        } else {
            self.always_on.remove(&app);
        }
    }

    pub fn is_always_on(&self, app: &AppName) -> bool {
        self.always_on.contains(app)
    }

    /// Stores the language and code table that are current as this app's. An ignored app is never stored.
    pub fn remember_app(&mut self, app: AppName) {
        if IGNORED_APPS.contains(&app.as_str()) {
            return;
        }
        let state = AppState::new(self.language, self.settings.typing().code_table());
        self.app_memory.insert(app, state);
    }

    /// Adopts what a known app last used, as far as the settings ask for it, and returns true. An app never seen
    /// before is remembered with the current state and returns false. The desktop shell and the settings window are
    /// ignored, even if an older version stored one of them.
    pub fn apply_app_switch(&mut self, app: &AppName) -> bool {
        if IGNORED_APPS.contains(&app.as_str()) {
            return false;
        }
        let Some(state) = self.app_memory.get(app).copied() else {
            self.remember_app(app.clone());
            return false;
        };
        if self.settings.switching().smart_switch().is_on() {
            self.language = state.language();
        }
        if self.settings.switching().remember_code_table().is_on() {
            self.settings.apply(SettingChange::CodeTable(state.code_table()));
        }
        true
    }
}

#[cfg(test)]
mod tests;
