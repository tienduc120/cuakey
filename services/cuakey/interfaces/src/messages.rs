//! The text each refusal shows. This is the one place an outcome becomes a message on screen, and every `match` is
//! exhaustive, so a new outcome cannot reach the user unworded.

use cuakey_application::{
    ExportMacrosError, ImportMacrosError, InvalidMacro, LoadMacrosError, LoadPreferencesError, QuickConvertError,
    ResetPreferencesError, SaveMacrosError, SavePreferencesError, UpdateError,
};

const SETTINGS_NOT_SAVED: &str = "Không lưu được cài đặt";

/// What the app tells the user about itself and the system, rather than about one setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notice {
    /// Quick convert put the converted text on the clipboard.
    ConvertDone,
    /// The user refused the administrator prompt, so the app runs with normal rights.
    ElevationRefused,
    /// The app could not start itself again as administrator.
    ElevationFailed,
    /// Starting with Windows as administrator can only be changed by an app that runs as administrator.
    StartupNeedsAdministrator,
    StartupNotChanged,
    ShortcutNotChanged,
    /// The app could not start at all.
    NotStarted,
    WindowNotOpened,
}

#[must_use]
pub const fn notice_message(notice: Notice) -> &'static str {
    match notice {
        Notice::ConvertDone => "Chuyển mã thành công!",
        Notice::ElevationRefused => "Không được cấp quyền Admin, CuaKey chạy với quyền thường",
        Notice::ElevationFailed => "Không khởi động lại được CuaKey với quyền Admin",
        Notice::StartupNeedsAdministrator => "Cần chạy CuaKey với quyền Admin để đổi cách khởi động cùng Windows",
        Notice::StartupNotChanged => "Không đổi được cài đặt khởi động cùng Windows",
        Notice::ShortcutNotChanged => "Không đổi được biểu tượng trên Desktop",
        Notice::NotStarted => "Không khởi động được CuaKey",
        Notice::WindowNotOpened => "Không mở được cửa sổ CuaKey",
    }
}

/// Why the stored preferences could not be read; the app then carries on with the defaults.
#[must_use]
pub const fn load_error_message(error: &LoadPreferencesError) -> &'static str {
    match error {
        LoadPreferencesError::Storage(_) => "Không đọc được cài đặt, CuaKey dùng cài đặt gốc",
    }
}

#[must_use]
pub const fn error_message(error: UpdateError) -> &'static str {
    match error {
        UpdateError::EmptyProfileId => "Tên hồ sơ không được để trống",
        UpdateError::EmptyAppName => "Tên ứng dụng không được để trống",
        UpdateError::EmptyHotkey => "Phím tắt cần có phím bổ trợ hoặc một phím",
        UpdateError::InvalidHotkeyKey => "Phím tắt không thể kết thúc bằng phím này",
        UpdateError::SwitchEqualsSettings => "Hai phím tắt không được trùng nhau",
        UpdateError::QuickConvertClashes => "Phím chuyển mã nhanh không được trùng với phím tắt khác",
    }
}

#[must_use]
pub const fn save_error_message(error: &SavePreferencesError) -> &'static str {
    match error {
        SavePreferencesError::Invalid(invalid) => error_message(*invalid),
        SavePreferencesError::Storage(_) => SETTINGS_NOT_SAVED,
    }
}

#[must_use]
pub const fn reset_error_message(error: &ResetPreferencesError) -> &'static str {
    match error {
        ResetPreferencesError::Invalid(invalid) => error_message(*invalid),
        ResetPreferencesError::Storage(_) => SETTINGS_NOT_SAVED,
    }
}

/// Why a macro was refused. The row that caused it stays in the table, so the message need not repeat its shortcut.
#[must_use]
pub const fn macro_error_message(error: &InvalidMacro) -> &'static str {
    match error {
        InvalidMacro::EmptyShortcut => "Từ gõ tắt không được để trống",
        InvalidMacro::Whitespace(_) => "Từ gõ tắt không được chứa khoảng trắng",
        InvalidMacro::Colon(_) => "Từ gõ tắt không được chứa dấu hai chấm",
        InvalidMacro::Semicolon(_) => "Từ gõ tắt không được bắt đầu bằng dấu chấm phẩy",
        InvalidMacro::LineBreak(_) => "Nội dung đầy đủ không được xuống dòng",
    }
}

#[must_use]
pub const fn load_macros_error_message(error: &LoadMacrosError) -> &'static str {
    match error {
        LoadMacrosError::Storage(_) => "Không đọc được bảng gõ tắt",
    }
}

#[must_use]
pub const fn save_macros_error_message(error: &SaveMacrosError) -> &'static str {
    match error {
        SaveMacrosError::Invalid(invalid) => macro_error_message(invalid),
        SaveMacrosError::Storage(_) => "Không lưu được bảng gõ tắt",
    }
}

#[must_use]
pub const fn import_macros_error_message(error: &ImportMacrosError) -> &'static str {
    match error {
        ImportMacrosError::Storage(_) => "Không nạp được bảng gõ tắt từ file",
    }
}

#[must_use]
pub const fn export_macros_error_message(error: &ExportMacrosError) -> &'static str {
    match error {
        ExportMacrosError::Storage(_) => "Không xuất được bảng gõ tắt ra file",
    }
}

#[must_use]
pub const fn convert_error_message(error: &QuickConvertError) -> &'static str {
    match error {
        QuickConvertError::NoText => "Bộ nhớ tạm không có văn bản để chuyển mã",
        QuickConvertError::Clipboard(_) => "Không dùng được bộ nhớ tạm",
        QuickConvertError::Storage(_) => "Không đọc được tùy chọn chuyển mã",
    }
}

#[cfg(test)]
mod tests;
