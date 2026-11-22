//! The state behind the macro window: the rows being edited, the two input fields, the selected row, and the import
//! choice that waits for an answer. It draws nothing and opens no dialog, so a test drives every button.

use std::path::PathBuf;

use cuakey_application::{ImportMode, MacroView, PreferencesView, Toggle};

use crate::UiAction;
use crate::settings_form::toggle;

/// Shortcuts in alphabetical order that ignores case; the exact text breaks a tie.
fn sort_rows(rows: &mut [MacroView]) {
    rows.sort_by(|a, b| {
        a.shortcut
            .to_lowercase()
            .cmp(&b.shortcut.to_lowercase())
            .then_with(|| a.shortcut.cmp(&b.shortcut))
    });
}

/// The macros being edited, the fields for a new one, and what the window shows around them.
///
/// Every change to the table returns the whole table to store. Whether a row is valid is the use case's call: when it
/// refuses, the caller shows the message and feeds the stored table back with [`MacroForm::set_macros`].
#[derive(Debug, Clone)]
pub struct MacroForm {
    view: PreferencesView,
    macros: Vec<MacroView>,
    shortcut_input: String,
    expansion_input: String,
    selected: Option<usize>,
    /// The file the user picked to import, until they choose between adding and replacing.
    pending_import: Option<PathBuf>,
    error: Option<&'static str>,
}

impl MacroForm {
    #[must_use]
    pub fn new(view: PreferencesView, mut macros: Vec<MacroView>) -> Self {
        sort_rows(&mut macros);
        Self {
            view,
            macros,
            shortcut_input: String::new(),
            expansion_input: String::new(),
            selected: None,
            pending_import: None,
            error: None,
        }
    }

    #[must_use]
    pub const fn view(&self) -> &PreferencesView {
        &self.view
    }

    #[must_use]
    pub fn macros(&self) -> &[MacroView] {
        &self.macros
    }

    #[must_use]
    pub const fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub const fn select(&mut self, row: Option<usize>) {
        self.selected = row;
    }

    pub const fn shortcut_mut(&mut self) -> &mut String {
        &mut self.shortcut_input
    }

    pub const fn expansion_mut(&mut self) -> &mut String {
        &mut self.expansion_input
    }

    /// Replaces the rows, after a load or an import, or to drop an edit the use case refused.
    pub fn set_macros(&mut self, mut macros: Vec<MacroView>) {
        sort_rows(&mut macros);
        self.macros = macros;
        self.selected = None;
    }

    /// Replaces the edited preferences, when another process changed them. Nothing is saved.
    pub fn set_preferences(&mut self, view: PreferencesView) {
        self.view = view;
    }

    /// Shows a message under the buttons, or clears it. The edits stay.
    pub const fn show_error(&mut self, message: Option<&'static str>) {
        self.error = message;
    }

    #[must_use]
    pub const fn error(&self) -> Option<&'static str> {
        self.error
    }

    /// `+ Thêm`: the two fields become a row, which replaces a row with the same shortcut. A blank shortcut adds
    /// nothing.
    pub fn add(&mut self) -> Option<UiAction> {
        let shortcut = self.shortcut_input.trim();
        if shortcut.is_empty() {
            return None;
        }
        let row = MacroView {
            shortcut: shortcut.to_owned(),
            expansion: std::mem::take(&mut self.expansion_input),
        };
        self.shortcut_input.clear();
        self.macros.retain(|existing| existing.shortcut != row.shortcut);
        self.macros.push(row);
        sort_rows(&mut self.macros);
        self.selected = None;
        Some(self.save_macros())
    }

    /// `- Xóa`: the selected row goes. Nothing selected removes nothing.
    pub fn remove(&mut self) -> Option<UiAction> {
        let row = self.selected.take()?;
        (row < self.macros.len()).then(|| {
            self.macros.remove(row);
            self.save_macros()
        })
    }

    fn save_macros(&self) -> UiAction {
        UiAction::SaveMacros(self.macros.clone())
    }

    /// The user picked a file to import; the window now asks whether to add or replace.
    pub fn choose_import_file(&mut self, path: PathBuf) {
        self.pending_import = Some(path);
    }

    #[must_use]
    pub const fn import_pending(&self) -> bool {
        self.pending_import.is_some()
    }

    /// The answer to the import question. Without a picked file there is nothing to import.
    pub fn choose_import_mode(&mut self, mode: ImportMode) -> Option<UiAction> {
        let path = self.pending_import.take()?;
        Some(UiAction::ImportMacros { path, mode })
    }

    pub fn cancel_import(&mut self) {
        self.pending_import = None;
    }

    /// The user picked where to write the macros.
    #[must_use]
    pub const fn export_to(path: PathBuf) -> UiAction {
        UiAction::ExportMacros { path }
    }

    /// The "match capitals" setting lives with the preferences, so it saves the whole view.
    pub fn set_match_case(&mut self, on: bool) -> UiAction {
        self.view.macros.match_case = toggle(on);
        UiAction::Save(self.view.clone())
    }

    #[must_use]
    pub const fn match_case(&self) -> Toggle {
        self.view.macros.match_case
    }
}

#[cfg(test)]
mod tests;
