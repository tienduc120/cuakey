use std::path::PathBuf;

use cuakey_application::{InterfaceLanguageValue, LoadPreferencesError, PreferencesView, StoreError};

use super::{Origin, Stores};

#[test]
fn only_what_the_store_holds_may_be_saved_over_it() {
    assert!(Origin::Stored.may_save());
    assert!(
        !Origin::Defaults.may_save(),
        "defaults in place of an unread file would replace what the user set"
    );
}

#[test]
fn a_failed_load_gives_the_defaults_and_says_so() {
    let failed: Result<PreferencesView, _> = Err(LoadPreferencesError::Storage(StoreError::Storage(
        "the file is locked".into(),
    )));

    let (view, origin, error) = Origin::of(failed);

    assert_eq!(view, PreferencesView::default());
    assert_eq!(origin, Origin::Defaults);
    assert!(error.is_some(), "the caller tells the user why");
}

#[test]
fn a_load_gives_what_the_store_holds() {
    let stored = PreferencesView {
        always_on: vec!["game.exe".to_owned()],
        ..PreferencesView::default()
    };

    let (view, origin, error) = Origin::of(Ok::<_, LoadPreferencesError>(stored.clone()));

    assert_eq!((view, origin), (stored, Origin::Stored));
    assert!(error.is_none());
}

/// A folder of its own in the temp folder for one test, removed when the test ends, whether it passed or not.
struct TempFolder(PathBuf);

impl TempFolder {
    fn new(test: &str) -> Self {
        let path = std::env::temp_dir().join(format!("cuakey-stores-{}-{test}", std::process::id()));
        // What an earlier run left would be read as stored.
        drop(std::fs::remove_dir_all(&path));
        Self(path)
    }
}

impl Drop for TempFolder {
    fn drop(&mut self) {
        // A folder left behind is not a failed test.
        drop(std::fs::remove_dir_all(&self.0));
    }
}

#[test]
fn the_interface_language_is_read_from_the_store() {
    let folder = TempFolder::new("interface");
    let stores = Stores::in_folder(folder.0.clone()).unwrap();
    assert_eq!(
        stores.interface_language(),
        InterfaceLanguageValue::English,
        "nothing is stored yet, so the default"
    );

    let mut prefs = PreferencesView::default();
    prefs.system.interface_language = InterfaceLanguageValue::Vietnamese;
    stores.save_preferences(&prefs).unwrap();

    assert_eq!(stores.interface_language(), InterfaceLanguageValue::Vietnamese);
}
