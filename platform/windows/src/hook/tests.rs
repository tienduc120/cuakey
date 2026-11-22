use super::{HookedKey, RawKey, RawMods, key_for_handler};
use crate::INJECTED_TAG;

const LETTER_A: u32 = 0x41;
const LEFT_SHIFT: u32 = 0xa0;
const RIGHT_CTRL: u32 = 0xa3;
/// `LLKHF_EXTENDED`.
const EXTENDED: u32 = 0x01;
/// `LLKHF_UP`.
const RELEASED: u32 = 0x80;

fn hooked(vk: u32, flags: u32, extra_info: usize) -> HookedKey {
    HookedKey {
        vk,
        scan: 0x1e,
        flags,
        extra_info,
    }
}

const SHIFT_HELD: RawMods = RawMods {
    shift: true,
    ctrl_alt_win: false,
    caps_lock: false,
};

#[test]
fn injected_events_filtered() {
    let ours = key_for_handler(hooked(LETTER_A, 0, INJECTED_TAG), || SHIFT_HELD);
    let users = key_for_handler(hooked(LETTER_A, 0, 0), || SHIFT_HELD);

    assert_eq!(ours, None, "a key the app sent never reaches the handler");
    assert_eq!(
        users,
        Some(RawKey {
            vk: 0x41,
            scan: 0x1e,
            down: true,
            extended: false,
            mods: SHIFT_HELD,
        }),
        "the user's key reaches the handler with the modifiers held at it"
    );
}

#[test]
fn injected_events_never_read_the_modifiers() {
    let mut read = false;

    key_for_handler(hooked(LETTER_A, 0, INJECTED_TAG), || {
        read = true;
        RawMods::default()
    });

    assert!(!read, "a key that passes straight through costs no system calls");
}

#[test]
fn modifier_keys_reach_the_handler() {
    let press = key_for_handler(hooked(LEFT_SHIFT, 0, 0), RawMods::default);
    let release = key_for_handler(hooked(LEFT_SHIFT, RELEASED, 0), || SHIFT_HELD);

    assert_eq!(
        press.map(|key| (key.vk, key.down)),
        Some((0xa0, true)),
        "Shift going down reaches the handler, which matches hotkeys on it"
    );
    assert_eq!(
        release.map(|key| (key.vk, key.down)),
        Some((0xa0, false)),
        "and so does Shift coming up"
    );
}

#[test]
fn extended_keys_are_marked() {
    let key = key_for_handler(hooked(RIGHT_CTRL, EXTENDED | RELEASED, 0), RawMods::default);

    assert_eq!(
        key.map(|key| (key.extended, key.down)),
        Some((true, false)),
        "right Ctrl is on the extended part of the keyboard, and this is its release"
    );
}

#[cfg(windows)]
#[test]
fn hook_flags_are_the_system_ones() {
    use windows::Win32::UI::WindowsAndMessaging::{LLKHF_EXTENDED, LLKHF_UP};

    assert_eq!(
        (super::EXTENDED_FLAG, super::RELEASED_FLAG),
        (LLKHF_EXTENDED.0, LLKHF_UP.0),
        "the flags read from each event are the ones Windows sets"
    );
}
