//! One open window of the window process, whichever kind it is, and whether what it shows was read from the stores.

use std::sync::Arc;

use cuakey_application::{InterfaceLanguageValue, MacroView, PreferencesView};
use cuakey_interfaces::{
    AlwaysOnWindow, ConvertWindow, MacroWindow, Notice, Platform, SettingsWindow, UiAction, WindowKind, labels,
    notice_message,
};
use eframe::egui::{Context, IconData, Ui, Vec2, ViewportBuilder, ViewportCommand, ViewportId};

use crate::args::window_name;
use crate::stores::{Origin, Stores};

#[derive(Debug)]
pub(super) struct Window {
    view: View,
    /// Whether the preferences it shows were read; a save built on defaults would replace what the user set.
    prefs: Origin,
    /// Whether the macros it shows were read; only the macro window shows any.
    macros: Origin,
    /// The title its native window shows, so a change of interface language can retitle it.
    shown_title: &'static str,
    /// The size it opened at. Its native window takes later sizes from what the window draws, which keeps it on
    /// screen; a viewport of its own given a new size would resize it again, past that.
    opened_size: Vec2,
}

#[derive(Debug)]
enum View {
    Settings(SettingsWindow),
    Macros(MacroWindow),
    Convert(ConvertWindow),
    AlwaysOn(AlwaysOnWindow),
}

impl View {
    const fn interface_language(&self) -> InterfaceLanguageValue {
        match self {
            Self::Settings(window) => window.interface_language(),
            Self::Macros(window) => window.interface_language(),
            Self::Convert(window) => window.interface_language(),
            Self::AlwaysOn(window) => window.interface_language(),
        }
    }

    const fn title(&self) -> &'static str {
        let text = labels(self.interface_language());
        match self {
            Self::Settings(_) => text.window_title,
            Self::Macros(_) => text.macro_window_title,
            Self::Convert(_) => text.convert_window_title,
            Self::AlwaysOn(_) => text.always_on_window_title,
        }
    }

    const fn size(&self) -> Vec2 {
        match self {
            Self::Settings(window) => window.size(),
            Self::Macros(_) => Vec2::new(740.0, 460.0),
            Self::Convert(_) => Vec2::new(570.0, 380.0),
            Self::AlwaysOn(_) => Vec2::new(455.0, 340.0),
        }
    }
}

impl Window {
    /// The window that shows `kind`, with what is stored now. When something cannot be read the window shows the
    /// defaults, says so, and saves none of its changes until the stored values reach it.
    pub(super) fn open(kind: WindowKind, stores: &Stores) -> Self {
        let (prefs, prefs_origin, prefs_error) = Origin::of(stores.load_preferences());
        let mut macros_origin = Origin::Stored;
        let view = match kind {
            WindowKind::Settings => View::Settings(SettingsWindow::new(prefs, Platform::Windows)),
            WindowKind::About => {
                let mut window = SettingsWindow::new(prefs, Platform::Windows);
                window.reveal_about();
                View::Settings(window)
            }
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
            shown_title: view.title(),
            opened_size: view.size(),
            view,
            prefs: prefs_origin,
            macros: macros_origin,
        };
        window.show_error(window.unread_message());
        window
    }

    /// Why this window saves nothing, when what it shows could not be read.
    fn unread_message(&self) -> Option<&'static str> {
        let language = self.interface_language();
        if !self.prefs.may_save() {
            Some(notice_message(Notice::SettingsUnreadNotSaved, language))
        } else if !self.macros.may_save() {
            Some(notice_message(Notice::MacrosUnreadNotSaved, language))
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

    /// The language the window speaks: the interface language of the preferences it shows.
    pub(super) const fn interface_language(&self) -> InterfaceLanguageValue {
        self.view.interface_language()
    }

    /// Draws the window into `ui` and returns what the user asked for this frame. When the interface language changed
    /// since the last frame, the native window takes the title in the new language.
    pub(super) fn show(&mut self, ui: &mut Ui) -> Vec<UiAction> {
        let actions = match &mut self.view {
            View::Settings(window) => window.show(ui),
            View::Macros(window) => window.show(ui),
            View::Convert(window) => window.show(ui),
            View::AlwaysOn(window) => window.show(ui),
        };
        self.follow_title(ui.ctx());
        actions
    }

    /// Tells the native window of `ctx`'s viewport its title when it no longer says what it shows.
    fn follow_title(&mut self, ctx: &Context) {
        let title = self.title();
        if title != self.shown_title {
            ctx.send_viewport_cmd(ViewportCommand::Title(title.to_owned()));
            self.shown_title = title;
        }
    }

    /// Expands a settings window and brings its about text into view; the other windows have no about text.
    pub(super) const fn reveal_about(&mut self) {
        match &mut self.view {
            View::Settings(window) => window.reveal_about(),
            View::Macros(_) | View::Convert(_) | View::AlwaysOn(_) => {}
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

    /// The title of the native window, in the window's interface language.
    pub(super) const fn title(&self) -> &'static str {
        self.view.title()
    }

    /// The size of what the window shows now, in points: the size it opens at, and the size it takes in the root
    /// viewport. The settings window changes it when it expands or collapses, and when its compact rows change height,
    /// as an error message comes or goes. Each other size holds every row and
    /// button, and a line for an error message, in either interface language, with the shared style in Segoe UI and in
    /// the fonts the windows may borrow instead.
    pub(super) const fn size(&self) -> Vec2 {
        self.view.size()
    }

    /// The native window this one opens in.
    pub(super) fn viewport(&self, icon: &Arc<IconData>) -> ViewportBuilder {
        ViewportBuilder::default()
            .with_title(self.title())
            .with_inner_size(self.opened_size)
            .with_icon(Arc::clone(icon))
    }

    /// The viewport of the window when it is not in the root one.
    pub(super) fn viewport_id(&self) -> ViewportId {
        ViewportId::from_hash_of(window_name(self.kind()))
    }
}
