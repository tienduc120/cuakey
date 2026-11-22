//! The background process: the keyboard hook, the focus watch, the notification-area icon, and the message window the
//! window process and installers post to. It loads no UI toolkit; the windows open in a process of their own.
//!
//! Three threads: the hook thread owns the typing session, as `hook` describes; the focus watch classifies the focused
//! control on its own, and the main thread runs the message loop, which serves the icon, its menu and the messages.

mod hook;
mod menu;
mod system;

use std::cell::{OnceCell, RefCell};
use std::io;
use std::path::Path;
use std::process::{Child, Command, ExitCode};
use std::rc::Rc;
use std::sync::{Arc, Mutex, mpsc};

use cuakey_application::{LanguageValue, LayoutFlags, MacroView, PreferencesView, SessionError, Toggle, TypingSession};
use cuakey_interfaces::{
    MenuEffect, MenuId, Notice, WindowKind, apply, convert_error_message, labels, load_error_message,
    load_macros_error_message, model, notice_message, save_error_message,
};
use platform_clock::SystemClock;
use platform_windows::{
    AppMessage, CreateMessageWindowError, FocusCell, FocusChange, FocusClass, FocusWatch, KeyboardHook, MessageWindow,
    RelaunchError, SingleInstanceError, SingleInstanceGuard, StartFocusWatchError, StartHookError, StartupError,
    StartupMode, Tray, TrayError, TrayEvent, apply_startup, beep, foreground_layout, ime_open, is_elevated, is_english,
    post_to, relaunch_elevated, set_desktop_shortcut, show_notice, show_notice_and_wait,
};

use self::hook::{HookHandler, HookWorld, Report, SessionUpdate};
use self::menu::{MenuMap, menu_app, tray_icon};
use self::system::{SystemChanges, startup_mode, system_changes};
use crate::args::{Mode, window_args};
use crate::processes::{BACKGROUND_CLASS, UI_CLASS, shown_window, take_instance, window_kind};
use crate::stores::{OpenStoresError, Origin, Stores};

/// This process, as the single-instance functions name it; its focus log plays no part there.
const THIS_PROCESS: Mode = Mode::Background { focus_log: None };

/// Why the background process could not start.
#[derive(Debug, thiserror::Error)]
enum StartError {
    #[error("could not take the single-instance mutex")]
    Instance(#[source] SingleInstanceError),
    #[error("could not open the stores")]
    Stores(#[source] OpenStoresError),
    #[error("could not start the typing session")]
    Session(#[source] SessionError),
    #[error("could not create the message window")]
    MessageWindow(#[source] CreateMessageWindowError),
    #[error("could not show the notification-area icon")]
    Tray(#[source] TrayError),
    #[error("could not start the focus watch")]
    FocusWatch(#[source] StartFocusWatchError),
    #[error("could not start the keyboard hook")]
    Hook(#[source] StartHookError),
}

/// Runs the background process until it is told to quit. `args` are this launch's own, handed on when it starts
/// itself again as administrator.
pub(crate) fn run(focus_log: Option<&Path>, args: &[String]) -> ExitCode {
    match start(focus_log, args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(error = ?error, "the background process could not start");
            // The process ends next, which would take a notice on a thread of its own with it.
            show_notice_and_wait(labels::NOTICE_TITLE, notice_message(Notice::NotStarted));
            ExitCode::FAILURE
        }
    }
}

fn start(focus_log: Option<&Path>, args: &[String]) -> Result<(), StartError> {
    let Some(guard) = take_instance(&THIS_PROCESS).map_err(StartError::Instance)? else {
        return Ok(());
    };
    let stores = Stores::open().map_err(StartError::Stores)?;
    let prefs = load_preferences(&stores);
    let macros = load_macros(&stores);
    let Some(_guard) = elevate_if_asked(guard, &prefs, args)? else {
        return Ok(());
    };
    // The defaults in place of unread preferences are not the user's way to start, so Windows keeps its own.
    if prefs.origin.may_save() {
        apply_startup_mode(startup_mode(&prefs.view.system));
    }
    serve(stores, prefs, macros, focus_log)
}

/// The preferences the background starts with, and whether they may be saved over the stored ones.
#[derive(Debug)]
struct Loaded {
    view: PreferencesView,
    origin: Origin,
}

/// The stored preferences, or the defaults after a notice when they cannot be read. The defaults are then never saved
/// in this run, so a read that failed for a moment cannot replace what the user set.
fn load_preferences(stores: &Stores) -> Loaded {
    let (view, origin, error) = Origin::of(stores.load_preferences());
    if let Some(error) = error {
        tracing::warn!(error = ?error, "could not load the preferences");
        notify(notice_message(Notice::DefaultsNotSaved));
    }
    Loaded { view, origin }
}

fn load_macros(stores: &Stores) -> Vec<MacroView> {
    stores.load_macros().unwrap_or_else(|error| {
        tracing::warn!(error = ?error, "could not load the macros");
        notify(load_macros_error_message(&error));
        Vec::new()
    })
}

/// Starts `CuaKey` again as administrator when the settings ask for it and this process is not. The guard goes first,
/// so the new process can take it; `None` means the new process runs and this one is done. A refused or failed prompt
/// carries on with normal rights, after a notice.
fn elevate_if_asked(
    guard: SingleInstanceGuard,
    prefs: &Loaded,
    args: &[String],
) -> Result<Option<SingleInstanceGuard>, StartError> {
    if prefs.view.system.run_as_administrator == Toggle::Off || is_elevated() {
        return Ok(Some(guard));
    }
    drop(guard);
    let notice = match relaunch_elevated(args) {
        Ok(()) => return Ok(None),
        Err(RelaunchError::Cancelled) => Notice::ElevationRefused,
        Err(error @ (RelaunchError::CurrentExe(_) | RelaunchError::Launch(_))) => {
            tracing::warn!(error = ?error, "could not start again as administrator");
            Notice::ElevationFailed
        }
    };
    notify(notice_message(notice));
    take_instance(&THIS_PROCESS).map_err(StartError::Instance)
}

/// Starts the threads and the icon, and runs the message loop until a `Quit`.
fn serve(stores: Stores, prefs: Loaded, macros: Vec<MacroView>, focus_log: Option<&Path>) -> Result<(), StartError> {
    let Loaded { view: prefs, origin } = prefs;
    let session = TypingSession::new(&prefs, &macros).map_err(StartError::Session)?;
    let (updates, session_updates) = mpsc::channel();
    let (reports, session_reports) = mpsc::channel();
    let last_app = Arc::new(Mutex::new(None));
    let language = prefs.language;
    let show_settings = prefs.system.show_settings_at_startup == Toggle::On;
    let background = Rc::new(Background {
        stores,
        origin,
        prefs: RefCell::new(prefs),
        macros: RefCell::new(macros),
        last_app: Arc::clone(&last_app),
        updates: updates.clone(),
        reports: session_reports,
        tray: OnceCell::new(),
        window_process: RefCell::new(None),
    });
    let window = {
        let background = Rc::clone(&background);
        MessageWindow::new(BACKGROUND_CLASS, move |message| background.on_message(message))
            .map_err(StartError::MessageWindow)?
    };
    background.show_tray(language)?;
    let cell = Arc::new(FocusCell::new());
    let watch = FocusWatch::start(Arc::clone(&cell), on_focus(updates, last_app), focus_log, SystemClock)
        .map_err(StartError::FocusWatch)?;
    let watch = Arc::new(watch);
    let world = WindowsWorld {
        cell,
        watch: Arc::clone(&watch),
        reports,
    };
    let mut handler = HookHandler::new(session, session_updates);
    let hook = KeyboardHook::start(move |event| handler.handle(event, &world)).map_err(StartError::Hook)?;
    if show_settings {
        background.open_window(WindowKind::Settings);
    }
    window.run_loop();
    // The icon goes first, so it is never left behind while the threads stop.
    drop(background);
    stop(hook, watch);
    Ok(())
}

/// What the focus watch does with each classification, on its thread: it notes the app for the menu and hands the
/// change to the session.
fn on_focus(
    updates: mpsc::Sender<SessionUpdate>,
    last_app: Arc<Mutex<Option<String>>>,
) -> impl Fn(FocusChange) + Send + 'static {
    move |change| {
        if change.app_changed
            && let Some(app) = menu_app(&change.app)
            && let Ok(mut last) = last_app.lock()
        {
            *last = Some(app.to_owned());
        }
        if updates.send(SessionUpdate::Focus(change)).is_err() {
            tracing::debug!("the session has stopped");
        }
    }
}

/// Stops the hook, and then the focus watch it was rechecking.
fn stop(hook: KeyboardHook, watch: Arc<FocusWatch>) {
    if let Err(error) = hook.stop() {
        tracing::warn!(error = ?error, "the keyboard hook did not stop cleanly");
    }
    // The hook's handler held the other reference, and it went with the hook thread.
    if let Ok(watch) = Arc::try_unwrap(watch)
        && let Err(error) = watch.stop()
    {
        tracing::warn!(error = ?error, "the focus watch did not stop cleanly");
    }
}

/// Windows as the hook handler sees it.
#[derive(Debug)]
struct WindowsWorld {
    cell: Arc<FocusCell>,
    watch: Arc<FocusWatch>,
    reports: mpsc::Sender<Report>,
}

impl HookWorld for WindowsWorld {
    fn focus(&self) -> FocusClass {
        self.cell.get()
    }

    fn layout(&self) -> LayoutFlags {
        LayoutFlags {
            ime_open: ime_open(),
            english_layout: is_english(foreground_layout()),
        }
    }

    fn recheck_focus(&self) {
        self.watch.recheck();
    }

    fn report(&self, report: Report) {
        // The main thread stops listening only as the process ends.
        if self.reports.send(report).is_ok()
            && let Err(error) = post_to(BACKGROUND_CLASS, AppMessage::Wake)
        {
            tracing::warn!(error = ?error, "could not wake the main thread");
        }
    }
}

/// The main thread's state: the preferences the menu shows, and the way to the session and the window process.
#[derive(Debug)]
struct Background {
    stores: Stores,
    /// Whether the preferences this run started with were read, and so may be saved, for the whole run.
    origin: Origin,
    /// The preferences as last loaded or stored.
    prefs: RefCell<PreferencesView>,
    macros: RefCell<Vec<MacroView>>,
    /// The last app in front the menu may offer, which the focus watch writes.
    last_app: Arc<Mutex<Option<String>>>,
    updates: mpsc::Sender<SessionUpdate>,
    reports: mpsc::Receiver<Report>,
    tray: OnceCell<Tray>,
    /// The window process this one started last, waited on rather than left behind.
    window_process: RefCell<Option<Child>>,
}

impl Background {
    /// Adds the icon. Its handler holds a weak reference, since the tray it is part of is held here.
    fn show_tray(self: &Rc<Self>, language: LanguageValue) -> Result<(), StartError> {
        let weak = Rc::downgrade(self);
        let tray = Tray::add(tray_icon(language), labels::WINDOW_TITLE, move |event| {
            if let Some(background) = weak.upgrade() {
                background.on_tray(event);
            }
        })
        .map_err(StartError::Tray)?;
        // Only this function sets the cell, once.
        drop(self.tray.set(tray));
        Ok(())
    }

    fn on_message(&self, message: AppMessage) {
        match message {
            AppMessage::Wake => self.take_reports(),
            AppMessage::Reload => self.reload(),
            AppMessage::ShowWindow(kind) => self.open_window(window_kind(kind)),
            // The message window ends the loop itself; the windows go with the background.
            AppMessage::Quit => tell_window_process(AppMessage::Quit),
        }
    }

    fn take_reports(&self) {
        while let Ok(report) = self.reports.try_recv() {
            match report {
                Report::SwitchedLanguage(prefs) => self.switched_language(prefs),
                Report::Remembered(prefs) => self.keep(prefs),
                Report::OpenSettings => self.open_window(WindowKind::Settings),
                Report::QuickConvert => self.quick_convert(),
            }
        }
    }

    fn switched_language(&self, prefs: PreferencesView) {
        if prefs.switching.beep == Toggle::On {
            beep();
        }
        self.keep(prefs);
    }

    /// Shows the language of the session's `session` preferences and stores what the session changed over the stored
    /// ones.
    fn keep(&self, session: PreferencesView) {
        self.show_language(session.language);
        let saved = self.save_over_stored(|stored| rebase_session(stored, &session));
        *self.prefs.borrow_mut() = saved.unwrap_or(session);
    }

    /// Stores a click on the menu entry `id` over the stored preferences, and hands the result to the session.
    fn keep_menu_change(&self, id: &MenuId) {
        // A hotkey may have changed the preferences while the menu was open, so the click applies to them as they are.
        let shown = self.prefs.borrow().clone();
        let prefs = self
            .save_over_stored(|stored| rebase_menu_change(stored, &shown, id.clone()))
            .unwrap_or_else(|| rebase_menu_change(shown.clone(), &shown, id.clone()));
        self.show_language(prefs.language);
        self.send_to_session(prefs.clone(), self.macros.borrow().clone());
        *self.prefs.borrow_mut() = prefs;
    }

    /// Saves `edit` of the stored preferences as they are now, tells the window process, and returns what was saved.
    /// Nothing is saved in a run that started on the defaults, nor when the stored preferences cannot be read now, since
    /// the save would put the defaults or an older copy in place of what the user set.
    fn save_over_stored(&self, edit: impl FnOnce(PreferencesView) -> PreferencesView) -> Option<PreferencesView> {
        if !self.origin.may_save() {
            return None;
        }
        let stored = match self.stores.load_preferences() {
            Ok(stored) => stored,
            Err(error) => {
                tracing::warn!(error = ?error, "could not load the preferences to save over");
                notify(load_error_message(&error));
                return None;
            }
        };
        let prefs = edit(stored);
        match self.stores.save_preferences(&prefs) {
            Ok(()) => {
                tell_window_process(AppMessage::Reload);
                Some(prefs)
            }
            Err(error) => {
                tracing::warn!(error = ?error, "could not save the preferences");
                notify(save_error_message(&error));
                None
            }
        }
    }

    /// Reads the preferences and macros the window process stored, and hands them to the session.
    fn reload(&self) {
        let prefs = match self.stores.load_preferences() {
            Ok(prefs) => prefs,
            Err(error) => {
                tracing::warn!(error = ?error, "could not load the preferences");
                notify(load_error_message(&error));
                return;
            }
        };
        let macros = self.stores.load_macros().unwrap_or_else(|error| {
            tracing::warn!(error = ?error, "could not load the macros");
            notify(load_macros_error_message(&error));
            self.macros.borrow().clone()
        });
        apply_system_changes(system_changes(&self.prefs.borrow().system, &prefs.system));
        self.show_language(prefs.language);
        self.send_to_session(prefs.clone(), macros.clone());
        *self.prefs.borrow_mut() = prefs;
        *self.macros.borrow_mut() = macros;
    }

    fn send_to_session(&self, prefs: PreferencesView, macros: Vec<MacroView>) {
        // The hook stops only after the message loop, so it is there to receive.
        if self.updates.send(SessionUpdate::Apply { prefs, macros }).is_err() {
            tracing::warn!("the session has stopped");
        }
    }

    fn show_language(&self, language: LanguageValue) {
        if let Some(tray) = self.tray.get()
            && let Err(error) = tray.set_icon(tray_icon(language))
        {
            tracing::warn!(error = ?error, "could not change the icon");
        }
    }

    fn on_tray(&self, event: TrayEvent) {
        match event {
            TrayEvent::Activate => self.open_window(WindowKind::Settings),
            TrayEvent::MenuRequested => self.show_menu(),
        }
    }

    fn show_menu(&self) {
        let Some(tray) = self.tray.get() else {
            return;
        };
        let last_app = self.last_app.lock().ok().and_then(|app| app.clone());
        let map = MenuMap::new(&model(&self.prefs.borrow(), last_app.as_deref()));
        let chosen = match tray.show_menu(map.items()) {
            Ok(chosen) => chosen,
            Err(error) => {
                tracing::warn!(error = ?error, "could not show the menu");
                return;
            }
        };
        let Some(id) = chosen.and_then(|id| map.chosen(id)).cloned() else {
            return;
        };
        // This only tells what the click does; a change is made again on the stored preferences when it is kept.
        let effect = apply(&mut self.prefs.borrow().clone(), id.clone());
        match effect {
            MenuEffect::Save => self.keep_menu_change(&id),
            MenuEffect::Open(kind) => self.open_window(kind),
            MenuEffect::QuickConvert => self.quick_convert(),
            MenuEffect::Exit => quit(),
            MenuEffect::None => {}
        }
    }

    /// Shows `kind` in the window process, starting one when none answers.
    fn open_window(&self, kind: WindowKind) {
        if post_to(UI_CLASS, AppMessage::ShowWindow(shown_window(kind))).is_ok() {
            return;
        }
        match start_window_process(kind) {
            Ok(child) => {
                if let Some(mut previous) = self.window_process.replace(Some(child))
                    && let Err(error) = previous.try_wait()
                {
                    tracing::warn!(error = ?error, "could not check on the previous window process");
                }
            }
            Err(error) => {
                tracing::warn!(error = ?error, "could not start the window process");
                notify(notice_message(Notice::WindowNotOpened));
            }
        }
    }

    fn quick_convert(&self) {
        match self.stores.quick_convert() {
            Ok(result) => {
                if result.notify {
                    notify(notice_message(Notice::ConvertDone));
                }
            }
            Err(error) => {
                tracing::warn!(error = ?error, "quick convert failed");
                notify(convert_error_message(&error));
            }
        }
    }
}

/// The preferences to store for the session's `session` ones: the fresh `stored` ones, with only what the session
/// changes itself taken from it: the language, the code table in force, and the apps it remembers. The window process
/// keeps the rest of the store current, so a hotkey or an app switch that lands before its `Reload` arrives must not
/// put back the settings that save replaced.
pub(crate) fn rebase_session(stored: PreferencesView, session: &PreferencesView) -> PreferencesView {
    let mut prefs = PreferencesView {
        language: session.language,
        app_memory: session.app_memory.clone(),
        ..stored
    };
    prefs.typing.code_table = session.typing.code_table;
    prefs
}

/// The preferences to store for a click on the menu entry `id`: the click applied to the `stored` preferences rebased
/// on `shown`, the preferences the menu was shown for.
pub(crate) fn rebase_menu_change(stored: PreferencesView, shown: &PreferencesView, id: MenuId) -> PreferencesView {
    let mut prefs = rebase_session(stored, shown);
    // The caller already knows the click is a change to store.
    apply(&mut prefs, id);
    prefs
}

fn start_window_process(kind: WindowKind) -> io::Result<Child> {
    Command::new(std::env::current_exe()?).args(window_args(kind)).spawn()
}

/// Ends the message loop through the message window, as an installer's `Quit` does.
fn quit() {
    if let Err(error) = post_to(BACKGROUND_CLASS, AppMessage::Quit) {
        tracing::warn!(error = ?error, "could not ask the message loop to end");
    }
}

/// Posts `message` to the window process, when one runs.
fn tell_window_process(message: AppMessage) {
    if let Err(error) = post_to(UI_CLASS, message) {
        tracing::debug!(error = ?error, "the window process did not take the message");
    }
}

fn apply_system_changes(changes: SystemChanges) {
    if let Some(mode) = changes.startup {
        apply_startup_mode(mode);
    }
    if let Some(shortcut) = changes.shortcut {
        put_shortcut(shortcut);
    }
}

fn apply_startup_mode(mode: StartupMode) {
    let notice = match apply_startup(mode) {
        Ok(()) => return,
        Err(StartupError::NeedsElevation) => Notice::StartupNeedsAdministrator,
        Err(error @ (StartupError::CurrentExe(_) | StartupError::Registry { .. } | StartupError::Task { .. })) => {
            tracing::warn!(error = ?error, "could not change how the app starts with Windows");
            Notice::StartupNotChanged
        }
    };
    notify(notice_message(notice));
}

fn put_shortcut(shortcut: Toggle) {
    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(error) => {
            tracing::warn!(error = ?error, "could not find the program's own path");
            notify(notice_message(Notice::ShortcutNotChanged));
            return;
        }
    };
    if let Err(error) = set_desktop_shortcut(shortcut == Toggle::On, &exe) {
        tracing::warn!(error = ?error, "could not change the desktop shortcut");
        notify(notice_message(Notice::ShortcutNotChanged));
    }
}

fn notify(text: &str) {
    show_notice(labels::NOTICE_TITLE, text);
}

#[cfg(test)]
mod tests;
