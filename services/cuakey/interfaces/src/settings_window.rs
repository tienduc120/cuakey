//! The settings window drawn with egui. It opens small, with the control group only, and the expand button shows the
//! other sections. Every change applies at once: a frame that changed a control returns one `Save` with the whole
//! edited view. What is shown and enabled lives in [`SettingsForm`]; this file only draws it.

use cuakey_application::{CodeTableValue, InputMethodValue, LanguageValue, PreferencesView};
use egui::Ui;

use crate::hotkey_editor::show_hotkey;
use crate::settings_form::{GatedControl, SettingsForm, is_on};
use crate::{Button, Platform, UiAction, labels, widgets};

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

/// The window's inner size with the control group only, the size it opens at. The group and the button column measure
/// 719 x 106 points in Segoe UI and at most 734 wide in the other fonts the window may borrow; the panel's margins and
/// a line for an error message come on top, so nothing scrolls or clips.
pub const COMPACT_SIZE: egui::Vec2 = egui::vec2(760.0, 150.0);

/// The window's inner size with every section: as wide as the compact window, and tall enough to show a useful part
/// of the sections, which scroll past that.
pub(crate) const EXPANDED_SIZE: egui::Vec2 = egui::vec2(COMPACT_SIZE.x, 620.0);

/// A checkbox for an on/off preference. Returns the new state when the user clicked it.
pub(crate) fn checkbox(ui: &mut Ui, label: &str, current: cuakey_application::Toggle) -> Option<bool> {
    let on = is_on(current);
    widgets::checkbox(ui, label, on).changed().then_some(!on)
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

fn input_method_and_code_table(ui: &mut Ui, form: &mut SettingsForm) {
    ui.horizontal(|ui| {
        if let Some(method) = combo(ui, "input_method", form.view().typing.input_method, &INPUT_METHODS) {
            form.set_input_method(method);
        }
        ui.label(labels::CODE_TABLE);
        if let Some(table) = combo(ui, "code_table", form.view().typing.code_table, &CODE_TABLES) {
            form.set_code_table(table);
        }
    });
}

/// The rows of the control group, as grid cells: a label, then its controls. Each hotkey editor sits in one cell and
/// draws the same fixed-width parts, so the modifier boxes and key fields of the two hotkey rows line up.
fn control_rows(ui: &mut Ui, form: &mut SettingsForm) {
    ui.label(labels::INPUT_METHOD);
    input_method_and_code_table(ui, form);
    ui.end_row();

    ui.label(labels::SWITCH_HOTKEY);
    if let Some(hotkey) = show_hotkey(ui, "switch_hotkey", form.view().switching.hotkey) {
        form.set_switch_hotkey(hotkey);
    }
    toggle_row!(ui, form, labels::BEEP, switching.beep, set_beep);
    ui.end_row();

    ui.label(labels::SETTINGS_HOTKEY);
    if let Some(hotkey) = show_hotkey(ui, "settings_hotkey", form.view().behaviours.settings_hotkey) {
        form.set_settings_hotkey(hotkey);
    }
    ui.end_row();

    ui.label(labels::LANGUAGE);
    if let Some(language) = radios(ui, form.view().language, &LANGUAGES) {
        form.set_language(language);
    }
    ui.end_row();
}

/// The control group, framed, with its title inside the frame: egui has no titled group box.
fn control_group(ui: &mut Ui, form: &mut SettingsForm) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.strong(labels::SECTION_CONTROL);
        egui::Grid::new("control_rows")
            .num_columns(3)
            .show(ui, |ui| control_rows(ui, form));
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
        actions.extend(Button::OpenAlwaysOn.action());
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
        actions.extend(Button::OpenMacros.action());
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

fn button_label(form: &SettingsForm, button: Button) -> &'static str {
    match button {
        Button::CloseWindow => labels::BUTTON_CLOSE,
        Button::ExitApp => labels::BUTTON_EXIT,
        Button::ResetToDefaults => labels::BUTTON_RESET,
        Button::OpenMacros => labels::MACRO_TABLE,
        Button::OpenAlwaysOn => labels::ALWAYS_ON_APPS,
        Button::ToggleExpanded if form.is_expanded() => labels::BUTTON_COLLAPSE,
        Button::ToggleExpanded => labels::BUTTON_EXPAND,
    }
}

/// Whether `button` sits in the column beside the control group. The reset sits under the expanded sections, away
/// from the buttons clicked every day.
const fn is_in_column(button: Button) -> bool {
    !matches!(button, Button::ResetToDefaults)
}

/// One width for every button of the column: the widest of their labels, both labels of the expand button included,
/// so the column keeps its place when that label changes.
fn column_width(ui: &Ui) -> f32 {
    let widest = [
        labels::BUTTON_CLOSE,
        labels::BUTTON_EXIT,
        labels::BUTTON_EXPAND,
        labels::BUTTON_COLLAPSE,
    ]
    .into_iter()
    .map(|label| {
        egui::WidgetText::from(label)
            .into_galley(
                ui,
                Some(egui::TextWrapMode::Extend),
                f32::INFINITY,
                egui::TextStyle::Button,
            )
            .size()
            .x
    })
    .fold(0.0, f32::max);
    widest + 2.0 * ui.spacing().button_padding.x
}

/// The column beside the control group, top to bottom. Returns the button clicked.
fn button_column(ui: &mut Ui, form: &SettingsForm) -> Option<Button> {
    let min_size = egui::vec2(column_width(ui), 0.0);
    ui.vertical(|ui| {
        let mut clicked = None;
        for &button in form.buttons().iter().filter(|button| is_in_column(**button)) {
            if ui
                .add(egui::Button::new(button_label(form, button)).min_size(min_size))
                .clicked()
            {
                clicked = Some(button);
            }
        }
        clicked
    })
    .inner
}

/// The buttons under the expanded sections. Returns the button clicked.
fn buttons_below(ui: &mut Ui, form: &SettingsForm) -> Option<Button> {
    let mut clicked = None;
    for &button in form.buttons().iter().filter(|button| !is_in_column(**button)) {
        if ui.button(button_label(form, button)).clicked() {
            clicked = Some(button);
        }
    }
    clicked
}

/// The sections only the expanded window shows, then the buttons under them. Returns the button clicked.
fn expanded_sections(ui: &mut Ui, form: &mut SettingsForm, actions: &mut Vec<UiAction>) -> Option<Button> {
    typing_section(ui, form, actions);
    macros_section(ui, form, actions);
    system_section(ui, form);
    about_section(ui);
    buttons_below(ui, form)
}

/// Acts on a clicked button. The expand button concerns this window alone: it resizes the window and asks the process
/// nothing.
fn press(ui: &Ui, form: &mut SettingsForm, button: Button, actions: &mut Vec<UiAction>) {
    if button == Button::ToggleExpanded {
        form.toggle_expanded();
        let size = if form.is_expanded() {
            EXPANDED_SIZE
        } else {
            COMPACT_SIZE
        };
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
    }
    actions.extend(button.action());
}

/// The control group with the button column beside it, the message of the last refused change under them, and, when
/// expanded, the other sections in a scroll area below. In compact mode nothing scrolls.
fn contents(ui: &mut Ui, form: &mut SettingsForm, actions: &mut Vec<UiAction>) {
    let mut clicked = ui
        .horizontal_top(|ui| {
            control_group(ui, form);
            button_column(ui, form)
        })
        .inner;
    if let Some(message) = form.error() {
        ui.colored_label(ui.visuals().error_fg_color, message);
    }
    if form.is_expanded() {
        let below = egui::ScrollArea::vertical()
            .show(ui, |ui| expanded_sections(ui, form, actions))
            .inner;
        clicked = clicked.or(below);
    }
    if let Some(button) = clicked {
        press(ui, form, button, actions);
    }
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

    /// Shows a message under the control group, or clears it. The user's edits stay.
    pub const fn show_error(&mut self, message: Option<&'static str>) {
        self.form.show_error(message);
    }

    /// Draws the window into `ui`, the root ui of its viewport, and returns what the user asked for this frame.
    pub fn show(&mut self, ui: &mut Ui) -> Vec<UiAction> {
        let mut actions = Vec::new();
        let form = &mut self.form;
        egui::CentralPanel::default().show(ui, |ui| contents(ui, form, &mut actions));
        if let Some(view) = form.take_save() {
            actions.insert(0, UiAction::Save(view));
        }
        actions
    }
}

#[cfg(test)]
mod tests;
