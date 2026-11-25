//! What the settings window asks the desktop to do, in values the desktop reads.

use std::path::PathBuf;

use cuakey_application::{ImportMode, MacroView, PreferencesView};

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
    /// The macro table the user edited, to be stored now.
    SaveMacros(Vec<MacroView>),
    /// Read the macro file at `path` into the stored macros.
    ImportMacros {
        path: PathBuf,
        mode: ImportMode,
    },
    /// Write the stored macros to the file at `path`.
    ExportMacros {
        path: PathBuf,
    },
    /// Convert the clipboard with these options: store the view first, then convert with what is stored.
    Convert(PreferencesView),
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
    /// Shows or hides the sections beyond the control group.
    ToggleExpanded,
}

impl Button {
    /// What the button asks of the process that owns the window. Expanding asks nothing: the window resizes itself.
    #[must_use]
    pub const fn action(self) -> Option<UiAction> {
        match self {
            Self::ExitApp => Some(UiAction::Exit),
            Self::ResetToDefaults => Some(UiAction::Reset),
            Self::CloseWindow => Some(UiAction::Close),
            Self::OpenMacros => Some(UiAction::Open(WindowKind::Macros)),
            Self::OpenAlwaysOn => Some(UiAction::Open(WindowKind::AlwaysOn)),
            Self::ToggleExpanded => None,
        }
    }
}

#[cfg(test)]
mod tests;
