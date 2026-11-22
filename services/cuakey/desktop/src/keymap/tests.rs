use cuakey_application::KeyName;

use super::to_key_name;

#[test]
fn vk_mapping() {
    let cases = [
        (0x41, KeyName::A),
        (0x5a, KeyName::Z),
        (0x30, KeyName::Digit0),
        (0x39, KeyName::Digit9),
        (0x70, KeyName::F1),
        (0x74, KeyName::F5),
        (0x7b, KeyName::F12),
        (0x20, KeyName::Space),
        (0x08, KeyName::Backspace),
        (0x0d, KeyName::Enter),
        (0x09, KeyName::Tab),
        (0x1b, KeyName::Escape),
        (0xbd, KeyName::Minus),
        (0xbb, KeyName::Equal),
        (0xdb, KeyName::LeftBracket),
        (0xdd, KeyName::RightBracket),
        (0xdc, KeyName::Backslash),
        (0xba, KeyName::Semicolon),
        (0xde, KeyName::Quote),
        (0xbc, KeyName::Comma),
        (0xbe, KeyName::Period),
        (0xbf, KeyName::Slash),
        (0xc0, KeyName::Backquote),
    ];
    for (vk, name) in cases {
        assert_eq!(to_key_name(vk), name, "virtual key {vk:#04x}");
    }
}

#[test]
fn each_side_of_a_modifier_is_that_modifier() {
    let cases = [
        (0x10, KeyName::Shift),
        (0xa0, KeyName::Shift),
        (0xa1, KeyName::Shift),
        (0x11, KeyName::Ctrl),
        (0xa2, KeyName::Ctrl),
        (0xa3, KeyName::Ctrl),
        (0x12, KeyName::Alt),
        (0xa4, KeyName::Alt),
        (0xa5, KeyName::Alt),
        (0x5b, KeyName::Win),
        (0x5c, KeyName::Win),
    ];
    for (vk, name) in cases {
        assert_eq!(to_key_name(vk), name, "virtual key {vk:#04x}");
    }
}

#[test]
fn keys_the_engine_does_not_know_are_other() {
    // Left arrow, Delete, numpad 0, Caps Lock.
    for vk in [0x25, 0x2e, 0x60, 0x14] {
        assert_eq!(to_key_name(vk), KeyName::Other, "virtual key {vk:#04x}");
    }
}

#[cfg(windows)]
mod hook_keys {
    use cuakey_application::{FocusValue, KeyCommand, KeyFlags, KeyName, LayoutFlags, OutputAction};
    use platform_windows::{FocusClass, RawKey, RawMods, SendOp};

    use super::super::{focus_value, key_command, send_ops};

    const A_DOWN: RawKey = RawKey {
        vk: 0x41,
        scan: 0x1e,
        down: true,
        extended: false,
        mods: RawMods {
            shift: true,
            ctrl_alt_win: false,
            caps_lock: true,
        },
    };

    #[test]
    fn raw_key_to_command_groups_flags() {
        let layout = LayoutFlags {
            ime_open: false,
            english_layout: true,
        };

        let command = key_command(A_DOWN, FocusValue::Text, "notepad.exe", layout);

        assert_eq!(
            command,
            KeyCommand {
                key: KeyName::A,
                down: true,
                flags: KeyFlags {
                    shift: true,
                    ctrl_alt_win: false,
                    caps_lock: true,
                },
                focus: FocusValue::Text,
                app: "notepad.exe",
                layout,
            }
        );
    }

    #[test]
    fn a_released_key_is_not_down() {
        let up = RawKey { down: false, ..A_DOWN };

        assert!(!key_command(up, FocusValue::Text, "", LayoutFlags::default()).down);
    }

    #[test]
    fn each_focus_class_has_its_value() {
        let cases = [
            (FocusClass::Pending, FocusValue::Pending),
            (FocusClass::Text, FocusValue::Text),
            (FocusClass::Password, FocusValue::Password),
            (FocusClass::NotText, FocusValue::NotText),
            (FocusClass::Unknown, FocusValue::Unknown),
        ];
        for (class, value) in cases {
            assert_eq!(focus_value(class), value);
        }
    }

    #[test]
    fn output_actions_become_send_ops_and_resend_the_key() {
        let actions = vec![
            OutputAction::Backspace,
            OutputAction::SelectPreviousChar,
            OutputAction::Unit(0x1ec7),
            OutputAction::Paste(vec![0x76, 0x1ec7]),
            OutputAction::ResendKey,
        ];

        assert_eq!(
            send_ops(actions, A_DOWN),
            [
                SendOp::Backspace,
                SendOp::SelectPreviousChar,
                SendOp::Unit(0x1ec7),
                SendOp::Paste(vec![0x76, 0x1ec7]),
                SendOp::ResendKey(A_DOWN),
            ]
        );
    }
}
