//! The shape of `settings.json`. Every group and every field has a default, so a file written by an older version loads;
//! a field this version does not know is ignored.

use std::collections::{BTreeMap, BTreeSet};

use cuakey_domain::{
    AppName, AppState, BehaviourSettings, CodeTable, CompatibilitySettings, ConvertCase, ConvertOptions,
    ConvertSettings, InputMethod, InterfaceLanguage, Language, MacroSettings, OnOff, Preferences, PreferencesError,
    ProfileId, SettingChange, Settings, SwitchSettings, SystemSettings, Theme, TypingSettings,
};
use serde::{Deserialize, Serialize};

use super::hotkeys::HotkeysFile;

/// The profile used when the file names none.
const DEFAULT_PROFILE: &str = "default";

/// A stored choice, read as the domain value it names.
pub(super) trait Stored: Copy {
    type Domain;

    /// The value, or `None` for a name this version does not know (one a newer version wrote).
    fn known(self) -> Option<Self::Domain>;
}

/// `value`, or `default` with a log line when the file held a value this version does not know.
fn or_default<T>(setting: &str, value: Option<T>, default: T) -> T {
    value.unwrap_or_else(|| {
        tracing::warn!(
            setting,
            "the preferences file holds a value this version does not know, so the default is kept"
        );
        default
    })
}

/// Declares an enum stored by name that mirrors a domain enum variant for variant. A name the enum does not list reads
/// as `Unknown`, which is never written, so one value from a newer version cannot fail the whole file.
macro_rules! stored_enum {
    ($name:ident mirrors $domain:ident { $($variant:ident),* $(,)? }) => {
        #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub(super) enum $name {
            $($variant,)*
            #[serde(other)]
            Unknown,
        }

        impl From<$domain> for $name {
            fn from(value: $domain) -> Self {
                match value {
                    $($domain::$variant => Self::$variant),*
                }
            }
        }

        impl Stored for $name {
            type Domain = $domain;

            fn known(self) -> Option<$domain> {
                match self {
                    $(Self::$variant => Some($domain::$variant),)*
                    Self::Unknown => None,
                }
            }
        }
    };
}

stored_enum!(OnOffFile mirrors OnOff { On, Off });
stored_enum!(InputMethodFile mirrors InputMethod { Telex, Vni, SimpleTelex });
stored_enum!(CodeTableFile mirrors CodeTable { Unicode, Tcvn3, VniWindows, UnicodeCompound, Cp1258 });
stored_enum!(LanguageFile mirrors Language { Vietnamese, English });
stored_enum!(InterfaceLanguageFile mirrors InterfaceLanguage { Vietnamese, English });
stored_enum!(ThemeFile mirrors Theme { Light, Dark });
stored_enum!(ConvertCaseFile mirrors ConvertCase { Keep, Upper, Lower, SentenceStart, EachWord });

/// Declares one group of settings: a stored struct whose fields are the domain group's, read back as the changes that
/// rebuild it. Its default is the domain group's default.
macro_rules! group {
    ($name:ident of $domain:ident { $($field:ident: $stored:ty => $change:ident),* $(,)? }) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(default)]
        pub(super) struct $name {
            $($field: $stored,)*
        }

        impl Default for $name {
            fn default() -> Self {
                Self::from(&$domain::default())
            }
        }

        impl From<&$domain> for $name {
            fn from(domain: &$domain) -> Self {
                Self {
                    $($field: domain.$field().into(),)*
                }
            }
        }

        impl $name {
            /// The changes that rebuild the group; a setting holding an unknown value is left out, so it keeps its default.
            fn changes(&self) -> Vec<SettingChange> {
                let mut changes = Vec::new();
                $(
                    match Stored::known(self.$field) {
                        Some(value) => changes.push(SettingChange::$change(value)),
                        None => tracing::warn!(
                            setting = stringify!($field),
                            "a setting holds a value this version does not know, so its default is kept"
                        ),
                    }
                )*
                changes
            }
        }
    };
}

group!(TypingFile of TypingSettings {
    input_method: InputMethodFile => InputMethod,
    code_table: CodeTableFile => CodeTable,
    new_tone_style: OnOffFile => NewToneStyle,
    spell_check: OnOffFile => SpellCheck,
    restore_invalid_word: OnOffFile => RestoreInvalidWord,
    allow_zwjf_consonants: OnOffFile => AllowZwjfConsonants,
    capitalize_sentences: OnOffFile => CapitalizeSentences,
    quick_telex: OnOffFile => QuickTelex,
    quick_first_consonant: OnOffFile => QuickFirstConsonant,
    quick_last_consonant: OnOffFile => QuickLastConsonant,
});

group!(MacrosFile of MacroSettings {
    use_macros: OnOffFile => UseMacros,
    macros_in_english: OnOffFile => MacrosInEnglish,
    match_case: OnOffFile => MacroMatchCase,
});

group!(SwitchingFile of SwitchSettings {
    beep: OnOffFile => BeepOnSwitch,
    smart_switch: OnOffFile => SmartSwitch,
    remember_code_table: OnOffFile => RememberCodeTable,
    work_in_other_languages: OnOffFile => WorkInOtherLanguages,
});

group!(CompatibilityFile of CompatibilitySettings {
    fix_autocomplete: OnOffFile => FixAutocomplete,
    chromium_fix: OnOffFile => ChromiumFix,
    clipboard_input: OnOffFile => ClipboardInput,
    store_app_support: OnOffFile => StoreAppSupport,
});

group!(SystemFile of SystemSettings {
    start_with_windows: OnOffFile => StartWithWindows,
    show_settings_at_startup: OnOffFile => ShowSettingsAtStartup,
    run_as_administrator: OnOffFile => RunAsAdministrator,
    desktop_shortcut: OnOffFile => DesktopShortcut,
    interface_language: InterfaceLanguageFile => InterfaceLanguage,
    theme: ThemeFile => Theme,
});

group!(BehavioursFile of BehaviourSettings {
    keep_w: OnOffFile => KeepW,
    only_text_boxes: OnOffFile => OnlyTextBoxes,
});

group!(ConvertFile of ConvertSettings {
    notify_when_done: OnOffFile => NotifyWhenConvertDone,
    options: ConvertOptionsFile => ConvertOptions,
});

/// What the conversion window last chose.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct ConvertOptionsFile {
    from: CodeTableFile,
    to: CodeTableFile,
    case: ConvertCaseFile,
    remove_accents: OnOffFile,
}

impl Default for ConvertOptionsFile {
    fn default() -> Self {
        Self::from(ConvertSettings::default().options())
    }
}

impl From<ConvertOptions> for ConvertOptionsFile {
    fn from(options: ConvertOptions) -> Self {
        Self {
            from: options.from().into(),
            to: options.to().into(),
            case: options.case().into(),
            remove_accents: options.remove_accents().into(),
        }
    }
}

impl Stored for ConvertOptionsFile {
    type Domain = ConvertOptions;

    /// Each choice that is unknown keeps its default, and the others load.
    fn known(self) -> Option<ConvertOptions> {
        let default = ConvertSettings::default().options();
        let options = ConvertOptions::new(
            or_default("convert.from", self.from.known(), default.from()),
            or_default("convert.to", self.to.known(), default.to()),
        )
        .with_case(or_default("convert.case", self.case.known(), default.case()))
        .with_remove_accents(or_default(
            "convert.remove_accents",
            self.remove_accents.known(),
            default.remove_accents(),
        ));
        Some(options)
    }
}

/// The language and code table that were current in one app.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(super) struct AppStateFile {
    language: Option<LanguageFile>,
    code_table: Option<CodeTableFile>,
}

/// Everything the preferences hold, as the file stores it.
#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct PreferencesFile {
    profile_id: String,
    language: LanguageFile,
    typing: TypingFile,
    macros: MacrosFile,
    switching: SwitchingFile,
    compatibility: CompatibilityFile,
    system: SystemFile,
    behaviours: BehavioursFile,
    convert: ConvertFile,
    hotkeys: HotkeysFile,
    always_on: Vec<String>,
    app_memory: BTreeMap<String, AppStateFile>,
}

impl Default for PreferencesFile {
    fn default() -> Self {
        Self {
            profile_id: DEFAULT_PROFILE.to_owned(),
            language: Language::English.into(),
            typing: TypingFile::default(),
            macros: MacrosFile::default(),
            switching: SwitchingFile::default(),
            compatibility: CompatibilityFile::default(),
            system: SystemFile::default(),
            behaviours: BehavioursFile::default(),
            convert: ConvertFile::default(),
            hotkeys: HotkeysFile::default(),
            always_on: Vec::new(),
            app_memory: BTreeMap::new(),
        }
    }
}

impl From<&Preferences> for PreferencesFile {
    fn from(preferences: &Preferences) -> Self {
        let settings = preferences.settings();
        Self {
            profile_id: preferences.id().as_str().to_owned(),
            language: preferences.language().into(),
            typing: settings.typing().into(),
            macros: settings.macros().into(),
            switching: settings.switching().into(),
            compatibility: settings.compatibility().into(),
            system: settings.system().into(),
            behaviours: settings.behaviours().into(),
            convert: settings.convert().into(),
            hotkeys: settings.into(),
            always_on: preferences
                .always_on_apps()
                .iter()
                .map(|app| app.as_str().to_owned())
                .collect(),
            app_memory: preferences
                .app_memory()
                .iter()
                .map(|(app, state)| {
                    let state = AppStateFile {
                        language: Some(state.language().into()),
                        code_table: Some(state.code_table().into()),
                    };
                    (app.as_str().to_owned(), state)
                })
                .collect(),
        }
    }
}

impl PreferencesFile {
    /// The preferences the file describes. Whatever is invalid in it is replaced by a default and logged.
    pub(super) fn into_preferences(self) -> Result<Preferences, PreferencesError> {
        let id = ProfileId::new(&self.profile_id).or_else(|_| ProfileId::new(DEFAULT_PROFILE))?;
        let settings = [
            self.typing.changes(),
            self.macros.changes(),
            self.switching.changes(),
            self.compatibility.changes(),
            self.system.changes(),
            self.behaviours.changes(),
            self.convert.changes(),
            self.hotkeys.resolve().changes().to_vec(),
        ]
        .into_iter()
        .flatten()
        .fold(Settings::default(), Settings::with);
        let always_on: BTreeSet<AppName> = self.always_on.iter().filter_map(|name| app_name(name)).collect();
        let app_memory: BTreeMap<AppName, AppState> = self
            .app_memory
            .iter()
            .filter_map(|(name, state)| {
                let state = known_state(name, *state)?;
                app_name(name).map(|app| (app, state))
            })
            .collect();
        Ok(Preferences::restore(
            id,
            settings,
            or_default("language", self.language.known(), Language::English),
            always_on,
            app_memory,
        ))
    }
}

/// What an app last used, or `None` when the entry lacks a value or holds one this version does not know.
fn known_state(app: &str, stored: AppStateFile) -> Option<AppState> {
    let language = stored.language.and_then(Stored::known);
    let code_table = stored.code_table.and_then(Stored::known);
    if let (Some(language), Some(code_table)) = (language, code_table) {
        return Some(AppState::new(language, code_table));
    }
    tracing::warn!(
        app,
        "an app memory entry is incomplete or holds an unknown value and was left out"
    );
    None
}

/// The app a stored name stands for, or `None` for a name that holds no file name.
fn app_name(stored: &str) -> Option<AppName> {
    match AppName::new(stored) {
        Ok(app) => Some(app),
        Err(error) => {
            tracing::warn!(app = stored, %error, "an app name in the preferences file is invalid and was left out");
            None
        }
    }
}
