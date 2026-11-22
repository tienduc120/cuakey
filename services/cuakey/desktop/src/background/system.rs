//! What the system settings ask of Windows: how `CuaKey` starts with it, and the shortcut on the desktop.

use cuakey_application::{SystemPrefs, Toggle};
use platform_windows::StartupMode;

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
