use super::{QuickConvert, QuickConvertError, QuickConvertResult};
use crate::testing::{FakeClipboard, FakePreferencesStore, Health};
use crate::{ClipboardError, CodeTableValue, ConvertCaseValue, PreferencesView, StoreError, Toggle};

/// Preferences whose saved conversion removes accents and writes capitals, and asks for no notification.
fn upper_without_accents() -> PreferencesView {
    let mut prefs = PreferencesView::default();
    prefs.convert.from = CodeTableValue::Unicode;
    prefs.convert.to = CodeTableValue::Unicode;
    prefs.convert.case = ConvertCaseValue::Upper;
    prefs.convert.remove_accents = Toggle::On;
    prefs.convert.notify_when_done = Toggle::Off;
    prefs
}

#[tokio::test]
async fn quick_convert_round_trip() {
    let store = FakePreferencesStore::holding(&upper_without_accents()).unwrap();
    let quick_convert = QuickConvert::new(FakeClipboard::holding("Tiếng Việt"), store);

    let result = quick_convert.execute().await.unwrap();

    assert_eq!(
        quick_convert.clipboard().text(),
        Some("TIENG VIET".to_owned()),
        "converted with the saved options and written back"
    );
    assert_eq!(result, QuickConvertResult { notify: false }, "as the preferences ask");
}

#[tokio::test]
async fn the_defaults_apply_when_nothing_was_saved() {
    let quick_convert = QuickConvert::new(
        FakeClipboard::holding("Tiếng Việt"),
        FakePreferencesStore::new(Health::Working),
    );

    let result = quick_convert.execute().await.unwrap();

    assert_eq!(
        quick_convert.clipboard().text(),
        Some("Tiếng Việt".to_owned()),
        "unicode to unicode"
    );
    assert_eq!(result, QuickConvertResult { notify: true }, "notifies by default");
}

#[tokio::test]
async fn a_clipboard_with_no_text_is_refused() {
    let quick_convert = QuickConvert::new(
        FakeClipboard::new(Health::Working),
        FakePreferencesStore::new(Health::Working),
    );

    let err = quick_convert.execute().await.unwrap_err();

    assert!(matches!(err, QuickConvertError::NoText), "{err:?}");
}

#[tokio::test]
async fn an_empty_clipboard_is_refused() {
    let quick_convert = QuickConvert::new(FakeClipboard::holding(""), FakePreferencesStore::new(Health::Working));

    let err = quick_convert.execute().await.unwrap_err();

    assert!(matches!(err, QuickConvertError::NoText), "{err:?}");
    assert_eq!(quick_convert.clipboard().text(), Some(String::new()), "nothing written");
}

#[tokio::test]
async fn a_clipboard_failure_keeps_its_cause() {
    let quick_convert = QuickConvert::new(
        FakeClipboard::new(Health::Broken),
        FakePreferencesStore::new(Health::Working),
    );

    let err = quick_convert.execute().await.unwrap_err();

    assert!(
        matches!(err, QuickConvertError::Clipboard(ClipboardError::Unavailable(_))),
        "{err:?}"
    );
}

#[tokio::test]
async fn a_storage_failure_keeps_its_cause() {
    let quick_convert = QuickConvert::new(
        FakeClipboard::holding("Tiếng Việt"),
        FakePreferencesStore::new(Health::Broken),
    );

    let err = quick_convert.execute().await.unwrap_err();

    assert!(
        matches!(err, QuickConvertError::Storage(StoreError::Storage(_))),
        "{err:?}"
    );
}
