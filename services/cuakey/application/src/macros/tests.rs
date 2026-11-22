use std::path::{Path, PathBuf};

use super::{
    ExportMacros, ExportMacrosCommand, ImportMacros, ImportMacrosCommand, ImportMacrosError, LoadMacros,
    LoadMacrosError, SaveMacros, SaveMacrosError,
};
use crate::testing::{FakeMacroStore, Health};
use crate::{ImportMode, InvalidMacro, MacroView, StoreError};

const FILE: &str = r"C:\Users\nam\macros.txt";

fn entry(shortcut: &str, expansion: &str) -> MacroView {
    MacroView {
        shortcut: shortcut.to_owned(),
        expansion: expansion.to_owned(),
    }
}

/// A store holding two macros, next to a file that holds one new macro and one that changes `ko`.
fn store_with_file() -> FakeMacroStore {
    FakeMacroStore::holding(&[entry("ko", "không"), entry("vn", "Việt Nam")])
        .unwrap()
        .with_file(Path::new(FILE), &[entry("ko", "có không"), entry("hn", "Hà Nội")])
        .unwrap()
}

fn import(mode: ImportMode) -> ImportMacrosCommand {
    ImportMacrosCommand {
        path: PathBuf::from(FILE),
        mode,
    }
}

#[tokio::test]
async fn load_gives_the_stored_macros_in_shortcut_order() {
    let store = FakeMacroStore::holding(&[entry("vn", "Việt Nam"), entry("ko", "không")]).unwrap();

    let macros = LoadMacros::new(store).execute().await.unwrap();

    assert_eq!(macros, vec![entry("ko", "không"), entry("vn", "Việt Nam")]);
}

#[tokio::test]
async fn load_keeps_the_cause_of_a_storage_failure() {
    let err = LoadMacros::new(FakeMacroStore::new(Health::Broken))
        .execute()
        .await
        .unwrap_err();

    assert!(
        matches!(err, LoadMacrosError::Storage(StoreError::Storage(_))),
        "{err:?}"
    );
}

#[tokio::test]
async fn save_stores_the_macros() {
    let save = SaveMacros::new(FakeMacroStore::new(Health::Working));

    save.execute(&[entry("ko", "không")]).await.unwrap();

    assert_eq!(save.store().stored(), vec![entry("ko", "không")]);
}

#[tokio::test]
async fn save_refuses_a_shortcut_with_a_colon() {
    let save = SaveMacros::new(FakeMacroStore::holding(&[entry("ko", "không")]).unwrap());

    let err = save
        .execute(&[entry("vn", "Việt Nam"), entry("a:b", "x")])
        .await
        .unwrap_err();

    assert!(
        matches!(&err, SaveMacrosError::Invalid(InvalidMacro::Colon(s)) if s == "a:b"),
        "{err:?}"
    );
    assert_eq!(save.store().stored(), vec![entry("ko", "không")], "nothing saved");
}

#[tokio::test]
async fn save_keeps_the_cause_of_a_storage_failure() {
    let err = SaveMacros::new(FakeMacroStore::new(Health::Broken))
        .execute(&[entry("ko", "không")])
        .await
        .unwrap_err();

    assert!(
        matches!(err, SaveMacrosError::Storage(StoreError::Storage(_))),
        "{err:?}"
    );
}

#[tokio::test]
async fn import_appends_to_the_stored_macros() {
    let import_macros = ImportMacros::new(store_with_file());

    let macros = import_macros.execute(import(ImportMode::Append)).await.unwrap();

    let expected = vec![entry("hn", "Hà Nội"), entry("ko", "có không"), entry("vn", "Việt Nam")];
    assert_eq!(macros, expected, "kept, added, and the file's ko");
    assert_eq!(import_macros.store().stored(), expected, "stored");
}

#[tokio::test]
async fn import_replaces_the_stored_macros() {
    let import_macros = ImportMacros::new(store_with_file());

    let macros = import_macros.execute(import(ImportMode::Replace)).await.unwrap();

    assert_eq!(macros, vec![entry("hn", "Hà Nội"), entry("ko", "có không")]);
}

#[tokio::test]
async fn import_keeps_the_cause_of_a_storage_failure() {
    let err = ImportMacros::new(FakeMacroStore::new(Health::Working))
        .execute(import(ImportMode::Append))
        .await
        .unwrap_err();

    assert!(
        matches!(err, ImportMacrosError::Storage(StoreError::Storage(_))),
        "a missing file: {err:?}"
    );
}

#[tokio::test]
async fn export_writes_the_stored_macros() {
    let export = ExportMacros::new(FakeMacroStore::holding(&[entry("ko", "không")]).unwrap());

    export
        .execute(ExportMacrosCommand {
            path: PathBuf::from(FILE),
        })
        .await
        .unwrap();

    assert_eq!(export.store().file(Path::new(FILE)), Some(vec![entry("ko", "không")]));
}

#[tokio::test]
async fn save_refuses_what_a_macro_file_cannot_hold() {
    let save = SaveMacros::new(FakeMacroStore::new(Health::Working));

    let semicolon = save.execute(&[entry(";sig", "x")]).await.unwrap_err();
    let line_break = save.execute(&[entry("sig", "a\nb")]).await.unwrap_err();

    assert!(
        matches!(&semicolon, SaveMacrosError::Invalid(InvalidMacro::Semicolon(s)) if s == ";sig"),
        "{semicolon:?}"
    );
    assert!(
        matches!(&line_break, SaveMacrosError::Invalid(InvalidMacro::LineBreak(s)) if s == "sig"),
        "{line_break:?}"
    );
}
