//! Starting `CuaKey` with Windows: a value under the user's `Run` key, or a scheduled task when it must run elevated.
//! Exactly one of the two is ever active.

use std::path::Path;

/// The name of the `Run` value and of the scheduled task.
pub(crate) const ENTRY_NAME: &str = "CuaKey";

/// How `CuaKey` starts when the user signs in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupMode {
    Off,
    /// With the user's normal rights.
    Normal,
    /// With the highest rights, through a scheduled task. Changing the task needs an elevated process.
    Elevated,
}

/// One change to the ways `CuaKey` starts.
#[cfg(any(windows, test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    SetRunValue,
    DeleteRunValue,
    CreateTask,
    DeleteTask,
}

/// The two changes that make `mode` the one way `CuaKey` starts. The one that touches the task goes first, since it is
/// the one that can be refused for want of rights, and a refusal then leaves the previous state as it was.
#[cfg(any(windows, test))]
const fn steps(mode: StartupMode) -> [Step; 2] {
    match mode {
        StartupMode::Off => [Step::DeleteTask, Step::DeleteRunValue],
        StartupMode::Normal => [Step::DeleteTask, Step::SetRunValue],
        StartupMode::Elevated => [Step::CreateTask, Step::DeleteRunValue],
    }
}

/// When the task starts.
#[cfg(any(windows, test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Trigger {
    /// When the current user signs in.
    AtLogon,
}

/// The rights the task runs with.
#[cfg(any(windows, test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RunLevel {
    Highest,
}

/// What the task does when a second start comes while it runs.
#[cfg(any(windows, test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Instances {
    IgnoreNew,
}

/// How the task treats running on battery.
#[cfg(any(windows, test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BatteryRules {
    /// Do not start while on battery.
    pub(crate) disallow_start: bool,
    /// Stop when the computer leaves mains power.
    pub(crate) stop_when_leaving_mains: bool,
}

/// What the scheduled task is registered with.
#[cfg(any(windows, test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TaskSpec {
    pub(crate) trigger: Trigger,
    pub(crate) run_level: RunLevel,
    pub(crate) battery: BatteryRules,
    /// An ISO 8601 duration; `PT0S` means no limit.
    pub(crate) time_limit: &'static str,
    /// The process priority class level, where 4 is normal.
    pub(crate) priority: i32,
    pub(crate) instances: Instances,
}

/// The task suits a typing app that runs all session: it starts and keeps running on battery, has no time limit, and
/// runs at normal priority.
#[cfg(any(windows, test))]
pub(crate) const fn task_spec() -> TaskSpec {
    TaskSpec {
        trigger: Trigger::AtLogon,
        run_level: RunLevel::Highest,
        battery: BatteryRules {
            disallow_start: false,
            stop_when_leaving_mains: false,
        },
        time_limit: "PT0S",
        priority: 4,
        instances: Instances::IgnoreNew,
    }
}

/// The program as the `Run` value and the task action name it: the path in quotes, so spaces do not split it.
#[must_use]
pub fn run_value(exe: &Path) -> String {
    format!("\"{}\"", exe.display())
}

#[cfg(windows)]
pub use self::system::{StartupError, apply};

#[cfg(windows)]
mod system {
    use std::io;
    use std::path::Path;

    use windows::Win32::Foundation::{E_ACCESSDENIED, ERROR_FILE_NOT_FOUND};
    use windows::Win32::System::Registry::{
        HKEY, HKEY_CURRENT_USER, KEY_SET_VALUE, REG_OPTION_NON_VOLATILE, REG_SZ, RegCloseKey, RegCreateKeyExW,
        RegDeleteValueW, RegOpenKeyExW, RegSetValueExW,
    };
    use windows::core::{HSTRING, PCWSTR, w};

    use super::{ENTRY_NAME, StartupMode, Step, run_value, steps};
    use crate::task_scheduler;

    /// Why the way `CuaKey` starts could not be changed.
    #[derive(Debug, thiserror::Error)]
    pub enum StartupError {
        #[error("could not find the program's own path")]
        CurrentExe(#[source] io::Error),
        #[error("could not {action} the startup entry in the registry")]
        Registry {
            action: &'static str,
            #[source]
            source: windows::core::Error,
        },
        #[error("could not {action} the startup task")]
        Task {
            action: &'static str,
            #[source]
            source: windows::core::Error,
        },
        /// The startup task can only be changed by an elevated process.
        #[error("changing the startup task needs `CuaKey` to run as administrator")]
        NeedsElevation,
    }

    /// Makes `mode` the one way `CuaKey` starts with Windows.
    pub fn apply(mode: StartupMode) -> Result<(), StartupError> {
        let exe = std::env::current_exe().map_err(StartupError::CurrentExe)?;
        for step in steps(mode) {
            match step {
                Step::SetRunValue => set_run_value(ENTRY_NAME, &exe)?,
                Step::DeleteRunValue => delete_run_value(ENTRY_NAME)?,
                Step::CreateTask => task_scheduler::create(&exe).map_err(create_task_error)?,
                Step::DeleteTask => task_scheduler::delete().or_else(delete_task_error)?,
            }
        }
        Ok(())
    }

    /// What a failure to create the task means: a refusal for want of rights, or a failure.
    pub(super) fn create_task_error(source: windows::core::Error) -> StartupError {
        if source.code() == E_ACCESSDENIED {
            StartupError::NeedsElevation
        } else {
            StartupError::Task {
                action: "create",
                source,
            }
        }
    }

    /// What a failure to delete the task means: no task is already as wanted.
    pub(super) fn delete_task_error(source: windows::core::Error) -> Result<(), StartupError> {
        if source.code() == ERROR_FILE_NOT_FOUND.to_hresult() {
            Ok(())
        } else if source.code() == E_ACCESSDENIED {
            Err(StartupError::NeedsElevation)
        } else {
            Err(StartupError::Task {
                action: "delete",
                source,
            })
        }
    }

    const RUN_KEY: PCWSTR = w!(r"Software\Microsoft\Windows\CurrentVersion\Run");

    /// An open registry key, closed on drop.
    struct Key(HKEY);

    impl Key {
        /// The `Run` key, created when it is missing.
        fn create_run() -> Result<Self, StartupError> {
            let mut key = HKEY::default();
            // SAFETY: the key name is a terminated string that outlives the call, `key` is a local the call fills, and
            // the optional arguments are null or plain flags.
            let result = unsafe {
                RegCreateKeyExW(
                    HKEY_CURRENT_USER,
                    RUN_KEY,
                    None,
                    PCWSTR::null(),
                    REG_OPTION_NON_VOLATILE,
                    KEY_SET_VALUE,
                    None,
                    &raw mut key,
                    None,
                )
            };
            result.ok().map_err(|source| registry_error("open", source))?;
            Ok(Self(key))
        }

        /// The `Run` key, or `None` when it does not exist.
        fn open_run() -> Result<Option<Self>, StartupError> {
            let mut key = HKEY::default();
            // SAFETY: the key name is a terminated string that outlives the call, and `key` is a local the call fills.
            let result = unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, RUN_KEY, None, KEY_SET_VALUE, &raw mut key) };
            if result == ERROR_FILE_NOT_FOUND {
                return Ok(None);
            }
            result.ok().map_err(|source| registry_error("open", source))?;
            Ok(Some(Self(key)))
        }
    }

    impl Drop for Key {
        fn drop(&mut self) {
            // SAFETY: the handle came from a successful open and is closed once, here.
            let closed = unsafe { RegCloseKey(self.0) };
            if let Err(error) = closed.ok() {
                tracing::warn!(error = ?error, "could not close a registry key");
            }
        }
    }

    fn registry_error(action: &'static str, source: windows::core::Error) -> StartupError {
        StartupError::Registry { action, source }
    }

    /// Writes the `Run` value `name`, pointing at `exe`.
    pub(super) fn set_run_value(name: &str, exe: &Path) -> Result<(), StartupError> {
        let key = Key::create_run()?;
        let data: Vec<u8> = run_value(exe)
            .encode_utf16()
            .chain(std::iter::once(0))
            .flat_map(u16::to_le_bytes)
            .collect();
        // SAFETY: the key is open for writing, the name is a terminated string, and the data is a terminated UTF-16
        // string as `REG_SZ` requires; both outlive the call.
        let result = unsafe { RegSetValueExW(key.0, &HSTRING::from(name), None, REG_SZ, Some(&data)) };
        result.ok().map_err(|source| registry_error("write", source))
    }

    /// Deletes the `Run` value `name`; a value that is not there is already as wanted.
    pub(super) fn delete_run_value(name: &str) -> Result<(), StartupError> {
        let Some(key) = Key::open_run()? else {
            return Ok(());
        };
        // SAFETY: the key is open for writing and the name is a terminated string that outlives the call.
        let result = unsafe { RegDeleteValueW(key.0, &HSTRING::from(name)) };
        if result == ERROR_FILE_NOT_FOUND {
            return Ok(());
        }
        result.ok().map_err(|source| registry_error("delete", source))
    }

    /// The text of the `Run` value `name`, or `None` when it is not there.
    #[cfg(test)]
    pub(super) fn read_run_value(name: &str) -> Result<Option<String>, StartupError> {
        use windows::Win32::System::Registry::{KEY_QUERY_VALUE, RRF_RT_REG_SZ, RegGetValueW};

        let mut key = HKEY::default();
        // SAFETY: the key name is a terminated string that outlives the call, and `key` is a local the call fills.
        let opened = unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, RUN_KEY, None, KEY_QUERY_VALUE, &raw mut key) };
        opened.ok().map_err(|source| registry_error("open", source))?;
        let key = Key(key);
        let mut buffer = [0_u16; 1024];
        let mut bytes = u32::try_from(size_of_val(&buffer)).unwrap_or(0);
        // SAFETY: the key is open for reading, the name is a terminated string, and the buffer is `bytes` long.
        let result = unsafe {
            RegGetValueW(
                key.0,
                PCWSTR::null(),
                &HSTRING::from(name),
                RRF_RT_REG_SZ,
                None,
                Some(buffer.as_mut_ptr().cast()),
                Some(&raw mut bytes),
            )
        };
        if result == ERROR_FILE_NOT_FOUND {
            return Ok(None);
        }
        result.ok().map_err(|source| registry_error("read", source))?;
        let units = usize::try_from(bytes).unwrap_or(0) / 2;
        let text = buffer.get(..units).unwrap_or_default();
        Ok(Some(String::from_utf16_lossy(text).trim_end_matches('\0').to_owned()))
    }
}

#[cfg(test)]
mod tests;
