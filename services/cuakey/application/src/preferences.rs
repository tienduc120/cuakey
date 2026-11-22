use cuakey_domain::{
    AppName, BehaviourSettings, CodeTable, CompatibilitySettings, ConvertCase, ConvertOptions, ConvertSettings, Hotkey,
    HotkeyKey, InputMethod, Language, MacroSettings, Modifier, ModifierSet, OnOff, Preferences, PreferencesError,
    ProfileId, SettingChange, Settings, SwitchSettings, SystemSettings, TypingSettings,
};

use crate::{KeyName, PreferencesStore, StoreError};

/// The profile used when nothing was saved yet.
const DEFAULT_PROFILE: &str = "default";

mirror_enum! {
    /// A setting that is on or off.
    Toggle mirrors OnOff { On, Off }
}

mirror_enum! {
    /// How typed keys become Vietnamese letters.
    InputMethodValue mirrors InputMethod { Telex, Vni, SimpleTelex }
}

mirror_enum! {
    /// The character encoding text is written in.
    CodeTableValue mirrors CodeTable { Unicode, Tcvn3, VniWindows, UnicodeCompound, Cp1258 }
}

mirror_enum! {
    /// The language the keyboard types in.
    LanguageValue mirrors Language { Vietnamese, English }
}

mirror_enum! {
    /// What a conversion does to the case of the letters.
    ConvertCaseValue mirrors ConvertCase { Keep, Upper, Lower, SentenceStart, EachWord }
}

impl Toggle {
    const fn is_on(self) -> bool {
        matches!(self, Self::On)
    }
}

/// A hotkey as plain values: the modifiers it holds and its key, if it has one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HotkeyView {
    pub ctrl: Toggle,
    pub alt: Toggle,
    pub win: Toggle,
    pub shift: Toggle,
    pub key: Option<KeyName>,
}

impl From<Hotkey> for HotkeyView {
    fn from(hotkey: Hotkey) -> Self {
        let held = |modifier| {
            if hotkey.modifiers().contains(modifier) {
                Toggle::On
            } else {
                Toggle::Off
            }
        };
        Self {
            ctrl: held(Modifier::Ctrl),
            alt: held(Modifier::Alt),
            win: held(Modifier::Win),
            shift: held(Modifier::Shift),
            key: hotkey.key().map(KeyName::from),
        }
    }
}

impl HotkeyView {
    fn to_hotkey(self) -> Result<Hotkey, UpdateError> {
        let modifiers = [
            (self.ctrl, Modifier::Ctrl),
            (self.alt, Modifier::Alt),
            (self.win, Modifier::Win),
            (self.shift, Modifier::Shift),
        ]
        .into_iter()
        .filter(|(held, _)| held.is_on())
        .fold(ModifierSet::empty(), |set, (_, modifier)| set.with(modifier));
        let key = match self.key.map(KeyName::to_hotkey_key) {
            None => None,
            Some(HotkeyKey::Key(code)) => Some(code),
            // A modifier is held, never the key a hotkey ends on.
            Some(HotkeyKey::Modifier(_)) => return Err(UpdateError::InvalidHotkeyKey),
        };
        Ok(Hotkey::new(modifiers, key)?)
    }
}

/// How keys are turned into Vietnamese.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypingPrefs {
    pub input_method: InputMethodValue,
    pub code_table: CodeTableValue,
    pub new_tone_style: Toggle,
    pub spell_check: Toggle,
    pub restore_invalid_word: Toggle,
    pub allow_zwjf_consonants: Toggle,
    pub capitalize_sentences: Toggle,
    pub quick_telex: Toggle,
    pub quick_first_consonant: Toggle,
    pub quick_last_consonant: Toggle,
}

impl From<&TypingSettings> for TypingPrefs {
    fn from(typing: &TypingSettings) -> Self {
        Self {
            input_method: typing.input_method().into(),
            code_table: typing.code_table().into(),
            new_tone_style: typing.new_tone_style().into(),
            spell_check: typing.spell_check().into(),
            restore_invalid_word: typing.restore_invalid_word().into(),
            allow_zwjf_consonants: typing.allow_zwjf_consonants().into(),
            capitalize_sentences: typing.capitalize_sentences().into(),
            quick_telex: typing.quick_telex().into(),
            quick_first_consonant: typing.quick_first_consonant().into(),
            quick_last_consonant: typing.quick_last_consonant().into(),
        }
    }
}

impl TypingPrefs {
    fn changes(&self) -> [SettingChange; 10] {
        [
            SettingChange::InputMethod(self.input_method.into()),
            SettingChange::CodeTable(self.code_table.into()),
            SettingChange::NewToneStyle(self.new_tone_style.into()),
            SettingChange::SpellCheck(self.spell_check.into()),
            SettingChange::RestoreInvalidWord(self.restore_invalid_word.into()),
            SettingChange::AllowZwjfConsonants(self.allow_zwjf_consonants.into()),
            SettingChange::CapitalizeSentences(self.capitalize_sentences.into()),
            SettingChange::QuickTelex(self.quick_telex.into()),
            SettingChange::QuickFirstConsonant(self.quick_first_consonant.into()),
            SettingChange::QuickLastConsonant(self.quick_last_consonant.into()),
        ]
    }
}

/// Shortcuts that expand to longer text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroPrefs {
    pub use_macros: Toggle,
    pub macros_in_english: Toggle,
    pub match_case: Toggle,
}

impl From<&MacroSettings> for MacroPrefs {
    fn from(macros: &MacroSettings) -> Self {
        Self {
            use_macros: macros.use_macros().into(),
            macros_in_english: macros.macros_in_english().into(),
            match_case: macros.match_case().into(),
        }
    }
}

impl MacroPrefs {
    fn changes(&self) -> [SettingChange; 3] {
        [
            SettingChange::UseMacros(self.use_macros.into()),
            SettingChange::MacrosInEnglish(self.macros_in_english.into()),
            SettingChange::MacroMatchCase(self.match_case.into()),
        ]
    }
}

/// Switching between Vietnamese and English.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchPrefs {
    pub hotkey: HotkeyView,
    pub beep: Toggle,
    pub smart_switch: Toggle,
    pub remember_code_table: Toggle,
    pub work_in_other_languages: Toggle,
}

impl From<&SwitchSettings> for SwitchPrefs {
    fn from(switching: &SwitchSettings) -> Self {
        Self {
            hotkey: switching.hotkey().into(),
            beep: switching.beep().into(),
            smart_switch: switching.smart_switch().into(),
            remember_code_table: switching.remember_code_table().into(),
            work_in_other_languages: switching.work_in_other_languages().into(),
        }
    }
}

impl SwitchPrefs {
    fn changes(&self) -> Result<[SettingChange; 5], UpdateError> {
        Ok([
            SettingChange::SwitchHotkey(self.hotkey.to_hotkey()?),
            SettingChange::BeepOnSwitch(self.beep.into()),
            SettingChange::SmartSwitch(self.smart_switch.into()),
            SettingChange::RememberCodeTable(self.remember_code_table.into()),
            SettingChange::WorkInOtherLanguages(self.work_in_other_languages.into()),
        ])
    }
}

/// Workarounds for apps that mishandle the way text is replaced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityPrefs {
    pub fix_autocomplete: Toggle,
    pub chromium_fix: Toggle,
    pub clipboard_input: Toggle,
    pub store_app_support: Toggle,
}

impl From<&CompatibilitySettings> for CompatibilityPrefs {
    fn from(compatibility: &CompatibilitySettings) -> Self {
        Self {
            fix_autocomplete: compatibility.fix_autocomplete().into(),
            chromium_fix: compatibility.chromium_fix().into(),
            clipboard_input: compatibility.clipboard_input().into(),
            store_app_support: compatibility.store_app_support().into(),
        }
    }
}

impl CompatibilityPrefs {
    fn changes(&self) -> [SettingChange; 4] {
        [
            SettingChange::FixAutocomplete(self.fix_autocomplete.into()),
            SettingChange::ChromiumFix(self.chromium_fix.into()),
            SettingChange::ClipboardInput(self.clipboard_input.into()),
            SettingChange::StoreAppSupport(self.store_app_support.into()),
        ]
    }
}

/// How the app starts and installs itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemPrefs {
    pub start_with_windows: Toggle,
    pub show_settings_at_startup: Toggle,
    pub run_as_administrator: Toggle,
    pub desktop_shortcut: Toggle,
}

impl From<&SystemSettings> for SystemPrefs {
    fn from(system: &SystemSettings) -> Self {
        Self {
            start_with_windows: system.start_with_windows().into(),
            show_settings_at_startup: system.show_settings_at_startup().into(),
            run_as_administrator: system.run_as_administrator().into(),
            desktop_shortcut: system.desktop_shortcut().into(),
        }
    }
}

impl SystemPrefs {
    fn changes(&self) -> [SettingChange; 4] {
        [
            SettingChange::StartWithWindows(self.start_with_windows.into()),
            SettingChange::ShowSettingsAtStartup(self.show_settings_at_startup.into()),
            SettingChange::RunAsAdministrator(self.run_as_administrator.into()),
            SettingChange::DesktopShortcut(self.desktop_shortcut.into()),
        ]
    }
}

/// The settings hotkey and where typing is converted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BehaviourPrefs {
    pub settings_hotkey: HotkeyView,
    pub keep_w: Toggle,
    pub only_text_boxes: Toggle,
}

impl From<&BehaviourSettings> for BehaviourPrefs {
    fn from(behaviours: &BehaviourSettings) -> Self {
        Self {
            settings_hotkey: behaviours.settings_hotkey().into(),
            keep_w: behaviours.keep_w().into(),
            only_text_boxes: behaviours.only_text_boxes().into(),
        }
    }
}

impl BehaviourPrefs {
    fn changes(&self) -> Result<[SettingChange; 3], UpdateError> {
        Ok([
            SettingChange::SettingsHotkey(self.settings_hotkey.to_hotkey()?),
            SettingChange::KeepW(self.keep_w.into()),
            SettingChange::OnlyTextBoxes(self.only_text_boxes.into()),
        ])
    }
}

/// The clipboard conversion tool, and the options quick convert reuses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvertPrefs {
    pub quick_convert_hotkey: Option<HotkeyView>,
    pub notify_when_done: Toggle,
    pub from: CodeTableValue,
    pub to: CodeTableValue,
    pub case: ConvertCaseValue,
    pub remove_accents: Toggle,
}

impl From<&ConvertSettings> for ConvertPrefs {
    fn from(convert: &ConvertSettings) -> Self {
        let options = convert.options();
        Self {
            quick_convert_hotkey: convert.quick_convert_hotkey().map(HotkeyView::from),
            notify_when_done: convert.notify_when_done().into(),
            from: options.from().into(),
            to: options.to().into(),
            case: options.case().into(),
            remove_accents: options.remove_accents().into(),
        }
    }
}

impl ConvertPrefs {
    fn changes(&self) -> Result<[SettingChange; 3], UpdateError> {
        let options = ConvertOptions::new(self.from.into(), self.to.into())
            .with_case(self.case.into())
            .with_remove_accents(self.remove_accents.into());
        Ok([
            SettingChange::QuickConvertHotkey(self.quick_convert_hotkey.map(HotkeyView::to_hotkey).transpose()?),
            SettingChange::NotifyWhenConvertDone(self.notify_when_done.into()),
            SettingChange::ConvertOptions(options),
        ])
    }
}

/// The language and code table one app last used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppMemoryView {
    pub app: String,
    pub language: LanguageValue,
    pub code_table: CodeTableValue,
}

/// Every preference as plain values, for the settings window and the session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreferencesView {
    pub profile_id: String,
    pub language: LanguageValue,
    pub typing: TypingPrefs,
    pub macros: MacroPrefs,
    pub switching: SwitchPrefs,
    pub compatibility: CompatibilityPrefs,
    pub system: SystemPrefs,
    pub behaviours: BehaviourPrefs,
    pub convert: ConvertPrefs,
    /// The executables converted in any control, not only in text boxes.
    pub always_on: Vec<String>,
    pub app_memory: Vec<AppMemoryView>,
}

impl Default for PreferencesView {
    fn default() -> Self {
        Self::with_settings(DEFAULT_PROFILE, &Settings::default(), Language::Vietnamese)
    }
}

impl PreferencesView {
    pub(crate) fn of(preferences: &Preferences) -> Self {
        Self {
            always_on: preferences
                .always_on_apps()
                .iter()
                .map(|app| app.as_str().to_owned())
                .collect(),
            app_memory: preferences
                .app_memory()
                .iter()
                .map(|(app, state)| AppMemoryView {
                    app: app.as_str().to_owned(),
                    language: state.language().into(),
                    code_table: state.code_table().into(),
                })
                .collect(),
            ..Self::with_settings(
                preferences.id().as_str(),
                preferences.settings(),
                preferences.language(),
            )
        }
    }

    /// The aggregate these values describe.
    pub(crate) fn to_preferences(&self) -> Result<Preferences, UpdateError> {
        let settings = self.settings()?;
        let code_table = settings.typing().code_table();
        let mut preferences = Preferences::new(ProfileId::new(&self.profile_id)?, settings)?;
        // The aggregate remembers an app with the language and code table in force, so each app's are put in force in
        // turn, and the current ones last.
        for memory in &self.app_memory {
            preferences.set_language(memory.language.into());
            preferences.update(SettingChange::CodeTable(memory.code_table.into()))?;
            preferences.remember_app(AppName::new(&memory.app)?);
        }
        preferences.set_language(self.language.into());
        preferences.update(SettingChange::CodeTable(code_table))?;
        for app in &self.always_on {
            preferences.set_always_on(AppName::new(app)?, OnOff::On);
        }
        Ok(preferences)
    }

    fn with_settings(profile_id: &str, settings: &Settings, language: Language) -> Self {
        Self {
            profile_id: profile_id.to_owned(),
            language: language.into(),
            typing: settings.typing().into(),
            macros: settings.macros().into(),
            switching: settings.switching().into(),
            compatibility: settings.compatibility().into(),
            system: settings.system().into(),
            behaviours: settings.behaviours().into(),
            convert: settings.convert().into(),
            always_on: Vec::new(),
            app_memory: Vec::new(),
        }
    }

    fn settings(&self) -> Result<Settings, UpdateError> {
        let changes = self
            .typing
            .changes()
            .into_iter()
            .chain(self.macros.changes())
            .chain(self.switching.changes()?)
            .chain(self.compatibility.changes())
            .chain(self.system.changes())
            .chain(self.behaviours.changes()?)
            .chain(self.convert.changes()?);
        Ok(changes.fold(Settings::default(), Settings::with))
    }
}

/// An app's file name in lower case, so `C:\Games\Game.EXE` and `game.exe` name the same app.
///
/// # Errors
///
/// Refuses text with no file name in it.
pub fn app_name(text: &str) -> Result<String, UpdateError> {
    Ok(AppName::new(text)?.as_str().to_owned())
}

/// Why a set of preferences was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum UpdateError {
    #[error("the profile id must not be empty")]
    EmptyProfileId,
    #[error("an app name must not be empty")]
    EmptyAppName,
    #[error("a hotkey needs a modifier or a key")]
    EmptyHotkey,
    #[error("a hotkey cannot end on this key")]
    InvalidHotkeyKey,
    #[error("the switch hotkey and the settings hotkey must differ")]
    SwitchEqualsSettings,
    #[error("the quick convert hotkey must differ from the switch and settings hotkeys")]
    QuickConvertClashes,
}

impl From<PreferencesError> for UpdateError {
    fn from(error: PreferencesError) -> Self {
        match error {
            PreferencesError::EmptyId => Self::EmptyProfileId,
            PreferencesError::EmptyAppName => Self::EmptyAppName,
            PreferencesError::EmptyHotkey => Self::EmptyHotkey,
            PreferencesError::UnknownHotkeyKey => Self::InvalidHotkeyKey,
            PreferencesError::SwitchEqualsSettings => Self::SwitchEqualsSettings,
            PreferencesError::QuickConvertClashes => Self::QuickConvertClashes,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LoadPreferencesError {
    #[error("could not load the preferences")]
    Storage(#[source] StoreError),
}

#[derive(Debug, thiserror::Error)]
pub enum SavePreferencesError {
    #[error("invalid preferences: {0}")]
    Invalid(#[from] UpdateError),
    #[error("could not save the preferences")]
    Storage(#[source] StoreError),
}

#[derive(Debug, thiserror::Error)]
pub enum ResetPreferencesError {
    #[error("invalid preferences: {0}")]
    Invalid(#[from] UpdateError),
    #[error("could not save the preferences")]
    Storage(#[source] StoreError),
}

/// Reads the preferences a previous run saved.
#[derive(Debug)]
pub struct LoadPreferences<S> {
    store: S,
}

impl<S: PreferencesStore> LoadPreferences<S> {
    pub const fn new(store: S) -> Self {
        Self { store }
    }

    /// The stored preferences, or the defaults when none were saved yet.
    pub async fn execute(&self) -> Result<PreferencesView, LoadPreferencesError> {
        let stored = self.store.load().await.map_err(LoadPreferencesError::Storage)?;
        Ok(stored.map_or_else(PreferencesView::default, |preferences| {
            PreferencesView::of(&preferences)
        }))
    }
}

/// Saves the preferences the settings window shows, once the aggregate accepts them.
#[derive(Debug)]
pub struct SavePreferences<S> {
    store: S,
}

impl<S: PreferencesStore> SavePreferences<S> {
    pub const fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn execute(&self, prefs: &PreferencesView) -> Result<(), SavePreferencesError> {
        let preferences = prefs.to_preferences()?;
        self.store
            .save(&preferences)
            .await
            .map_err(SavePreferencesError::Storage)
    }

    #[cfg(test)]
    const fn store(&self) -> &S {
        &self.store
    }
}

/// Puts every setting back to its default, keeping the apps that are always on and what each app last used.
#[derive(Debug)]
pub struct ResetPreferences<S> {
    store: S,
}

impl<S: PreferencesStore> ResetPreferences<S> {
    pub const fn new(store: S) -> Self {
        Self { store }
    }

    /// Saves the defaults and returns them.
    pub async fn execute(&self, current: &PreferencesView) -> Result<PreferencesView, ResetPreferencesError> {
        let defaults = PreferencesView {
            profile_id: current.profile_id.clone(),
            always_on: current.always_on.clone(),
            app_memory: current.app_memory.clone(),
            ..PreferencesView::default()
        };
        let preferences = defaults.to_preferences()?;
        self.store
            .save(&preferences)
            .await
            .map_err(ResetPreferencesError::Storage)?;
        Ok(PreferencesView::of(&preferences))
    }

    #[cfg(test)]
    const fn store(&self) -> &S {
        &self.store
    }
}

#[cfg(test)]
mod tests;
