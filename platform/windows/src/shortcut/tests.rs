use std::path::PathBuf;

use super::{create_shortcut_at, remove_shortcut_at};

/// A shortcut path in the temp folder that is removed at the end of the test.
struct TempShortcut(PathBuf);

impl TempShortcut {
    fn new(tag: &str) -> Self {
        Self(std::env::temp_dir().join(format!("cuakey-test-{tag}-{}.lnk", std::process::id())))
    }
}

impl Drop for TempShortcut {
    fn drop(&mut self) {
        drop(std::fs::remove_file(&self.0));
    }
}

#[test]
fn shortcut_is_created_then_removed() {
    let shortcut = TempShortcut::new("create");
    let exe = std::env::current_exe().unwrap();

    create_shortcut_at(&shortcut.0, &exe).unwrap();
    let created = shortcut.0.is_file();
    remove_shortcut_at(&shortcut.0).unwrap();
    let removed = !shortcut.0.exists();

    assert!(created, "the shortcut file exists after creating it");
    assert!(removed, "the shortcut file is gone after removing it");
}

#[test]
fn removing_a_missing_shortcut_is_success() {
    let shortcut = TempShortcut::new("missing");

    assert!(
        remove_shortcut_at(&shortcut.0).is_ok(),
        "there is nothing to remove, which is the wanted state"
    );
}
