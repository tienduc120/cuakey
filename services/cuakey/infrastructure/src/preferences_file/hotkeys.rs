//! The three hotkeys as the file stores them, and how invalid ones fall back to the defaults.

use cuakey_domain::{BehaviourSettings, Hotkey, Modifier, ModifierSet, SettingChange, Settings, SwitchSettings};
use serde::{Deserialize, Serialize};

use crate::key_names::{key_code, key_name};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ModifierFile {
    Ctrl,
    Alt,
    Win,
    Shift,
    /// A modifier a newer version wrote. It is never written, and a hotkey holding it is invalid.
    #[serde(other)]
    Unknown,
}

const MODIFIERS: [(Modifier, ModifierFile); 4] = [
    (Modifier::Ctrl, ModifierFile::Ctrl),
    (Modifier::Alt, ModifierFile::Alt),
    (Modifier::Win, ModifierFile::Win),
    (Modifier::Shift, ModifierFile::Shift),
];

impl ModifierFile {
    const fn known(self) -> Option<Modifier> {
        match self {
            Self::Ctrl => Some(Modifier::Ctrl),
            Self::Alt => Some(Modifier::Alt),
            Self::Win => Some(Modifier::Win),
            Self::Shift => Some(Modifier::Shift),
            Self::Unknown => None,
        }
    }
}

/// A hotkey: the modifiers it holds, and the name of its key if it has one.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct HotkeyFile {
    modifiers: Vec<ModifierFile>,
    key: Option<String>,
}

impl From<Hotkey> for HotkeyFile {
    fn from(hotkey: Hotkey) -> Self {
        Self {
            modifiers: MODIFIERS
                .iter()
                .filter(|(modifier, _)| hotkey.modifiers().contains(*modifier))
                .map(|(_, stored)| *stored)
                .collect(),
            key: hotkey.key().and_then(key_name).map(str::to_owned),
        }
    }
}

impl HotkeyFile {
    /// The hotkey this describes, or `None` when it is empty or names no key. `which` says whose it is, for the log.
    fn parse(&self, which: &str) -> Option<Hotkey> {
        let key = match self.key.as_deref().map(|name| (name, key_code(name))) {
            None => None,
            Some((_, Some(code))) => Some(code),
            Some((name, None)) => {
                tracing::warn!(
                    hotkey = which,
                    key = name,
                    "a hotkey names no known key, so its default is used"
                );
                return None;
            }
        };
        let mut modifiers = ModifierSet::empty();
        for stored in &self.modifiers {
            let Some(modifier) = stored.known() else {
                tracing::warn!(
                    hotkey = which,
                    "a hotkey holds a modifier this version does not know, so its default is used"
                );
                return None;
            };
            modifiers = modifiers.with(modifier);
        }
        match Hotkey::new(modifiers, key) {
            Ok(hotkey) => Some(hotkey),
            Err(error) => {
                tracing::warn!(hotkey = which, %error, "a hotkey is invalid, so its default is used");
                None
            }
        }
    }
}

/// The switch, settings, and quick convert hotkeys.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct HotkeysFile {
    switch: HotkeyFile,
    settings: HotkeyFile,
    quick_convert: Option<HotkeyFile>,
}

impl Default for HotkeysFile {
    fn default() -> Self {
        Self::from(&Settings::default())
    }
}

impl From<&Settings> for HotkeysFile {
    fn from(settings: &Settings) -> Self {
        Self {
            switch: settings.switching().hotkey().into(),
            settings: settings.behaviours().settings_hotkey().into(),
            quick_convert: settings.convert().quick_convert_hotkey().map(Into::into),
        }
    }
}

/// Three hotkeys that no two are the same.
#[derive(Debug, Clone, Copy)]
pub(super) struct Hotkeys {
    switch: Hotkey,
    settings: Hotkey,
    quick_convert: Option<Hotkey>,
}

impl Hotkeys {
    pub(super) fn changes(self) -> [SettingChange; 3] {
        [
            SettingChange::SwitchHotkey(self.switch),
            SettingChange::SettingsHotkey(self.settings),
            SettingChange::QuickConvertHotkey(self.quick_convert),
        ]
    }

    fn defaults() -> Self {
        Self {
            switch: SwitchSettings::default().hotkey(),
            settings: BehaviourSettings::default().settings_hotkey(),
            quick_convert: None,
        }
    }
}

impl HotkeysFile {
    /// Hotkeys no two of which are the same. A stored hotkey that is invalid, or that clashes with one before it,
    /// takes its default (quick convert has none, so it is off); when the default settings hotkey clashes with the
    /// switch hotkey in turn, every hotkey takes its default.
    pub(super) fn resolve(&self) -> Hotkeys {
        let defaults = Hotkeys::defaults();
        let switch = self.switch.parse("switch").unwrap_or(defaults.switch);
        let settings = match self.settings.parse("settings") {
            Some(settings) if settings != switch => settings,
            Some(_) => {
                tracing::warn!("the settings hotkey is the switch hotkey, so its default is used");
                defaults.settings
            }
            None => defaults.settings,
        };
        if settings == switch {
            tracing::warn!("the default settings hotkey is the switch hotkey, so every hotkey takes its default");
            return defaults;
        }
        let quick_convert = self
            .quick_convert
            .as_ref()
            .and_then(|stored| stored.parse("quick_convert"))
            .filter(|quick| {
                let free = *quick != switch && *quick != settings;
                if !free {
                    tracing::warn!("the quick convert hotkey is another hotkey, so it is off");
                }
                free
            });
        Hotkeys {
            switch,
            settings,
            quick_convert,
        }
    }
}
