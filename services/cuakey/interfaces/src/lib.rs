//! What this service shows the user. Only this layer maps an outcome to a message on screen, with an exhaustive
//! `match` (ADR-0011, ADR-0014). Its state is plain data: a UI toolkit draws it, and a test reads it.

mod fonts;
mod hotkey_editor;
pub mod labels;
mod messages;
mod settings_form;
mod settings_window;
mod ui_action;

pub use fonts::{FontError, install_system_font};
pub use hotkey_editor::{key_from_egui, key_label, key_to_egui, press_key};
pub use messages::{error_message, reset_error_message, save_error_message};
pub use settings_form::{GatedControl, SettingsForm};
pub use settings_window::SettingsWindow;
pub use ui_action::{Button, Platform, UiAction, WindowKind};
