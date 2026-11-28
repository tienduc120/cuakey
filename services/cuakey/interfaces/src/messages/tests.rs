use cuakey_application::{
    ClipboardError, ExportMacrosError, ImportMacrosError, InterfaceLanguageValue, InvalidMacro, LoadMacrosError,
    LoadPreferencesError, QuickConvertError, ResetPreferencesError, SaveMacrosError, SavePreferencesError, StoreError,
    UpdateError,
};

use super::{
    Notice, convert_error_message, error_message, export_macros_error_message, import_macros_error_message,
    load_error_message, load_macros_error_message, macro_error_message, notice_message, reset_error_message,
    save_error_message, save_macros_error_message,
};

const VIETNAMESE: InterfaceLanguageValue = InterfaceLanguageValue::Vietnamese;

const LANGUAGES: [InterfaceLanguageValue; 2] = [InterfaceLanguageValue::Vietnamese, InterfaceLanguageValue::English];

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

/// Fails unless every message is worded, and none says the same as another.
fn assert_all_differ(messages: &[&str]) {
    for (index, message) in messages.iter().enumerate() {
        assert_ne!(*message, "");
        assert!(!messages[..index].contains(message), "{message} repeats");
    }
}

#[test]
fn equal_hotkeys_show_error() {
    assert_eq!(
        error_message(UpdateError::SwitchEqualsSettings, VIETNAMESE),
        "Hai phím tắt không được trùng nhau"
    );
}

#[test]
fn every_update_error_has_its_own_message() {
    for language in LANGUAGES {
        let messages: Vec<&str> = ALL_UPDATE_ERRORS
            .into_iter()
            .map(|error| error_message(error, language))
            .collect();
        assert_all_differ(&messages);
    }
}

#[test]
fn an_invalid_save_or_reset_shows_the_update_message() {
    for language in LANGUAGES {
        for error in ALL_UPDATE_ERRORS {
            assert_eq!(
                save_error_message(&SavePreferencesError::Invalid(error), language),
                error_message(error, language)
            );
            assert_eq!(
                reset_error_message(&ResetPreferencesError::Invalid(error), language),
                error_message(error, language)
            );
        }
    }
}

#[test]
fn a_storage_failure_says_the_settings_were_not_saved() {
    let saved = save_error_message(&SavePreferencesError::Storage(storage_error()), VIETNAMESE);
    let reset = reset_error_message(&ResetPreferencesError::Storage(storage_error()), VIETNAMESE);
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
    for language in LANGUAGES {
        let messages: Vec<&str> = all_invalid_macros()
            .iter()
            .map(|invalid| macro_error_message(invalid, language))
            .collect();
        assert_all_differ(&messages);
    }
}

#[test]
fn a_semicolon_and_a_line_break_are_worded() {
    assert_eq!(
        macro_error_message(&InvalidMacro::Semicolon("; x".into()), VIETNAMESE),
        "Từ gõ tắt không được bắt đầu bằng dấu chấm phẩy"
    );
    assert_eq!(
        macro_error_message(&InvalidMacro::LineBreak("x".into()), VIETNAMESE),
        "Nội dung đầy đủ không được xuống dòng"
    );
}

#[test]
fn an_invalid_macro_save_shows_the_macro_message() {
    for language in LANGUAGES {
        for invalid in all_invalid_macros() {
            let expected = macro_error_message(&invalid, language);
            assert_eq!(
                save_macros_error_message(&SaveMacrosError::Invalid(invalid), language),
                expected
            );
        }
    }
}

#[test]
fn each_macro_storage_failure_has_its_own_message() {
    for language in LANGUAGES {
        assert_all_differ(&[
            load_macros_error_message(&LoadMacrosError::Storage(storage_error()), language),
            save_macros_error_message(&SaveMacrosError::Storage(storage_error()), language),
            import_macros_error_message(&ImportMacrosError::Storage(storage_error()), language),
            export_macros_error_message(&ExportMacrosError::Storage(storage_error()), language),
        ]);
    }
}

#[test]
fn each_quick_convert_failure_has_its_own_message() {
    for language in LANGUAGES {
        assert_all_differ(&[
            convert_error_message(&QuickConvertError::NoText, language),
            convert_error_message(
                &QuickConvertError::Clipboard(ClipboardError::Unavailable("busy".into())),
                language,
            ),
            convert_error_message(&QuickConvertError::Storage(storage_error()), language),
        ]);
    }
}

#[test]
fn a_failed_load_says_the_settings_were_not_read() {
    assert_eq!(
        load_error_message(&LoadPreferencesError::Storage(storage_error()), VIETNAMESE),
        "Không đọc được cài đặt"
    );
}

#[test]
fn running_on_defaults_says_nothing_is_saved_until_a_restart() {
    assert_eq!(
        notice_message(Notice::DefaultsNotSaved, VIETNAMESE),
        "Không đọc được cài đặt. CuaKey dùng cài đặt gốc và không lưu thay đổi cho đến khi được khởi động lại"
    );
}

#[test]
fn a_window_built_on_an_unread_file_says_its_changes_are_not_saved() {
    assert_eq!(
        notice_message(Notice::SettingsUnreadNotSaved, VIETNAMESE),
        "Không đọc được cài đặt nên thay đổi ở cửa sổ này không được lưu"
    );
    assert_eq!(
        notice_message(Notice::MacrosUnreadNotSaved, VIETNAMESE),
        "Không đọc được bảng gõ tắt nên thay đổi ở cửa sổ này không được lưu"
    );
}

const ALL_NOTICES: [Notice; 12] = [
    Notice::ConvertDone,
    Notice::DefaultsNotSaved,
    Notice::SettingsUnreadNotSaved,
    Notice::MacrosUnreadNotSaved,
    Notice::ElevationRefused,
    Notice::ElevationFailed,
    Notice::StartupNeedsAdministrator,
    Notice::StartupOffInTaskManager,
    Notice::StartupNotChanged,
    Notice::ShortcutNotChanged,
    Notice::NotStarted,
    Notice::WindowNotOpened,
];

#[test]
fn a_startup_the_user_turned_off_says_where_to_turn_it_on() {
    assert_eq!(
        notice_message(Notice::StartupOffInTaskManager, InterfaceLanguageValue::English),
        "Starting with Windows is turned off for CuaKey in Task Manager. Turn it on under Startup apps"
    );
    assert_eq!(
        notice_message(Notice::StartupOffInTaskManager, VIETNAMESE),
        "CuaKey đang bị tắt khởi động cùng Windows trong Task Manager. Hãy bật lại ở mục Ứng dụng khởi động \
         (Startup apps)"
    );
}

#[test]
fn a_finished_quick_convert_says_so() {
    assert_eq!(notice_message(Notice::ConvertDone, VIETNAMESE), "Chuyển mã thành công!");
}

#[test]
fn every_notice_has_its_own_message() {
    for language in LANGUAGES {
        let messages: Vec<&str> = ALL_NOTICES
            .into_iter()
            .map(|notice| notice_message(notice, language))
            .collect();
        assert_all_differ(&messages);
    }
}

#[test]
fn messages_follow_the_interface_language() {
    let error = UpdateError::SwitchEqualsSettings;

    let vietnamese = error_message(error, InterfaceLanguageValue::Vietnamese);
    let english = error_message(error, InterfaceLanguageValue::English);

    assert_eq!(vietnamese, "Hai phím tắt không được trùng nhau");
    assert_eq!(english, "The two hotkeys must differ");
}
