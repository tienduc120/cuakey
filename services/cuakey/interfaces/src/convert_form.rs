//! The state behind the conversion window: the conversion options being edited. Every change returns the whole view
//! to store, and the convert button asks to convert with what was stored. It draws nothing, so a test drives every
//! control.

use cuakey_application::{CodeTableValue, ConvertCaseValue, HotkeyView, PreferencesView, Toggle};

use crate::UiAction;
use crate::settings_form::toggle;

/// A hotkey with no modifier and no key is "no hotkey".
fn is_empty(hotkey: HotkeyView) -> bool {
    [hotkey.ctrl, hotkey.alt, hotkey.win, hotkey.shift]
        .iter()
        .all(|modifier| *modifier == Toggle::Off)
        && hotkey.key.is_none()
}

/// The conversion options being edited, and the message under the buttons.
#[derive(Debug, Clone)]
pub struct ConvertForm {
    view: PreferencesView,
    error: Option<&'static str>,
}

impl ConvertForm {
    #[must_use]
    pub const fn new(view: PreferencesView) -> Self {
        Self { view, error: None }
    }

    #[must_use]
    pub const fn view(&self) -> &PreferencesView {
        &self.view
    }

    /// Replaces the edited preferences, when another process changed them. Nothing is saved.
    pub fn set_preferences(&mut self, view: PreferencesView) {
        self.view = view;
    }

    /// Shows a message under the buttons, or clears it.
    pub const fn show_error(&mut self, message: Option<&'static str>) {
        self.error = message;
    }

    #[must_use]
    pub const fn error(&self) -> Option<&'static str> {
        self.error
    }

    fn save(&self) -> UiAction {
        UiAction::Save(self.view.clone())
    }

    /// The four case boxes are one choice: ticking one unticks the others, and unticking the ticked one keeps the case.
    pub fn set_case(&mut self, case: ConvertCaseValue, on: bool) -> UiAction {
        if on {
            self.view.convert.case = case;
        } else if self.view.convert.case == case {
            self.view.convert.case = ConvertCaseValue::Keep;
        }
        self.save()
    }

    #[must_use]
    pub fn is_case_ticked(&self, case: ConvertCaseValue) -> bool {
        self.view.convert.case == case
    }

    pub fn set_remove_accents(&mut self, on: bool) -> UiAction {
        self.view.convert.remove_accents = toggle(on);
        self.save()
    }

    pub fn set_notify_when_done(&mut self, on: bool) -> UiAction {
        self.view.convert.notify_when_done = toggle(on);
        self.save()
    }

    pub fn set_quick_hotkey(&mut self, hotkey: HotkeyView) -> UiAction {
        self.view.convert.quick_convert_hotkey = (!is_empty(hotkey)).then_some(hotkey);
        self.save()
    }

    pub fn set_from(&mut self, table: CodeTableValue) -> UiAction {
        self.view.convert.from = table;
        self.save()
    }

    pub fn set_to(&mut self, table: CodeTableValue) -> UiAction {
        self.view.convert.to = table;
        self.save()
    }

    /// `Đảo`: the source table becomes the target and the other way round.
    pub fn swap_tables(&mut self) -> UiAction {
        let convert = &mut self.view.convert;
        std::mem::swap(&mut convert.from, &mut convert.to);
        self.save()
    }

    /// `Chuyển mã`: convert the clipboard with the options now on screen.
    #[must_use]
    pub fn convert(&self) -> UiAction {
        UiAction::Convert(self.view.clone())
    }

    /// `Đóng`.
    #[must_use]
    pub const fn close() -> UiAction {
        UiAction::Close
    }
}

#[cfg(test)]
mod tests;
