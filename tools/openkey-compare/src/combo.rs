//! The settings a typing comparison runs under: an input method, a code table, the language, and the typing and macro
//! options both engines share.

use std::fmt;

use cuakey_domain::{CodeTable, ComposerSettings, InputMethod, Language, OnOff, SettingChange, Settings};

/// The code tables, in the order the report lists them.
pub const CODE_TABLES: [CodeTable; 5] = [
    CodeTable::Unicode,
    CodeTable::Tcvn3,
    CodeTable::VniWindows,
    CodeTable::UnicodeCompound,
    CodeTable::Cp1258,
];

/// An option that is on or off in both engines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Toggle {
    SpellCheck,
    Restore,
    ModernTones,
    QuickTelex,
    QuickFirst,
    QuickLast,
    Capitals,
    Macros,
    MacrosInEnglish,
    MatchCase,
}

impl Toggle {
    pub const ALL: [Self; 10] = [
        Self::SpellCheck,
        Self::Restore,
        Self::ModernTones,
        Self::QuickTelex,
        Self::QuickFirst,
        Self::QuickLast,
        Self::Capitals,
        Self::Macros,
        Self::MacrosInEnglish,
        Self::MatchCase,
    ];

    /// The options that change how a word is typed.
    pub const TYPING: [Self; 7] = [
        Self::SpellCheck,
        Self::Restore,
        Self::ModernTones,
        Self::QuickTelex,
        Self::QuickFirst,
        Self::QuickLast,
        Self::Capitals,
    ];

    /// The options that change whether and how a shortcut expands, with the typing options that meet them.
    pub const MACROS: [Self; 6] = [
        Self::SpellCheck,
        Self::Restore,
        Self::Capitals,
        Self::Macros,
        Self::MacrosInEnglish,
        Self::MatchCase,
    ];

    /// The word for it in a settings label, present when it is on.
    pub const fn token(self) -> &'static str {
        match self {
            Self::SpellCheck => "spell-check",
            Self::Restore => "restore",
            Self::ModernTones => "modern-tones",
            Self::QuickTelex => "quick-telex",
            Self::QuickFirst => "quick-first",
            Self::QuickLast => "quick-last",
            Self::Capitals => "capitals",
            Self::Macros => "macros",
            Self::MacrosInEnglish => "macros-in-english",
            Self::MatchCase => "match-caps",
        }
    }

    /// "spell-check on", "capitals off".
    pub fn state(self, on: bool) -> String {
        format!("{} {}", self.token(), if on { "on" } else { "off" })
    }

    /// Spell check, restoring an invalid word, and macros are on unless turned off; the rest are off.
    pub const fn on_by_default(self) -> bool {
        matches!(self, Self::SpellCheck | Self::Restore | Self::Macros)
    }

    const fn bit(self) -> u16 {
        1 << self as u16
    }
}

/// Which of the toggles are on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Toggles(u16);

impl Toggles {
    pub fn defaults() -> Self {
        Self(Toggle::ALL.iter().filter(|t| t.on_by_default()).map(|t| t.bit()).sum())
    }

    pub const fn is_on(self, toggle: Toggle) -> bool {
        self.0 & toggle.bit() != 0
    }

    #[must_use]
    pub const fn with(self, toggle: Toggle, on: OnOff) -> Self {
        match on {
            OnOff::On => Self(self.0 | toggle.bit()),
            OnOff::Off => Self(self.0 & !toggle.bit()),
        }
    }

    /// The defaults, with `varied` set as `bits` says: bit `n` for `varied[n]`.
    fn varying(varied: &[Toggle], bits: usize) -> Self {
        varied
            .iter()
            .enumerate()
            .fold(Self::defaults(), |toggles, (at, toggle)| {
                toggles.with(*toggle, on_off(bits >> at & 1 == 1))
            })
    }
}

/// One combination of settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Combo {
    pub method: InputMethod,
    pub table: CodeTable,
    pub language: Language,
    pub toggles: Toggles,
}

impl Combo {
    /// `method` with every other setting at its default.
    pub fn default_for(method: InputMethod) -> Self {
        Self {
            method,
            table: CodeTable::Unicode,
            language: Language::Vietnamese,
            toggles: Toggles::defaults(),
        }
    }

    /// Every combination for `method` of the code tables, `languages`, and each setting of `varied`; the other
    /// toggles stay at their defaults.
    pub fn all_for(
        method: InputMethod,
        varied: &'static [Toggle],
        languages: &'static [Language],
    ) -> impl Iterator<Item = Self> {
        CODE_TABLES.into_iter().flat_map(move |table| {
            languages.iter().flat_map(move |language| {
                (0..1usize << varied.len()).map(move |bits| Self {
                    method,
                    table,
                    language: *language,
                    toggles: Toggles::varying(varied, bits),
                })
            })
        })
    }

    /// How many combinations `all_for` gives for one method.
    pub const fn count_for(varied: &[Toggle], languages: &[Language]) -> usize {
        (CODE_TABLES.len() * languages.len()) << varied.len()
    }

    pub const fn is_on(self, toggle: Toggle) -> bool {
        self.toggles.is_on(toggle)
    }

    /// The settings that differ from the defaults, in words.
    pub fn differences(self) -> Vec<String> {
        let defaults = Self::default_for(InputMethod::Telex);
        let mut differences = Vec::new();
        if self.method != defaults.method {
            differences.push(method_name(self.method).to_owned());
        }
        if self.table != defaults.table {
            differences.push(table_name(self.table).to_owned());
        }
        if self.language != defaults.language {
            differences.push("English".to_owned());
        }
        differences.extend(
            Toggle::ALL
                .into_iter()
                .filter(|toggle| self.is_on(*toggle) != defaults.is_on(*toggle))
                .map(|toggle| toggle.state(self.is_on(toggle))),
        );
        differences
    }

    /// Every setting that can differ from the defaults, in words, in the order a report lists them.
    pub fn possible_differences() -> Vec<String> {
        let mut all = vec![
            method_name(InputMethod::Vni).to_owned(),
            method_name(InputMethod::SimpleTelex).to_owned(),
        ];
        all.extend(CODE_TABLES.iter().skip(1).map(|table| table_name(*table).to_owned()));
        all.push("English".to_owned());
        all.extend(
            Toggle::ALL
                .into_iter()
                .map(|toggle| toggle.state(!toggle.on_by_default())),
        );
        all
    }

    /// The settings as `CuaKey`'s typing engine reads them. Keeping `w` at the start of a word is `CuaKey`'s own
    /// behaviour, so it is off: `OpenKey` has no such setting.
    pub fn cuakey_settings(self) -> ComposerSettings {
        let on_off = |toggle| on_off(self.is_on(toggle));
        let settings = [
            SettingChange::InputMethod(self.method),
            SettingChange::CodeTable(self.table),
            SettingChange::SpellCheck(on_off(Toggle::SpellCheck)),
            SettingChange::RestoreInvalidWord(on_off(Toggle::Restore)),
            SettingChange::NewToneStyle(on_off(Toggle::ModernTones)),
            SettingChange::QuickTelex(on_off(Toggle::QuickTelex)),
            SettingChange::QuickFirstConsonant(on_off(Toggle::QuickFirst)),
            SettingChange::QuickLastConsonant(on_off(Toggle::QuickLast)),
            SettingChange::CapitalizeSentences(on_off(Toggle::Capitals)),
            SettingChange::UseMacros(on_off(Toggle::Macros)),
            SettingChange::MacrosInEnglish(on_off(Toggle::MacrosInEnglish)),
            SettingChange::MacroMatchCase(on_off(Toggle::MatchCase)),
            SettingChange::KeepW(OnOff::Off),
        ]
        .into_iter()
        .fold(Settings::default(), Settings::with);
        ComposerSettings::from_settings(&settings, self.language)
    }

    /// The settings as tokens an accepted entry's filter is matched against: the method, the table, the language,
    /// and each toggle, `!` in front when it is off.
    pub fn tokens(self) -> Vec<String> {
        let mut tokens = vec![
            method_token(self.method).to_owned(),
            table_token(self.table).to_owned(),
            language_token(self.language).to_owned(),
        ];
        tokens.extend(
            Toggle::ALL
                .into_iter()
                .map(|toggle| toggle_token(toggle, self.is_on(toggle))),
        );
        tokens
    }

    /// Every token a filter may name.
    pub fn known_tokens() -> Vec<String> {
        let methods = [InputMethod::Telex, InputMethod::Vni, InputMethod::SimpleTelex].map(method_token);
        let languages = [Language::Vietnamese, Language::English].map(language_token);
        let mut known: Vec<String> = methods.into_iter().chain(languages).map(str::to_owned).collect();
        known.extend(CODE_TABLES.map(table_token).into_iter().map(str::to_owned));
        for toggle in Toggle::ALL {
            known.extend([toggle_token(toggle, true), toggle_token(toggle, false)]);
        }
        known
    }
}

pub fn toggle_token(toggle: Toggle, on: bool) -> String {
    if on {
        toggle.token().to_owned()
    } else {
        format!("!{}", toggle.token())
    }
}

/// The label a report names a combination by: the input method, the code table, `english` when typing in English,
/// then each toggle that is on.
impl fmt::Display for Combo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", method_token(self.method), table_token(self.table))?;
        if self.language == Language::English {
            f.write_str(" english")?;
        }
        for toggle in Toggle::ALL.into_iter().filter(|t| self.is_on(*t)) {
            write!(f, " {}", toggle.token())?;
        }
        Ok(())
    }
}

const fn on_off(on: bool) -> OnOff {
    if on { OnOff::On } else { OnOff::Off }
}

pub const fn method_name(method: InputMethod) -> &'static str {
    match method {
        InputMethod::Telex => "Telex",
        InputMethod::Vni => "VNI",
        InputMethod::SimpleTelex => "Simple Telex",
    }
}

pub const fn table_name(table: CodeTable) -> &'static str {
    match table {
        CodeTable::Unicode => "Unicode",
        CodeTable::Tcvn3 => "TCVN3",
        CodeTable::VniWindows => "VNI Windows",
        CodeTable::UnicodeCompound => "Unicode compound",
        CodeTable::Cp1258 => "CP1258",
    }
}

pub const fn method_token(method: InputMethod) -> &'static str {
    match method {
        InputMethod::Telex => "telex",
        InputMethod::Vni => "vni",
        InputMethod::SimpleTelex => "simple-telex",
    }
}

pub const fn table_token(table: CodeTable) -> &'static str {
    match table {
        CodeTable::Unicode => "unicode",
        CodeTable::Tcvn3 => "tcvn3",
        CodeTable::VniWindows => "vni-windows",
        CodeTable::UnicodeCompound => "unicode-compound",
        CodeTable::Cp1258 => "cp1258",
    }
}

pub const fn language_token(language: Language) -> &'static str {
    match language {
        Language::Vietnamese => "vietnamese",
        Language::English => "english",
    }
}

#[cfg(test)]
mod tests;
