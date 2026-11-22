use std::cell::{Cell, RefCell};
use std::sync::mpsc;

use cuakey_application::{AppMemoryView, LanguageValue, LayoutFlags, PreferencesView, Toggle, TypingSession};
use platform_windows::{FocusChange, FocusClass, HookEvent, HookReply, RawKey, RawMods, SendOp};

use super::{HookHandler, HookWorld, Report, SessionUpdate, focus_moved, store_app, switch_needs_save};

const NO_MODS: RawMods = RawMods {
    shift: false,
    ctrl_alt_win: false,
    caps_lock: false,
};
const CTRL_HELD: RawMods = RawMods {
    ctrl_alt_win: true,
    ..NO_MODS
};

const A: u16 = 0x41;
const TAB: u16 = 0x09;
const F5: u16 = 0x74;
const LEFT_CTRL: u16 = 0xa2;
const LEFT_SHIFT: u16 = 0xa0;

fn down(vk: u16, mods: RawMods) -> HookEvent {
    HookEvent::Key(RawKey {
        vk,
        scan: 0,
        down: true,
        extended: false,
        mods,
    })
}

fn up(vk: u16, mods: RawMods) -> HookEvent {
    match down(vk, mods) {
        HookEvent::Key(key) => HookEvent::Key(RawKey { down: false, ..key }),
        HookEvent::MouseDown => HookEvent::MouseDown,
    }
}

fn change(app: &str, app_changed: bool, class: FocusClass) -> FocusChange {
    FocusChange {
        app: app.to_owned(),
        app_changed,
        class,
    }
}

/// Windows as the handler sees it: a text box in front, an English layout, and a record of what it was told.
struct FakeWorld {
    focus: Cell<FocusClass>,
    rechecks: Cell<usize>,
    reports: RefCell<Vec<Report>>,
}

impl FakeWorld {
    fn new() -> Self {
        Self {
            focus: Cell::new(FocusClass::Text),
            rechecks: Cell::new(0),
            reports: RefCell::new(Vec::new()),
        }
    }
}

impl HookWorld for FakeWorld {
    fn focus(&self) -> FocusClass {
        self.focus.get()
    }

    fn layout(&self) -> LayoutFlags {
        LayoutFlags {
            ime_open: false,
            english_layout: true,
        }
    }

    fn recheck_focus(&self) {
        self.rechecks.set(self.rechecks.get() + 1);
    }

    fn report(&self, report: Report) {
        self.reports.borrow_mut().push(report);
    }
}

fn handler_with(prefs: &PreferencesView) -> (HookHandler, mpsc::Sender<SessionUpdate>) {
    let (updates, received) = mpsc::channel();
    let session = TypingSession::new(prefs, &[]).unwrap();
    (HookHandler::new(session, received), updates)
}

fn handler() -> (HookHandler, mpsc::Sender<SessionUpdate>) {
    handler_with(&PreferencesView::default())
}

fn press(handler: &mut HookHandler, world: &FakeWorld, events: &[HookEvent]) -> Vec<HookReply> {
    events.iter().map(|event| handler.handle(*event, world)).collect()
}

#[test]
fn store_app_needs_the_store_frame_and_the_setting() {
    assert!(store_app("applicationframehost.exe", Toggle::On));
    assert!(
        !store_app("applicationframehost.exe", Toggle::Off),
        "the setting is off"
    );
    assert!(!store_app("notepad.exe", Toggle::On), "only a store app's frame counts");
}

#[test]
fn focus_moves_to_a_new_app_or_a_new_kind_of_control() {
    assert!(
        focus_moved(None, &change("notepad.exe", true, FocusClass::Text)),
        "the first classification"
    );
    assert!(focus_moved(
        Some(FocusClass::Text),
        &change("chrome.exe", true, FocusClass::Text)
    ));
    assert!(focus_moved(
        Some(FocusClass::Text),
        &change("notepad.exe", false, FocusClass::Password)
    ));
    assert!(
        !focus_moved(Some(FocusClass::Text), &change("notepad.exe", false, FocusClass::Text)),
        "a recheck that finds the same kind of control in the same app"
    );
}

#[test]
fn an_app_switch_is_saved_only_when_it_changed_what_is_stored() {
    let before = PreferencesView::default();
    let mut remembered = before.clone();
    remembered.app_memory.push(AppMemoryView {
        app: "notepad.exe".to_owned(),
        language: before.language,
        code_table: before.typing.code_table,
    });
    let mut switched = before.clone();
    switched.language = LanguageValue::English;

    assert!(!switch_needs_save(&before, &before.clone()));
    assert!(switch_needs_save(&before, &remembered), "a new app is remembered");
    assert!(switch_needs_save(&before, &switched), "the app's language was adopted");
}

#[test]
fn a_mouse_click_rechecks_the_focus() {
    let (mut handler, _updates) = handler();
    let world = FakeWorld::new();

    let reply = handler.handle(HookEvent::MouseDown, &world);

    assert_eq!(reply, HookReply::default(), "the click goes through");
    assert_eq!(world.rechecks.get(), 1);
}

#[test]
fn a_focus_moving_key_rechecks_once_it_is_handled() {
    let (mut handler, _updates) = handler();
    let world = FakeWorld::new();

    press(&mut handler, &world, &[down(A, NO_MODS), up(A, NO_MODS)]);
    assert_eq!(world.rechecks.get(), 0, "a letter leaves the focus where it is");
    press(&mut handler, &world, &[down(TAB, NO_MODS), up(TAB, NO_MODS)]);
    assert_eq!(world.rechecks.get(), 1, "Tab moves it, once, as it goes down");
}

#[test]
fn typing_in_a_text_box_types_vietnamese() {
    let (mut handler, _updates) = handler();
    let world = FakeWorld::new();

    let replies = press(&mut handler, &world, &[down(A, NO_MODS), down(A, NO_MODS)]);

    let second = replies.last().unwrap();
    assert!(second.swallow, "the second a becomes part of the letter");
    assert!(second.ops.contains(&SendOp::Unit(0xe2)), "{:?}", second.ops);
}

#[test]
fn the_language_hotkey_reports_the_switch() {
    let (mut handler, _updates) = handler();
    let world = FakeWorld::new();

    press(
        &mut handler,
        &world,
        &[
            down(LEFT_CTRL, NO_MODS),
            down(LEFT_SHIFT, CTRL_HELD),
            up(LEFT_SHIFT, CTRL_HELD),
        ],
    );

    let reports = world.reports.borrow();
    let [Report::SwitchedLanguage(prefs)] = reports.as_slice() else {
        panic!("one switch is reported: {reports:?}");
    };
    assert_eq!(prefs.language, LanguageValue::English);
}

#[test]
fn a_recheck_while_the_hotkey_is_held_does_not_forget_it() {
    let (mut handler, updates) = handler();
    let world = FakeWorld::new();
    updates
        .send(SessionUpdate::Focus(change("notepad.exe", true, FocusClass::Text)))
        .unwrap();
    press(
        &mut handler,
        &world,
        &[down(LEFT_CTRL, NO_MODS), down(LEFT_SHIFT, CTRL_HELD)],
    );
    world.reports.borrow_mut().clear();

    // Shift went down with Ctrl held, which asked for a recheck; its answer finds the same text box.
    updates
        .send(SessionUpdate::Focus(change("notepad.exe", false, FocusClass::Text)))
        .unwrap();
    press(&mut handler, &world, &[up(LEFT_SHIFT, CTRL_HELD)]);

    assert!(
        matches!(world.reports.borrow().as_slice(), [Report::SwitchedLanguage(_)]),
        "{:?}",
        world.reports.borrow()
    );
}

#[test]
fn the_settings_hotkey_reports_open_settings() {
    let (mut handler, _updates) = handler();
    let world = FakeWorld::new();
    let ctrl_shift = RawMods {
        shift: true,
        ..CTRL_HELD
    };

    let replies = press(
        &mut handler,
        &world,
        &[
            down(LEFT_CTRL, NO_MODS),
            down(LEFT_SHIFT, CTRL_HELD),
            down(F5, ctrl_shift),
            up(F5, ctrl_shift),
        ],
    );

    assert!(replies.last().unwrap().swallow, "the app in front never sees F5");
    assert_eq!(*world.reports.borrow(), [Report::OpenSettings]);
}

#[test]
fn an_app_switch_that_remembers_the_app_reports_the_preferences() {
    let (mut handler, updates) = handler();
    let world = FakeWorld::new();

    updates
        .send(SessionUpdate::Focus(change("notepad.exe", true, FocusClass::Text)))
        .unwrap();
    handler.handle(HookEvent::MouseDown, &world);
    updates
        .send(SessionUpdate::Focus(change("notepad.exe", false, FocusClass::NotText)))
        .unwrap();
    handler.handle(HookEvent::MouseDown, &world);

    let reports = world.reports.borrow();
    let [Report::Remembered(prefs)] = reports.as_slice() else {
        panic!("only the new app is reported: {reports:?}");
    };
    assert_eq!(
        prefs.app_memory.first().map(|memory| memory.app.as_str()),
        Some("notepad.exe")
    );
}

#[test]
fn the_desktop_shell_is_never_remembered() {
    let (mut handler, updates) = handler();
    let world = FakeWorld::new();

    updates
        .send(SessionUpdate::Focus(change("explorer.exe", true, FocusClass::NotText)))
        .unwrap();
    handler.handle(HookEvent::MouseDown, &world);

    assert!(world.reports.borrow().is_empty(), "{:?}", world.reports.borrow());
}

#[test]
fn backspaces_go_to_a_store_app_as_characters_once_the_setting_is_on() {
    let (mut handler, updates) = handler();
    let world = FakeWorld::new();
    updates
        .send(SessionUpdate::Focus(change(
            "applicationframehost.exe",
            true,
            FocusClass::Text,
        )))
        .unwrap();
    assert!(
        !handler.handle(down(A, NO_MODS), &world).store_app,
        "the setting starts off"
    );

    let mut prefs = PreferencesView::default();
    prefs.compatibility.store_app_support = Toggle::On;
    updates
        .send(SessionUpdate::Apply {
            prefs,
            macros: Vec::new(),
        })
        .unwrap();

    assert!(handler.handle(down(A, NO_MODS), &world).store_app);
}

#[test]
fn keys_pass_untouched_while_the_focus_is_pending() {
    let (mut handler, _updates) = handler();
    let world = FakeWorld::new();
    world.focus.set(FocusClass::Pending);

    let replies = press(&mut handler, &world, &[down(A, NO_MODS), down(A, NO_MODS)]);

    assert!(
        replies.iter().all(|reply| *reply == HookReply::default()),
        "{replies:?}"
    );
}
