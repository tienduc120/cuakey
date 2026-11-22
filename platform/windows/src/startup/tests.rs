use std::path::Path;

use super::{RunEntry, StartupMode, TaskEntry, plan, run_value, schtasks_create_args, schtasks_delete_args};

#[test]
fn run_value_quotes_path() {
    let value = run_value(Path::new(r"C:\Program Files\CuaKey\cuakey.exe"));

    assert_eq!(
        value, r#""C:\Program Files\CuaKey\cuakey.exe""#,
        "a path with spaces is quoted"
    );
}

#[test]
fn schtasks_args_exact() {
    let create = schtasks_create_args(Path::new(r"C:\Program Files\CuaKey\cuakey.exe"));
    let delete = schtasks_delete_args();

    assert_eq!(
        create,
        [
            "/create",
            "/sc",
            "onlogon",
            "/tn",
            "CuaKey",
            "/rl",
            "highest",
            "/tr",
            r#""C:\Program Files\CuaKey\cuakey.exe""#,
            "/f",
        ],
        "the task runs the quoted program at logon with the highest rights"
    );
    assert_eq!(
        delete,
        ["/delete", "/tn", "CuaKey", "/f"],
        "the task is deleted without asking"
    );
}

#[test]
fn exactly_one_way_to_start_is_active() {
    let off = plan(StartupMode::Off);
    let normal = plan(StartupMode::Normal);
    let elevated = plan(StartupMode::Elevated);

    assert_eq!(
        (off.run, off.task),
        (RunEntry::Remove, TaskEntry::Remove),
        "off removes both"
    );
    assert_eq!(
        (normal.run, normal.task),
        (RunEntry::Set, TaskEntry::Remove),
        "a normal start uses the registry value only"
    );
    assert_eq!(
        (elevated.run, elevated.task),
        (RunEntry::Remove, TaskEntry::Create),
        "an elevated start uses the scheduled task only"
    );
}

#[cfg(windows)]
mod registry {
    use std::path::Path;

    use super::super::system::{delete_run_value, read_run_value, set_run_value};
    use super::run_value;

    /// Removes the test value even when an assertion fails.
    struct Cleanup(String);

    impl Drop for Cleanup {
        fn drop(&mut self) {
            drop(delete_run_value(&self.0));
        }
    }

    #[test]
    #[ignore = "writes to the current user's registry"]
    fn startup_registry_round_trip() {
        let name = format!("CuaKeyTest-{}", std::process::id());
        let _cleanup = Cleanup(name.clone());
        let exe = Path::new(r"C:\Program Files\CuaKey\cuakey.exe");

        set_run_value(&name, exe).unwrap();
        let written = read_run_value(&name).unwrap();
        delete_run_value(&name).unwrap();
        let after_delete = read_run_value(&name).unwrap();
        let second_delete = delete_run_value(&name);

        assert_eq!(written, Some(run_value(exe)), "the value reads back as it was written");
        assert_eq!(after_delete, None, "the value is gone after the delete");
        assert!(second_delete.is_ok(), "deleting what is not there is success");
    }
}
