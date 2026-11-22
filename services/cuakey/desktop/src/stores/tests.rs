use cuakey_application::{LoadPreferencesError, PreferencesView, StoreError};

use super::Origin;

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
