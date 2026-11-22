use std::path::{Path, PathBuf};

use super::{FontError, read_first_font};

struct TestDir(PathBuf);

impl TestDir {
    fn new(test: &str) -> Self {
        let path = std::env::temp_dir().join(format!("cuakey-fonts-{}-{test}", std::process::id()));
        drop(std::fs::remove_dir_all(&path));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn put(&self, name: &str, bytes: &[u8]) {
        std::fs::write(self.0.join(name), bytes).unwrap();
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        // A leftover in the temp folder is not a failed test.
        drop(std::fs::remove_dir_all(&self.0));
    }
}

#[test]
fn segoe_ui_comes_first() {
    let dir = TestDir::new("segoe");
    dir.put("arial.ttf", b"arial");
    dir.put("segoeui.ttf", b"segoe");
    let (name, bytes) = read_first_font(dir.path()).unwrap();
    assert_eq!(name, "segoeui.ttf");
    assert_eq!(bytes, b"segoe");
}

#[test]
fn a_missing_font_falls_back_in_order() {
    let dir = TestDir::new("fallback");
    dir.put("arial.ttf", b"arial");
    assert_eq!(read_first_font(dir.path()).unwrap().0, "arial.ttf");
    dir.put("tahoma.ttf", b"tahoma");
    assert_eq!(read_first_font(dir.path()).unwrap().0, "tahoma.ttf");
}

#[test]
fn no_font_names_the_folder() {
    let dir = TestDir::new("none");
    let error = read_first_font(dir.path()).unwrap_err();
    let FontError::NoneLoaded { folder } = &error;
    assert_eq!(folder, dir.path());
    assert!(error.to_string().contains(&dir.path().display().to_string()));
}
