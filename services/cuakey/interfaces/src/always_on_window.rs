//! The always-on window drawn with egui. Every add or remove returns the whole edited view to store. What the buttons
//! do lives in [`AlwaysOnForm`]; this file only draws it, in the interface language of the view it edits.

use cuakey_application::{InterfaceLanguageValue, PreferencesView};
use egui::Ui;

use crate::UiAction;
use crate::always_on_form::AlwaysOnForm;
use crate::labels::labels_for;

fn app_list(ui: &mut Ui, form: &mut AlwaysOnForm) {
    let mut clicked = None;
    let selected = form.selected();
    egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
        for (index, app) in form.view().always_on.iter().enumerate() {
            if ui.selectable_label(selected == Some(index), app).clicked() {
                clicked = Some(index);
            }
        }
    });
    if clicked.is_some() {
        form.select(clicked);
    }
}

fn input_row(ui: &mut Ui, form: &mut AlwaysOnForm, actions: &mut Vec<UiAction>) {
    let text = labels_for(form.view());
    ui.horizontal(|ui| {
        ui.add(egui::TextEdit::singleline(form.input_mut()).hint_text(text.always_on_hint));
        if ui.button(text.always_on_add).clicked() {
            actions.extend(form.add());
        }
        if ui.button(text.always_on_remove).clicked() {
            actions.extend(form.remove());
        }
    });
}

fn contents(ui: &mut Ui, form: &mut AlwaysOnForm, actions: &mut Vec<UiAction>) {
    app_list(ui, form);
    input_row(ui, form, actions);
    if let Some(message) = form.error() {
        ui.colored_label(ui.visuals().error_fg_color, message);
    }
    if ui.button(labels_for(form.view()).button_close).clicked() {
        actions.push(AlwaysOnForm::close());
    }
}

/// The always-on window: draw it every frame, act on what it returns.
#[derive(Debug, Clone)]
pub struct AlwaysOnWindow {
    form: AlwaysOnForm,
}

impl AlwaysOnWindow {
    #[must_use]
    pub const fn new(prefs: PreferencesView) -> Self {
        Self {
            form: AlwaysOnForm::new(prefs),
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

    /// Shows a message under the field, or clears it. The user's edits stay.
    pub const fn show_error(&mut self, message: Option<&'static str>) {
        self.form.show_error(message);
    }

    /// Draws the window into `ui`, the root ui of its viewport, and returns what the user asked for this frame.
    pub fn show(&mut self, ui: &mut Ui) -> Vec<UiAction> {
        let mut actions = Vec::new();
        let form = &mut self.form;
        egui::CentralPanel::default().show(ui, |ui| contents(ui, form, &mut actions));
        actions
    }
}

#[cfg(test)]
mod tests;
