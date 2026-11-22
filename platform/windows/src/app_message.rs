//! The messages one `CuaKey` process sends another, and the number each travels as.

/// A window the background process can be asked to show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowKind {
    Settings,
    Macros,
    Convert,
    AlwaysOn,
    About,
}

/// What one process asks the other to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMessage {
    /// Read the preferences again.
    Reload,
    ShowWindow(WindowKind),
    /// Exit.
    Quit,
}

const RELOAD: usize = 1;
const QUIT: usize = 2;
/// The code of the first window kind; the others follow it in order, clear of the plain messages.
const SHOW_WINDOW_BASE: usize = 0x10;

impl AppMessage {
    /// The number this message travels as.
    #[must_use]
    pub const fn encode(self) -> usize {
        match self {
            Self::Reload => RELOAD,
            Self::Quit => QUIT,
            Self::ShowWindow(WindowKind::Settings) => SHOW_WINDOW_BASE,
            Self::ShowWindow(WindowKind::Macros) => SHOW_WINDOW_BASE + 1,
            Self::ShowWindow(WindowKind::Convert) => SHOW_WINDOW_BASE + 2,
            Self::ShowWindow(WindowKind::AlwaysOn) => SHOW_WINDOW_BASE + 3,
            Self::ShowWindow(WindowKind::About) => SHOW_WINDOW_BASE + 4,
        }
    }

    /// The message a number stands for, or `None` for one this version does not know, such as a newer version's.
    #[must_use]
    pub const fn decode(code: usize) -> Option<Self> {
        Some(match code {
            RELOAD => Self::Reload,
            QUIT => Self::Quit,
            code if code == SHOW_WINDOW_BASE => Self::ShowWindow(WindowKind::Settings),
            code if code == SHOW_WINDOW_BASE + 1 => Self::ShowWindow(WindowKind::Macros),
            code if code == SHOW_WINDOW_BASE + 2 => Self::ShowWindow(WindowKind::Convert),
            code if code == SHOW_WINDOW_BASE + 3 => Self::ShowWindow(WindowKind::AlwaysOn),
            code if code == SHOW_WINDOW_BASE + 4 => Self::ShowWindow(WindowKind::About),
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests;
