//! The macro window drawn with egui. A change to the table returns it whole for the caller to store. File dialogs are
//! opened here and nowhere else: the pure state in [`MacroForm`] only receives the path the user chose.

use std::path::PathBuf;

use cuakey_application::{ImportMode, MacroView, PreferencesView};
use egui::{Context, Ui};

use crate::macro_form::MacroForm;
use crate::settings_window::checkbox;
use crate::{UiAction, labels};

const FILE_EXTENSIONS: [&str; 1] = ["txt"];

fn pick_file_to_open() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter(labels::MACRO_FILE_FILTER, &FILE_EXTENSIONS)
        .pick_file()
}

fn pick_file_to_save() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter(labels::MACRO_FILE_FILTER, &FILE_EXTENSIONS)
        .save_file()
}

fn row_clicked(ui: &mut Ui, row: &MacroView, is_selected: bool) -> bool {
    let on_shortcut = ui.selectable_label(is_selected, &row.shortcut).clicked();
    let on_expansion = ui.selectable_label(is_selected, &row.expansion).clicked();
    ui.end_row();
    on_shortcut || on_expansion
}

/// The table of macros. Returns the row the user clicked this frame.
fn grid(ui: &mut Ui, table: &[MacroView], selected: Option<usize>) -> Option<usize> {
    let mut clicked = None;
    egui::Grid::new("macro_rows")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            ui.strong(labels::MACRO_SHORTCUT_COLUMN);
            ui.strong(labels::MACRO_EXPANSION_COLUMN);
            ui.end_row();
            for (index, row) in table.iter().enumerate() {
                if row_clicked(ui, row, selected == Some(index)) {
                    clicked = Some(index);
                }
            }
        });
    clicked
}

fn rows(ui: &mut Ui, form: &mut MacroForm) {
    let clicked = egui::ScrollArea::vertical()
        .max_height(260.0)
        .show(ui, |ui| grid(ui, form.macros(), form.selected()))
        .inner;
    if clicked.is_some() {
        form.select(clicked);
    }
}

fn new_row(ui: &mut Ui, form: &mut MacroForm, actions: &mut Vec<UiAction>) {
    ui.horizontal(|ui| {
        ui.add(egui::TextEdit::singleline(form.shortcut_mut()).hint_text(labels::MACRO_SHORTCUT_COLUMN));
        ui.add(egui::TextEdit::singleline(form.expansion_mut()).hint_text(labels::MACRO_EXPANSION_COLUMN));
        if ui.button(labels::MACRO_ADD).clicked() {
            actions.extend(form.add());
        }
        if ui.button(labels::MACRO_REMOVE).clicked() {
            actions.extend(form.remove());
        }
    });
}

fn file_buttons(ui: &mut Ui, form: &mut MacroForm, actions: &mut Vec<UiAction>) {
    ui.horizontal(|ui| {
        if ui.button(labels::MACRO_IMPORT).clicked()
            && let Some(path) = pick_file_to_open()
        {
            form.choose_import_file(path);
        }
        if ui.button(labels::MACRO_EXPORT).clicked()
            && let Some(path) = pick_file_to_save()
        {
            actions.push(MacroForm::export_to(path));
        }
    });
}

fn import_buttons(ui: &mut Ui, form: &mut MacroForm, actions: &mut Vec<UiAction>) {
    ui.horizontal(|ui| {
        for (label, mode) in [
            (labels::MACRO_IMPORT_APPEND, ImportMode::Append),
            (labels::MACRO_IMPORT_REPLACE, ImportMode::Replace),
        ] {
            if ui.button(label).clicked() {
                actions.extend(form.choose_import_mode(mode));
            }
        }
        if ui.button(labels::MACRO_IMPORT_CANCEL).clicked() {
            form.cancel_import();
        }
    });
}

/// The question that follows a picked import file: add to the stored macros, or replace them.
fn import_question(ctx: &Context, form: &mut MacroForm, actions: &mut Vec<UiAction>) {
    if !form.import_pending() {
        return;
    }
    egui::Window::new(labels::MACRO_IMPORT_QUESTION)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| import_buttons(ui, form, actions));
}

fn contents(ui: &mut Ui, form: &mut MacroForm, actions: &mut Vec<UiAction>) {
    rows(ui, form);
    new_row(ui, form, actions);
    file_buttons(ui, form, actions);
    if let Some(on) = checkbox(ui, labels::MACRO_MATCH_CAPITALS, form.match_case()) {
        actions.push(form.set_match_case(on));
    }
    if let Some(message) = form.error() {
        ui.colored_label(ui.visuals().error_fg_color, message);
    }
}

/// The macro window: draw it every frame, act on what it returns.
#[derive(Debug, Clone)]
pub struct MacroWindow {
    form: MacroForm,
}

impl MacroWindow {
    #[must_use]
    pub fn new(prefs: PreferencesView, macros: Vec<MacroView>) -> Self {
        Self {
            form: MacroForm::new(prefs, macros),
        }
    }

    /// Replaces the rows, after a load or an import, or to drop an edit the use case refused.
    pub fn set_macros(&mut self, macros: Vec<MacroView>) {
        self.form.set_macros(macros);
    }

    /// Replaces the edited preferences, when the background process changed something.
    pub fn set_preferences(&mut self, prefs: PreferencesView) {
        self.form.set_preferences(prefs);
    }

    /// Shows a message under the buttons, or clears it. The user's edits stay.
    pub const fn show_error(&mut self, message: Option<&'static str>) {
        self.form.show_error(message);
    }

    /// Draws the window into `ui`, the root ui of its viewport, and returns what the user asked for this frame.
    pub fn show(&mut self, ui: &mut Ui) -> Vec<UiAction> {
        let mut actions = Vec::new();
        let form = &mut self.form;
        egui::CentralPanel::default().show(ui, |ui| contents(ui, form, &mut actions));
        import_question(ui.ctx(), form, &mut actions);
        actions
    }
}

#[cfg(test)]
mod tests;
