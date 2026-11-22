use super::roaming_app_data;

#[test]
fn roaming_app_data_is_an_existing_folder() {
    let folder = roaming_app_data().unwrap();

    assert!(folder.is_absolute(), "the folder is a full path");
    assert!(folder.is_dir(), "the folder exists");
}
