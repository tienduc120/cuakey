use cuakey_application::{ResetPreferencesError, SavePreferencesError, StoreError, UpdateError};

use super::{error_message, reset_error_message, save_error_message};

const ALL_UPDATE_ERRORS: [UpdateError; 6] = [
    UpdateError::EmptyProfileId,
    UpdateError::EmptyAppName,
    UpdateError::EmptyHotkey,
    UpdateError::InvalidHotkeyKey,
    UpdateError::SwitchEqualsSettings,
    UpdateError::QuickConvertClashes,
];

fn storage_error() -> StoreError {
    StoreError::Storage("disk full".into())
}

#[test]
fn equal_hotkeys_show_error() {
    assert_eq!(
        error_message(&UpdateError::SwitchEqualsSettings),
        "Hai phím tắt không được trùng nhau"
    );
}

#[test]
fn every_update_error_has_its_own_message() {
    let messages: Vec<&str> = ALL_UPDATE_ERRORS.iter().map(error_message).collect();
    for (index, message) in messages.iter().enumerate() {
        assert_ne!(*message, "");
        assert!(!messages[..index].contains(message), "{message} repeats");
    }
}

#[test]
fn an_invalid_save_or_reset_shows_the_update_message() {
    for error in ALL_UPDATE_ERRORS {
        assert_eq!(
            save_error_message(&SavePreferencesError::Invalid(error)),
            error_message(&error)
        );
        assert_eq!(
            reset_error_message(&ResetPreferencesError::Invalid(error)),
            error_message(&error)
        );
    }
}

#[test]
fn a_storage_failure_says_the_settings_were_not_saved() {
    let saved = save_error_message(&SavePreferencesError::Storage(storage_error()));
    let reset = reset_error_message(&ResetPreferencesError::Storage(storage_error()));
    assert_eq!(saved, "Không lưu được cài đặt");
    assert_eq!(reset, saved);
}
