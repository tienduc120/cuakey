//! The startup task the package declares, which starts a Store install of `CuaKey` when the user signs in. Whether it
//! runs is the user's to decide in Task Manager, so the app can turn it on only while the user has not turned it off
//! there.

use windows::ApplicationModel::{StartupTask, StartupTaskState};
use windows::Win32::Foundation::E_UNEXPECTED;
use windows::core::HSTRING;

/// The startup task's id, as the package manifest declares it.
pub const PACKAGE_STARTUP_TASK_ID: &str = "CuaKey";

/// Asks Windows to turn the task on, and returns the state it is in afterwards.
pub(crate) fn enable() -> windows::core::Result<StartupTaskState> {
    in_free_thread(|| task()?.RequestEnableAsync()?.join())
}

/// Turns the task off.
pub(crate) fn disable() -> windows::core::Result<()> {
    in_free_thread(|| task()?.Disable())
}

fn task() -> windows::core::Result<StartupTask> {
    StartupTask::GetAsync(&HSTRING::from(PACKAGE_STARTUP_TASK_ID))?.join()
}

/// Runs `work` on a thread of its own, which joins the process's multithreaded apartment, since waiting on a Windows
/// Runtime operation from a single-threaded apartment's thread can wait forever.
fn in_free_thread<T: Send>(work: impl FnOnce() -> windows::core::Result<T> + Send) -> windows::core::Result<T> {
    std::thread::scope(|scope| scope.spawn(work).join()).unwrap_or_else(|_panic| Err(E_UNEXPECTED.into()))
}
