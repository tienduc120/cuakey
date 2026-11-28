use crate::{
    CodeTable, ConvertOptions, Hotkey, InputMethod, InterfaceLanguage, KeyCode, Modifier, ModifierSet, OnOff,
    PreferencesError, SettingChange, Theme,
};

/// Declares read-only accessors for fields that are all `Copy`.
macro_rules! getters {
    ($($name:ident: $ty:ty),* $(,)?) => {
        $(
            pub const fn $name(&self) -> $ty {
                self.$name
            }
        )*
    };
}

/// How keys are turned into Vietnamese.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypingSettings {
    input_method: InputMethod,
    code_table: CodeTable,
    new_tone_style: OnOff,
    spell_check: OnOff,
    restore_invalid_word: OnOff,
    allow_zwjf_consonants: OnOff,
    capitalize_sentences: OnOff,
    quick_telex: OnOff,
    quick_first_consonant: OnOff,
    quick_last_consonant: OnOff,
}

impl TypingSettings {
    getters! {
        input_method: InputMethod,
        code_table: CodeTable,
        new_tone_style: OnOff,
        spell_check: OnOff,
        restore_invalid_word: OnOff,
        allow_zwjf_consonants: OnOff,
        capitalize_sentences: OnOff,
        quick_telex: OnOff,
        quick_first_consonant: OnOff,
        quick_last_consonant: OnOff,
    }
}

impl Default for TypingSettings {
    fn default() -> Self {
        Self {
            input_method: InputMethod::Telex,
            code_table: CodeTable::Unicode,
            new_tone_style: OnOff::Off,
            spell_check: OnOff::On,
            restore_invalid_word: OnOff::On,
            allow_zwjf_consonants: OnOff::Off,
            capitalize_sentences: OnOff::Off,
            quick_telex: OnOff::Off,
            quick_first_consonant: OnOff::Off,
            quick_last_consonant: OnOff::Off,
        }
    }
}

/// Shortcuts that expand to longer text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroSettings {
    use_macros: OnOff,
    macros_in_english: OnOff,
    match_case: OnOff,
}

impl MacroSettings {
    getters! {
        use_macros: OnOff,
        macros_in_english: OnOff,
        match_case: OnOff,
    }
}

impl Default for MacroSettings {
    fn default() -> Self {
        Self {
            use_macros: OnOff::On,
            macros_in_english: OnOff::Off,
            match_case: OnOff::Off,
        }
    }
}

/// Switching between Vietnamese and English.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchSettings {
    hotkey: Hotkey,
    beep: OnOff,
    smart_switch: OnOff,
    remember_code_table: OnOff,
    work_in_other_languages: OnOff,
}

impl SwitchSettings {
    getters! {
        hotkey: Hotkey,
        beep: OnOff,
        smart_switch: OnOff,
        remember_code_table: OnOff,
        work_in_other_languages: OnOff,
    }
}

impl Default for SwitchSettings {
    fn default() -> Self {
        Self {
            hotkey: Hotkey {
                modifiers: ModifierSet::empty().with(Modifier::Ctrl).with(Modifier::Shift),
                key: None,
            },
            beep: OnOff::Off,
            smart_switch: OnOff::Off,
            remember_code_table: OnOff::Off,
            work_in_other_languages: OnOff::On,
        }
    }
}

/// Workarounds for apps that mishandle the way text is replaced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilitySettings {
    fix_autocomplete: OnOff,
    chromium_fix: OnOff,
    clipboard_input: OnOff,
    store_app_support: OnOff,
}

impl CompatibilitySettings {
    getters! {
        fix_autocomplete: OnOff,
        chromium_fix: OnOff,
        clipboard_input: OnOff,
        store_app_support: OnOff,
    }
}

impl Default for CompatibilitySettings {
    fn default() -> Self {
        Self {
            fix_autocomplete: OnOff::On,
            chromium_fix: OnOff::Off,
            clipboard_input: OnOff::Off,
            store_app_support: OnOff::Off,
        }
    }
}

/// How the app starts, installs itself, which language it speaks, and how its windows look.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemSettings {
    start_with_windows: OnOff,
    show_settings_at_startup: OnOff,
    run_as_administrator: OnOff,
    desktop_shortcut: OnOff,
    interface_language: InterfaceLanguage,
    theme: Theme,
}

impl SystemSettings {
    getters! {
        start_with_windows: OnOff,
        show_settings_at_startup: OnOff,
        run_as_administrator: OnOff,
        desktop_shortcut: OnOff,
        interface_language: InterfaceLanguage,
        theme: Theme,
    }
}

impl Default for SystemSettings {
    fn default() -> Self {
        Self {
            start_with_windows: OnOff::On,
            show_settings_at_startup: OnOff::On,
            run_as_administrator: OnOff::Off,
            desktop_shortcut: OnOff::Off,
            interface_language: InterfaceLanguage::English,
            theme: Theme::Dark,
        }
    }
}

/// The settings hotkey and where typing is converted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BehaviourSettings {
    settings_hotkey: Hotkey,
    keep_w: OnOff,
    only_text_boxes: OnOff,
}

impl BehaviourSettings {
    getters! {
        settings_hotkey: Hotkey,
        keep_w: OnOff,
        only_text_boxes: OnOff,
    }
}

impl Default for BehaviourSettings {
    fn default() -> Self {
        Self {
            settings_hotkey: Hotkey {
                modifiers: ModifierSet::empty().with(Modifier::Ctrl).with(Modifier::Shift),
                key: Some(KeyCode::F5),
            },
            keep_w: OnOff::On,
            only_text_boxes: OnOff::On,
        }
    }
}

/// The clipboard text conversion tool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvertSettings {
    quick_convert_hotkey: Option<Hotkey>,
    notify_when_done: OnOff,
    /// What the conversion window last chose; quick convert reuses it.
    options: ConvertOptions,
}

impl ConvertSettings {
    getters! {
        quick_convert_hotkey: Option<Hotkey>,
        notify_when_done: OnOff,
        options: ConvertOptions,
    }
}

impl Default for ConvertSettings {
    fn default() -> Self {
        Self {
            quick_convert_hotkey: None,
            notify_when_done: OnOff::On,
            options: ConvertOptions::new(CodeTable::Unicode, CodeTable::Unicode),
        }
    }
}

/// Every setting the user can change, in small groups.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Settings {
    typing: TypingSettings,
    macros: MacroSettings,
    switching: SwitchSettings,
    compatibility: CompatibilitySettings,
    system: SystemSettings,
    behaviours: BehaviourSettings,
    convert: ConvertSettings,
}

impl Settings {
    pub const fn typing(&self) -> &TypingSettings {
        &self.typing
    }

    pub const fn macros(&self) -> &MacroSettings {
        &self.macros
    }

    pub const fn switching(&self) -> &SwitchSettings {
        &self.switching
    }

    pub const fn compatibility(&self) -> &CompatibilitySettings {
        &self.compatibility
    }

    pub const fn system(&self) -> &SystemSettings {
        &self.system
    }

    pub const fn behaviours(&self) -> &BehaviourSettings {
        &self.behaviours
    }

    pub const fn convert(&self) -> &ConvertSettings {
        &self.convert
    }

    /// The settings with one change applied. It does not check the hotkeys: `Preferences` does that.
    #[must_use]
    pub fn with(mut self, change: SettingChange) -> Self {
        self.apply(change);
        self
    }

    pub(crate) fn apply(&mut self, change: SettingChange) {
        match change {
            SettingChange::InputMethod(v) => self.typing.input_method = v,
            SettingChange::CodeTable(v) => self.typing.code_table = v,
            SettingChange::NewToneStyle(v) => self.typing.new_tone_style = v,
            SettingChange::SpellCheck(v) => self.typing.spell_check = v,
            SettingChange::RestoreInvalidWord(v) => self.typing.restore_invalid_word = v,
            SettingChange::AllowZwjfConsonants(v) => self.typing.allow_zwjf_consonants = v,
            SettingChange::CapitalizeSentences(v) => self.typing.capitalize_sentences = v,
            SettingChange::QuickTelex(v) => self.typing.quick_telex = v,
            SettingChange::QuickFirstConsonant(v) => self.typing.quick_first_consonant = v,
            SettingChange::QuickLastConsonant(v) => self.typing.quick_last_consonant = v,
            SettingChange::UseMacros(v) => self.macros.use_macros = v,
            SettingChange::MacrosInEnglish(v) => self.macros.macros_in_english = v,
            SettingChange::MacroMatchCase(v) => self.macros.match_case = v,
            SettingChange::SwitchHotkey(v) => self.switching.hotkey = v,
            SettingChange::BeepOnSwitch(v) => self.switching.beep = v,
            SettingChange::SmartSwitch(v) => self.switching.smart_switch = v,
            SettingChange::RememberCodeTable(v) => self.switching.remember_code_table = v,
            SettingChange::WorkInOtherLanguages(v) => self.switching.work_in_other_languages = v,
            SettingChange::FixAutocomplete(v) => self.compatibility.fix_autocomplete = v,
            SettingChange::ChromiumFix(v) => self.compatibility.chromium_fix = v,
            SettingChange::ClipboardInput(v) => self.compatibility.clipboard_input = v,
            SettingChange::StoreAppSupport(v) => self.compatibility.store_app_support = v,
            SettingChange::StartWithWindows(v) => self.system.start_with_windows = v,
            SettingChange::ShowSettingsAtStartup(v) => self.system.show_settings_at_startup = v,
            SettingChange::RunAsAdministrator(v) => self.system.run_as_administrator = v,
            SettingChange::DesktopShortcut(v) => self.system.desktop_shortcut = v,
            SettingChange::InterfaceLanguage(v) => self.system.interface_language = v,
            SettingChange::Theme(v) => self.system.theme = v,
            SettingChange::SettingsHotkey(v) => self.behaviours.settings_hotkey = v,
            SettingChange::KeepW(v) => self.behaviours.keep_w = v,
            SettingChange::OnlyTextBoxes(v) => self.behaviours.only_text_boxes = v,
            SettingChange::QuickConvertHotkey(v) => self.convert.quick_convert_hotkey = v,
            SettingChange::NotifyWhenConvertDone(v) => self.convert.notify_when_done = v,
            SettingChange::ConvertOptions(v) => self.convert.options = v,
        }
    }

    /// Checks that no two hotkeys are the same.
    pub(crate) fn check_hotkeys(&self) -> Result<(), PreferencesError> {
        let switch = self.switching.hotkey;
        let settings = self.behaviours.settings_hotkey;
        if switch == settings {
            return Err(PreferencesError::SwitchEqualsSettings);
        }
        match self.convert.quick_convert_hotkey {
            Some(quick) if quick == switch || quick == settings => Err(PreferencesError::QuickConvertClashes),
            _ => Ok(()),
        }
    }
}
