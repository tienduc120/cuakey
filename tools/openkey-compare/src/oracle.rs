//! `OpenKey`'s typing engine, compiled from its fetched sources with `shim/shim.cpp`, behind a safe interface. The
//! engine keeps its state in globals, so one value of [`OpenKey`] stands for it, and only one exists at a time.

use std::sync::atomic::{AtomicBool, Ordering};

use cuakey_domain::{CodeTable, InputMethod, KeyCode, KeyPress};

use crate::combo::{Combo, Toggle};

/// The most characters one reply carries; the shim's buffer has this size.
const MAX_SENDS: usize = 32;

#[repr(C)]
struct ShimSettings {
    input_method: i32,
    code_table: i32,
    spell_check: i32,
    restore: i32,
    modern_tones: i32,
    quick_telex: i32,
    quick_first: i32,
    quick_last: i32,
    capitals: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct ShimSend {
    units: [u16; 2],
    count: u16,
    key: u16,
}

#[repr(C)]
struct ShimReply {
    action: i32,
    backspaces: i32,
    after: i32,
    count: i32,
    sends: [ShimSend; MAX_SENDS],
}

unsafe extern "C" {
    fn shim_configure(settings: *const ShimSettings);
    fn shim_start_line();
    fn shim_key(vk: u16, caps_status: u8, control: i32, reply: *mut ShimReply);
    fn shim_click();
    fn shim_new_session();
    fn shim_key_send(vk: u16, caps: i32, send: *mut ShimSend);
}

/// Set while an [`OpenKey`] exists.
static TAKEN: AtomicBool = AtomicBool::new(false);

/// What the host does with one key, as the engine answers it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// The key goes on to the text box.
    PassThrough,
    /// The key is swallowed; delete `backspaces` characters and send `sends`.
    Replace,
    /// As `Replace`, then type the key itself.
    ReplaceThenKey,
    /// As `ReplaceThenKey`, then the next key starts a new word.
    ReplaceThenKeyThenNewWord,
    /// A reply this tool does not model, with the shim's number for it.
    Unmodelled(i32),
}

/// What the key was, as far as the host keeps track of characters typed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum After {
    WordBreak,
    Delete,
    Typed,
    Other,
}

/// One character the host sends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Send {
    /// One or two code units.
    Units { units: [u16; 2], count: usize },
    /// A key, sent as a key press so the text box types it.
    Key(KeyCode),
}

impl Send {
    pub fn units(&self) -> &[u16] {
        match self {
            Self::Units { units, count } => units.get(..*count).unwrap_or(units),
            Self::Key(_) => &[],
        }
    }
}

/// The engine's answer to one key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reply {
    pub action: Action,
    pub backspaces: usize,
    pub after: After,
    pub sends: Vec<Send>,
}

#[derive(Debug)]
pub struct OpenKey {
    _only_through_take: (),
}

#[expect(
    clippy::unused_self,
    reason = "the engine lives in globals: holding the one handle is what makes a call to it exclusive"
)]
impl OpenKey {
    /// The engine, unless another value already stands for it.
    pub fn take() -> Option<Self> {
        TAKEN
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
            .then_some(Self { _only_through_take: () })
    }

    /// Takes `combo`'s settings and starts afresh.
    pub fn configure(&mut self, combo: Combo) {
        let on = |toggle| i32::from(combo.is_on(toggle));
        let settings = ShimSettings {
            input_method: method_number(combo.method),
            code_table: table_number(combo.table),
            spell_check: on(Toggle::SpellCheck),
            restore: on(Toggle::Restore),
            modern_tones: on(Toggle::ModernTones),
            quick_telex: on(Toggle::QuickTelex),
            quick_first: on(Toggle::QuickFirst),
            quick_last: on(Toggle::QuickLast),
            capitals: on(Toggle::Capitals),
        };
        // SAFETY: `settings` is a live value of the layout the shim reads, and `self` is the only handle on the engine.
        unsafe { shim_configure(&raw const settings) };
    }

    /// Forgets everything typed, as the engine is when it starts.
    pub fn start_line(&mut self) {
        // SAFETY: no arguments; `self` is the only handle on the engine.
        unsafe { shim_start_line() };
    }

    /// One mouse button event; a click is two.
    pub fn mouse_button(&mut self) {
        // SAFETY: no arguments; `self` is the only handle on the engine.
        unsafe { shim_click() };
    }

    pub fn new_word(&mut self) {
        // SAFETY: no arguments; `self` is the only handle on the engine.
        unsafe { shim_new_session() };
    }

    pub fn press(&mut self, key: KeyPress) -> Reply {
        let mut reply = ShimReply {
            action: 0,
            backspaces: 0,
            after: 0,
            count: 0,
            sends: [ShimSend::default(); MAX_SENDS],
        };
        let (shift, caps) = (key.mods().shift().is_on(), key.mods().caps_lock().is_on());
        // The engine is told Shift or Caps Lock, and neither when both are on.
        let caps_status = match (shift, caps) {
            (true, false) => 1,
            (false, true) => 2,
            (true, true) | (false, false) => 0,
        };
        let control = i32::from(key.mods().ctrl_or_alt().is_on());
        // SAFETY: `reply` is a live, writable value of the layout the shim fills; `self` is the only handle.
        unsafe { shim_key(virtual_key(key.code()), caps_status, control, &raw mut reply) };
        let count = usize::try_from(reply.count).unwrap_or(0);
        Reply {
            action: action(reply.action),
            backspaces: usize::try_from(reply.backspaces).unwrap_or(0),
            after: after(reply.after),
            sends: reply.sends.iter().take(count).map(|send| send_of(*send)).collect(),
        }
    }

    /// How the host sends `key` itself after a reply; `upper` when Shift or Caps Lock is on.
    pub fn key_send(&mut self, key: KeyCode, upper: bool) -> Send {
        let mut send = ShimSend::default();
        // SAFETY: `send` is a live, writable value of the layout the shim fills; `self` is the only handle.
        unsafe { shim_key_send(virtual_key(key), i32::from(upper), &raw mut send) };
        send_of(send)
    }
}

impl Drop for OpenKey {
    fn drop(&mut self) {
        TAKEN.store(false, Ordering::Release);
    }
}

fn action(number: i32) -> Action {
    match number {
        0 => Action::PassThrough,
        1 => Action::Replace,
        2 => Action::ReplaceThenKey,
        3 => Action::ReplaceThenKeyThenNewWord,
        other => Action::Unmodelled(other),
    }
}

fn after(number: i32) -> After {
    match number {
        1 => After::WordBreak,
        2 => After::Delete,
        3 => After::Typed,
        _ => After::Other,
    }
}

fn send_of(send: ShimSend) -> Send {
    if send.count == 0 {
        Send::Key(key_of(send.key))
    } else {
        Send::Units {
            units: send.units,
            count: usize::from(send.count.min(2)),
        }
    }
}

const fn method_number(method: InputMethod) -> i32 {
    match method {
        InputMethod::Telex => 0,
        InputMethod::Vni => 1,
        InputMethod::SimpleTelex => 2,
    }
}

const fn table_number(table: CodeTable) -> i32 {
    match table {
        CodeTable::Unicode => 0,
        CodeTable::Tcvn3 => 1,
        CodeTable::VniWindows => 2,
        CodeTable::UnicodeCompound => 3,
        CodeTable::Cp1258 => 4,
    }
}

/// Each key with its Windows virtual-key code, as Microsoft documents them.
const VIRTUAL_KEYS: [(KeyCode, u16); 65] = [
    (KeyCode::A, 0x41),
    (KeyCode::B, 0x42),
    (KeyCode::C, 0x43),
    (KeyCode::D, 0x44),
    (KeyCode::E, 0x45),
    (KeyCode::F, 0x46),
    (KeyCode::G, 0x47),
    (KeyCode::H, 0x48),
    (KeyCode::I, 0x49),
    (KeyCode::J, 0x4a),
    (KeyCode::K, 0x4b),
    (KeyCode::L, 0x4c),
    (KeyCode::M, 0x4d),
    (KeyCode::N, 0x4e),
    (KeyCode::O, 0x4f),
    (KeyCode::P, 0x50),
    (KeyCode::Q, 0x51),
    (KeyCode::R, 0x52),
    (KeyCode::S, 0x53),
    (KeyCode::T, 0x54),
    (KeyCode::U, 0x55),
    (KeyCode::V, 0x56),
    (KeyCode::W, 0x57),
    (KeyCode::X, 0x58),
    (KeyCode::Y, 0x59),
    (KeyCode::Z, 0x5a),
    (KeyCode::Digit0, 0x30),
    (KeyCode::Digit1, 0x31),
    (KeyCode::Digit2, 0x32),
    (KeyCode::Digit3, 0x33),
    (KeyCode::Digit4, 0x34),
    (KeyCode::Digit5, 0x35),
    (KeyCode::Digit6, 0x36),
    (KeyCode::Digit7, 0x37),
    (KeyCode::Digit8, 0x38),
    (KeyCode::Digit9, 0x39),
    (KeyCode::F1, 0x70),
    (KeyCode::F2, 0x71),
    (KeyCode::F3, 0x72),
    (KeyCode::F4, 0x73),
    (KeyCode::F5, 0x74),
    (KeyCode::F6, 0x75),
    (KeyCode::F7, 0x76),
    (KeyCode::F8, 0x77),
    (KeyCode::F9, 0x78),
    (KeyCode::F10, 0x79),
    (KeyCode::F11, 0x7a),
    (KeyCode::F12, 0x7b),
    (KeyCode::Space, 0x20),
    (KeyCode::Backspace, 0x08),
    (KeyCode::Enter, 0x0d),
    (KeyCode::Tab, 0x09),
    (KeyCode::Escape, 0x1b),
    (KeyCode::Minus, 0xbd),
    (KeyCode::Equal, 0xbb),
    (KeyCode::LeftBracket, 0xdb),
    (KeyCode::RightBracket, 0xdd),
    (KeyCode::Backslash, 0xdc),
    (KeyCode::Semicolon, 0xba),
    (KeyCode::Quote, 0xde),
    (KeyCode::Comma, 0xbc),
    (KeyCode::Period, 0xbe),
    (KeyCode::Slash, 0xbf),
    (KeyCode::Backquote, 0xc0),
    (KeyCode::Other, 0xff),
];

fn virtual_key(code: KeyCode) -> u16 {
    VIRTUAL_KEYS
        .iter()
        .find(|(key, _)| *key == code)
        .map_or(0xff, |(_, vk)| *vk)
}

fn key_of(vk: u16) -> KeyCode {
    VIRTUAL_KEYS
        .iter()
        .find(|(_, number)| *number == vk)
        .map_or(KeyCode::Other, |(key, _)| *key)
}
