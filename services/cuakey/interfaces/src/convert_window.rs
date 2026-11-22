//! The conversion window drawn with egui. Every change returns the whole edited view to store; `Chuyển mã` returns a
//! request to convert. What the controls do lives in [`ConvertForm`]; this file only draws it.

use cuakey_application::{ConvertCaseValue, HotkeyView, PreferencesView, Toggle};
use egui::{Context, Ui};

use crate::convert_form::ConvertForm;
use crate::hotkey_editor::show_hotkey;
use crate::settings_window::{CODE_TABLES, checkbox, combo};
use crate::{UiAction, labels};

const CASES: [(ConvertCaseValue, &str); 4] = [
    (ConvertCaseValue::Upper, labels::CONVERT_UPPER),
    (ConvertCaseValue::Lower, labels::CONVERT_LOWER),
    (ConvertCaseValue::SentenceStart, labels::CONVERT_SENTENCE_START),
    (ConvertCaseValue::EachWord, labels::CONVERT_EACH_WORD),
];

/// What the hotkey field shows while no quick convert key is set.
const NO_HOTKEY: HotkeyView = HotkeyView {
    ctrl: Toggle::Off,
    alt: Toggle::Off,
    win: Toggle::Off,
    shift: Toggle::Off,
    key: None,
};

fn case_boxes(ui: &mut Ui, form: &mut ConvertForm, actions: &mut Vec<UiAction>) {
    for (case, label) in CASES {
        let mut on = form.is_case_ticked(case);
        if ui.checkbox(&mut on, label).changed() {
            actions.push(form.set_case(case, on));
        }
    }
}

fn option_boxes(ui: &mut Ui, form: &mut ConvertForm, actions: &mut Vec<UiAction>) {
    if let Some(on) = checkbox(ui, labels::CONVERT_REMOVE_ACCENTS, form.view().convert.remove_accents) {
        actions.push(form.set_remove_accents(on));
    }
    if let Some(on) = checkbox(ui, labels::CONVERT_NOTIFY, form.view().convert.notify_when_done) {
        actions.push(form.set_notify_when_done(on));
    }
    ui.horizontal(|ui| {
        ui.label(labels::CONVERT_HOTKEY);
        let hotkey = form.view().convert.quick_convert_hotkey.unwrap_or(NO_HOTKEY);
        if let Some(edited) = show_hotkey(ui, "quick_convert_hotkey", hotkey) {
            actions.push(form.set_quick_hotkey(edited));
        }
    });
}

fn table_row(ui: &mut Ui, form: &mut ConvertForm, actions: &mut Vec<UiAction>) {
    ui.horizontal(|ui| {
        ui.label(labels::CONVERT_FROM);
        if let Some(table) = combo(ui, "convert_from", form.view().convert.from, &CODE_TABLES) {
            actions.push(form.set_from(table));
        }
        if ui.button(labels::CONVERT_SWAP).clicked() {
            actions.push(form.swap_tables());
        }
        ui.label(labels::CONVERT_TO);
        if let Some(table) = combo(ui, "convert_to", form.view().convert.to, &CODE_TABLES) {
            actions.push(form.set_to(table));
        }
    });
}

fn bottom_bar(ui: &mut Ui, form: &ConvertForm, actions: &mut Vec<UiAction>) {
    if let Some(message) = form.error() {
        ui.colored_label(ui.visuals().error_fg_color, message);
    }
    ui.horizontal(|ui| {
        if ui.button(labels::CONVERT_RUN).clicked() {
            actions.push(form.convert());
        }
        if ui.button(labels::BUTTON_CLOSE).clicked() {
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

    /// Replaces the edited preferences, when the background process changed something.
    pub fn set_preferences(&mut self, prefs: PreferencesView) {
        self.form.set_preferences(prefs);
    }

    /// Shows a message under the buttons, or clears it. The user's edits stay.
    pub const fn show_error(&mut self, message: Option<&'static str>) {
        self.form.show_error(message);
    }

    /// Draws the window into `ctx` and returns what the user asked for this frame. A frame that changed several
    /// controls returns a save for each; the last holds every change.
    pub fn show(&mut self, ctx: &Context) -> Vec<UiAction> {
        let mut actions = Vec::new();
        let form = &mut self.form;
        ctx.root_ui(|ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                case_boxes(ui, form, &mut actions);
                option_boxes(ui, form, &mut actions);
                table_row(ui, form, &mut actions);
                bottom_bar(ui, form, &mut actions);
            });
        });
        actions
    }
}

#[cfg(test)]
mod tests;
