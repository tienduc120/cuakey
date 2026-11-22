use cuakey_application::{ImportMode, MacroStore as _};
use cuakey_domain::{MacroEntry, MacroTable};

use super::{HEADER, TextMacroStore, parse_macros};
use crate::test_dir::TestDir;

fn table(entries: &[(&str, &str)]) -> MacroTable {
    let mut table = MacroTable::default();
    for (shortcut, expansion) in entries {
        table.add(MacroEntry::new(shortcut, expansion).unwrap());
    }
    table
}

fn pairs(table: &MacroTable) -> Vec<(String, String)> {
    table
        .entries()
        .map(|entry| (entry.shortcut().to_owned(), entry.expansion().to_owned()))
        .collect()
}

fn expect(entries: &[(&str, &str)]) -> Vec<(String, String)> {
    entries
        .iter()
        .map(|(a, b)| ((*a).to_owned(), (*b).to_owned()))
        .collect()
}

#[tokio::test]
async fn macro_file_format() {
    let dir = TestDir::new("macro_format");
    let store = TextMacroStore::new(dir.path().to_path_buf());

    store
        .save(&table(&[("btw", "by the way"), ("tp", "thành phố")]))
        .await
        .unwrap();

    let text = std::fs::read(dir.path().join("macros.txt")).unwrap();
    let expected = format!("{HEADER}\nbtw:by the way\ntp:thành phố\n");
    assert_eq!(
        text,
        expected.as_bytes(),
        "header first, one shortcut:expansion per line, LF, no BOM"
    );
    assert_eq!(
        HEADER, ";Compatible OpenKey Macro Data file for UniKey*** version=1 ***",
        "the header other programs recognise"
    );
}

#[test]
fn macro_file_parsing() {
    let text = "\u{feff};Compatible OpenKey Macro Data file for UniKey*** version=1 ***\r\n\
                \r\n\
                ; a comment\r\n\
                btw:by the way\r\n\
                t:thời gian: 12:30\n\
                no separator here\n\
                :no shortcut\n\
                two words:bad shortcut\n\
                e:\n\
                btw:by the way, again\n";

    let parsed = parse_macros(text);

    assert_eq!(
        pairs(&parsed),
        expect(&[("btw", "by the way, again"), ("e", ""), ("t", "thời gian: 12:30")]),
        "the BOM and CRLF are accepted, only the first colon separates, bad lines are skipped, a later line wins"
    );
}

#[tokio::test]
async fn saved_macros_load_again() {
    let dir = TestDir::new("macro_round_trip");
    let store = TextMacroStore::new(dir.path().to_path_buf());
    let saved = table(&[("btw", "by the way"), ("t", "thời gian: 12:30")]);

    store.save(&saved).await.unwrap();

    assert_eq!(store.load().await.unwrap(), saved);
    assert_eq!(
        dir.file_names(),
        ["macros.txt"],
        "the temporary file is gone after the rename"
    );
}

#[tokio::test]
async fn no_file_is_an_empty_table() {
    let dir = TestDir::new("macro_missing");

    let loaded = TextMacroStore::new(dir.path().to_path_buf()).load().await.unwrap();

    assert_eq!(loaded, MacroTable::default());
}

#[tokio::test]
async fn a_line_break_in_an_expansion_does_not_split_its_line() {
    let dir = TestDir::new("macro_newline");
    let store = TextMacroStore::new(dir.path().to_path_buf());

    store.save(&table(&[("sig", "best\r\nregards")])).await.unwrap();

    assert_eq!(pairs(&store.load().await.unwrap()), expect(&[("sig", "best regards")]));
}

#[tokio::test]
async fn import_append_vs_replace() {
    let dir = TestDir::new("macro_import");
    let store = TextMacroStore::new(dir.path().join("config"));
    store
        .save(&table(&[("a", "stored a"), ("b", "stored b")]))
        .await
        .unwrap();
    let file = dir.path().join("other.txt");
    std::fs::write(&file, "b:file b\nc:file c\n").unwrap();

    let appended = store.import(&file, ImportMode::Append).await.unwrap();

    assert_eq!(
        pairs(&appended),
        expect(&[("a", "stored a"), ("b", "file b"), ("c", "file c")]),
        "append keeps what is stored, and the file's expansion wins for a shared shortcut"
    );
    assert_eq!(store.load().await.unwrap(), appended, "the import is stored");

    let replaced = store.import(&file, ImportMode::Replace).await.unwrap();

    assert_eq!(
        pairs(&replaced),
        expect(&[("b", "file b"), ("c", "file c")]),
        "replace keeps only the file's"
    );
    assert_eq!(store.load().await.unwrap(), replaced);
}

#[tokio::test]
async fn export_writes_the_stored_macros_to_the_path() {
    let dir = TestDir::new("macro_export");
    let store = TextMacroStore::new(dir.path().join("config"));
    store.save(&table(&[("btw", "by the way")])).await.unwrap();
    let target = dir.path().join("exported.txt");

    store.export(&target).await.unwrap();

    let text = std::fs::read_to_string(&target).unwrap();
    assert_eq!(text, format!("{HEADER}\nbtw:by the way\n"));
}

#[tokio::test]
async fn importing_a_file_that_is_not_there_is_a_storage_error() {
    let dir = TestDir::new("macro_import_missing");
    let store = TextMacroStore::new(dir.path().to_path_buf());

    let result = store.import(&dir.path().join("nothing.txt"), ImportMode::Append).await;

    assert!(result.is_err(), "the user asked for a file that does not exist");
}
