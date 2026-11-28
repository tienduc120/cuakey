use std::ffi::OsStr;
use std::path::Path;

use super::alias_path;

#[test]
fn the_alias_lives_in_the_users_windows_apps_folder() {
    let local = Path::new(r"C:\Users\someone\AppData\Local");

    let alias = alias_path(local);

    // Compared part by part, since CI also runs this on Linux, where a backslash is no separator.
    let below_local: Vec<&OsStr> = alias.strip_prefix(local).unwrap().iter().collect();
    assert_eq!(
        below_local,
        ["Microsoft", "WindowsApps", "cuakey.exe"],
        "Windows keeps every execution alias of the user in this one folder"
    );
}

#[cfg(windows)]
mod install {
    use windows::Win32::Foundation::{APPMODEL_ERROR_NO_PACKAGE, ERROR_INSUFFICIENT_BUFFER};

    use super::super::system::install_from;
    use super::super::{Install, program_path};

    #[test]
    fn a_process_without_a_package_is_loose() {
        assert_eq!(install_from(APPMODEL_ERROR_NO_PACKAGE), Install::Loose);
    }

    #[test]
    fn a_process_whose_package_name_needs_room_is_packaged() {
        assert_eq!(install_from(ERROR_INSUFFICIENT_BUFFER), Install::Packaged);
    }

    #[test]
    fn the_test_process_runs_loose() {
        assert_eq!(
            Install::current(),
            Install::Loose,
            "a test runs from the build folder, not a package"
        );
    }

    #[test]
    fn a_loose_program_starts_by_its_own_path() {
        let path = program_path().unwrap();

        assert_eq!(path, std::env::current_exe().unwrap());
    }
}
