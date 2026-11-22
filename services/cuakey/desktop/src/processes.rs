//! How the two processes find each other: a named mutex keeps each to one instance, and a message window of a fixed
//! class receives what the other one sends.

use cuakey_interfaces::WindowKind;
use platform_windows::{AppMessage, SingleInstanceError, SingleInstanceGuard, WindowKind as ShownWindow, post_to};

use crate::args::Mode;

pub(crate) const BACKGROUND_MUTEX: &str = r"Local\CuaKey.Background";
/// The message window of the background process, which installers and updaters also post `Quit` to.
pub(crate) const BACKGROUND_CLASS: &str = "CuaKey.Messages";
pub(crate) const UI_MUTEX: &str = r"Local\CuaKey.Ui";
pub(crate) const UI_CLASS: &str = "CuaKey.UiMessages";

/// The mutex that keeps `mode` to one instance.
pub(crate) const fn instance_mutex(mode: &Mode) -> &'static str {
    match mode {
        Mode::Background { .. } => BACKGROUND_MUTEX,
        Mode::Ui(_) => UI_MUTEX,
    }
}

/// Where a second launch of `mode` sends its request before it exits, and what it asks: the running background opens
/// the settings, and the running window process opens the window this launch was for.
pub(crate) const fn second_launch(mode: &Mode) -> (&'static str, AppMessage) {
    match mode {
        Mode::Background { .. } => (BACKGROUND_CLASS, AppMessage::ShowWindow(ShownWindow::Settings)),
        Mode::Ui(kind) => (UI_CLASS, AppMessage::ShowWindow(shown_window(*kind))),
    }
}

/// Takes the single-instance mutex of `mode`. When another instance holds it, that instance is handed this launch's
/// request and `None` comes back: this process has nothing left to do.
pub(crate) fn take_instance(mode: &Mode) -> Result<Option<SingleInstanceGuard>, SingleInstanceError> {
    let guard = SingleInstanceGuard::acquire(instance_mutex(mode))?;
    if guard.is_none() {
        let (class, message) = second_launch(mode);
        // The other instance may still be starting, before its window exists; the request is then lost, as a click
        // on a program that is not ready yet is.
        if let Err(error) = post_to(class, message) {
            tracing::warn!(error = ?error, "could not hand the request to the running instance");
        }
    }
    Ok(guard)
}

/// A window kind as it travels between the processes.
pub(crate) const fn shown_window(kind: WindowKind) -> ShownWindow {
    match kind {
        WindowKind::Settings => ShownWindow::Settings,
        WindowKind::Macros => ShownWindow::Macros,
        WindowKind::Convert => ShownWindow::Convert,
        WindowKind::AlwaysOn => ShownWindow::AlwaysOn,
        WindowKind::About => ShownWindow::About,
    }
}

/// A window kind that arrived from the other process.
pub(crate) const fn window_kind(shown: ShownWindow) -> WindowKind {
    match shown {
        ShownWindow::Settings => WindowKind::Settings,
        ShownWindow::Macros => WindowKind::Macros,
        ShownWindow::Convert => WindowKind::Convert,
        ShownWindow::AlwaysOn => WindowKind::AlwaysOn,
        ShownWindow::About => WindowKind::About,
    }
}

#[cfg(test)]
mod tests;
