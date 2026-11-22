//! The task that starts `CuaKey` at sign-in with the highest rights, through the Task Scheduler.

use std::path::Path;

use windows::Win32::Foundation::VARIANT_BOOL;
use windows::Win32::Security::Authentication::Identity::{GetUserNameExW, NameSamCompatible};
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
use windows::Win32::System::TaskScheduler::{
    IExecAction, ILogonTrigger, ITaskDefinition, ITaskFolder, ITaskService, TASK_ACTION_EXEC, TASK_CREATE_OR_UPDATE,
    TASK_INSTANCES_IGNORE_NEW, TASK_LOGON_INTERACTIVE_TOKEN, TASK_RUNLEVEL_HIGHEST, TASK_TRIGGER_LOGON, TaskScheduler,
};
use windows::Win32::System::Variant::VARIANT;
use windows::core::{BSTR, Interface, PWSTR};

use crate::com::ComScope;
use crate::startup::{ENTRY_NAME, Instances, RunLevel, Trigger, run_value, task_spec};

/// Registers the task for `exe`, replacing one of the same name.
pub(crate) fn create(exe: &Path) -> windows::core::Result<()> {
    let _com = ComScope::enter()?;
    register(exe)
}

/// Deletes the task. A missing task fails with `ERROR_FILE_NOT_FOUND`, which the caller treats as done.
pub(crate) fn delete() -> windows::core::Result<()> {
    let _com = ComScope::enter()?;
    let (_service, folder) = connect()?;
    // SAFETY: the name is a string that outlives the call, and no flags are set.
    unsafe { folder.DeleteTask(&BSTR::from(ENTRY_NAME), 0) }
}

// Every interface below is a local of a function that returns before the scope that initialised COM ends.

/// The service, connected to this machine as the current user, and its root folder.
fn connect() -> windows::core::Result<(ITaskService, ITaskFolder)> {
    // SAFETY: COM is initialised on this thread by the caller's scope, and the class and interface match.
    let service: ITaskService = unsafe { CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER) }?;
    let none = VARIANT::default();
    // SAFETY: the empty variants ask for the local machine and the current user, and outlive the call.
    unsafe { service.Connect(&none, &none, &none, &none) }?;
    // SAFETY: the name is a string that outlives the call.
    let folder = unsafe { service.GetFolder(&BSTR::from(r"\")) }?;
    Ok((service, folder))
}

fn register(exe: &Path) -> windows::core::Result<()> {
    let spec = task_spec();
    let (service, folder) = connect()?;
    // SAFETY: the flags argument is reserved and zero.
    let definition = unsafe { service.NewTask(0) }?;
    add_logon_trigger(&definition, spec.trigger)?;
    set_run_level(&definition, spec.run_level)?;
    add_exec_action(&definition, exe)?;
    apply_settings(&definition)?;
    let none = VARIANT::default();
    // SAFETY: the name and definition outlive the call; the empty variants mean the current user and no password, as
    // an interactive token needs.
    unsafe {
        folder.RegisterTaskDefinition(
            &BSTR::from(ENTRY_NAME),
            &definition,
            TASK_CREATE_OR_UPDATE.0,
            &none,
            &none,
            TASK_LOGON_INTERACTIVE_TOKEN,
            &none,
        )
    }?;
    Ok(())
}

fn add_logon_trigger(definition: &ITaskDefinition, trigger: Trigger) -> windows::core::Result<()> {
    let Trigger::AtLogon = trigger;
    // SAFETY: the call takes no arguments and returns the definition's collection.
    let triggers = unsafe { definition.Triggers() }?;
    // SAFETY: the type is a valid trigger type.
    let created = unsafe { triggers.Create(TASK_TRIGGER_LOGON) }?;
    let logon: ILogonTrigger = created.cast()?;
    // The trigger fires for the current user only, not for whoever signs in.
    // SAFETY: the name is a string that outlives the call.
    unsafe { logon.SetUserId(&current_user()?) }
}

/// The current user as `DOMAIN\name`.
fn current_user() -> windows::core::Result<BSTR> {
    let mut buffer = [0_u16; 256];
    let mut length = u32::try_from(buffer.len()).unwrap_or(0);
    let name = PWSTR(buffer.as_mut_ptr());
    // SAFETY: `name` points to `length` writable units, which the call fills and terminates, setting `length` to the
    // count without the terminator.
    let read = unsafe { GetUserNameExW(NameSamCompatible, Some(name), &raw mut length) };
    if !read {
        return Err(windows::core::Error::from_thread());
    }
    let written = buffer.get(..usize::try_from(length).unwrap_or(0)).unwrap_or_default();
    Ok(BSTR::from_wide(written))
}

fn set_run_level(definition: &ITaskDefinition, level: RunLevel) -> windows::core::Result<()> {
    let RunLevel::Highest = level;
    // SAFETY: the call takes no arguments and returns the definition's principal.
    let principal = unsafe { definition.Principal() }?;
    // SAFETY: the value is a valid run level.
    unsafe { principal.SetRunLevel(TASK_RUNLEVEL_HIGHEST) }?;
    // SAFETY: the value is a valid logon type.
    unsafe { principal.SetLogonType(TASK_LOGON_INTERACTIVE_TOKEN) }
}

fn add_exec_action(definition: &ITaskDefinition, exe: &Path) -> windows::core::Result<()> {
    // SAFETY: the call takes no arguments and returns the definition's collection.
    let actions = unsafe { definition.Actions() }?;
    // SAFETY: the type is a valid action type.
    let created = unsafe { actions.Create(TASK_ACTION_EXEC) }?;
    let exec: IExecAction = created.cast()?;
    // SAFETY: the string outlives the call, which copies it.
    unsafe { exec.SetPath(&BSTR::from(run_value(exe))) }
}

fn apply_settings(definition: &ITaskDefinition) -> windows::core::Result<()> {
    let spec = task_spec();
    // SAFETY: the call takes no arguments and returns the definition's settings.
    let settings = unsafe { definition.Settings() }?;
    // SAFETY: the value is a plain boolean.
    unsafe { settings.SetDisallowStartIfOnBatteries(VARIANT_BOOL::from(spec.battery.disallow_start)) }?;
    // SAFETY: the value is a plain boolean.
    unsafe { settings.SetStopIfGoingOnBatteries(VARIANT_BOOL::from(spec.battery.stop_when_leaving_mains)) }?;
    // SAFETY: the string outlives the call, which copies it.
    unsafe { settings.SetExecutionTimeLimit(&BSTR::from(spec.time_limit)) }?;
    // SAFETY: the value is within the priority range of 0 to 10.
    unsafe { settings.SetPriority(spec.priority) }?;
    let Instances::IgnoreNew = spec.instances;
    // SAFETY: the value is a valid policy.
    unsafe { settings.SetMultipleInstances(TASK_INSTANCES_IGNORE_NEW) }
}
