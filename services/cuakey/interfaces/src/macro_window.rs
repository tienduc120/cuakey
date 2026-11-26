//! The macro window drawn with egui. A change to the table returns it whole for the caller to store. File dialogs are
//! opened here and nowhere else: the pure state in [`MacroForm`] only receives the path the user chose.

use std::path::PathBuf;

use cuakey_application::{ImportMode, InterfaceLanguageValue, MacroView, PreferencesView};
use egui::{Context, Ui};

use crate::UiAction;
use crate::labels::{Labels, labels_for};
use crate::macro_form::MacroForm;
use crate::settings_window::checkbox;

#[cfg(windows)]
const FILE_EXTENSIONS: [&str; 1] = ["txt"];

#[cfg(windows)]
fn pick_file_to_open(filter: &str) -> Option<PathBuf> {
    rfd::FileDialog::new().add_filter(filter, &FILE_EXTENSIONS).pick_file()
}

#[cfg(windows)]
fn pick_file_to_save(filter: &str) -> Option<PathBuf> {
    rfd::FileDialog::new().add_filter(filter, &FILE_EXTENSIONS).save_file()
}

/// The app runs on Windows only, so elsewhere the open button finds no file.
#[cfg(not(windows))]
const fn pick_file_to_open(_filter: &str) -> Option<PathBuf> {
    None
}

/// The app runs on Windows only, so elsewhere the save button finds no file.
#[cfg(not(windows))]
const fn pick_file_to_save(_filter: &str) -> Option<PathBuf> {
    None
}

fn row_clicked(ui: &mut Ui, row: &MacroView, is_selected: bool) -> bool {
    let on_shortcut = ui.selectable_label(is_selected, &row.shortcut).clicked();
    let on_expansion = ui.selectable_label(is_selected, &row.expansion).clicked();
    ui.end_row();
    on_shortcut || on_expansion
}

/// The table of macros, its column titles in the language of `text`. Returns the row the user clicked this frame.
fn grid(ui: &mut Ui, text: &Labels, table: &[MacroView], selected: Option<usize>) -> Option<usize> {
    let mut clicked = None;
    egui::Grid::new("macro_rows")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            ui.strong(text.macro_shortcut_column);
            ui.strong(text.macro_expansion_column);
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
    let text = labels_for(form.view());
    let clicked = egui::ScrollArea::vertical()
        .max_height(260.0)
        .show(ui, |ui| grid(ui, text, form.macros(), form.selected()))
        .inner;
    if clicked.is_some() {
        form.select(clicked);
    }
}

fn new_row(ui: &mut Ui, form: &mut MacroForm, actions: &mut Vec<UiAction>) {
    let text = labels_for(form.view());
    ui.horizontal(|ui| {
        ui.add(egui::TextEdit::singleline(form.shortcut_mut()).hint_text(text.macro_shortcut_column));
        ui.add(egui::TextEdit::singleline(form.expansion_mut()).hint_text(text.macro_expansion_column));
        if ui.button(text.macro_add).clicked() {
            actions.extend(form.add());
        }
        if ui.button(text.macro_remove).clicked() {
            actions.extend(form.remove());
        }
    });
}

fn file_buttons(ui: &mut Ui, form: &mut MacroForm, actions: &mut Vec<UiAction>) {
    let text = labels_for(form.view());
    ui.horizontal(|ui| {
        if ui.button(text.macro_import).clicked()
            && let Some(path) = pick_file_to_open(text.macro_file_filter)
        {
            form.choose_import_file(path);
        }
        if ui.button(text.macro_export).clicked()
            && let Some(path) = pick_file_to_save(text.macro_file_filter)
        {
            actions.push(MacroForm::export_to(path));
        }
    });
}

fn import_buttons(ui: &mut Ui, form: &mut MacroForm, actions: &mut Vec<UiAction>) {
    let text = labels_for(form.view());
    ui.horizontal(|ui| {
        for (label, mode) in [
            (text.macro_import_append, ImportMode::Append),
            (text.macro_import_replace, ImportMode::Replace),
        ] {
            if ui.button(label).clicked() {
                actions.extend(form.choose_import_mode(mode));
            }
        }
        if ui.button(text.macro_import_cancel).clicked() {
            form.cancel_import();
        }
    });
}

/// The question that follows a picked import file: add to the stored macros, or replace them.
fn import_question(ctx: &Context, form: &mut MacroForm, actions: &mut Vec<UiAction>) {
    if !form.import_pending() {
        return;
    }
    // egui names a window after its title unless given an id, and the title changes with the language.
    egui::Window::new(labels_for(form.view()).macro_import_question)
        .id(egui::Id::unique("macro_import_question"))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| import_buttons(ui, form, actions));
}

fn contents(ui: &mut Ui, form: &mut MacroForm, actions: &mut Vec<UiAction>) {
    rows(ui, form);
    new_row(ui, form, actions);
    file_buttons(ui, form, actions);
    let match_capitals = labels_for(form.view()).macro_match_capitals;
    if let Some(on) = checkbox(ui, match_capitals, form.match_case()) {
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

    /// The language the window speaks: the interface language of the preferences it was given.
    #[must_use]
    pub const fn interface_language(&self) -> InterfaceLanguageValue {
        self.form.view().system.interface_language
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
