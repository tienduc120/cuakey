use std::path::Path;

use super::temporary_path;

#[test]
fn each_process_writes_its_own_temporary_file() {
    let path = Path::new("CuaKey").join("settings.json");

    let background = temporary_path(&path, 1200);
    let windows = temporary_path(&path, 3400);

    assert_ne!(
        background, windows,
        "two processes saving at once never write into the same file"
    );
    assert_eq!(
        background.parent(),
        path.parent(),
        "beside the file it replaces, so the rename stays in one folder"
    );
}
