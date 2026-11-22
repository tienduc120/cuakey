use super::{SendOp, segments, store_app_backspace_messages};

#[test]
fn a_paste_starts_a_new_batch() {
    let ops = [
        SendOp::Backspace,
        SendOp::Backspace,
        SendOp::Paste(vec![0x1ec7]),
        SendOp::Unit(0x74),
        SendOp::Paste(vec![0x0111]),
    ];

    let batches: Vec<&[SendOp]> = segments(&ops).collect();

    assert_eq!(
        batches,
        [&ops[..2], &ops[2..4], &ops[4..]],
        "the keys before a paste go out before the clipboard changes for it"
    );
}

#[test]
fn ops_without_a_paste_go_out_in_one_batch() {
    let ops = [SendOp::Backspace, SendOp::Unit(0x1ec7)];

    let batches: Vec<&[SendOp]> = segments(&ops).collect();

    assert_eq!(batches, [&ops[..]], "one batch, so no real key can land in the middle");
}

#[test]
fn nothing_to_send_is_no_batch() {
    let batches: Vec<&[SendOp]> = segments(&[]).collect();

    assert!(batches.is_empty(), "an empty reply sends nothing");
}

#[test]
fn store_app_gets_two_backspace_chars_per_backspace() {
    let ops = [
        SendOp::Backspace,
        SendOp::Unit(0x1ec7),
        SendOp::Backspace,
        SendOp::SelectPreviousChar,
    ];

    let messages = store_app_backspace_messages(&ops);

    assert_eq!(
        messages, 4,
        "two WM_CHAR backspaces for each of the two backspaces, none for the rest"
    );
}

#[cfg(windows)]
mod inputs {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        INPUT, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP,
        KEYEVENTF_UNICODE, VIRTUAL_KEY, VK_BACK, VK_INSERT, VK_LEFT, VK_SHIFT,
    };

    use super::super::inject::build_inputs;
    use crate::{INJECTED_TAG, RawKey, RawMods, SendOp};

    const PRESS: KEYBD_EVENT_FLAGS = KEYBD_EVENT_FLAGS(0);

    /// The keyboard event inside `input`, which must be one.
    fn keyboard_event(input: &INPUT) -> KEYBDINPUT {
        assert_eq!(
            input.r#type, INPUT_KEYBOARD,
            "every input the app sends is a keyboard event"
        );
        // SAFETY: the type checked above says the keyboard field is the one that was written.
        unsafe { input.Anonymous.ki }
    }

    /// The events for `ops` while the user holds no Shift.
    fn events(ops: &[SendOp]) -> Vec<KEYBDINPUT> {
        build_inputs(ops, false).iter().map(keyboard_event).collect()
    }

    fn events_while_shift_held(ops: &[SendOp]) -> Vec<KEYBDINPUT> {
        build_inputs(ops, true).iter().map(keyboard_event).collect()
    }

    fn key(vk: VIRTUAL_KEY, flags: KEYBD_EVENT_FLAGS) -> KEYBDINPUT {
        KEYBDINPUT {
            wVk: vk,
            wScan: 0,
            dwFlags: flags,
            time: 0,
            dwExtraInfo: INJECTED_TAG,
        }
    }

    fn unit(unit: u16, flags: KEYBD_EVENT_FLAGS) -> KEYBDINPUT {
        KEYBDINPUT {
            wVk: VIRTUAL_KEY(0),
            wScan: unit,
            dwFlags: KEYEVENTF_UNICODE | flags,
            time: 0,
            dwExtraInfo: INJECTED_TAG,
        }
    }

    #[test]
    fn backspace_inputs() {
        let sent = events(&[SendOp::Backspace]);

        assert_eq!(
            sent,
            [key(VK_BACK, PRESS), key(VK_BACK, KEYEVENTF_KEYUP)],
            "Backspace down then up, both tagged as ours"
        );
    }

    #[test]
    fn unit_inputs_are_unicode() {
        let sent = events(&[SendOp::Unit(0x1ec7)]);

        assert_eq!(
            sent,
            [unit(0x1ec7, PRESS), unit(0x1ec7, KEYEVENTF_KEYUP)],
            "the code unit travels in the scan code, so the layout cannot change it"
        );
    }

    #[test]
    fn select_previous_char_sequence() {
        let sent = events(&[SendOp::SelectPreviousChar]);

        assert_eq!(
            sent,
            [
                key(VK_SHIFT, PRESS),
                key(VK_LEFT, KEYEVENTF_EXTENDEDKEY),
                key(VK_LEFT, KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP),
                key(VK_SHIFT, KEYEVENTF_KEYUP),
            ],
            "Shift+Left, with Left as the arrow key rather than the number pad"
        );
    }

    #[test]
    fn paste_is_shift_insert() {
        let sent = events(&[SendOp::Paste(vec![0x1ec7])]);

        assert_eq!(
            sent,
            [
                key(VK_SHIFT, PRESS),
                key(VK_INSERT, KEYEVENTF_EXTENDEDKEY),
                key(VK_INSERT, KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP),
                key(VK_SHIFT, KEYEVENTF_KEYUP),
            ],
            "Shift+Insert pastes in more apps than Ctrl+V, and the text itself goes by the clipboard"
        );
    }

    #[test]
    fn select_previous_char_keeps_a_held_shift() {
        let sent = events_while_shift_held(&[SendOp::SelectPreviousChar]);

        assert_eq!(
            sent,
            [
                key(VK_LEFT, KEYEVENTF_EXTENDEDKEY),
                key(VK_LEFT, KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP),
            ],
            "the user's Shift already makes Left select, and releasing it would turn the next letters lowercase"
        );
    }

    #[test]
    fn paste_keeps_a_held_shift() {
        let sent = events_while_shift_held(&[SendOp::Paste(vec![0x1ec7])]);

        assert_eq!(
            sent,
            [
                key(VK_INSERT, KEYEVENTF_EXTENDEDKEY),
                key(VK_INSERT, KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP),
            ],
            "the user's Shift already makes Insert paste, and releasing it would turn the next letters lowercase"
        );
    }

    #[test]
    fn a_resent_key_is_only_pressed() {
        let typed = RawKey {
            vk: 0xbe,
            scan: 0x34,
            down: true,
            extended: true,
            mods: RawMods::default(),
        };

        let sent = events(&[SendOp::ResendKey(typed)]);

        assert_eq!(
            sent,
            [KEYBDINPUT {
                wVk: VIRTUAL_KEY(0xbe),
                wScan: 0x34,
                dwFlags: KEYEVENTF_EXTENDEDKEY,
                time: 0,
                dwExtraInfo: INJECTED_TAG,
            }],
            "the key goes down again as it came in; the user's own key-up lifts it"
        );
    }

    #[test]
    fn ops_go_out_in_order() {
        let sent = events(&[SendOp::Backspace, SendOp::Unit(0x74)]);

        assert_eq!(
            sent,
            [
                key(VK_BACK, PRESS),
                key(VK_BACK, KEYEVENTF_KEYUP),
                unit(0x74, PRESS),
                unit(0x74, KEYEVENTF_KEYUP),
            ],
            "each op's keys follow the previous op's"
        );
    }
}
