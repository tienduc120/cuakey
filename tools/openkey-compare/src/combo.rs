//! The settings a comparison runs under: an input method, a code table, and the typing options both engines share.

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

/// A typing option that is on or off in both engines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Toggle {
    SpellCheck,
    Restore,
    ModernTones,
    QuickTelex,
    QuickFirst,
    QuickLast,
    Capitals,
}

impl Toggle {
    pub const ALL: [Self; 7] = [
        Self::SpellCheck,
        Self::Restore,
        Self::ModernTones,
        Self::QuickTelex,
        Self::QuickFirst,
        Self::QuickLast,
        Self::Capitals,
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
        }
    }

    /// "spell-check on", "capitals off".
    pub fn state(self, on: bool) -> String {
        format!("{} {}", self.token(), if on { "on" } else { "off" })
    }

    /// Spell check and restoring an invalid word are on unless turned off; the rest are off.
    pub const fn on_by_default(self) -> bool {
        matches!(self, Self::SpellCheck | Self::Restore)
    }

    const fn bit(self) -> u8 {
        1 << self as u8
    }
}

/// Which of the toggles are on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Toggles(u8);

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

    /// Every setting of the toggles, `bits` read as one bit per toggle.
    fn from_bits(bits: u8) -> Self {
        Self(bits & ((1 << Toggle::ALL.len()) - 1))
    }
}

/// One combination of settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Combo {
    pub method: InputMethod,
    pub table: CodeTable,
    pub toggles: Toggles,
}

impl Combo {
    /// How many combinations there are for one input method.
    pub const PER_METHOD: usize = CODE_TABLES.len() << Toggle::ALL.len();

    /// `method` with every other setting at its default.
    pub fn default_for(method: InputMethod) -> Self {
        Self {
            method,
            table: CodeTable::Unicode,
            toggles: Toggles::defaults(),
        }
    }

    /// Every combination for `method`.
    pub fn all_for(method: InputMethod) -> impl Iterator<Item = Self> {
        let toggle_settings = 1u8 << Toggle::ALL.len();
        CODE_TABLES.into_iter().flat_map(move |table| {
            (0..toggle_settings).map(move |bits| Self {
                method,
                table,
                toggles: Toggles::from_bits(bits),
            })
        })
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
            SettingChange::KeepW(OnOff::Off),
        ]
        .into_iter()
        .fold(Settings::default(), Settings::with);
        ComposerSettings::from_settings(&settings, Language::Vietnamese)
    }
}

/// The label a report and `accepted-differences.toml` name a combination by: the input method, the code table, then
/// each toggle that is on.
impl fmt::Display for Combo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", method_token(self.method), table_token(self.table))?;
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

const fn method_token(method: InputMethod) -> &'static str {
    match method {
        InputMethod::Telex => "telex",
        InputMethod::Vni => "vni",
        InputMethod::SimpleTelex => "simple-telex",
    }
}

const fn table_token(table: CodeTable) -> &'static str {
    match table {
        CodeTable::Unicode => "unicode",
        CodeTable::Tcvn3 => "tcvn3",
        CodeTable::VniWindows => "vni-windows",
        CodeTable::UnicodeCompound => "unicode-compound",
        CodeTable::Cp1258 => "cp1258",
    }
}

#[cfg(test)]
mod tests;
