//! The settings window drawn with egui. Every change applies at once: a frame that changed a control returns one
//! `Save` with the whole edited view. What is shown and enabled lives in [`SettingsForm`]; this file only draws it.

use cuakey_application::{CodeTableValue, InputMethodValue, LanguageValue, PreferencesView};
use egui::{Context, Ui};

use crate::hotkey_editor::show_hotkey;
use crate::settings_form::{GatedControl, SettingsForm, is_on};
use crate::{Button, Platform, UiAction, labels};

const INPUT_METHODS: [(InputMethodValue, &str); 3] = [
    (InputMethodValue::Telex, labels::INPUT_METHOD_TELEX),
    (InputMethodValue::Vni, labels::INPUT_METHOD_VNI),
    (InputMethodValue::SimpleTelex, labels::INPUT_METHOD_SIMPLE_TELEX),
];

pub(crate) const CODE_TABLES: [(CodeTableValue, &str); 5] = [
    (CodeTableValue::Unicode, labels::CODE_TABLE_UNICODE),
    (CodeTableValue::Tcvn3, labels::CODE_TABLE_TCVN3),
    (CodeTableValue::VniWindows, labels::CODE_TABLE_VNI_WINDOWS),
    (CodeTableValue::UnicodeCompound, labels::CODE_TABLE_UNICODE_COMPOUND),
    (CodeTableValue::Cp1258, labels::CODE_TABLE_CP1258),
];

const LANGUAGES: [(LanguageValue, &str); 2] = [
    (LanguageValue::Vietnamese, labels::LANGUAGE_VIETNAMESE),
    (LanguageValue::English, labels::LANGUAGE_ENGLISH),
];

/// A checkbox for an on/off preference. Returns the new state when the user clicked it.
pub(crate) fn checkbox(ui: &mut Ui, label: &str, current: cuakey_application::Toggle) -> Option<bool> {
    let mut on = is_on(current);
    ui.checkbox(&mut on, label).changed().then_some(on)
}

/// A checkbox that is hidden or greyed by the form's rules.
fn gated_checkbox(
    ui: &mut Ui,
    form: &SettingsForm,
    control: GatedControl,
    label: &str,
    current: cuakey_application::Toggle,
) -> Option<bool> {
    if !form.is_visible(control) {
        return None;
    }
    ui.add_enabled_ui(form.is_enabled(control), |ui| checkbox(ui, label, current))
        .inner
}

/// A drop-down of choices. Returns the new choice when the user picked another.
pub(crate) fn combo<T: Copy + PartialEq>(ui: &mut Ui, id_salt: &str, current: T, choices: &[(T, &str)]) -> Option<T> {
    let mut selected = current;
    let shown = choices
        .iter()
        .find(|(value, _)| *value == current)
        .map_or("", |(_, label)| *label);
    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(shown)
        .show_ui(ui, |ui| {
            for (value, label) in choices {
                ui.selectable_value(&mut selected, *value, *label);
            }
        });
    (selected != current).then_some(selected)
}

/// Radio buttons side by side. Returns the new choice when the user picked another.
fn radios<T: Copy + PartialEq>(ui: &mut Ui, current: T, choices: &[(T, &str)]) -> Option<T> {
    let mut selected = current;
    ui.horizontal(|ui| {
        for (value, label) in choices {
            ui.radio_value(&mut selected, *value, *label);
        }
    });
    (selected != current).then_some(selected)
}

/// Draws a checkbox for `$path` of the view and stores a click through `$setter`.
macro_rules! toggle_row {
    ($ui:expr, $form:expr, $label:expr, $($path:ident).+, $setter:ident) => {
        if let Some(on) = checkbox($ui, $label, $form.view().$($path).+) {
            $form.$setter(on);
        }
    };
}

/// Like `toggle_row!`, for a control the form can hide or grey.
macro_rules! gated_row {
    ($ui:expr, $form:expr, $control:expr, $label:expr, $($path:ident).+, $setter:ident) => {
        if let Some(on) = gated_checkbox($ui, $form, $control, $label, $form.view().$($path).+) {
            $form.$setter(on);
        }
    };
}

fn control_section(ui: &mut Ui, form: &mut SettingsForm) {
    ui.heading(labels::SECTION_CONTROL);
    ui.horizontal(|ui| {
        ui.label(labels::INPUT_METHOD);
        if let Some(method) = combo(ui, "input_method", form.view().typing.input_method, &INPUT_METHODS) {
            form.set_input_method(method);
        }
        ui.label(labels::CODE_TABLE);
        if let Some(table) = combo(ui, "code_table", form.view().typing.code_table, &CODE_TABLES) {
            form.set_code_table(table);
        }
    });
    ui.horizontal(|ui| {
        ui.label(labels::SWITCH_HOTKEY);
        if let Some(hotkey) = show_hotkey(ui, "switch_hotkey", form.view().switching.hotkey) {
            form.set_switch_hotkey(hotkey);
        }
        toggle_row!(ui, form, labels::BEEP, switching.beep, set_beep);
    });
    ui.horizontal(|ui| {
        ui.label(labels::SETTINGS_HOTKEY);
        if let Some(hotkey) = show_hotkey(ui, "settings_hotkey", form.view().behaviours.settings_hotkey) {
            form.set_settings_hotkey(hotkey);
        }
    });
    ui.horizontal(|ui| {
        ui.label(labels::LANGUAGE);
        if let Some(language) = radios(ui, form.view().language, &LANGUAGES) {
            form.set_language(language);
        }
    });
}

fn typing_section(ui: &mut Ui, form: &mut SettingsForm, actions: &mut Vec<UiAction>) {
    ui.heading(labels::SECTION_TYPING);
    toggle_row!(
        ui,
        form,
        labels::NEW_TONE_STYLE,
        typing.new_tone_style,
        set_new_tone_style
    );
    gated_row!(
        ui,
        form,
        GatedControl::FixAutocomplete,
        labels::FIX_AUTOCOMPLETE,
        compatibility.fix_autocomplete,
        set_fix_autocomplete
    );
    toggle_row!(ui, form, labels::SPELL_CHECK, typing.spell_check, set_spell_check);
    ui.indent("spelling", |ui| {
        gated_row!(
            ui,
            form,
            GatedControl::RestoreInvalidWord,
            labels::RESTORE_INVALID_WORD,
            typing.restore_invalid_word,
            set_restore_invalid_word
        );
        gated_row!(
            ui,
            form,
            GatedControl::AllowZwjfConsonants,
            labels::ALLOW_ZWJF_CONSONANTS,
            typing.allow_zwjf_consonants,
            set_allow_zwjf_consonants
        );
    });
    toggle_row!(
        ui,
        form,
        labels::CAPITALIZE_SENTENCES,
        typing.capitalize_sentences,
        set_capitalize_sentences
    );
    toggle_row!(ui, form, labels::SMART_SWITCH, switching.smart_switch, set_smart_switch);
    toggle_row!(
        ui,
        form,
        labels::REMEMBER_CODE_TABLE,
        switching.remember_code_table,
        set_remember_code_table
    );
    toggle_row!(
        ui,
        form,
        labels::WORK_IN_OTHER_LANGUAGES,
        switching.work_in_other_languages,
        set_work_in_other_languages
    );
    toggle_row!(ui, form, labels::KEEP_W, behaviours.keep_w, set_keep_w);
    toggle_row!(
        ui,
        form,
        labels::ONLY_TEXT_BOXES,
        behaviours.only_text_boxes,
        set_only_text_boxes
    );
    if ui.button(labels::ALWAYS_ON_APPS).clicked() {
        actions.push(Button::OpenAlwaysOn.action());
    }
}

fn macros_section(ui: &mut Ui, form: &mut SettingsForm, actions: &mut Vec<UiAction>) {
    ui.heading(labels::SECTION_MACROS);
    toggle_row!(ui, form, labels::USE_MACROS, macros.use_macros, set_use_macros);
    toggle_row!(
        ui,
        form,
        labels::MACROS_IN_ENGLISH,
        macros.macros_in_english,
        set_macros_in_english
    );
    toggle_row!(
        ui,
        form,
        labels::MACRO_MATCH_CASE,
        macros.match_case,
        set_macro_match_case
    );
    if ui.button(labels::MACRO_TABLE).clicked() {
        actions.push(Button::OpenMacros.action());
    }
    toggle_row!(ui, form, labels::QUICK_TELEX, typing.quick_telex, set_quick_telex);
    toggle_row!(
        ui,
        form,
        labels::QUICK_FIRST_CONSONANT,
        typing.quick_first_consonant,
        set_quick_first_consonant
    );
    toggle_row!(
        ui,
        form,
        labels::QUICK_LAST_CONSONANT,
        typing.quick_last_consonant,
        set_quick_last_consonant
    );
}

fn system_section(ui: &mut Ui, form: &mut SettingsForm) {
    ui.heading(labels::SECTION_SYSTEM);
    toggle_row!(
        ui,
        form,
        labels::START_WITH_WINDOWS,
        system.start_with_windows,
        set_start_with_windows
    );
    toggle_row!(
        ui,
        form,
        labels::SHOW_SETTINGS_AT_STARTUP,
        system.show_settings_at_startup,
        set_show_settings_at_startup
    );
    gated_row!(
        ui,
        form,
        GatedControl::DesktopShortcut,
        labels::DESKTOP_SHORTCUT,
        system.desktop_shortcut,
        set_desktop_shortcut
    );
    gated_row!(
        ui,
        form,
        GatedControl::RunAsAdministrator,
        labels::RUN_AS_ADMINISTRATOR,
        system.run_as_administrator,
        set_run_as_administrator
    );
    gated_row!(
        ui,
        form,
        GatedControl::StoreAppSupport,
        labels::STORE_APP_SUPPORT,
        compatibility.store_app_support,
        set_store_app_support
    );
    gated_row!(
        ui,
        form,
        GatedControl::ChromiumFix,
        labels::CHROMIUM_FIX,
        compatibility.chromium_fix,
        set_chromium_fix
    );
    gated_row!(
        ui,
        form,
        GatedControl::ClipboardInput,
        labels::CLIPBOARD_INPUT,
        compatibility.clipboard_input,
        set_clipboard_input
    );
}

fn about_section(ui: &mut Ui) {
    ui.heading(labels::SECTION_ABOUT);
    ui.label(labels::ABOUT_NAME);
    ui.label(labels::ABOUT_VERSION);
    ui.label(labels::ABOUT_LICENSE);
    ui.hyperlink(labels::ABOUT_URL);
    ui.label(labels::ABOUT_CREDITS);
}

/// The buttons under the settings, and the message of the last refused change.
fn bottom_bar(ui: &mut Ui, form: &SettingsForm, actions: &mut Vec<UiAction>) {
    if let Some(message) = form.error() {
        ui.colored_label(ui.visuals().error_fg_color, message);
    }
    ui.horizontal(|ui| {
        for (label, button) in [
            (labels::BUTTON_EXIT, Button::ExitApp),
            (labels::BUTTON_RESET, Button::ResetToDefaults),
            (labels::BUTTON_CLOSE, Button::CloseWindow),
        ] {
            if ui.button(label).clicked() {
                actions.push(button.action());
            }
        }
    });
}

fn all_sections(ui: &mut Ui, form: &mut SettingsForm, actions: &mut Vec<UiAction>) {
    control_section(ui, form);
    typing_section(ui, form, actions);
    macros_section(ui, form, actions);
    system_section(ui, form);
    about_section(ui);
}

/// The settings window: draw it every frame, act on what it returns.
#[derive(Debug, Clone)]
pub struct SettingsWindow {
    form: SettingsForm,
}

impl SettingsWindow {
    #[must_use]
    pub const fn new(prefs: PreferencesView, platform: Platform) -> Self {
        Self {
            form: SettingsForm::new(prefs, platform),
        }
    }

    /// Replaces the edited preferences, after a reset or when the background process changed something.
    pub fn set_preferences(&mut self, prefs: PreferencesView) {
        self.form.set_preferences(prefs);
    }

    /// Shows a message under the buttons, or clears it. The user's edits stay.
    pub const fn show_error(&mut self, message: Option<&'static str>) {
        self.form.show_error(message);
    }

    /// Draws the window into `ctx` and returns what the user asked for this frame.
    pub fn show(&mut self, ctx: &Context) -> Vec<UiAction> {
        let mut actions = Vec::new();
        let form = &mut self.form;
        ctx.root_ui(|ui| {
            egui::Panel::bottom("buttons").show(ui, |ui| bottom_bar(ui, form, &mut actions));
            egui::CentralPanel::default().show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| all_sections(ui, form, &mut actions));
            });
        });
        if let Some(view) = form.take_save() {
            actions.insert(0, UiAction::Save(view));
        }
        actions
    }
}

#[cfg(test)]
mod tests;
