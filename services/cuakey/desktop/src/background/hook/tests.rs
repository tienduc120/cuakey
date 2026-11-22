use std::cell::{Cell, RefCell};
use std::sync::mpsc;

use cuakey_application::{AppMemoryView, LanguageValue, LayoutFlags, PreferencesView, Toggle, TypingSession};
use platform_windows::{FocusChange, FocusClass, HookEvent, HookReply, RawKey, RawMods, SendOp};

use super::{HookHandler, HookWorld, Report, SessionUpdate, rechecks_focus, store_app, switch_needs_save};

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

fn key(event: HookEvent) -> RawKey {
    match event {
        HookEvent::Key(key) => key,
        HookEvent::MouseDown => panic!("not a key"),
    }
}

#[test]
fn a_modifier_never_asks_for_a_recheck() {
    // Shift, Ctrl, Alt and Win: either side, and the ones that name no side.
    let modifiers = [0x10, 0xa0, 0xa1, 0x11, 0xa2, 0xa3, 0x12, 0xa4, 0xa5, 0x5b, 0x5c];
    for vk in modifiers {
        for mods in [NO_MODS, CTRL_HELD] {
            for event in [down(vk, mods), up(vk, mods)] {
                assert!(!rechecks_focus(key(event), NOT_SWALLOWED), "{event:?}");
            }
        }
    }
}

const NOT_SWALLOWED: bool = false;

#[test]
fn other_keys_recheck_when_they_usually_move_the_focus() {
    assert!(rechecks_focus(key(down(TAB, NO_MODS)), NOT_SWALLOWED));
    assert!(rechecks_focus(key(down(A, CTRL_HELD)), NOT_SWALLOWED), "a shortcut");
    assert!(!rechecks_focus(key(down(A, NO_MODS)), NOT_SWALLOWED), "a letter");
    assert!(
        !rechecks_focus(key(up(TAB, NO_MODS)), NOT_SWALLOWED),
        "once, as it goes down"
    );
    assert!(
        !rechecks_focus(key(down(F5, CTRL_HELD)), true),
        "a key the session swallowed never reached the app"
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
fn the_modifiers_of_a_hotkey_never_ask_for_a_recheck() {
    let (mut handler, _updates) = handler();
    let world = FakeWorld::new();

    press(
        &mut handler,
        &world,
        &[
            down(LEFT_CTRL, NO_MODS),
            down(LEFT_SHIFT, CTRL_HELD),
            up(LEFT_SHIFT, CTRL_HELD),
            up(LEFT_CTRL, CTRL_HELD),
        ],
    );

    assert_eq!(world.rechecks.get(), 0, "a modifier alone never moves the focus");
}

#[test]
fn every_focus_report_starts_a_new_word() {
    let (mut handler, updates) = handler();
    let world = FakeWorld::new();
    updates
        .send(SessionUpdate::Focus(change("notepad.exe", true, FocusClass::Text)))
        .unwrap();
    press(&mut handler, &world, &[down(A, NO_MODS)]);

    // A program moved the focus to another text box of the same app, with no key or click.
    updates
        .send(SessionUpdate::Focus(change("notepad.exe", false, FocusClass::Text)))
        .unwrap();
    let reply = handler.handle(down(A, NO_MODS), &world);

    assert_eq!(
        reply,
        HookReply::default(),
        "the a starts a word of its own in the new box"
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
    // F5 never reached the app, so it moved nothing; a recheck's answer would forget the hotkey before F5 came up.
    assert_eq!(world.rechecks.get(), 0);
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
