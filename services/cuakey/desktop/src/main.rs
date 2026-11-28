//! Starts the app. Process-wide things happen here and nowhere else. This is the composition root: the one place that
//! knows which adapter backs each port (ADR-0014).
//!
//! One executable plays two parts. Started with no arguments it is the background process, which holds the keyboard
//! hook and the notification-area icon and loads no UI toolkit; started with `--ui <window>` it is the window process,
//! which the background starts to show a window and which ends when its last window closes.
#![windows_subsystem = "windows"]

// The command line and the key names are plain code, built and tested on every platform; the rest is Windows.
#[cfg(any(windows, test))]
mod args;
#[cfg(windows)]
mod background;
#[cfg(any(windows, test))]
mod keymap;
#[cfg(windows)]
mod processes;
#[cfg(windows)]
mod stores;
#[cfg(windows)]
mod ui_process;

use std::process::ExitCode;

#[cfg(windows)]
fn main() -> ExitCode {
    // A command line the app wrote itself is always Unicode; any other is refused with the rest.
    let Ok(args) = std::env::args_os()
        .skip(1)
        .map(std::ffi::OsString::into_string)
        .collect::<Result<Vec<String>, _>>()
    else {
        return ExitCode::FAILURE;
    };
    match args::parse(&args) {
        Ok(args::Mode::Background { focus_log }) => background::run(focus_log.as_deref(), &args),
        Ok(args::Mode::Ui(kind)) => ui_process::run(kind),
        Err(error) => {
            tracing::warn!(error = %error, "the command line was refused");
            ExitCode::FAILURE
        }
    }
}

/// The app runs on Windows only, so elsewhere it starts nothing.
#[cfg(not(windows))]
fn main() -> ExitCode {
    ExitCode::FAILURE
}

#[cfg(test)]
mod tests;
