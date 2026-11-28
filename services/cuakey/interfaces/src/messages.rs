//! The text each refusal shows, in the interface language. This is the one place an outcome becomes a message on
//! screen, and every `match` is exhaustive, so a new outcome cannot reach the user unworded.

use cuakey_application::{
    ExportMacrosError, ImportMacrosError, InterfaceLanguageValue, InvalidMacro, LoadMacrosError, LoadPreferencesError,
    QuickConvertError, ResetPreferencesError, SaveMacrosError, SavePreferencesError, UpdateError,
};

use crate::labels::labels;

/// What the app tells the user about itself and the system, rather than about one setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notice {
    /// Quick convert put the converted text on the clipboard.
    ConvertDone,
    /// The stored preferences could not be read at start, so the app runs on the defaults and stores nothing until it
    /// starts again, rather than put the defaults in place of what the user set.
    DefaultsNotSaved,
    /// A window opened on preferences that could not be read, so it saves none of its changes.
    SettingsUnreadNotSaved,
    /// The macro window opened on macros that could not be read, so it saves none of its changes.
    MacrosUnreadNotSaved,
    /// The user refused the administrator prompt, so the app runs with normal rights.
    ElevationRefused,
    /// The app could not start itself again as administrator.
    ElevationFailed,
    /// Starting with Windows as administrator can only be changed by an app that runs as administrator.
    StartupNeedsAdministrator,
    /// The user turned `CuaKey` off among the startup apps in Task Manager, where only they can turn it on again.
    StartupOffInTaskManager,
    StartupNotChanged,
    ShortcutNotChanged,
    /// The app could not start at all.
    NotStarted,
    WindowNotOpened,
}

#[must_use]
pub const fn notice_message(notice: Notice, language: InterfaceLanguageValue) -> &'static str {
    let text = labels(language);
    match notice {
        Notice::ConvertDone => text.notice_convert_done,
        Notice::DefaultsNotSaved => text.notice_defaults_not_saved,
        Notice::SettingsUnreadNotSaved => text.notice_settings_unread_not_saved,
        Notice::MacrosUnreadNotSaved => text.notice_macros_unread_not_saved,
        Notice::ElevationRefused => text.notice_elevation_refused,
        Notice::ElevationFailed => text.notice_elevation_failed,
        Notice::StartupNeedsAdministrator => text.notice_startup_needs_administrator,
        Notice::StartupOffInTaskManager => text.notice_startup_off_in_task_manager,
        Notice::StartupNotChanged => text.notice_startup_not_changed,
        Notice::ShortcutNotChanged => text.notice_shortcut_not_changed,
        Notice::NotStarted => text.notice_not_started,
        Notice::WindowNotOpened => text.notice_window_not_opened,
    }
}

/// Why the stored preferences could not be read.
#[must_use]
pub const fn load_error_message(error: &LoadPreferencesError, language: InterfaceLanguageValue) -> &'static str {
    match error {
        LoadPreferencesError::Storage(_) => labels(language).settings_not_read,
    }
}

#[must_use]
pub const fn error_message(error: UpdateError, language: InterfaceLanguageValue) -> &'static str {
    let text = labels(language);
    match error {
        UpdateError::EmptyProfileId => text.empty_profile_id,
        UpdateError::EmptyAppName => text.empty_app_name,
        UpdateError::EmptyHotkey => text.empty_hotkey,
        UpdateError::InvalidHotkeyKey => text.invalid_hotkey_key,
        UpdateError::SwitchEqualsSettings => text.switch_equals_settings,
        UpdateError::QuickConvertClashes => text.quick_convert_clashes,
    }
}

#[must_use]
pub const fn save_error_message(error: &SavePreferencesError, language: InterfaceLanguageValue) -> &'static str {
    match error {
        SavePreferencesError::Invalid(invalid) => error_message(*invalid, language),
        SavePreferencesError::Storage(_) => labels(language).settings_not_saved,
    }
}

#[must_use]
pub const fn reset_error_message(error: &ResetPreferencesError, language: InterfaceLanguageValue) -> &'static str {
    match error {
        ResetPreferencesError::Invalid(invalid) => error_message(*invalid, language),
        ResetPreferencesError::Storage(_) => labels(language).settings_not_saved,
    }
}

/// Why a macro was refused. The row that caused it stays in the table, so the message need not repeat its shortcut.
#[must_use]
pub const fn macro_error_message(error: &InvalidMacro, language: InterfaceLanguageValue) -> &'static str {
    let text = labels(language);
    match error {
        InvalidMacro::EmptyShortcut => text.macro_empty_shortcut,
        InvalidMacro::Whitespace(_) => text.macro_whitespace,
        InvalidMacro::Colon(_) => text.macro_colon,
        InvalidMacro::Semicolon(_) => text.macro_semicolon,
        InvalidMacro::LineBreak(_) => text.macro_line_break,
    }
}

#[must_use]
pub const fn load_macros_error_message(error: &LoadMacrosError, language: InterfaceLanguageValue) -> &'static str {
    match error {
        LoadMacrosError::Storage(_) => labels(language).macros_not_read,
    }
}

#[must_use]
pub const fn save_macros_error_message(error: &SaveMacrosError, language: InterfaceLanguageValue) -> &'static str {
    match error {
        SaveMacrosError::Invalid(invalid) => macro_error_message(invalid, language),
        SaveMacrosError::Storage(_) => labels(language).macros_not_saved,
    }
}

#[must_use]
pub const fn import_macros_error_message(error: &ImportMacrosError, language: InterfaceLanguageValue) -> &'static str {
    match error {
        ImportMacrosError::Storage(_) => labels(language).macros_not_imported,
    }
}

#[must_use]
pub const fn export_macros_error_message(error: &ExportMacrosError, language: InterfaceLanguageValue) -> &'static str {
    match error {
        ExportMacrosError::Storage(_) => labels(language).macros_not_exported,
    }
}

#[must_use]
pub const fn convert_error_message(error: &QuickConvertError, language: InterfaceLanguageValue) -> &'static str {
    let text = labels(language);
    match error {
        QuickConvertError::NoText => text.convert_no_text,
        QuickConvertError::Clipboard(_) => text.clipboard_unavailable,
        QuickConvertError::Storage(_) => text.convert_options_not_read,
    }
}

#[cfg(test)]
mod tests;
