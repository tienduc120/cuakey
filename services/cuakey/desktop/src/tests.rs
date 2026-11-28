/// The manifest of the Microsoft Store package, which declares what the app asks Windows for by name.
const PACKAGE_MANIFEST: &str = include_str!("../package/AppxManifest.xml");

#[test]
fn the_package_starts_this_program() {
    assert!(
        PACKAGE_MANIFEST.contains(r#"Executable="cuakey.exe""#),
        "the package runs the executable this crate builds"
    );
}

#[cfg(windows)]
mod names_the_app_asks_for {
    use platform_windows::{EXECUTION_ALIAS, PACKAGE_STARTUP_TASK_ID};

    use super::PACKAGE_MANIFEST;

    #[test]
    fn the_package_declares_the_alias_the_app_starts_itself_by() {
        assert!(
            PACKAGE_MANIFEST.contains(&format!(r#"Alias="{EXECUTION_ALIAS}""#)),
            "the scheduled task, the desktop shortcut and the restart as administrator start the program by this alias"
        );
    }

    #[test]
    fn the_package_declares_the_startup_task_the_app_turns_on() {
        assert!(
            PACKAGE_MANIFEST.contains(&format!(r#"TaskId="{PACKAGE_STARTUP_TASK_ID}""#)),
            "Windows finds the startup task only by the id the manifest gives it"
        );
    }
}
