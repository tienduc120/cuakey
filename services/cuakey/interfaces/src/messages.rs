//! The text each refusal shows. This is the one place an outcome becomes a message on screen, and every `match` is
//! exhaustive, so a new outcome cannot reach the user unworded.

use cuakey_application::{ResetPreferencesError, SavePreferencesError, UpdateError};

const SETTINGS_NOT_SAVED: &str = "Không lưu được cài đặt";

#[must_use]
#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "callers hold the error by reference, as the save and reset errors are"
)]
pub const fn error_message(error: &UpdateError) -> &'static str {
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
        SavePreferencesError::Invalid(invalid) => error_message(invalid),
        SavePreferencesError::Storage(_) => SETTINGS_NOT_SAVED,
    }
}

#[must_use]
pub const fn reset_error_message(error: &ResetPreferencesError) -> &'static str {
    match error {
        ResetPreferencesError::Invalid(invalid) => error_message(invalid),
        ResetPreferencesError::Storage(_) => SETTINGS_NOT_SAVED,
    }
}

#[cfg(test)]
mod tests;
