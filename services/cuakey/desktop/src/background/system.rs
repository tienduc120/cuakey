//! What the system settings ask of Windows: how `CuaKey` starts with it, and the shortcut on the desktop.

use cuakey_application::{SystemPrefs, Toggle};
use cuakey_interfaces::Notice;
use platform_windows::{StartupError, StartupMode};

/// What a change of the system settings must do in Windows; `None` leaves that part as it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct SystemChanges {
    pub(crate) startup: Option<StartupMode>,
    pub(crate) shortcut: Option<Toggle>,
}

/// How `CuaKey` starts with Windows for `system`.
pub(crate) const fn startup_mode(system: &SystemPrefs) -> StartupMode {
    match (system.start_with_windows, system.run_as_administrator) {
        (Toggle::Off, _) => StartupMode::Off,
        (Toggle::On, Toggle::Off) => StartupMode::Normal,
        (Toggle::On, Toggle::On) => StartupMode::Elevated,
    }
}

/// Why the startup mode is being applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Occasion {
    /// The app started, and keeps Windows in step with the settings.
    Start,
    /// The user changed the settings.
    Change,
}

/// The notice for a startup mode that could not be applied on `occasion`, if any. The user's own choice in Task Manager
/// goes unremarked at start, since the app starts every session; it is pointed out only when the user asks for the
/// opposite.
pub(crate) const fn startup_notice(error: &StartupError, occasion: Occasion) -> Option<Notice> {
    match error {
        StartupError::NeedsElevation => Some(Notice::StartupNeedsAdministrator),
        StartupError::DisabledByUser => match occasion {
            Occasion::Start => None,
            Occasion::Change => Some(Notice::StartupOffInTaskManager),
        },
        StartupError::ProgramPath(_)
        | StartupError::Registry { .. }
        | StartupError::Task { .. }
        | StartupError::PackageStartup { .. }
        | StartupError::PackageStartupOff(_) => Some(Notice::StartupNotChanged),
    }
}

/// What going from `before` to `after` changes in Windows.
pub(crate) fn system_changes(before: &SystemPrefs, after: &SystemPrefs) -> SystemChanges {
    let starts_differently = before.start_with_windows != after.start_with_windows
        || before.run_as_administrator != after.run_as_administrator;
    SystemChanges {
        startup: starts_differently.then(|| startup_mode(after)),
        shortcut: (before.desktop_shortcut != after.desktop_shortcut).then_some(after.desktop_shortcut),
    }
}

#[cfg(test)]
mod tests;
