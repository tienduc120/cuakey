//! The window process: the settings, macro, conversion and always-on windows, opened on demand by the background
//! process. Each open window has a native window of its own; the first is eframe's root viewport, and the process
//! ends when the last one closes.

mod system;
mod window;

use std::cell::RefCell;
use std::path::PathBuf;
use std::process::ExitCode;
use std::rc::Rc;
use std::sync::Arc;

use cuakey_application::{ImportMode, MacroView, PreferencesView};
use cuakey_interfaces::{
    Notice, UiAction, WindowKind, convert_error_message, export_macros_error_message, import_macros_error_message,
    install_system_font, labels, load_error_message, notice_message, reset_error_message, save_error_message,
    save_macros_error_message,
};
use eframe::egui::{Context, IconData, Ui, ViewportCommand, ViewportId};
use platform_windows::{
    AppMessage, KnownFolderError, MessageWindow, SingleInstanceError, post_to, show_notice, show_notice_and_wait,
    system_fonts,
};

use self::window::Window;
use crate::args::Mode;
use crate::processes::{BACKGROUND_CLASS, UI_CLASS, take_instance, window_kind};
use crate::stores::{OpenStoresError, Origin, Stores};

/// The name eframe gives the app.
const APP_NAME: &str = "CuaKey";

/// Why the window process could not start or stopped.
#[derive(Debug, thiserror::Error)]
enum StartError {
    #[error("could not take the single-instance mutex")]
    Instance(#[source] SingleInstanceError),
    #[error("could not open the stores")]
    Stores(#[source] OpenStoresError),
    #[error("could not find the folder of the system fonts")]
    Fonts(#[source] KnownFolderError),
    #[error("the windows failed")]
    Windows(#[source] eframe::Error),
}

/// Runs the window process with `kind` open, until its last window closes.
pub(crate) fn run(kind: WindowKind) -> ExitCode {
    match start(kind) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(error = ?error, "the window process failed");
            // The process ends next, which would take a notice on a thread of its own with it.
            show_notice_and_wait(labels::NOTICE_TITLE, notice_message(Notice::WindowNotOpened));
            ExitCode::FAILURE
        }
    }
}

fn start(kind: WindowKind) -> Result<(), StartError> {
    let Some(_guard) = take_instance(&Mode::Ui(kind)).map_err(StartError::Instance)? else {
        return Ok(());
    };
    let stores = Stores::open().map_err(StartError::Stores)?;
    let fonts = system_fonts().map_err(StartError::Fonts)?;
    let icon = Arc::new(window_icon());
    let first = Window::open(kind, &stores);
    let options = eframe::NativeOptions {
        viewport: first.viewport(&icon),
        centered: true,
        // The windows bring their own font; eframe would only warn about the ones it is missing.
        system_font_fallback: false,
        // eframe's clipboard is not built in, and it would swallow Ctrl+V; the windows take the shortcuts themselves.
        clipboard_shortcuts: false,
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(
        APP_NAME,
        options,
        Box::new(move |creation| {
            install_system_font(&creation.egui_ctx, &fonts)?;
            let inbox = Rc::new(RefCell::new(Vec::new()));
            let messages = {
                let inbox = Rc::clone(&inbox);
                let ctx = creation.egui_ctx.clone();
                MessageWindow::new(UI_CLASS, move |message| {
                    inbox.borrow_mut().push(message);
                    ctx.request_repaint();
                })?
            };
            Ok(Box::new(UiApp {
                stores,
                windows: vec![first],
                icon,
                inbox,
                _messages: messages,
            }))
        }),
    )
    .map_err(StartError::Windows)
}

/// The icon of every window: the "V" icon the executable shows, from the icon file it was built with.
fn window_icon() -> IconData {
    largest_png(include_bytes!("../../../../icon/icon_V.ico"))
        .and_then(|png| eframe::icon_data::from_png_bytes(png).ok())
        // An empty icon is ignored, so the window keeps the system's rather than eframe's own.
        .unwrap_or_default()
}

/// The windows and what they need.
#[derive(Debug)]
struct UiApp {
    stores: Stores,
    /// The open windows, one of each kind at most; the first is drawn in the root viewport.
    windows: Vec<Window>,
    icon: Arc<IconData>,
    /// What the background process posted, taken at the start of each pass.
    inbox: Rc<RefCell<Vec<AppMessage>>>,
    /// The message window the background process posts to, alive as long as the windows.
    _messages: MessageWindow,
}

/// What one window asked for in one pass.
#[derive(Debug)]
struct Shown {
    kind: WindowKind,
    actions: Vec<UiAction>,
    /// The user closed its native window.
    closing: bool,
}

impl eframe::App for UiApp {
    fn logic(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let messages = std::mem::take(&mut *self.inbox.borrow_mut());
        for message in messages {
            self.on_message(ctx, message);
        }
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let mut shown = Vec::new();
        if let Some(first) = self.windows.first_mut() {
            let actions = draw(first, ui);
            let closing = ui.input(|input| input.viewport().close_requested());
            shown.push(Shown {
                kind: first.kind(),
                actions,
                closing,
            });
        }
        for window in self.windows.iter_mut().skip(1) {
            let kind = window.kind();
            let (actions, closing) =
                ctx.show_viewport_immediate(window.viewport_id(), window.viewport(&self.icon), |ui, _class| {
                    let actions = draw(window, ui);
                    (actions, ui.input(|input| input.viewport().close_requested()))
                });
            shown.push(Shown { kind, actions, closing });
        }
        for Shown { kind, actions, closing } in shown {
            for action in actions {
                self.act(&ctx, kind, action);
            }
            if closing {
                self.close(&ctx, kind);
            }
        }
    }
}

/// Gives the root viewport to `window` once the window it held closed: the root stays open, takes the window's title,
/// and takes the size of what the window shows now.
fn move_into_root(ctx: &Context, window: &Window) {
    ctx.send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::CancelClose);
    ctx.send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::Title(window.title().to_owned()));
    ctx.send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::InnerSize(window.size()));
}

/// Draws `window` into `ui`, the root ui of its viewport, with the clipboard and the browser at hand.
fn draw(window: &mut Window, ui: &mut Ui) -> Vec<UiAction> {
    let ctx = ui.ctx().clone();
    system::before_drawing(&ctx);
    let actions = window.show(ui);
    system::after_drawing(&ctx);
    actions
}

impl UiApp {
    fn on_message(&mut self, ctx: &Context, message: AppMessage) {
        match message {
            AppMessage::Reload => match self.stores.load_preferences() {
                Ok(prefs) => self.set_preferences(&prefs),
                Err(error) => tracing::warn!(error = ?error, "could not load the preferences"),
            },
            AppMessage::ShowWindow(kind) => self.open(ctx, window_kind(kind)),
            AppMessage::Quit => self.close_all(ctx),
            AppMessage::Wake => {}
        }
    }

    fn act(&mut self, ctx: &Context, source: WindowKind, action: UiAction) {
        match action {
            UiAction::Save(edited) => {
                self.store(source, edited);
            }
            UiAction::Reset => self.reset(source),
            UiAction::Exit => {
                tell_background(AppMessage::Quit);
                self.close_all(ctx);
            }
            UiAction::Open(kind) => self.open(ctx, kind),
            UiAction::SaveMacros(macros) => self.save_macros(source, &macros),
            UiAction::ImportMacros { path, mode } => self.import_macros(source, path, mode),
            UiAction::ExportMacros { path } => self.export_macros(source, path),
            UiAction::Convert(edited) => self.convert(source, edited),
            UiAction::Close => self.close(ctx, source),
        }
    }

    /// Stores what `source` edited on top of the fresh preferences, shows the result in every window, and tells the
    /// background process. Says whether it was stored.
    fn store(&mut self, source: WindowKind, edited: PreferencesView) -> bool {
        if !self.origin_of(source, Window::prefs_origin).may_save() {
            self.show_error(source, Some(notice_message(Notice::SettingsUnreadNotSaved)));
            return false;
        }
        let stored = match self.stores.load_preferences() {
            Ok(stored) => stored,
            Err(error) => {
                self.show_error(source, Some(load_error_message(&error)));
                return false;
            }
        };
        let prefs = rebase(stored, edited, edit_of(source));
        match self.stores.save_preferences(&prefs) {
            Ok(()) => {
                self.show_error(source, None);
                self.set_preferences(&prefs);
                tell_background(AppMessage::Reload);
                true
            }
            Err(error) => {
                self.show_error(source, Some(save_error_message(&error)));
                false
            }
        }
    }

    fn reset(&mut self, source: WindowKind) {
        let reset = self
            .stores
            .load_preferences()
            .map_err(|error| load_error_message(&error))
            .and_then(|current| {
                self.stores
                    .reset_preferences(&current)
                    .map_err(|error| reset_error_message(&error))
            });
        match reset {
            Ok(prefs) => {
                self.show_error(source, None);
                self.set_preferences(&prefs);
                tell_background(AppMessage::Reload);
            }
            Err(message) => self.show_error(source, Some(message)),
        }
    }

    fn save_macros(&mut self, source: WindowKind, macros: &[MacroView]) {
        if !self.origin_of(source, Window::macros_origin).may_save() {
            self.show_error(source, Some(notice_message(Notice::MacrosUnreadNotSaved)));
            return;
        }
        match self.stores.save_macros(macros) {
            Ok(()) => {
                self.show_error(source, None);
                tell_background(AppMessage::Reload);
            }
            // The refused row stays in the table, so the user can see which one to correct.
            Err(error) => self.show_error(source, Some(save_macros_error_message(&error))),
        }
    }

    fn import_macros(&mut self, source: WindowKind, path: PathBuf, mode: ImportMode) {
        match self.stores.import_macros(path, mode) {
            Ok(macros) => {
                self.show_error(source, None);
                if let Some(window) = self.window_mut(source) {
                    window.set_macros(macros);
                }
                tell_background(AppMessage::Reload);
            }
            Err(error) => self.show_error(source, Some(import_macros_error_message(&error))),
        }
    }

    fn export_macros(&mut self, source: WindowKind, path: PathBuf) {
        let message = self
            .stores
            .export_macros(path)
            .err()
            .map(|error| export_macros_error_message(&error));
        self.show_error(source, message);
    }

    /// Stores the options the conversion window shows, then converts the clipboard with them.
    fn convert(&mut self, source: WindowKind, edited: PreferencesView) {
        if !self.store(source, edited) {
            return;
        }
        match self.stores.quick_convert() {
            Ok(result) => {
                if result.notify {
                    show_notice(labels::NOTICE_TITLE, notice_message(Notice::ConvertDone));
                }
            }
            Err(error) => self.show_error(source, Some(convert_error_message(&error))),
        }
    }

    /// Brings `kind` to the front, opening it when it is not open.
    fn open(&mut self, ctx: &Context, kind: WindowKind) {
        let shown = shown_kind(kind);
        let Some(index) = self.windows.iter().position(|window| window.kind() == shown) else {
            // The kind asked for, not the one shown: the About menu item opens the settings window expanded.
            self.windows.push(Window::open(kind, &self.stores));
            return;
        };
        let viewport = match self.windows.get(index) {
            Some(window) if index > 0 => window.viewport_id(),
            Some(_) | None => ViewportId::ROOT,
        };
        ctx.send_viewport_cmd_to(viewport, ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd_to(viewport, ViewportCommand::Focus);
    }

    /// Closes `kind`. A window in a viewport of its own goes once it is no longer drawn; when the window in the root
    /// viewport closes, the next one moves into it, since closing the root viewport ends the process.
    fn close(&mut self, ctx: &Context, kind: WindowKind) {
        let Some(index) = self.windows.iter().position(|window| window.kind() == kind) else {
            return;
        };
        self.windows.remove(index);
        if index > 0 {
            return;
        }
        match self.windows.first() {
            Some(next) => move_into_root(ctx, next),
            None => ctx.send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::Close),
        }
    }

    fn close_all(&mut self, ctx: &Context) {
        self.windows.clear();
        ctx.send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::Close);
    }

    fn set_preferences(&mut self, prefs: &PreferencesView) {
        for window in &mut self.windows {
            window.set_preferences(prefs.clone());
        }
    }

    fn show_error(&mut self, source: WindowKind, message: Option<&'static str>) {
        if let Some(window) = self.window_mut(source) {
            window.show_error(message);
        }
    }

    fn window_mut(&mut self, kind: WindowKind) -> Option<&mut Window> {
        self.windows.iter_mut().find(|window| window.kind() == kind)
    }

    /// Where what `kind` shows came from, as `origin` reads it; a window that is gone saves nothing.
    fn origin_of(&self, kind: WindowKind, origin: fn(&Window) -> Origin) -> Origin {
        self.windows
            .iter()
            .find(|window| window.kind() == kind)
            .map_or(Origin::Defaults, origin)
    }
}

/// Posts `message` to the background process; a window process started on its own has none to tell.
fn tell_background(message: AppMessage) {
    if let Err(error) = post_to(BACKGROUND_CLASS, message) {
        tracing::debug!(error = ?error, "the background process did not take the message");
    }
}

/// What a window's save may change in the stored preferences.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Edit {
    /// The setting groups and the language.
    Settings,
    /// The apps that are always on.
    AlwaysOn,
}

pub(crate) const fn edit_of(kind: WindowKind) -> Edit {
    match kind {
        WindowKind::AlwaysOn => Edit::AlwaysOn,
        WindowKind::Settings | WindowKind::Macros | WindowKind::Convert | WindowKind::About => Edit::Settings,
    }
}

/// The preferences to store for a save of `edited`: the fresh `stored` ones, with only what the window edits taken
/// from it. The background process keeps the language and the remembered apps in the store as they change, so a
/// window opened before then would otherwise put back what it was shown.
pub(crate) fn rebase(stored: PreferencesView, edited: PreferencesView, edit: Edit) -> PreferencesView {
    match edit {
        Edit::Settings => PreferencesView {
            language: edited.language,
            typing: edited.typing,
            macros: edited.macros,
            switching: edited.switching,
            compatibility: edited.compatibility,
            system: edited.system,
            behaviours: edited.behaviours,
            convert: edited.convert,
            ..stored
        },
        Edit::AlwaysOn => PreferencesView {
            always_on: edited.always_on,
            ..stored
        },
    }
}

/// The window that shows `kind`: "about" is the last section of the settings window.
pub(crate) const fn shown_kind(kind: WindowKind) -> WindowKind {
    match kind {
        WindowKind::About => WindowKind::Settings,
        WindowKind::Settings | WindowKind::Macros | WindowKind::Convert | WindowKind::AlwaysOn => kind,
    }
}

const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";
const ICO_HEADER_LEN: usize = 6;
const ICO_ENTRY_LEN: usize = 16;

/// The largest picture in an icon file that is stored as a PNG, as eframe takes a window icon.
pub(crate) fn largest_png(ico: &[u8]) -> Option<&[u8]> {
    let count = usize::from(u16::from_le_bytes(ico.get(4..6)?.try_into().ok()?));
    let directory = ico.get(ICO_HEADER_LEN..ICO_HEADER_LEN + count * ICO_ENTRY_LEN)?;
    directory
        .as_chunks::<ICO_ENTRY_LEN>()
        .0
        .iter()
        .filter_map(|entry| {
            let (width, picture) = icon_picture(ico, entry)?;
            picture.starts_with(PNG_SIGNATURE).then_some((width, picture))
        })
        .max_by_key(|(width, _)| *width)
        .map(|(_, picture)| picture)
}

/// The width and the bytes of the picture a directory entry of `ico` describes.
fn icon_picture<'a>(ico: &'a [u8], entry: &[u8; ICO_ENTRY_LEN]) -> Option<(u32, &'a [u8])> {
    // The width, height, colour count, a reserved byte, planes and bits per pixel, then the picture's size and where
    // it starts.
    let [width, _, _, _, _, _, _, _, s0, s1, s2, s3, o0, o1, o2, o3] = *entry;
    // A width of 0 stands for 256.
    let width = if width == 0 { 256 } else { u32::from(width) };
    let size = usize::try_from(u32::from_le_bytes([s0, s1, s2, s3])).ok()?;
    let offset = usize::try_from(u32::from_le_bytes([o0, o1, o2, o3])).ok()?;
    Some((width, ico.get(offset..offset.checked_add(size)?)?))
}

#[cfg(test)]
mod tests;
