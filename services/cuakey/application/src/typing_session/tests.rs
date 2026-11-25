use cuakey_domain::{CodeTable, encode};

use super::TypingSession;
use crate::{
    AppMemoryView, AppSwitchResult, CodeTableValue, CommandValue, FocusValue, InputMethodValue, InvalidMacro,
    KeyCommand, KeyFlags, KeyName, KeyResult, LanguageValue, LayoutFlags, MacroView, OutputAction, PreferencesView,
    SessionError, Toggle,
};

const LETTERS: [KeyName; 26] = [
    KeyName::A,
    KeyName::B,
    KeyName::C,
    KeyName::D,
    KeyName::E,
    KeyName::F,
    KeyName::G,
    KeyName::H,
    KeyName::I,
    KeyName::J,
    KeyName::K,
    KeyName::L,
    KeyName::M,
    KeyName::N,
    KeyName::O,
    KeyName::P,
    KeyName::Q,
    KeyName::R,
    KeyName::S,
    KeyName::T,
    KeyName::U,
    KeyName::V,
    KeyName::W,
    KeyName::X,
    KeyName::Y,
    KeyName::Z,
];

const DIGITS: [KeyName; 10] = [
    KeyName::Digit0,
    KeyName::Digit1,
    KeyName::Digit2,
    KeyName::Digit3,
    KeyName::Digit4,
    KeyName::Digit5,
    KeyName::Digit6,
    KeyName::Digit7,
    KeyName::Digit8,
    KeyName::Digit9,
];

/// The key that types `c` on a US layout, and whether Shift is held for it.
fn key_for(c: char) -> (KeyName, bool) {
    let key = match c.to_ascii_lowercase() {
        letter @ 'a'..='z' => LETTERS[('a'..='z').position(|l| l == letter).unwrap()],
        digit @ '0'..='9' => DIGITS[('0'..='9').position(|d| d == digit).unwrap()],
        ' ' => KeyName::Space,
        other => panic!("no key types {other:?}"),
    };
    (key, c.is_ascii_uppercase())
}

fn macro_view(shortcut: &str, expansion: &str) -> MacroView {
    MacroView {
        shortcut: shortcut.to_owned(),
        expansion: expansion.to_owned(),
    }
}

/// A control on screen, driven key by key through the session the way the keyboard hook drives it.
struct Typist {
    session: TypingSession,
    screen: String,
    focus: FocusValue,
    app: String,
    layout: LayoutFlags,
    caps_lock: bool,
    results: Vec<KeyResult>,
}

impl Typist {
    fn new(prefs: &PreferencesView, macros: &[MacroView]) -> Self {
        Self {
            session: TypingSession::new(prefs, macros).unwrap(),
            screen: String::new(),
            focus: FocusValue::Text,
            app: "notepad.exe".to_owned(),
            layout: LayoutFlags {
                ime_open: false,
                english_layout: true,
            },
            caps_lock: false,
            results: Vec::new(),
        }
    }

    fn with_defaults() -> Self {
        Self::new(&PreferencesView::default(), &[])
    }

    fn with(prefs: &PreferencesView) -> Self {
        Self::new(prefs, &[])
    }

    fn focused(mut self, focus: FocusValue) -> Self {
        self.focus = focus;
        self
    }

    fn in_app(mut self, app: &str) -> Self {
        app.clone_into(&mut self.app);
        self
    }

    /// Types `text`, one key per character, and returns what the control shows.
    fn type_text(&mut self, text: &str) -> &str {
        for c in text.chars() {
            let (key, shift) = key_for(c);
            let typed = if shift == self.caps_lock {
                c.to_ascii_lowercase()
            } else {
                c.to_ascii_uppercase()
            };
            let flags = KeyFlags {
                shift,
                ctrl_alt_win: false,
                caps_lock: self.caps_lock,
            };
            self.tap(key, flags, Some(typed));
        }
        &self.screen
    }

    /// Presses and releases `key`; `typed` is what the app types when the key reaches it.
    fn tap(&mut self, key: KeyName, flags: KeyFlags, typed: Option<char>) {
        let down = self.event(key, true, flags);
        self.show(&down, typed);
        let up = self.event(key, false, flags);
        self.show(&up, None);
    }

    /// Presses `keys` in order and releases them in reverse, with the modifiers held so far, and returns every result.
    fn chord(&mut self, keys: &[KeyName]) -> Vec<KeyResult> {
        let mut flags = KeyFlags {
            caps_lock: self.caps_lock,
            ..KeyFlags::default()
        };
        let mut results = Vec::new();
        for &key in keys {
            results.push(self.event(key, true, flags));
            flags.shift |= key == KeyName::Shift;
            flags.ctrl_alt_win |= matches!(key, KeyName::Ctrl | KeyName::Alt | KeyName::Win);
        }
        for &key in keys.iter().rev() {
            results.push(self.event(key, false, flags));
        }
        results
    }

    fn event(&mut self, key: KeyName, down: bool, flags: KeyFlags) -> KeyResult {
        let result = self.session.handle_key(KeyCommand {
            key,
            down,
            flags,
            focus: self.focus,
            app: &self.app,
            layout: self.layout,
        });
        self.results.push(result.clone());
        result
    }

    /// What the app shows after one result: the key itself unless it was swallowed, then each action.
    fn show(&mut self, result: &KeyResult, typed: Option<char>) {
        if !result.swallow {
            self.put(typed);
        }
        for action in &result.actions {
            match action {
                // The selected character is replaced by what comes next.
                OutputAction::Backspace | OutputAction::SelectPreviousChar => {
                    self.screen.pop();
                }
                OutputAction::Unit(unit) => self.screen.push(char::from_u32(u32::from(*unit)).unwrap()),
                OutputAction::Paste(units) => self.screen.push_str(&String::from_utf16(units).unwrap()),
                OutputAction::ResendKey => self.put(typed),
            }
        }
    }

    fn put(&mut self, typed: Option<char>) {
        self.screen.extend(typed);
    }

    fn nothing_sent(&self) -> bool {
        self.results
            .iter()
            .all(|r| !r.swallow && r.actions.is_empty() && r.command.is_none())
    }
}

fn commands(results: &[KeyResult]) -> Vec<CommandValue> {
    results.iter().filter_map(|r| r.command).collect()
}

#[test]
fn text_box_converts() {
    let mut typist = Typist::with_defaults();

    let screen = typist.type_text("vieetj");

    assert_eq!(screen, "việt", "telex in a text box");
}

#[test]
fn shift_and_caps_lock_set_the_case() {
    let mut shifted = Typist::with_defaults();
    let mut caps = Typist::with_defaults();
    caps.caps_lock = true;

    let (first_capital, all_capitals) = (shifted.type_text("Vieetj").to_owned(), caps.type_text("vieetj"));

    assert_eq!(first_capital, "Việt", "shift on the first letter");
    assert_eq!(all_capitals, "VIỆT", "caps lock");
}

#[test]
fn password_passes_everything() {
    let mut typist =
        Typist::new(&PreferencesView::default(), &[macro_view("ko", "không")]).focused(FocusValue::Password);

    let screen = typist.type_text("vieetj ko ").to_owned();

    assert_eq!(screen, "vieetj ko ", "the password box gets every key as typed");
    assert!(typist.nothing_sent(), "nothing swallowed or sent: {:?}", typist.results);
}

#[test]
fn pending_passes() {
    let mut typist = Typist::with_defaults().focused(FocusValue::Pending);

    let screen = typist.type_text("vieetj").to_owned();

    assert_eq!(screen, "vieetj", "a control still being classified gets keys as typed");
    assert!(typist.nothing_sent(), "{:?}", typist.results);
}

#[test]
fn not_text_needs_always_on() {
    let always_on = PreferencesView {
        always_on: vec!["game.exe".to_owned()],
        ..PreferencesView::default()
    };
    let mut plain = Typist::with_defaults().focused(FocusValue::NotText).in_app("game.exe");
    let mut listed = Typist::with(&always_on).focused(FocusValue::NotText).in_app("game.exe");

    let (passed, converted) = (plain.type_text("vieetj").to_owned(), listed.type_text("vieetj"));

    assert_eq!(passed, "vieetj", "not a text box, and the app is not always on");
    assert_eq!(converted, "việt", "the app is always on");
}

#[test]
fn always_on_matches_any_case() {
    let prefs = PreferencesView {
        always_on: vec!["game.exe".to_owned()],
        ..PreferencesView::default()
    };
    let mut upper = Typist::with(&prefs).focused(FocusValue::NotText).in_app("Game.EXE");
    let mut path = Typist::with(&prefs)
        .focused(FocusValue::NotText)
        .in_app(r"C:\Games\Game.EXE");

    let (by_name, by_path) = (upper.type_text("vieetj").to_owned(), path.type_text("vieetj"));

    assert_eq!(by_name, "việt", "the file name in another case");
    assert_eq!(by_path, "việt", "a full path");
}

#[test]
fn other_layout_passes_when_option_off() {
    let mut prefs = PreferencesView::default();
    prefs.switching.work_in_other_languages = Toggle::Off;
    let mut typist = Typist::with(&prefs);
    typist.layout.english_layout = false;

    let screen = typist.type_text("vieetj").to_owned();

    assert_eq!(screen, "vieetj", "another keyboard language is active");
    assert!(typist.nothing_sent(), "{:?}", typist.results);
}

#[test]
fn other_layout_converts_when_option_on() {
    let mut typist = Typist::with_defaults();
    typist.layout.english_layout = false;

    let screen = typist.type_text("vieetj");

    assert_eq!(screen, "việt", "keeps working by default");
}

#[test]
fn ime_open_passes() {
    let mut typist = Typist::with_defaults();
    typist.layout.ime_open = true;

    let screen = typist.type_text("vieetj").to_owned();
    let chord = typist.chord(&[KeyName::Ctrl, KeyName::Shift]);

    assert_eq!(screen, "vieetj", "the input method editor gets every key");
    assert_eq!(commands(&chord), vec![], "no hotkey either");
    assert!(typist.nothing_sent(), "{:?}", typist.results);
}

/// Shuts one of the gates that let a key through to the app untouched.
type CloseGate = fn(&mut Typist);

#[test]
fn a_key_a_gate_passes_ends_the_word() {
    let mut prefs = PreferencesView::default();
    prefs.switching.work_in_other_languages = Toggle::Off;
    let gates: [(&str, CloseGate); 3] = [
        ("an open input method editor", |typist| {
            typist.layout.ime_open = true;
        }),
        ("another keyboard language", |typist| {
            typist.layout.english_layout = false;
        }),
        ("a control that is not a text box", |typist| {
            typist.focus = FocusValue::NotText;
        }),
    ];

    for (gate, close) in gates {
        let mut typist = Typist::with(&prefs);
        typist.type_text("vie");
        let open = (typist.layout, typist.focus);
        close(&mut typist);
        typist.type_text("x");
        (typist.layout, typist.focus) = open;

        let screen = typist.type_text("s");

        assert_eq!(
            screen, "viexs",
            "{gate}: the s after the passed x starts a word of its own"
        );
    }
}

#[test]
fn switch_hotkey_toggles_and_remembers_app() {
    let mut typist = Typist::with_defaults().in_app("zalo.exe");

    let results = typist.chord(&[KeyName::Ctrl, KeyName::Shift]);

    assert_eq!(
        commands(&results),
        vec![CommandValue::SwitchLanguage],
        "fires once, on release"
    );
    let prefs = typist.session.preferences();
    assert_eq!(prefs.language, LanguageValue::English, "the language flipped");
    assert_eq!(
        prefs.app_memory,
        vec![AppMemoryView {
            app: "zalo.exe".to_owned(),
            language: LanguageValue::English,
            code_table: CodeTableValue::Unicode,
        }],
        "smart switch remembers it for the app"
    );
    assert_eq!(typist.type_text("vieetj"), "vieetj", "english types keys as they are");
}

#[test]
fn switching_back_types_vietnamese_again() {
    let mut typist = Typist::with_defaults();

    typist.chord(&[KeyName::Ctrl, KeyName::Shift]);
    typist.chord(&[KeyName::Ctrl, KeyName::Shift]);

    assert_eq!(typist.type_text("vieetj"), "việt");
}

#[test]
fn settings_hotkey_opens_settings_and_hides_its_key() {
    let mut typist = Typist::with_defaults();

    let results = typist.chord(&[KeyName::Ctrl, KeyName::Shift, KeyName::F5]);

    assert_eq!(
        commands(&results),
        vec![CommandValue::OpenSettings],
        "opens settings, never also switches"
    );
    assert!(
        results[2].swallow && results[3].swallow,
        "the app never sees F5: {results:?}"
    );
}

#[test]
fn settings_hotkey_in_password_box_passes() {
    let mut typist = Typist::with_defaults().focused(FocusValue::Password);

    let results = typist.chord(&[KeyName::Ctrl, KeyName::Shift, KeyName::F5]);

    assert!(
        results.iter().all(|r| !r.swallow && r.command.is_none()),
        "a hotkey with a key is not intercepted in a password box: {results:?}"
    );
}

#[test]
fn focus_change_breaks_word() {
    let mut typist = Typist::with_defaults();
    typist.type_text("a");

    typist.session.focus_changed();

    assert_eq!(typist.type_text("s"), "as", "the s starts a new word, so it is no tone");
}

#[test]
fn w_right_after_a_focus_change_stays_w() {
    let mut typist = Typist::with_defaults();
    typist.type_text("u");

    typist.session.focus_changed();

    assert_eq!(typist.type_text("w"), "uw", "a w that starts a word stays w");
}

#[test]
fn a_click_breaks_the_word() {
    let mut typist = Typist::with_defaults();
    typist.type_text("a");

    typist.session.mouse_click();

    assert_eq!(typist.type_text("s"), "as");
}

#[test]
fn a_language_switch_breaks_the_word() {
    let mut typist = Typist::with_defaults();
    typist.type_text("a");

    typist.chord(&[KeyName::Ctrl, KeyName::Shift]);
    typist.chord(&[KeyName::Ctrl, KeyName::Shift]);

    assert_eq!(typist.type_text("s"), "as");
}

#[test]
fn a_focus_change_forgets_a_half_pressed_hotkey() {
    let mut typist = Typist::with_defaults();
    typist.event(KeyName::Ctrl, true, KeyFlags::default());
    typist.event(KeyName::Shift, true, KeyFlags::default());

    typist.session.focus_changed();
    let released = typist.event(KeyName::Shift, false, KeyFlags::default());

    assert_eq!(released.command, None, "a key-up after the focus moved fires nothing");
}

#[test]
fn a_modifier_alone_does_not_end_the_word() {
    let mut typist = Typist::with_defaults();
    typist.type_text("vie");

    typist.chord(&[KeyName::Shift]);

    assert_eq!(typist.type_text("etj"), "việt", "shift never reaches the typing engine");
}

#[test]
fn a_chord_ends_the_word() {
    let mut typist = Typist::with_defaults();
    typist.type_text("a");
    let ctrl = KeyFlags {
        ctrl_alt_win: true,
        ..KeyFlags::default()
    };

    typist.tap(KeyName::C, ctrl, None);

    assert_eq!(typist.type_text("s"), "as", "ctrl+c is for the app and ends the word");
}

#[test]
fn english_macros_replace_the_word() {
    let mut prefs = PreferencesView {
        language: LanguageValue::English,
        ..PreferencesView::default()
    };
    prefs.macros.macros_in_english = Toggle::On;
    let mut typist = Typist::new(&prefs, &[macro_view("ko", "không")]);

    let screen = typist.type_text("ko ");

    assert_eq!(screen, "không ", "the expansion replaces the shortcut in english too");
}

#[test]
fn chromium_fix_selects_the_previous_character_in_chrome() {
    let mut prefs = PreferencesView::default();
    prefs.compatibility.chromium_fix = Toggle::On;
    let mut typist = Typist::with(&prefs).in_app("chrome.exe");
    typist.type_text("a");

    let result = typist.event(KeyName::S, true, KeyFlags::default());

    assert_eq!(
        result.actions,
        vec![OutputAction::SelectPreviousChar, OutputAction::Unit(0xe1)],
        "select the a, then type á over it"
    );
}

#[test]
fn clipboard_input_pastes_the_replacement() {
    let mut prefs = PreferencesView::default();
    prefs.compatibility.clipboard_input = Toggle::On;
    prefs.compatibility.fix_autocomplete = Toggle::Off;
    let mut typist = Typist::with(&prefs);
    typist.type_text("a");

    let result = typist.event(KeyName::S, true, KeyFlags::default());

    assert_eq!(
        result.actions,
        vec![OutputAction::Backspace, OutputAction::Paste(vec![0xe1])]
    );
}

#[test]
fn the_output_is_written_in_the_code_table() {
    let mut prefs = PreferencesView::default();
    prefs.typing.code_table = CodeTableValue::VniWindows;
    prefs.compatibility.fix_autocomplete = Toggle::Off;
    let mut typist = Typist::with(&prefs);
    typist.type_text("a");

    let result = typist.event(KeyName::S, true, KeyFlags::default());

    let mut expected = vec![OutputAction::Backspace];
    expected.extend(encode("á", CodeTable::VniWindows).into_iter().map(OutputAction::Unit));
    assert_eq!(result.actions, expected);
}

#[test]
fn an_app_switch_restores_what_the_app_last_used() {
    let prefs = PreferencesView {
        app_memory: vec![AppMemoryView {
            app: "zalo.exe".to_owned(),
            language: LanguageValue::English,
            code_table: CodeTableValue::Unicode,
        }],
        ..PreferencesView::default()
    };
    let mut typist = Typist::with(&prefs).in_app("zalo.exe");

    let switched = typist.session.app_switched(r"C:\Apps\Zalo.exe");

    assert_eq!(
        switched,
        AppSwitchResult {
            language_changed: true,
            code_table_changed: false,
        }
    );
    assert_eq!(typist.type_text("vieetj"), "vieetj", "zalo was last used in english");
}

#[test]
fn apply_takes_the_new_settings() {
    let mut typist = Typist::with_defaults();
    let mut prefs = PreferencesView::default();
    prefs.typing.input_method = InputMethodValue::Vni;

    typist.session.apply(&prefs, &[], None).unwrap();

    assert_eq!(typist.type_text("vie6t5"), "việt", "vni now");
}

#[test]
fn a_code_table_chosen_for_an_app_stays_with_it() {
    let mut typist = Typist::with_defaults();
    typist.session.app_switched("word.exe");
    let mut chosen = typist.session.preferences();
    chosen.typing.code_table = CodeTableValue::VniWindows;

    typist.session.apply(&chosen, &[], Some("word.exe")).unwrap();
    typist.session.app_switched("zalo.exe");
    typist.session.app_switched("word.exe");

    assert_eq!(
        typist.session.preferences().typing.code_table,
        CodeTableValue::VniWindows,
        "coming back to word keeps the code table chosen while it was in front"
    );
}

#[test]
fn settings_change_from_cuakey_window_keeps_previous_apps_language() {
    let prefs = PreferencesView {
        app_memory: vec![
            AppMemoryView {
                app: "code.exe".to_owned(),
                language: LanguageValue::English,
                code_table: CodeTableValue::Unicode,
            },
            // Written by an older version that treated the settings window as a program.
            AppMemoryView {
                app: "cuakey.exe".to_owned(),
                language: LanguageValue::Vietnamese,
                code_table: CodeTableValue::Unicode,
            },
        ],
        ..PreferencesView::default()
    };
    let mut typist = Typist::with(&prefs);
    typist.session.app_switched("code.exe");
    typist.session.app_switched("cuakey.exe");
    let mut chosen = typist.session.preferences();
    chosen.typing.code_table = CodeTableValue::VniWindows;

    typist.session.apply(&chosen, &[], Some("code.exe")).unwrap();

    let after = typist.session.preferences();
    let code = after.app_memory.iter().find(|memory| memory.app == "code.exe").unwrap();
    assert_eq!(code.language, LanguageValue::English, "language");
    assert_eq!(code.code_table, CodeTableValue::VniWindows, "code table");
}

#[test]
fn a_change_that_keeps_the_language_and_code_table_leaves_the_app_memory_alone() {
    let remembered = AppMemoryView {
        app: "word.exe".to_owned(),
        language: LanguageValue::English,
        code_table: CodeTableValue::Unicode,
    };
    let mut prefs = PreferencesView {
        app_memory: vec![remembered.clone()],
        ..PreferencesView::default()
    };
    let mut typist = Typist::with(&prefs);
    prefs.typing.input_method = InputMethodValue::Vni;

    typist.session.apply(&prefs, &[], Some("word.exe")).unwrap();

    assert_eq!(typist.session.preferences().app_memory, vec![remembered]);
}

#[test]
fn an_invalid_macro_is_refused() {
    let made = TypingSession::new(&PreferencesView::default(), &[macro_view("k o", "không")]);

    assert!(
        matches!(&made, Err(SessionError::Macro(InvalidMacro::Whitespace(s))) if s == "k o"),
        "{made:?}"
    );
}
