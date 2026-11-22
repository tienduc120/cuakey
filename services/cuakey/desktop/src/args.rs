//! The command line. With no arguments `cuakey` is the background process; `--ui <window>` is the window process,
//! which the background starts with the window to open.

use std::path::PathBuf;

use cuakey_interfaces::WindowKind;

const UI: &str = "--ui";
const FOCUS_LOG: &str = "--focus-log";

/// What this process is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Mode {
    /// The keyboard hook and the notification-area icon. With a path, the focus watch logs each classification there.
    Background { focus_log: Option<PathBuf> },
    /// The windows, opening this one first.
    Ui(WindowKind),
}

/// Why the command line was refused.
#[derive(Debug, thiserror::Error)]
pub(crate) enum ArgsError {
    #[error("unexpected argument {0}")]
    Unexpected(String),
    #[error("{0} needs a value")]
    MissingValue(String),
    #[error("no window is called {0}")]
    UnknownWindow(String),
}

/// The mode `args`, without the program's own name, ask for.
pub(crate) fn parse(args: &[String]) -> Result<Mode, ArgsError> {
    match args {
        [] => Ok(Mode::Background { focus_log: None }),
        [flag, value, rest @ ..] if flag == UI || flag == FOCUS_LOG => {
            if let Some(extra) = rest.first() {
                return Err(ArgsError::Unexpected(extra.clone()));
            }
            if flag == FOCUS_LOG {
                return Ok(Mode::Background {
                    focus_log: Some(PathBuf::from(value)),
                });
            }
            window_kind(value)
                .map(Mode::Ui)
                .ok_or_else(|| ArgsError::UnknownWindow(value.clone()))
        }
        [flag] if flag == UI || flag == FOCUS_LOG => Err(ArgsError::MissingValue(flag.clone())),
        [other, ..] => Err(ArgsError::Unexpected(other.clone())),
    }
}

/// The arguments that start the window process with `kind` open.
pub(crate) const fn window_args(kind: WindowKind) -> [&'static str; 2] {
    [UI, window_name(kind)]
}

/// The name `--ui` takes for `kind`.
pub(crate) const fn window_name(kind: WindowKind) -> &'static str {
    match kind {
        WindowKind::Settings => "settings",
        WindowKind::Macros => "macros",
        WindowKind::Convert => "convert",
        WindowKind::AlwaysOn => "always-on",
        WindowKind::About => "about",
    }
}

fn window_kind(name: &str) -> Option<WindowKind> {
    [
        WindowKind::Settings,
        WindowKind::Macros,
        WindowKind::Convert,
        WindowKind::AlwaysOn,
        WindowKind::About,
    ]
    .into_iter()
    .find(|kind| window_name(*kind) == name)
}

#[cfg(test)]
mod tests;
