use cuakey_application::{
    ClipboardError, ExportMacrosError, ImportMacrosError, InvalidMacro, LoadMacrosError, QuickConvertError,
    ResetPreferencesError, SaveMacrosError, SavePreferencesError, StoreError, UpdateError,
};

use super::{
    convert_error_message, error_message, export_macros_error_message, import_macros_error_message,
    load_macros_error_message, macro_error_message, reset_error_message, save_error_message, save_macros_error_message,
};

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
        error_message(UpdateError::SwitchEqualsSettings),
        "Hai phím tắt không được trùng nhau"
    );
}

#[test]
fn every_update_error_has_its_own_message() {
    let messages: Vec<&str> = ALL_UPDATE_ERRORS.into_iter().map(error_message).collect();
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
            error_message(error)
        );
        assert_eq!(
            reset_error_message(&ResetPreferencesError::Invalid(error)),
            error_message(error)
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

fn all_invalid_macros() -> [InvalidMacro; 5] {
    let shortcut = || "ab".to_owned();
    [
        InvalidMacro::EmptyShortcut,
        InvalidMacro::Whitespace(shortcut()),
        InvalidMacro::Colon(shortcut()),
        InvalidMacro::Semicolon(shortcut()),
        InvalidMacro::LineBreak(shortcut()),
    ]
}

#[test]
fn every_invalid_macro_has_its_own_message() {
    let messages: Vec<&str> = all_invalid_macros().iter().map(macro_error_message).collect();
    for (index, message) in messages.iter().enumerate() {
        assert_ne!(*message, "");
        assert!(!messages[..index].contains(message), "{message} repeats");
    }
}

#[test]
fn a_semicolon_and_a_line_break_are_worded() {
    assert_eq!(
        macro_error_message(&InvalidMacro::Semicolon("; x".into())),
        "Từ gõ tắt không được bắt đầu bằng dấu chấm phẩy"
    );
    assert_eq!(
        macro_error_message(&InvalidMacro::LineBreak("x".into())),
        "Nội dung đầy đủ không được xuống dòng"
    );
}

#[test]
fn an_invalid_macro_save_shows_the_macro_message() {
    for invalid in all_invalid_macros() {
        let expected = macro_error_message(&invalid);
        assert_eq!(save_macros_error_message(&SaveMacrosError::Invalid(invalid)), expected);
    }
}

#[test]
fn each_macro_storage_failure_has_its_own_message() {
    let messages = [
        load_macros_error_message(&LoadMacrosError::Storage(storage_error())),
        save_macros_error_message(&SaveMacrosError::Storage(storage_error())),
        import_macros_error_message(&ImportMacrosError::Storage(storage_error())),
        export_macros_error_message(&ExportMacrosError::Storage(storage_error())),
    ];
    for (index, message) in messages.iter().enumerate() {
        assert_ne!(*message, "");
        assert!(!messages[..index].contains(message), "{message} repeats");
    }
}

#[test]
fn each_quick_convert_failure_has_its_own_message() {
    let messages = [
        convert_error_message(&QuickConvertError::NoText),
        convert_error_message(&QuickConvertError::Clipboard(ClipboardError::Unavailable(
            "busy".into(),
        ))),
        convert_error_message(&QuickConvertError::Storage(storage_error())),
    ];
    for (index, message) in messages.iter().enumerate() {
        assert_ne!(*message, "");
        assert!(!messages[..index].contains(message), "{message} repeats");
    }
}
