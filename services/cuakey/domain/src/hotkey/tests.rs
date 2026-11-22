use alloc::vec::Vec;

use super::KeyDirection::{self, Down, Up};
use super::{HotkeyAction, HotkeyBindings, HotkeyKey, HotkeyMatcher, HotkeyOutcome};
use crate::{Hotkey, KeyCode, Modifier, ModifierSet, OnOff, PreferencesError, SettingChange, Settings};

const CTRL: HotkeyKey = HotkeyKey::Modifier(Modifier::Ctrl);
const SHIFT: HotkeyKey = HotkeyKey::Modifier(Modifier::Shift);
const ALT: HotkeyKey = HotkeyKey::Modifier(Modifier::Alt);

fn key(code: KeyCode) -> HotkeyKey {
    HotkeyKey::Key(code)
}

/// Feeds events to a matcher with the default hotkeys in a normal text box.
struct Rig {
    matcher: HotkeyMatcher,
    bindings: HotkeyBindings,
    password: OnOff,
}

impl Rig {
    fn new() -> Self {
        Self::with(&Settings::default())
    }

    fn with(settings: &Settings) -> Self {
        Self {
            matcher: HotkeyMatcher::new(),
            bindings: HotkeyBindings::from_settings(settings),
            password: OnOff::Off,
        }
    }

    fn send(&mut self, key: HotkeyKey, direction: KeyDirection) -> HotkeyOutcome {
        self.matcher.on_key(key, direction, self.bindings, self.password)
    }
}

fn outcome(swallow: bool, fire: Option<HotkeyAction>) -> HotkeyOutcome {
    HotkeyOutcome::new(swallow, fire)
}

#[test]
fn ctrl_shift_release_switches() {
    let mut rig = Rig::new();

    assert_eq!(rig.send(CTRL, Down), outcome(false, None));
    assert_eq!(rig.send(SHIFT, Down), outcome(false, None));
    assert_eq!(rig.send(SHIFT, Up), outcome(false, Some(HotkeyAction::SwitchLanguage)));
    assert_eq!(
        rig.send(CTRL, Up),
        outcome(false, None),
        "the second release fires nothing more"
    );
}

#[test]
fn ctrl_shift_f5_opens_settings_only() {
    let mut rig = Rig::new();
    let events = [
        (CTRL, Down),
        (SHIFT, Down),
        (key(KeyCode::F5), Down),
        (key(KeyCode::F5), Up),
        (SHIFT, Up),
        (CTRL, Up),
    ];

    let outcomes: Vec<HotkeyOutcome> = events.into_iter().map(|(k, d)| rig.send(k, d)).collect();
    let fired: Vec<HotkeyAction> = outcomes.iter().filter_map(|o| o.fire()).collect();

    assert!(outcomes[2].swallow(), "F5 down is swallowed");
    assert!(outcomes[3].swallow(), "F5 up is swallowed");
    assert_eq!(outcomes[2].fire(), None, "not on press");
    assert_eq!(outcomes[3].fire(), Some(HotkeyAction::OpenSettings), "on release");
    assert_eq!(fired, [HotkeyAction::OpenSettings], "no switch on the modifier release");
}

#[test]
fn f5_repeat_fires_once() {
    let mut rig = Rig::new();
    rig.send(CTRL, Down);
    rig.send(SHIFT, Down);

    let first = rig.send(key(KeyCode::F5), Down);
    let repeat = rig.send(key(KeyCode::F5), Down);
    let release = rig.send(key(KeyCode::F5), Up);

    assert_eq!(first, outcome(true, None));
    assert_eq!(repeat, outcome(true, None));
    assert_eq!(release, outcome(true, Some(HotkeyAction::OpenSettings)));
}

#[test]
fn ctrl_shift_t_does_nothing() {
    let mut rig = Rig::new();
    rig.send(CTRL, Down);
    rig.send(SHIFT, Down);

    assert_eq!(rig.send(key(KeyCode::T), Down), outcome(false, None));
    assert_eq!(rig.send(key(KeyCode::T), Up), outcome(false, None));
    assert_eq!(
        rig.send(SHIFT, Up),
        outcome(false, None),
        "a key went down, so no switch"
    );
    assert_eq!(rig.send(CTRL, Up), outcome(false, None));
}

#[test]
fn superset_chord_does_not_fire() {
    let mut rig = Rig::new();
    rig.send(CTRL, Down);
    rig.send(ALT, Down);
    rig.send(SHIFT, Down);

    assert_eq!(rig.send(SHIFT, Up), outcome(false, None));
    assert_eq!(rig.send(ALT, Up), outcome(false, None));
    assert_eq!(rig.send(CTRL, Up), outcome(false, None));
}

#[test]
fn password_box_key_hotkey_passes() {
    let mut rig = Rig::new();
    rig.password = OnOff::On;
    rig.send(CTRL, Down);
    rig.send(SHIFT, Down);

    assert_eq!(rig.send(key(KeyCode::F5), Down), outcome(false, None));
    assert_eq!(rig.send(key(KeyCode::F5), Up), outcome(false, None));
    assert_eq!(rig.send(SHIFT, Up), outcome(false, None));
    assert_eq!(rig.send(CTRL, Up), outcome(false, None));
}

#[test]
fn password_box_modifier_only_still_fires() {
    let mut rig = Rig::new();
    rig.password = OnOff::On;
    rig.send(CTRL, Down);
    rig.send(SHIFT, Down);

    assert_eq!(rig.send(SHIFT, Up), outcome(false, Some(HotkeyAction::SwitchLanguage)));
}

#[test]
fn reset_clears_chord() {
    let mut rig = Rig::new();
    rig.send(CTRL, Down);
    rig.send(SHIFT, Down);

    rig.matcher.reset();

    assert_eq!(rig.send(SHIFT, Up), outcome(false, None), "the chord was forgotten");
    assert_eq!(rig.send(CTRL, Up), outcome(false, None));
    rig.send(CTRL, Down);
    rig.send(SHIFT, Down);
    assert_eq!(
        rig.send(SHIFT, Up),
        outcome(false, Some(HotkeyAction::SwitchLanguage)),
        "a fresh chord works after a reset"
    );
}

#[test]
fn stale_modifier_does_not_fire() {
    let mut rig = Rig::new();

    // Releases of keys that went down while nothing was watching.
    assert_eq!(rig.send(SHIFT, Up), outcome(false, None));
    assert_eq!(rig.send(CTRL, Up), outcome(false, None));
    assert_eq!(rig.send(key(KeyCode::F5), Up), outcome(false, None));

    // Ctrl's release was lost; the reset on focus change must stop it from joining the next chord.
    rig.send(CTRL, Down);
    rig.matcher.reset();
    rig.send(SHIFT, Down);
    assert_eq!(
        rig.send(SHIFT, Up),
        outcome(false, None),
        "Shift alone is not the switch hotkey"
    );
}

#[test]
fn key_only_hotkey_fires_on_release() {
    let f9 = Hotkey::new(ModifierSet::empty(), Some(KeyCode::F9)).unwrap();
    let settings = Settings::default().with(SettingChange::QuickConvertHotkey(Some(f9)));
    let mut rig = Rig::with(&settings);

    assert_eq!(rig.send(key(KeyCode::F9), Down), outcome(true, None));
    assert_eq!(
        rig.send(key(KeyCode::F9), Up),
        outcome(true, Some(HotkeyAction::QuickConvert))
    );
}

#[test]
fn modifier_events_are_never_swallowed() {
    let mut rig = Rig::new();
    let events = [
        (CTRL, Down),
        (SHIFT, Down),
        (key(KeyCode::F5), Down),
        (key(KeyCode::F5), Up),
        (SHIFT, Up),
        (CTRL, Up),
    ];

    for (k, d) in events {
        let o = rig.send(k, d);
        if matches!(k, HotkeyKey::Modifier(_)) {
            assert!(!o.swallow(), "{k:?} {d:?}");
        }
    }
}

#[test]
fn hotkey_on_an_unknown_key_is_refused() {
    assert_eq!(
        Hotkey::new(ModifierSet::empty().with(Modifier::Ctrl), Some(KeyCode::Other)),
        Err(PreferencesError::UnknownHotkeyKey)
    );
}

#[test]
fn bare_f5_after_ctrl_shift_is_not_a_hotkey() {
    let mut rig = Rig::new();
    rig.send(CTRL, Down);
    rig.send(SHIFT, Down);
    rig.send(SHIFT, Up);
    rig.send(CTRL, Up);

    assert_eq!(rig.send(key(KeyCode::F5), Down), outcome(false, None));
    assert_eq!(rig.send(key(KeyCode::F5), Up), outcome(false, None));
}

#[test]
fn key_only_hotkey_fires_after_earlier_modifier_use() {
    let f9 = Hotkey::new(ModifierSet::empty(), Some(KeyCode::F9)).unwrap();
    let settings = Settings::default().with(SettingChange::QuickConvertHotkey(Some(f9)));
    let mut rig = Rig::with(&settings);
    rig.send(CTRL, Down);
    rig.send(key(KeyCode::C), Down);
    rig.send(key(KeyCode::C), Up);
    rig.send(CTRL, Up);

    assert_eq!(rig.send(key(KeyCode::F9), Down), outcome(true, None));
    assert_eq!(
        rig.send(key(KeyCode::F9), Up),
        outcome(true, Some(HotkeyAction::QuickConvert))
    );
}

#[test]
fn chord_survives_a_partial_release() {
    let mut rig = Rig::new();
    rig.send(CTRL, Down);
    rig.send(SHIFT, Down);

    // Shift comes up before any key, so the switch hotkey fires then.
    assert_eq!(rig.send(SHIFT, Up), outcome(false, Some(HotkeyAction::SwitchLanguage)));
    // Ctrl is still held, so the chord is still Ctrl+Shift and F5 completes the settings hotkey.
    assert_eq!(rig.send(key(KeyCode::F5), Down), outcome(true, None));
    assert_eq!(
        rig.send(key(KeyCode::F5), Up),
        outcome(true, Some(HotkeyAction::OpenSettings))
    );
    assert_eq!(rig.send(CTRL, Up), outcome(false, None));
}
