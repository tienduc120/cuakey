use super::{roaming_app_data, system_fonts};

#[test]
fn roaming_app_data_is_an_existing_folder() {
    let folder = roaming_app_data().unwrap();

    assert!(folder.is_absolute(), "the folder is a full path");
    assert!(folder.is_dir(), "the folder exists");
}

#[test]
fn system_fonts_is_the_folder_windows_keeps_its_fonts_in() {
    let folder = system_fonts().unwrap();

    assert!(folder.is_dir(), "the folder exists");
    assert!(folder.join("segoeui.ttf").is_file(), "it holds the system's own font");
}
