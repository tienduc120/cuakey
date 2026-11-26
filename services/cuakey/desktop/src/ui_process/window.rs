//! One open window of the window process, whichever kind it is, and whether what it shows was read from the stores.

use std::sync::Arc;

use cuakey_application::{InterfaceLanguageValue, MacroView, PreferencesView, WindowPlace, WindowPlaces};
use cuakey_interfaces::{
    AlwaysOnWindow, ConvertWindow, MacroWindow, Notice, Platform, SettingsWindow, UiAction, WindowKind, labels,
    notice_message,
};
use eframe::egui::{Context, IconData, Pos2, Ui, Vec2, ViewportBuilder, ViewportCommand, ViewportId};
use platform_windows::ScreenRect;

use super::place::{correction, opening_point, placed_window, reported_place};
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
    /// Where its native window opens, in points: the place it last closed at. Without one it opens where it always
    /// did, the first window centred and the others where Windows puts them.
    opens_at: Option<Pos2>,
    /// That place, until the native window first says where it is.
    settling_at: Option<WindowPlace>,
    /// Where its native window was when last drawn, kept for when it closes.
    last_place: Option<WindowPlace>,
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
            Self::Convert(_) => Vec2::new(680.0, 380.0),
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
            opens_at: None,
            settling_at: None,
            last_place: None,
        };
        window.show_error(window.unread_message());
        window
    }

    /// The window opening at its place in `places`, at egui's `zoom`, when its title bar there is on a monitor as
    /// `on_a_monitor` says. It keeps its size: a place left over from the expanded window may put the bottom of the
    /// compact one off the screen, and the user can still take hold of the title bar.
    pub(super) fn at_place(
        mut self,
        places: &WindowPlaces,
        zoom: f32,
        on_a_monitor: impl FnOnce(ScreenRect) -> bool,
    ) -> Self {
        let saved = places.get(placed_window(self.kind()));
        self.opens_at = saved.and_then(|saved| opening_point(saved, self.size(), zoom, on_a_monitor));
        self.settling_at = saved.filter(|_| self.opens_at.is_some());
        self
    }

    /// Whether the window opens at a place it remembers, rather than where it would without one.
    pub(super) const fn opens_at_place(&self) -> bool {
        self.opens_at.is_some()
    }

    /// Where its native window was when last drawn, if it said.
    pub(super) const fn last_place(&self) -> Option<WindowPlace> {
        self.last_place
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
        self.follow_place(ui.ctx());
        actions
    }

    /// Notes where the native window of `ctx`'s viewport is. The first time it says, a window that landed off the place
    /// it opened at is moved onto it, once, so the user can move it from then on.
    fn follow_place(&mut self, ctx: &Context) {
        let Some((now, pixels_per_point)) =
            ctx.input(|input| reported_place(input).map(|place| (place, input.pixels_per_point)))
        else {
            return;
        };
        if let Some(target) = self.settling_at.take()
            && let Some(point) = correction(target, now, pixels_per_point)
        {
            ctx.send_viewport_cmd(ViewportCommand::OuterPosition(point));
        }
        self.last_place = Some(now);
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

    /// The native window this one opens in. The user can neither resize nor maximize it: each size holds what the
    /// window shows, and only the window itself changes it. The place it opens at stays the same in every frame, so
    /// it never moves the native window back after the user moved it.
    pub(super) fn viewport(&self, icon: &Arc<IconData>) -> ViewportBuilder {
        let builder = ViewportBuilder::default()
            .with_title(self.title())
            .with_inner_size(self.opened_size)
            .with_resizable(false)
            .with_maximize_button(false)
            .with_icon(Arc::clone(icon));
        match self.opens_at {
            Some(point) => builder.with_position(point),
            None => builder,
        }
    }

    /// The viewport of the window when it is not in the root one.
    pub(super) fn viewport_id(&self) -> ViewportId {
        ViewportId::from_hash_of(window_name(self.kind()))
    }
}
