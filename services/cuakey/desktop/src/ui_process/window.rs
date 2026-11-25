//! One open window of the window process, whichever kind it is, and whether what it shows was read from the stores.

use std::sync::Arc;

use cuakey_application::{MacroView, PreferencesView};
use cuakey_interfaces::{
    AlwaysOnWindow, COMPACT_SIZE, ConvertWindow, MacroWindow, Notice, Platform, SettingsWindow, UiAction, WindowKind,
    labels, notice_message,
};
use eframe::egui::{IconData, Ui, Vec2, ViewportBuilder, ViewportId};

use super::shown_kind;
use crate::args::window_name;
use crate::stores::{Origin, Stores};

#[derive(Debug)]
pub(super) struct Window {
    view: View,
    /// Whether the preferences it shows were read; a save built on defaults would replace what the user set.
    prefs: Origin,
    /// Whether the macros it shows were read; only the macro window shows any.
    macros: Origin,
}

#[derive(Debug)]
enum View {
    Settings(SettingsWindow),
    Macros(MacroWindow),
    Convert(ConvertWindow),
    AlwaysOn(AlwaysOnWindow),
}

impl Window {
    /// The window that shows `kind`, with what is stored now. When something cannot be read the window shows the
    /// defaults, says so, and saves none of its changes until the stored values reach it.
    pub(super) fn open(kind: WindowKind, stores: &Stores) -> Self {
        let (prefs, prefs_origin, prefs_error) = Origin::of(stores.load_preferences());
        let mut macros_origin = Origin::Stored;
        let view = match shown_kind(kind) {
            WindowKind::Settings | WindowKind::About => View::Settings(SettingsWindow::new(prefs, Platform::Windows)),
            WindowKind::Macros => {
                let (macros, origin, error) = Origin::of(stores.load_macros());
                if let Some(error) = error {
                    tracing::warn!(error = ?error, "could not load the macros");
                }
                macros_origin = origin;
                View::Macros(MacroWindow::new(prefs, macros))
            }
            WindowKind::Convert => View::Convert(ConvertWindow::new(prefs)),
            WindowKind::AlwaysOn => View::AlwaysOn(AlwaysOnWindow::new(prefs)),
        };
        if let Some(error) = prefs_error {
            tracing::warn!(error = ?error, "could not load the preferences");
        }
        let mut window = Self {
            view,
            prefs: prefs_origin,
            macros: macros_origin,
        };
        window.show_error(window.unread_message());
        window
    }

    /// Why this window saves nothing, when what it shows could not be read.
    fn unread_message(&self) -> Option<&'static str> {
        if !self.prefs.may_save() {
            Some(notice_message(Notice::SettingsUnreadNotSaved))
        } else if !self.macros.may_save() {
            Some(notice_message(Notice::MacrosUnreadNotSaved))
        } else {
            None
        }
    }

    /// Whether a save of this window's preferences may be written.
    pub(super) const fn prefs_origin(&self) -> Origin {
        self.prefs
    }

    /// Whether a save of this window's macros may be written.
    pub(super) const fn macros_origin(&self) -> Origin {
        self.macros
    }

    pub(super) const fn kind(&self) -> WindowKind {
        match self.view {
            View::Settings(_) => WindowKind::Settings,
            View::Macros(_) => WindowKind::Macros,
            View::Convert(_) => WindowKind::Convert,
            View::AlwaysOn(_) => WindowKind::AlwaysOn,
        }
    }

    pub(super) fn show(&mut self, ui: &mut Ui) -> Vec<UiAction> {
        match &mut self.view {
            View::Settings(window) => window.show(ui),
            View::Macros(window) => window.show(ui),
            View::Convert(window) => window.show(ui),
            View::AlwaysOn(window) => window.show(ui),
        }
    }

    /// Shows `prefs`, which come from the store, so a save built on them may be written from now on.
    pub(super) fn set_preferences(&mut self, prefs: PreferencesView) {
        match &mut self.view {
            View::Settings(window) => window.set_preferences(prefs),
            View::Macros(window) => window.set_preferences(prefs),
            View::Convert(window) => window.set_preferences(prefs),
            View::AlwaysOn(window) => window.set_preferences(prefs),
        }
        if !self.prefs.may_save() {
            self.prefs = Origin::Stored;
            self.show_error(self.unread_message());
        }
    }

    /// Replaces the macro window's rows with `macros`, which come from the store; other windows show no macros.
    pub(super) fn set_macros(&mut self, macros: Vec<MacroView>) {
        match &mut self.view {
            View::Macros(window) => window.set_macros(macros),
            View::Settings(_) | View::Convert(_) | View::AlwaysOn(_) => return,
        }
        if !self.macros.may_save() {
            self.macros = Origin::Stored;
            self.show_error(self.unread_message());
        }
    }

    pub(super) const fn show_error(&mut self, message: Option<&'static str>) {
        match &mut self.view {
            View::Settings(window) => window.show_error(message),
            View::Macros(window) => window.show_error(message),
            View::Convert(window) => window.show_error(message),
            View::AlwaysOn(window) => window.show_error(message),
        }
    }

    pub(super) const fn title(&self) -> &'static str {
        match self.view {
            View::Settings(_) => labels::WINDOW_TITLE,
            View::Macros(_) => labels::MACRO_WINDOW_TITLE,
            View::Convert(_) => labels::CONVERT_WINDOW_TITLE,
            View::AlwaysOn(_) => labels::ALWAYS_ON_WINDOW_TITLE,
        }
    }

    /// The size the window opens at, in points.
    pub(super) const fn size(&self) -> Vec2 {
        match self.view {
            View::Settings(_) => COMPACT_SIZE,
            View::Macros(_) => Vec2::new(480.0, 460.0),
            View::Convert(_) => Vec2::new(440.0, 300.0),
            View::AlwaysOn(_) => Vec2::new(360.0, 340.0),
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
