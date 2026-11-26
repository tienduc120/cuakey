//! The state behind the always-on window: the apps that always type Vietnamese, the field for a new one, and the
//! selected row. It draws nothing, so a test drives every button.

use cuakey_application::{PreferencesView, app_name};

use crate::{UiAction, error_message};

/// The apps being edited and the field for a new one.
#[derive(Debug, Clone)]
pub struct AlwaysOnForm {
    view: PreferencesView,
    input: String,
    selected: Option<usize>,
    error: Option<&'static str>,
}

impl AlwaysOnForm {
    #[must_use]
    pub const fn new(view: PreferencesView) -> Self {
        Self {
            view,
            input: String::new(),
            selected: None,
            error: None,
        }
    }

    #[must_use]
    pub const fn view(&self) -> &PreferencesView {
        &self.view
    }

    #[must_use]
    pub const fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub const fn select(&mut self, row: Option<usize>) {
        self.selected = row;
    }

    pub const fn input_mut(&mut self) -> &mut String {
        &mut self.input
    }

    /// Replaces the edited preferences, when another process changed them. Nothing is saved.
    pub fn set_preferences(&mut self, view: PreferencesView) {
        self.view = view;
        self.selected = None;
    }

    /// Shows a message under the field, or clears it.
    pub const fn show_error(&mut self, message: Option<&'static str>) {
        self.error = message;
    }

    #[must_use]
    pub const fn error(&self) -> Option<&'static str> {
        self.error
    }

    /// `+ Thêm`: the field's text becomes the app's lower-case file name. Blank text is refused with a message, and an
    /// app already listed is ignored.
    pub fn add(&mut self) -> Option<UiAction> {
        let name = match app_name(&self.input) {
            Ok(name) => name,
            Err(error) => {
                self.error = Some(error_message(error, self.view.system.interface_language));
                return None;
            }
        };
        self.error = None;
        self.input.clear();
        if self.view.always_on.contains(&name) {
            return None;
        }
        self.view.always_on.push(name);
        self.view.always_on.sort();
        self.selected = None;
        Some(UiAction::Save(self.view.clone()))
    }

    /// `- Xóa`: the selected app goes. Nothing selected removes nothing.
    pub fn remove(&mut self) -> Option<UiAction> {
        let row = self.selected.take()?;
        (row < self.view.always_on.len()).then(|| {
            self.view.always_on.remove(row);
            UiAction::Save(self.view.clone())
        })
    }

    /// `Đóng`.
    #[must_use]
    pub const fn close() -> UiAction {
        UiAction::Close
    }
}

#[cfg(test)]
mod tests;
