//! What the settings window asks the desktop to do, in values the desktop reads.

use cuakey_application::PreferencesView;

/// The system the window runs on. Options only Windows has are hidden elsewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    Other,
}

/// A window the desktop can open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowKind {
    Settings,
    Macros,
    Convert,
    AlwaysOn,
    About,
}

/// A request from the window to the process that owns it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiAction {
    /// The user changed a control: the whole edited view, to be stored now.
    Save(PreferencesView),
    /// Quit the app.
    Exit,
    /// Put every preference back to its default.
    Reset,
    Open(WindowKind),
    /// Close this window.
    Close,
}

/// A button of the window that asks for something other than a changed value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    ExitApp,
    ResetToDefaults,
    CloseWindow,
    OpenMacros,
    OpenAlwaysOn,
}

impl Button {
    #[must_use]
    pub const fn action(self) -> UiAction {
        match self {
            Self::ExitApp => UiAction::Exit,
            Self::ResetToDefaults => UiAction::Reset,
            Self::CloseWindow => UiAction::Close,
            Self::OpenMacros => UiAction::Open(WindowKind::Macros),
            Self::OpenAlwaysOn => UiAction::Open(WindowKind::AlwaysOn),
        }
    }
}

#[cfg(test)]
mod tests;
