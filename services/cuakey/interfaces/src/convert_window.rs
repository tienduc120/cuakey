//! The conversion window drawn with egui. Every change returns the whole edited view to store; the convert button
//! returns a request to convert. What the controls do lives in [`ConvertForm`]; this file only draws it, in the
//! interface language of the view it edits.

use cuakey_application::{ConvertCaseValue, HotkeyView, InterfaceLanguageValue, PreferencesView, Toggle};
use egui::Ui;

use crate::UiAction;
use crate::convert_form::ConvertForm;
use crate::hotkey_editor::show_hotkey;
use crate::labels::{Labels, labels_for};
use crate::settings_window::{checkbox, code_tables, combo};
use crate::widgets;

const fn cases(text: &Labels) -> [(ConvertCaseValue, &'static str); 4] {
    [
        (ConvertCaseValue::Upper, text.convert_upper),
        (ConvertCaseValue::Lower, text.convert_lower),
        (ConvertCaseValue::SentenceStart, text.convert_sentence_start),
        (ConvertCaseValue::EachWord, text.convert_each_word),
    ]
}

/// What the hotkey field shows while no quick convert key is set.
const NO_HOTKEY: HotkeyView = HotkeyView {
    ctrl: Toggle::Off,
    alt: Toggle::Off,
    win: Toggle::Off,
    shift: Toggle::Off,
    key: None,
};

fn case_boxes(ui: &mut Ui, form: &mut ConvertForm, actions: &mut Vec<UiAction>) {
    for (case, label) in cases(labels_for(form.view())) {
        let on = form.is_case_ticked(case);
        if widgets::checkbox(ui, label, on).changed() {
            actions.push(form.set_case(case, !on));
        }
    }
}

fn option_boxes(ui: &mut Ui, form: &mut ConvertForm, actions: &mut Vec<UiAction>) {
    let text = labels_for(form.view());
    if let Some(on) = checkbox(ui, text.convert_remove_accents, form.view().convert.remove_accents) {
        actions.push(form.set_remove_accents(on));
    }
    if let Some(on) = checkbox(ui, text.convert_notify, form.view().convert.notify_when_done) {
        actions.push(form.set_notify_when_done(on));
    }
    ui.horizontal(|ui| {
        ui.label(text.convert_hotkey);
        let hotkey = form.view().convert.quick_convert_hotkey.unwrap_or(NO_HOTKEY);
        let language = form.view().system.interface_language;
        if let Some(edited) = show_hotkey(ui, "quick_convert_hotkey", hotkey, language) {
            actions.push(form.set_quick_hotkey(edited));
        }
    });
}

fn table_row(ui: &mut Ui, form: &mut ConvertForm, actions: &mut Vec<UiAction>) {
    let text = labels_for(form.view());
    ui.horizontal(|ui| {
        ui.label(text.convert_from);
        if let Some(table) = combo(ui, "convert_from", form.view().convert.from, code_tables, text) {
            actions.push(form.set_from(table));
        }
        if ui.button(text.convert_swap).clicked() {
            actions.push(form.swap_tables());
        }
        ui.label(text.convert_to);
        if let Some(table) = combo(ui, "convert_to", form.view().convert.to, code_tables, text) {
            actions.push(form.set_to(table));
        }
    });
}

fn bottom_bar(ui: &mut Ui, form: &ConvertForm, actions: &mut Vec<UiAction>) {
    let text = labels_for(form.view());
    if let Some(message) = form.error() {
        ui.colored_label(ui.visuals().error_fg_color, message);
    }
    ui.horizontal(|ui| {
        if ui.button(text.convert_run).clicked() {
            actions.push(form.convert());
        }
        if ui.button(text.button_close).clicked() {
            actions.push(ConvertForm::close());
        }
    });
}

/// The conversion window: draw it every frame, act on what it returns.
#[derive(Debug, Clone)]
pub struct ConvertWindow {
    form: ConvertForm,
}

impl ConvertWindow {
    #[must_use]
    pub const fn new(prefs: PreferencesView) -> Self {
        Self {
            form: ConvertForm::new(prefs),
        }
    }

    /// The language the window speaks: the interface language of the preferences it edits.
    #[must_use]
    pub const fn interface_language(&self) -> InterfaceLanguageValue {
        self.form.view().system.interface_language
    }

    /// Replaces the edited preferences, when the background process changed something.
    pub fn set_preferences(&mut self, prefs: PreferencesView) {
        self.form.set_preferences(prefs);
    }

    /// Shows a message under the buttons, or clears it. The user's edits stay.
    pub const fn show_error(&mut self, message: Option<&'static str>) {
        self.form.show_error(message);
    }

    /// Draws the window into `ui`, the root ui of its viewport, and returns what the user asked for this frame. A frame
    /// that changed several controls returns a save for each; the last holds every change.
    pub fn show(&mut self, ui: &mut Ui) -> Vec<UiAction> {
        let mut actions = Vec::new();
        let form = &mut self.form;
        egui::CentralPanel::default().show(ui, |ui| {
            case_boxes(ui, form, &mut actions);
            option_boxes(ui, form, &mut actions);
            table_row(ui, form, &mut actions);
            bottom_bar(ui, form, &mut actions);
        });
        actions
    }
}

#[cfg(test)]
mod tests;
