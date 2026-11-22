//! One open window of the window process, whichever kind it is.

use std::sync::Arc;

use cuakey_application::{MacroView, PreferencesView};
use cuakey_interfaces::{
    AlwaysOnWindow, ConvertWindow, MacroWindow, Platform, SettingsWindow, UiAction, WindowKind, labels,
    load_error_message, load_macros_error_message,
};
use eframe::egui::{IconData, Ui, Vec2, ViewportBuilder, ViewportId};

use super::shown_kind;
use crate::args::window_name;
use crate::stores::Stores;

#[derive(Debug)]
pub(super) enum Window {
    Settings(SettingsWindow),
    Macros(MacroWindow),
    Convert(ConvertWindow),
    AlwaysOn(AlwaysOnWindow),
}

impl Window {
    /// The window that shows `kind`, with the preferences as stored now. What could not be read is shown in it.
    pub(super) fn open(kind: WindowKind, stores: &Stores) -> Self {
        let (prefs, mut error) = match stores.load_preferences() {
            Ok(prefs) => (prefs, None),
            Err(error) => (PreferencesView::default(), Some(load_error_message(&error))),
        };
        let mut window = match shown_kind(kind) {
            WindowKind::Settings | WindowKind::About => Self::Settings(SettingsWindow::new(prefs, Platform::Windows)),
            WindowKind::Macros => {
                let macros = stores.load_macros().unwrap_or_else(|failure| {
                    error = Some(load_macros_error_message(&failure));
                    Vec::new()
                });
                Self::Macros(MacroWindow::new(prefs, macros))
            }
            WindowKind::Convert => Self::Convert(ConvertWindow::new(prefs)),
            WindowKind::AlwaysOn => Self::AlwaysOn(AlwaysOnWindow::new(prefs)),
        };
        window.show_error(error);
        window
    }

    pub(super) const fn kind(&self) -> WindowKind {
        match self {
            Self::Settings(_) => WindowKind::Settings,
            Self::Macros(_) => WindowKind::Macros,
            Self::Convert(_) => WindowKind::Convert,
            Self::AlwaysOn(_) => WindowKind::AlwaysOn,
        }
    }

    pub(super) fn show(&mut self, ui: &mut Ui) -> Vec<UiAction> {
        match self {
            Self::Settings(window) => window.show(ui),
            Self::Macros(window) => window.show(ui),
            Self::Convert(window) => window.show(ui),
            Self::AlwaysOn(window) => window.show(ui),
        }
    }

    pub(super) fn set_preferences(&mut self, prefs: PreferencesView) {
        match self {
            Self::Settings(window) => window.set_preferences(prefs),
            Self::Macros(window) => window.set_preferences(prefs),
            Self::Convert(window) => window.set_preferences(prefs),
            Self::AlwaysOn(window) => window.set_preferences(prefs),
        }
    }

    /// Replaces the macro window's rows; other windows show no macros.
    pub(super) fn set_macros(&mut self, macros: Vec<MacroView>) {
        match self {
            Self::Macros(window) => window.set_macros(macros),
            Self::Settings(_) | Self::Convert(_) | Self::AlwaysOn(_) => {}
        }
    }

    pub(super) const fn show_error(&mut self, message: Option<&'static str>) {
        match self {
            Self::Settings(window) => window.show_error(message),
            Self::Macros(window) => window.show_error(message),
            Self::Convert(window) => window.show_error(message),
            Self::AlwaysOn(window) => window.show_error(message),
        }
    }

    pub(super) const fn title(&self) -> &'static str {
        match self {
            Self::Settings(_) => labels::WINDOW_TITLE,
            Self::Macros(_) => labels::MACRO_WINDOW_TITLE,
            Self::Convert(_) => labels::CONVERT_WINDOW_TITLE,
            Self::AlwaysOn(_) => labels::ALWAYS_ON_WINDOW_TITLE,
        }
    }

    /// The size the window opens at, in points.
    pub(super) const fn size(&self) -> Vec2 {
        match self {
            Self::Settings(_) => Vec2::new(520.0, 640.0),
            Self::Macros(_) => Vec2::new(480.0, 460.0),
            Self::Convert(_) => Vec2::new(440.0, 300.0),
            Self::AlwaysOn(_) => Vec2::new(360.0, 340.0),
        }
    }

    /// The native window this one opens in.
    pub(super) fn viewport(&self, icon: &Arc<IconData>) -> ViewportBuilder {
        ViewportBuilder::default()
            .with_title(self.title())
            .with_inner_size(self.size())
            .with_icon(Arc::clone(icon))
    }

    /// The viewport of the window when it is not in the root one.
    pub(super) fn viewport_id(&self) -> ViewportId {
        ViewportId::from_hash_of(window_name(self.kind()))
    }
}
