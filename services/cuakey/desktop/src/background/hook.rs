//! The keyboard hook's side of the background process. The handler owns the typing session and answers each key at
//! once: it reads and writes no file, and leaves the stores, the icon and the other process to the main thread.
//! Everything else reaches it through one channel, read at the start of each event, and what it cannot do itself it
//! reports to the main thread.

use std::sync::mpsc;

use cuakey_application::{CommandValue, LayoutFlags, MacroView, PreferencesView, Toggle, TypingSession};
use platform_windows::{FocusChange, FocusClass, HookEvent, HookReply, RawKey, focus_moving_key};

use crate::keymap::{focus_value, is_modifier, key_command, send_ops};

/// The frame every Microsoft Store app's window sits in.
const STORE_APP_FRAME: &str = "applicationframehost.exe";

/// What the rest of the process hands the session.
#[derive(Debug)]
pub(crate) enum SessionUpdate {
    /// The preferences and macros as stored now.
    Apply {
        prefs: PreferencesView,
        macros: Vec<MacroView>,
    },
    /// A classification of the focused control from the focus watch.
    Focus(FocusChange),
}

/// What the session asks of the main thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Report {
    /// The language hotkey switched the language: beep, show it, and store the session's preferences, given here.
    SwitchedLanguage(PreferencesView),
    /// An app switch changed what the session would store; these are its preferences now.
    Remembered(PreferencesView),
    OpenSettings,
    QuickConvert,
}

/// What the handler reads from Windows and tells the rest of the process, so a test can stand in for both.
pub(crate) trait HookWorld {
    fn focus(&self) -> FocusClass;
    fn layout(&self) -> LayoutFlags;
    /// Marks the focus pending and has the focus watch classify it again.
    fn recheck_focus(&self);
    /// Hands `report` to the main thread and wakes it.
    fn report(&self, report: Report);
}

/// The keyboard hook's handler.
#[derive(Debug)]
pub(crate) struct HookHandler {
    session: TypingSession,
    updates: mpsc::Receiver<SessionUpdate>,
    /// The app in front, as the last classification named it.
    app: String,
    store_app_support: Toggle,
}

impl HookHandler {
    pub(crate) fn new(session: TypingSession, updates: mpsc::Receiver<SessionUpdate>) -> Self {
        let store_app_support = session.preferences().compatibility.store_app_support;
        Self {
            session,
            updates,
            app: String::new(),
            store_app_support,
        }
    }

    /// The answer to one event from the hook.
    pub(crate) fn handle(&mut self, event: HookEvent, world: &impl HookWorld) -> HookReply {
        self.take_updates(world);
        match event {
            HookEvent::MouseDown => {
                self.session.mouse_click();
                world.recheck_focus();
                HookReply::default()
            }
            HookEvent::Key(key) => self.handle_key(key, world),
        }
    }

    fn handle_key(&mut self, key: RawKey, world: &impl HookWorld) -> HookReply {
        let command = key_command(key, focus_value(world.focus()), &self.app, world.layout());
        let result = self.session.handle_key(command);
        // The key itself was judged by the control it was typed into; whatever comes after it may not be.
        if rechecks_focus(key, result.swallow) {
            world.recheck_focus();
        }
        match result.command {
            Some(CommandValue::SwitchLanguage) => world.report(Report::SwitchedLanguage(self.session.preferences())),
            Some(CommandValue::OpenSettings) => world.report(Report::OpenSettings),
            Some(CommandValue::QuickConvert) => world.report(Report::QuickConvert),
            None => {}
        }
        HookReply {
            swallow: result.swallow,
            ops: send_ops(result.actions, key),
            store_app: store_app(&self.app, self.store_app_support),
        }
    }

    fn take_updates(&mut self, world: &impl HookWorld) {
        while let Ok(update) = self.updates.try_recv() {
            match update {
                SessionUpdate::Apply { prefs, macros } => self.apply(&prefs, &macros),
                SessionUpdate::Focus(change) => self.focus(change, world),
            }
        }
    }

    fn apply(&mut self, prefs: &PreferencesView, macros: &[MacroView]) {
        match self.session.apply(prefs, macros) {
            Ok(()) => self.store_app_support = prefs.compatibility.store_app_support,
            // The main thread read both from the stores, which hold only what was valid; the session keeps what it had.
            Err(error) => tracing::warn!(error = ?error, "the session refused the stored preferences"),
        }
    }

    /// Every classification starts a new word: the focus may have moved to another control of the same kind, and the
    /// word typed in the old one must never be edited in the new one.
    fn focus(&mut self, change: FocusChange, world: &impl HookWorld) {
        self.session.focus_changed();
        if change.app_changed {
            let before = self.session.preferences();
            self.session.app_switched(&change.app);
            let after = self.session.preferences();
            if switch_needs_save(&before, &after) {
                world.report(Report::Remembered(after));
            }
        }
        self.app = change.app;
    }
}

/// Whether the app in front gets each backspace as a character too: only a Microsoft Store app, and only with the
/// setting on, since every other app would see two backspaces.
pub(crate) fn store_app(app: &str, support: Toggle) -> bool {
    support == Toggle::On && app == STORE_APP_FRAME
}

/// Whether the hook has the focus classified again after `key`: a key that usually moves the focus, as it goes down.
/// Never for a modifier, which moves nothing by itself, nor for a key the session swallowed, which the app never saw.
/// Each classification starts a new word and forgets the hotkey being pressed, so a recheck in the middle of one, as
/// Shift with Ctrl held or F5 in Ctrl+Shift+F5 would ask for, would keep it from firing.
pub(crate) fn rechecks_focus(key: RawKey, swallowed: bool) -> bool {
    key.down && !swallowed && !is_modifier(key.vk) && focus_moving_key(key.vk, key.mods.ctrl_alt_win)
}

/// Whether an app switch changed what the store keeps for the session: the language or code table it adopted, or the
/// apps it remembers. Only then is the store written, so moving between known apps costs nothing.
pub(crate) fn switch_needs_save(before: &PreferencesView, after: &PreferencesView) -> bool {
    before.language != after.language
        || before.typing.code_table != after.typing.code_table
        || before.app_memory != after.app_memory
}

#[cfg(test)]
mod tests;
