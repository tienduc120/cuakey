//! A directory of its own for a test, removed when the test ends.

use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(crate) struct TestDir {
    path: PathBuf,
}

impl TestDir {
    /// Names the directory after the process and the test, which a crate names once, so tests running side by side
    /// never share one; a leftover of an earlier run is cleared.
    pub(crate) fn new(test: &str) -> Self {
        let path = std::env::temp_dir().join(format!("cuakey-{}-{test}", std::process::id()));
        drop(std::fs::remove_dir_all(&path));
        std::fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    /// The names of the files in the directory, sorted.
    pub(crate) fn file_names(&self) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(&self.path)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        // A directory that cannot be removed is a leftover in the temp folder, not a failed test.
        drop(std::fs::remove_dir_all(&self.path));
    }
}
