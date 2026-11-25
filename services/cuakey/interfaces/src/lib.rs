//! What this service shows the user. Only this layer maps an outcome to a message on screen, with an exhaustive
//! `match` (ADR-0011, ADR-0014). Its state is plain data: a UI toolkit draws it, and a test reads it.

mod always_on_form;
mod always_on_window;
mod convert_form;
mod convert_window;
mod fonts;
mod hotkey_editor;
pub mod labels;
mod macro_form;
mod macro_window;
mod messages;
mod settings_form;
mod settings_window;
mod tray_menu;
mod ui_action;
mod widgets;

pub use always_on_form::AlwaysOnForm;
pub use always_on_window::AlwaysOnWindow;
pub use convert_form::ConvertForm;
pub use convert_window::ConvertWindow;
pub use fonts::{FontError, install_system_font};
pub use hotkey_editor::{key_from_egui, key_label, key_to_egui, press_key};
pub use macro_form::MacroForm;
pub use macro_window::MacroWindow;
pub use messages::{
    Notice, convert_error_message, error_message, export_macros_error_message, import_macros_error_message,
    load_error_message, load_macros_error_message, macro_error_message, notice_message, reset_error_message,
    save_error_message, save_macros_error_message,
};
pub use settings_form::{GatedControl, SettingsForm};
pub use settings_window::{COMPACT_SIZE, EXPANDED_SIZE, SettingsWindow};
pub use tray_menu::{MenuEffect, MenuEntry, MenuId, apply, model};
pub use ui_action::{Button, Platform, UiAction, WindowKind};
