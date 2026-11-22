//! Starting `CuaKey` with Windows: a value under the user's `Run` key, or a scheduled task when it must run elevated.
//! Exactly one of the two is ever active.

use std::path::Path;

/// The name of the `Run` value and of the scheduled task.
const ENTRY_NAME: &str = "CuaKey";

/// How `CuaKey` starts when the user signs in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupMode {
    Off,
    /// With the user's normal rights.
    Normal,
    /// With the highest rights, through a scheduled task. Creating it needs an elevated process.
    Elevated,
}

/// What to do with the `Run` value.
#[cfg(any(windows, test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunEntry {
    Set,
    Remove,
}

/// What to do with the scheduled task.
#[cfg(any(windows, test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaskEntry {
    Create,
    Remove,
}

/// What a mode asks of the `Run` value and the task.
#[cfg(any(windows, test))]
#[derive(Debug, Clone, Copy)]
struct Plan {
    run: RunEntry,
    task: TaskEntry,
}

#[cfg(any(windows, test))]
const fn plan(mode: StartupMode) -> Plan {
    match mode {
        StartupMode::Off => Plan {
            run: RunEntry::Remove,
            task: TaskEntry::Remove,
        },
        StartupMode::Normal => Plan {
            run: RunEntry::Set,
            task: TaskEntry::Remove,
        },
        StartupMode::Elevated => Plan {
            run: RunEntry::Remove,
            task: TaskEntry::Create,
        },
    }
}

/// The program as the `Run` value and the task's command line name it: the path in quotes, so spaces do not split it.
#[must_use]
pub fn run_value(exe: &Path) -> String {
    format!("\"{}\"", exe.display())
}

/// The `schtasks` arguments that create the task running `exe` at logon with the highest rights.
#[must_use]
pub fn schtasks_create_args(exe: &Path) -> Vec<String> {
    ["/create", "/sc", "onlogon", "/tn", ENTRY_NAME, "/rl", "highest", "/tr"]
        .into_iter()
        .map(String::from)
        .chain([run_value(exe), "/f".to_owned()])
        .collect()
}

/// The `schtasks` arguments that delete the task without asking.
#[must_use]
pub fn schtasks_delete_args() -> Vec<String> {
    ["/delete", "/tn", ENTRY_NAME, "/f"].map(String::from).to_vec()
}

#[cfg(windows)]
pub use self::system::{StartupError, apply};

#[cfg(windows)]
mod system {
    use std::io;
    use std::os::windows::process::CommandExt;
    use std::path::Path;
    use std::process::Command;

    use windows::Win32::Foundation::ERROR_FILE_NOT_FOUND;
    use windows::Win32::System::Registry::{
        HKEY, HKEY_CURRENT_USER, KEY_SET_VALUE, REG_OPTION_NON_VOLATILE, REG_SZ, RegCloseKey, RegCreateKeyExW,
        RegDeleteValueW, RegOpenKeyExW, RegSetValueExW,
    };
    use windows::core::{HSTRING, PCWSTR, w};

    use super::{ENTRY_NAME, Plan, RunEntry, StartupMode, TaskEntry, plan, run_value};
    use crate::elevation::is_elevated;

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
        #[error("could not run schtasks")]
        Schtasks(#[source] io::Error),
        /// A task with the highest rights can only be created by an elevated process.
        #[error("starting with the highest rights needs CuaKey to run as administrator")]
        NeedsElevation,
        #[error("schtasks failed with exit code {code:?}")]
        TaskFailed { code: Option<i32> },
    }

    /// Makes `mode` the one way `CuaKey` starts with Windows. What is to be added goes in before what is to be removed,
    /// so a failure leaves `CuaKey` starting one way or the other.
    pub fn apply(mode: StartupMode) -> Result<(), StartupError> {
        let exe = std::env::current_exe().map_err(StartupError::CurrentExe)?;
        let Plan { run, task } = plan(mode);
        if task == TaskEntry::Create {
            create_task(&exe)?;
        }
        match run {
            RunEntry::Set => set_run_value(ENTRY_NAME, &exe)?,
            RunEntry::Remove => delete_run_value(ENTRY_NAME)?,
        }
        if task == TaskEntry::Remove {
            delete_task()?;
        }
        Ok(())
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

    /// The `CREATE_NO_WINDOW` flag, so running `schtasks` does not flash a console.
    const NO_WINDOW: u32 = 0x0800_0000;

    /// Runs `schtasks` with `args` and returns its exit code, or `None` when a signal ended it.
    fn schtasks(args: &[String]) -> Result<Option<i32>, StartupError> {
        let output = Command::new("schtasks.exe")
            .args(args)
            .creation_flags(NO_WINDOW)
            .output()
            .map_err(StartupError::Schtasks)?;
        Ok(output.status.code())
    }

    fn create_task(exe: &Path) -> Result<(), StartupError> {
        match schtasks(&super::schtasks_create_args(exe))? {
            Some(0) => Ok(()),
            code if !is_elevated() => {
                tracing::debug!(?code, "creating the startup task failed without elevation");
                Err(StartupError::NeedsElevation)
            }
            code => Err(StartupError::TaskFailed { code }),
        }
    }

    /// Deletes the task when there is one.
    fn delete_task() -> Result<(), StartupError> {
        let query = ["/query", "/tn", ENTRY_NAME].map(String::from);
        if schtasks(&query)? != Some(0) {
            return Ok(());
        }
        match schtasks(&super::schtasks_delete_args())? {
            Some(0) => Ok(()),
            code => Err(StartupError::TaskFailed { code }),
        }
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
