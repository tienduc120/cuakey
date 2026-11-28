//! The state behind the settings window: the preferences being edited, which controls are shown or enabled, the
//! message under the controls and the buttons of the bottom bar. It holds no drawing, so a test drives every control
//! without a window.

use cuakey_application::{
    CodeTableValue, HotkeyView, InputMethodValue, InterfaceLanguageValue, LanguageValue, PreferencesView, ThemeValue,
    Toggle,
};

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

/// The buttons at the bar's right end, left to right, whatever the window shows.
const BAR_RIGHT: [Button; 3] = [Button::ToggleExpanded, Button::ExitApp, Button::CloseWindow];

/// Where the window's buttons sit in the bar along its bottom edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonBar {
    /// The button at the left end, apart from the others, if any.
    pub left: Option<Button>,
    /// The buttons at the right end, left to right.
    pub right: [Button; 3],
}

impl ButtonBar {
    /// The bar of a window that shows every section when `expanded`, or only the controls used every day. The reset,
    /// which only the expanded window offers, sits alone at the left end, away from the buttons clicked every day.
    #[must_use]
    pub const fn for_expanded(expanded: bool) -> Self {
        Self {
            left: if expanded { Some(Button::ResetToDefaults) } else { None },
            right: BAR_RIGHT,
        }
    }
}

/// The preferences being edited, and what the window shows around them.
#[derive(Debug, Clone)]
pub struct SettingsForm {
    view: PreferencesView,
    platform: Platform,
    /// A control changed since the last `take_save`.
    changed: bool,
    error: Option<&'static str>,
    /// The window shows every section, not only the controls used every day.
    expanded: bool,
}

impl SettingsForm {
    /// The form of a window that opens small, with the controls used every day only.
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

    /// Shows a message under the controls used every day, or clears it. The edits stay, so the user can correct them.
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

    /// Whether the window shows every section, or only the controls used every day.
    #[must_use]
    pub const fn is_expanded(&self) -> bool {
        self.expanded
    }

    /// Shows every section, or only the controls used every day. No setting changes, so nothing is saved.
    pub const fn toggle_expanded(&mut self) {
        self.expanded = !self.expanded;
    }

    /// The buttons of the bottom bar and where they sit.
    #[must_use]
    pub const fn bar(&self) -> ButtonBar {
        ButtonBar::for_expanded(self.expanded)
    }

    pub fn set_language(&mut self, language: LanguageValue) {
        self.view.language = language;
        self.changed = true;
    }

    /// The language the app's windows, menus and messages speak.
    pub fn set_interface_language(&mut self, language: InterfaceLanguageValue) {
        self.view.system.interface_language = language;
        self.changed = true;
    }

    /// Switches the app's windows from light to dark, or back: the button in the window's corner.
    pub const fn toggle_theme(&mut self) {
        self.view.system.theme = match self.view.system.theme {
            ThemeValue::Light => ThemeValue::Dark,
            ThemeValue::Dark => ThemeValue::Light,
        };
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
