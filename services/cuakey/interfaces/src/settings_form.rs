//! The state behind the settings window: the preferences being edited, which controls are shown or enabled, and the
//! message under the buttons. It holds no drawing, so a test drives every control without a window.

use cuakey_application::{CodeTableValue, HotkeyView, InputMethodValue, LanguageValue, PreferencesView, Toggle};

use crate::{Button, Platform};

/// A control whose visibility or enabled state depends on the platform or on another option.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatedControl {
    FixAutocomplete,
    ChromiumFix,
    DesktopShortcut,
    RunAsAdministrator,
    StoreAppSupport,
    ClipboardInput,
    RestoreInvalidWord,
    AllowZwjfConsonants,
}

#[must_use]
pub(crate) const fn toggle(on: bool) -> Toggle {
    if on { Toggle::On } else { Toggle::Off }
}

#[must_use]
pub(crate) const fn is_on(toggle: Toggle) -> bool {
    matches!(toggle, Toggle::On)
}

/// Declares a setter for each on/off control: it stores the box's state in the field and marks the form changed.
macro_rules! toggle_setters {
    ($($setter:ident => $($field:ident).+),* $(,)?) => {
        $(
            pub fn $setter(&mut self, on: bool) {
                self.view.$($field).+ = toggle(on);
                self.changed = true;
            }
        )*
    };
}

const COMPACT_BUTTONS: [Button; 3] = [Button::CloseWindow, Button::ExitApp, Button::ToggleExpanded];

const EXPANDED_BUTTONS: [Button; 4] = [
    Button::CloseWindow,
    Button::ExitApp,
    Button::ToggleExpanded,
    Button::ResetToDefaults,
];

/// The preferences being edited, and what the window shows around them.
#[derive(Debug, Clone)]
pub struct SettingsForm {
    view: PreferencesView,
    platform: Platform,
    /// A control changed since the last `take_save`.
    changed: bool,
    error: Option<&'static str>,
    /// The window shows every section, not only the control group.
    expanded: bool,
}

impl SettingsForm {
    /// The form of a window that opens small, with the control group only.
    #[must_use]
    pub const fn new(view: PreferencesView, platform: Platform) -> Self {
        Self {
            view,
            platform,
            changed: false,
            error: None,
            expanded: false,
        }
    }

    #[must_use]
    pub const fn view(&self) -> &PreferencesView {
        &self.view
    }

    /// Replaces the edited preferences, after a reset or when another process changed them. Nothing is saved.
    pub fn set_preferences(&mut self, view: PreferencesView) {
        self.view = view;
        self.changed = false;
    }

    /// Shows a message under the control group, or clears it. The edits stay, so the user can correct them.
    pub const fn show_error(&mut self, message: Option<&'static str>) {
        self.error = message;
    }

    #[must_use]
    pub const fn error(&self) -> Option<&'static str> {
        self.error
    }

    /// The whole edited view when a control changed since the last call. The window applies changes at once, so a
    /// frame that changed a control ends in one save.
    pub fn take_save(&mut self) -> Option<PreferencesView> {
        std::mem::take(&mut self.changed).then(|| self.view.clone())
    }

    /// Options only Windows has are not drawn elsewhere, and keep their value.
    #[must_use]
    pub const fn is_visible(&self, control: GatedControl) -> bool {
        match control {
            GatedControl::FixAutocomplete
            | GatedControl::ChromiumFix
            | GatedControl::DesktopShortcut
            | GatedControl::RunAsAdministrator
            | GatedControl::StoreAppSupport
            | GatedControl::ClipboardInput => matches!(self.platform, Platform::Windows),
            GatedControl::RestoreInvalidWord | GatedControl::AllowZwjfConsonants => true,
        }
    }

    /// A disabled control is drawn greyed and keeps its value.
    #[must_use]
    pub const fn is_enabled(&self, control: GatedControl) -> bool {
        match control {
            GatedControl::RestoreInvalidWord | GatedControl::AllowZwjfConsonants => is_on(self.view.typing.spell_check),
            GatedControl::ChromiumFix => is_on(self.view.compatibility.fix_autocomplete),
            GatedControl::FixAutocomplete
            | GatedControl::DesktopShortcut
            | GatedControl::RunAsAdministrator
            | GatedControl::StoreAppSupport
            | GatedControl::ClipboardInput => true,
        }
    }

    /// Whether the window shows every section, or only the control group.
    #[must_use]
    pub const fn is_expanded(&self) -> bool {
        self.expanded
    }

    /// Shows every section, or only the control group. No setting changes, so nothing is saved.
    pub const fn toggle_expanded(&mut self) {
        self.expanded = !self.expanded;
    }

    /// The buttons the window offers, in the order it draws them: the column beside the control group, then the
    /// reset, which only the expanded window offers, under its sections.
    #[must_use]
    pub const fn buttons(&self) -> &'static [Button] {
        if self.expanded {
            &EXPANDED_BUTTONS
        } else {
            &COMPACT_BUTTONS
        }
    }

    pub fn set_language(&mut self, language: LanguageValue) {
        self.view.language = language;
        self.changed = true;
    }

    pub fn set_input_method(&mut self, input_method: InputMethodValue) {
        self.view.typing.input_method = input_method;
        self.changed = true;
    }

    pub fn set_code_table(&mut self, code_table: CodeTableValue) {
        self.view.typing.code_table = code_table;
        self.changed = true;
    }

    pub fn set_switch_hotkey(&mut self, hotkey: HotkeyView) {
        self.view.switching.hotkey = hotkey;
        self.changed = true;
    }

    pub fn set_settings_hotkey(&mut self, hotkey: HotkeyView) {
        self.view.behaviours.settings_hotkey = hotkey;
        self.changed = true;
    }

    toggle_setters! {
        set_new_tone_style => typing.new_tone_style,
        set_spell_check => typing.spell_check,
        set_restore_invalid_word => typing.restore_invalid_word,
        set_allow_zwjf_consonants => typing.allow_zwjf_consonants,
        set_capitalize_sentences => typing.capitalize_sentences,
        set_quick_telex => typing.quick_telex,
        set_quick_first_consonant => typing.quick_first_consonant,
        set_quick_last_consonant => typing.quick_last_consonant,
        set_use_macros => macros.use_macros,
        set_macros_in_english => macros.macros_in_english,
        set_macro_match_case => macros.match_case,
        set_beep => switching.beep,
        set_smart_switch => switching.smart_switch,
        set_remember_code_table => switching.remember_code_table,
        set_work_in_other_languages => switching.work_in_other_languages,
        set_fix_autocomplete => compatibility.fix_autocomplete,
        set_chromium_fix => compatibility.chromium_fix,
        set_clipboard_input => compatibility.clipboard_input,
        set_store_app_support => compatibility.store_app_support,
        set_start_with_windows => system.start_with_windows,
        set_show_settings_at_startup => system.show_settings_at_startup,
        set_run_as_administrator => system.run_as_administrator,
        set_desktop_shortcut => system.desktop_shortcut,
        set_keep_w => behaviours.keep_w,
        set_only_text_boxes => behaviours.only_text_boxes,
    }
}

#[cfg(test)]
mod tests;
