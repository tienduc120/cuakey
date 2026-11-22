use cuakey_domain::{
    AppName, Composer, ComposerSettings, Edit, FocusState, HotkeyAction, HotkeyBindings, HotkeyKey, HotkeyMatcher,
    KeyCode, KeyDirection, KeyMods, KeyPress, Language, Modifier, OnOff, OutputContext, OutputOp, Preferences,
    plan_output, should_convert,
};

use crate::macros::macro_table;
use crate::{InvalidMacro, MacroView, PreferencesView, UpdateError};

/// Declares `KeyName` with one variant per domain key code plus the four modifiers, and the conversions between them.
macro_rules! key_names {
    ($($key:ident),* $(,)?) => {
        /// A key as the keyboard hook reports it: a key the typing engine knows, a modifier, or `Other`.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum KeyName {
            $($key,)*
            Ctrl,
            Alt,
            Win,
            Shift,
        }

        impl KeyName {
            /// The key as the hotkey matcher sees it: modifiers apart from the keys that may type.
            pub(crate) const fn to_hotkey_key(self) -> HotkeyKey {
                match self {
                    $(Self::$key => HotkeyKey::Key(KeyCode::$key),)*
                    Self::Ctrl => HotkeyKey::Modifier(Modifier::Ctrl),
                    Self::Alt => HotkeyKey::Modifier(Modifier::Alt),
                    Self::Win => HotkeyKey::Modifier(Modifier::Win),
                    Self::Shift => HotkeyKey::Modifier(Modifier::Shift),
                }
            }
        }

        impl From<KeyCode> for KeyName {
            fn from(code: KeyCode) -> Self {
                match code {
                    $(KeyCode::$key => Self::$key),*
                }
            }
        }
    };
}

key_names!(
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Space,
    Backspace,
    Enter,
    Tab,
    Escape,
    Minus,
    Equal,
    LeftBracket,
    RightBracket,
    Backslash,
    Semicolon,
    Quote,
    Comma,
    Period,
    Slash,
    Backquote,
    Other,
);

mirror_enum! {
    /// What is known about the control that has the keyboard focus.
    FocusValue mirrors FocusState { Pending, Text, Password, NotText, Unknown }
}

mirror_enum! {
    /// What a hotkey asks the app to do.
    CommandValue mirrors HotkeyAction { SwitchLanguage, OpenSettings, QuickConvert }
}

/// The modifiers held and Caps Lock as one key goes, as the platform saw them at that key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyFlags {
    pub shift: bool,
    /// Ctrl, Alt or Win held: the key is a shortcut for the app, never text.
    pub ctrl_alt_win: bool,
    pub caps_lock: bool,
}

impl KeyFlags {
    const fn key_mods(self) -> KeyMods {
        KeyMods::none()
            .with_shift(on_off(self.shift))
            .with_caps_lock(on_off(self.caps_lock))
            .with_ctrl_or_alt(on_off(self.ctrl_alt_win))
    }
}

/// The input language the focused app is using.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LayoutFlags {
    /// An input method editor (Chinese, Japanese, Korean) is composing text.
    pub ime_open: bool,
    /// The active keyboard layout is an English one.
    pub english_layout: bool,
}

/// One key event from the keyboard hook. `app` is the executable of the app in front, borrowed so a key costs no
/// allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyCommand<'a> {
    pub key: KeyName,
    pub down: bool,
    pub flags: KeyFlags,
    pub focus: FocusValue,
    pub app: &'a str,
    pub layout: LayoutFlags,
}

/// One step of sending a replacement to the focused app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputAction {
    Backspace,
    /// A UTF-16 code unit typed as text.
    Unit(u16),
    /// Shift+Left, which selects the character before the caret so the next key replaces it.
    SelectPreviousChar,
    /// Text pasted through the clipboard, as UTF-16 code units.
    Paste(Vec<u16>),
    /// The key that came in, sent again after the replacement.
    ResendKey,
}

impl From<OutputOp> for OutputAction {
    fn from(op: OutputOp) -> Self {
        match op {
            OutputOp::Backspace => Self::Backspace,
            OutputOp::Unit(unit) => Self::Unit(unit),
            OutputOp::SelectPreviousChar => Self::SelectPreviousChar,
            OutputOp::Paste(units) => Self::Paste(units),
            OutputOp::ResendKey => Self::ResendKey,
        }
    }
}

/// What the keyboard hook does with one key event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyResult {
    /// The app must not see the key.
    pub swallow: bool,
    /// What to send in its place, in order.
    pub actions: Vec<OutputAction>,
    /// A hotkey fired.
    pub command: Option<CommandValue>,
}

impl KeyResult {
    /// The key goes to the app untouched.
    const fn pass() -> Self {
        Self {
            swallow: false,
            actions: Vec::new(),
            command: None,
        }
    }
}

/// What changed when another app came to the front.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AppSwitchResult {
    pub language_changed: bool,
    pub code_table_changed: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("the preferences are invalid")]
    Preferences(#[from] UpdateError),
    #[error("a macro is invalid")]
    Macro(#[from] InvalidMacro),
}

/// The browsers that take a replacement better when the character before the caret is selected first.
const CHROMIUM_APPS: [&str; 3] = ["chrome.exe", "msedge.exe", "brave.exe"];

const fn on_off(on: bool) -> OnOff {
    if on { OnOff::On } else { OnOff::Off }
}

/// What the session knows of the app in front, worked out when the app changes rather than on every key.
#[derive(Debug)]
struct FrontApp {
    /// The executable as the keyboard hook names it.
    raw: String,
    always_on: OnOff,
    chromium: OnOff,
}

impl FrontApp {
    fn of(raw: String, preferences: &Preferences) -> Self {
        let (always_on, chromium) = AppName::new(&raw).map_or((false, false), |name| {
            (preferences.is_always_on(&name), CHROMIUM_APPS.contains(&name.as_str()))
        });
        Self {
            raw,
            always_on: on_off(always_on),
            chromium: on_off(chromium),
        }
    }
}

/// Turns each key event into what to send: hotkeys first, then whether the focused control takes Vietnamese, then the
/// typing engine and the keys that put its edit on screen.
#[derive(Debug)]
pub struct TypingSession {
    preferences: Preferences,
    composer: Composer,
    hotkeys: HotkeyMatcher,
    bindings: HotkeyBindings,
    front: FrontApp,
    /// The word before the key, kept in one buffer so a key allocates nothing once typing is under way.
    word_before: String,
}

impl TypingSession {
    pub fn new(prefs: &PreferencesView, macros: &[MacroView]) -> Result<Self, SessionError> {
        let preferences = prefs.to_preferences()?;
        let mut composer = Composer::new(ComposerSettings::from_settings(
            preferences.settings(),
            preferences.language(),
        ));
        composer.set_macros(macro_table(macros)?);
        Ok(Self {
            bindings: HotkeyBindings::from_settings(preferences.settings()),
            front: FrontApp::of(String::new(), &preferences),
            preferences,
            composer,
            hotkeys: HotkeyMatcher::new(),
            word_before: String::new(),
        })
    }

    pub fn handle_key(&mut self, cmd: KeyCommand<'_>) -> KeyResult {
        if cmd.layout.ime_open {
            return KeyResult::pass();
        }
        let focus = FocusState::from(cmd.focus);
        let key = cmd.key.to_hotkey_key();
        let direction = if cmd.down { KeyDirection::Down } else { KeyDirection::Up };
        let password_box = on_off(focus == FocusState::Password);
        let outcome = self.hotkeys.on_key(key, direction, self.bindings, password_box);
        if let Some(action) = outcome.fire() {
            if action == HotkeyAction::SwitchLanguage {
                self.switch_language(cmd.app);
            }
            return KeyResult {
                swallow: outcome.swallow(),
                actions: Vec::new(),
                command: Some(action.into()),
            };
        }
        if outcome.swallow() {
            return KeyResult {
                swallow: true,
                ..KeyResult::pass()
            };
        }
        match key {
            HotkeyKey::Key(code) if cmd.down => self.type_key(code, focus, cmd),
            // A modifier never reaches the typing engine, which would end the word on it; a key-up types nothing.
            HotkeyKey::Key(_) | HotkeyKey::Modifier(_) => KeyResult::pass(),
        }
    }

    pub fn mouse_click(&mut self) {
        self.composer.break_word();
    }

    /// Starts a new word. A key-up lost while the focus moved must not fire a hotkey later, so the chord is forgotten.
    pub fn focus_changed(&mut self) {
        self.composer.break_word();
        self.hotkeys.reset();
    }

    /// Adopts what the app last used, as the settings ask, and starts a new word.
    pub fn app_switched(&mut self, app: &str) -> AppSwitchResult {
        let Ok(name) = AppName::new(app) else {
            return AppSwitchResult::default();
        };
        let language = self.preferences.language();
        let code_table = self.preferences.settings().typing().code_table();
        self.preferences.apply_app_switch(&name);
        self.refresh_composer();
        AppSwitchResult {
            language_changed: self.preferences.language() != language,
            code_table_changed: self.preferences.settings().typing().code_table() != code_table,
        }
    }

    /// Takes new preferences and macros. The word being typed and any half-pressed hotkey are dropped with the old ones.
    pub fn apply(&mut self, prefs: &PreferencesView, macros: &[MacroView]) -> Result<(), SessionError> {
        *self = Self::new(prefs, macros)?;
        Ok(())
    }

    pub fn preferences(&self) -> PreferencesView {
        PreferencesView::of(&self.preferences)
    }

    fn type_key(&mut self, code: KeyCode, focus: FocusState, cmd: KeyCommand<'_>) -> KeyResult {
        if self.front.raw != cmd.app {
            self.front = FrontApp::of(cmd.app.to_owned(), &self.preferences);
        }
        let settings = self.preferences.settings();
        let converts = should_convert(focus, settings.behaviours().only_text_boxes(), self.front.always_on);
        let layout_allows = cmd.layout.english_layout || settings.switching().work_in_other_languages().is_on();
        if !converts || !layout_allows {
            return KeyResult::pass();
        }
        self.word_before.clear();
        self.word_before.push_str(self.composer.word());
        let edit = self.composer.press(KeyPress::new(code, cmd.flags.key_mods()));
        let actions = plan_output(&edit, &self.word_before, self.output_context())
            .into_iter()
            .map(OutputAction::from)
            .collect();
        KeyResult {
            swallow: matches!(edit, Edit::Replace { .. }),
            actions,
            command: None,
        }
    }

    fn output_context(&self) -> OutputContext {
        let settings = self.preferences.settings();
        let compatibility = settings.compatibility();
        OutputContext::new(settings.typing().code_table())
            .with_browser_fix(compatibility.fix_autocomplete())
            .with_chromium_fix(compatibility.chromium_fix())
            .with_chromium_app(self.front.chromium)
            .with_clipboard(compatibility.clipboard_input())
    }

    /// Flips the language, remembers it for the app in front when smart switch is on, and starts a new word.
    fn switch_language(&mut self, app: &str) {
        let language = match self.preferences.language() {
            Language::Vietnamese => Language::English,
            Language::English => Language::Vietnamese,
        };
        self.preferences.set_language(language);
        if self.preferences.settings().switching().smart_switch().is_on()
            && let Ok(app) = AppName::new(app)
        {
            self.preferences.remember_app(app);
        }
        self.refresh_composer();
    }

    /// Hands the composer the settings and language in force; it drops the word being typed.
    fn refresh_composer(&mut self) {
        self.composer.set_settings(ComposerSettings::from_settings(
            self.preferences.settings(),
            self.preferences.language(),
        ));
    }
}

#[cfg(test)]
mod tests;
