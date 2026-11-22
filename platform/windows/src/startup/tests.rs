use std::path::Path;

use super::{BatteryRules, Instances, RunLevel, StartupMode, Step, Trigger, run_value, steps, task_spec};

#[test]
fn run_value_quotes_path() {
    let value = run_value(Path::new(r"C:\Program Files\CuaKey\cuakey.exe"));

    assert_eq!(
        value, r#""C:\Program Files\CuaKey\cuakey.exe""#,
        "a path with spaces is quoted"
    );
}

#[test]
fn task_settings_suit_a_long_running_typing_app() {
    let spec = task_spec();

    assert_eq!(spec.trigger, Trigger::AtLogon, "the task starts when the user signs in");
    assert_eq!(
        spec.run_level,
        RunLevel::Highest,
        "the task runs with the highest rights"
    );
    assert_eq!(
        spec.battery,
        BatteryRules {
            disallow_start: false,
            stop_when_leaving_mains: false,
        },
        "the task starts on battery and keeps running on it"
    );
    assert_eq!(spec.time_limit, "PT0S", "the task is never stopped for running long");
    assert_eq!(spec.priority, 4, "the task runs at normal priority");
    assert_eq!(spec.instances, Instances::IgnoreNew, "a second start is ignored");
}

#[test]
fn the_step_that_can_need_elevation_goes_first() {
    assert_eq!(
        steps(StartupMode::Off),
        [Step::DeleteTask, Step::DeleteRunValue],
        "off removes both, task first"
    );
    assert_eq!(
        steps(StartupMode::Normal),
        [Step::DeleteTask, Step::SetRunValue],
        "a normal start drops the task before it adds the registry value"
    );
    assert_eq!(
        steps(StartupMode::Elevated),
        [Step::CreateTask, Step::DeleteRunValue],
        "an elevated start creates the task before it drops the registry value"
    );
}

#[cfg(windows)]
mod task_errors {
    use windows::Win32::Foundation::{E_ACCESSDENIED, E_FAIL, ERROR_FILE_NOT_FOUND};

    use super::super::system::{StartupError, create_task_error, delete_task_error};

    #[test]
    fn denied_task_change_needs_elevation() {
        let denied = || windows::core::Error::from(E_ACCESSDENIED);

        assert!(matches!(create_task_error(denied()), StartupError::NeedsElevation));
        assert!(matches!(delete_task_error(denied()), Err(StartupError::NeedsElevation)));
    }

    #[test]
    fn deleting_a_missing_task_is_success() {
        let missing = windows::core::Error::from(ERROR_FILE_NOT_FOUND.to_hresult());

        assert!(
            delete_task_error(missing).is_ok(),
            "there is no task, which is the wanted state"
        );
    }

    #[test]
    fn other_task_failures_keep_their_cause() {
        let error = windows::core::Error::from(E_FAIL);

        assert!(matches!(delete_task_error(error), Err(StartupError::Task { .. })));
    }
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

#[cfg(windows)]
mod live_task {
    use std::path::Path;

    use super::super::system::delete_task_error;
    use super::{run_value, task_spec};
    use crate::task_scheduler::{self, Registered};

    /// Deletes the test task even when an assertion fails.
    struct Cleanup(String);

    impl Drop for Cleanup {
        fn drop(&mut self) {
            drop(task_scheduler::delete(&self.0));
        }
    }

    #[test]
    #[ignore = "registers a scheduled task; needs an elevated process"]
    fn startup_task_round_trip() {
        let name = format!("CuaKeyTest-{}", std::process::id());
        let _cleanup = Cleanup(name.clone());
        // A program that does not exist, so the task would run nothing if the cleanup ever failed.
        let exe = Path::new(r"C:\CuaKeyTest\missing.exe");
        let spec = task_spec();

        task_scheduler::create(&name, exe).unwrap();
        let registered = task_scheduler::read_back(&name).unwrap();
        task_scheduler::delete(&name).unwrap();
        let second_delete = task_scheduler::delete(&name).or_else(delete_task_error);

        assert_eq!(
            registered,
            Registered {
                disallow_start: spec.battery.disallow_start,
                stop_when_leaving_mains: spec.battery.stop_when_leaving_mains,
                time_limit: spec.time_limit.to_owned(),
                priority: spec.priority,
                run_level_highest: true,
                trigger_user: task_scheduler::expected_trigger_user().unwrap(),
                action_path: run_value(exe),
            },
            "the task reads back as it was registered"
        );
        assert!(second_delete.is_ok(), "deleting a task that is not there is success");
    }
}
