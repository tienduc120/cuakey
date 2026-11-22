use eframe::egui::{Event, Key, Modifiers, OpenUrl, OutputCommand};

use super::{
    ClipboardShortcut, SystemCommand, clipboard_shortcut, take_clipboard_shortcuts, take_system_commands, wants_paste,
};

/// Ctrl as egui-winit reports it on Windows, where Ctrl is also the command key.
const CTRL: Modifiers = Modifiers {
    ctrl: true,
    command: true,
    ..Modifiers::NONE
};

fn key_event(key: Key, modifiers: Modifiers, pressed: bool) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers,
    }
}

#[test]
fn the_clipboard_shortcuts_are_the_windows_ones() {
    let cases = [
        (Key::C, CTRL, Some(ClipboardShortcut::Copy)),
        (Key::Insert, CTRL, Some(ClipboardShortcut::Copy)),
        (Key::X, CTRL, Some(ClipboardShortcut::Cut)),
        (Key::Delete, Modifiers::SHIFT, Some(ClipboardShortcut::Cut)),
        (Key::V, CTRL, Some(ClipboardShortcut::Paste)),
        (Key::Insert, Modifiers::SHIFT, Some(ClipboardShortcut::Paste)),
        (Key::Paste, Modifiers::NONE, Some(ClipboardShortcut::Paste)),
        (Key::V, Modifiers::NONE, None),
        (Key::V, Modifiers { shift: true, ..CTRL }, None),
        (Key::Delete, Modifiers::NONE, None),
        (Key::A, CTRL, None),
    ];
    for (key, modifiers, shortcut) in cases {
        assert_eq!(
            clipboard_shortcut(key, modifiers),
            shortcut,
            "{key:?} with {modifiers:?}"
        );
    }
}

#[test]
fn a_paste_becomes_the_clipboard_text_with_plain_line_ends() {
    let mut events = vec![key_event(Key::V, CTRL, true), key_event(Key::V, CTRL, false)];

    take_clipboard_shortcuts(&mut events, Some("một\r\nhai"));

    assert_eq!(
        events,
        [Event::Paste("một\nhai".to_owned()), key_event(Key::V, CTRL, false)],
        "the release of the key goes on as it is"
    );
}

#[test]
fn a_paste_with_nothing_to_paste_is_dropped() {
    for clipboard in [None, Some("")] {
        let mut events = vec![key_event(Key::Insert, Modifiers::SHIFT, true)];

        take_clipboard_shortcuts(&mut events, clipboard);

        assert_eq!(events, [], "{clipboard:?}");
    }
}

#[test]
fn copy_and_cut_take_the_place_of_their_keys() {
    let mut events = vec![
        key_event(Key::C, CTRL, true),
        key_event(Key::Delete, Modifiers::SHIFT, true),
    ];

    take_clipboard_shortcuts(&mut events, None);

    assert_eq!(events, [Event::Copy, Event::Cut], "Shift+Delete never also deletes");
}

#[test]
fn other_events_go_on_untouched() {
    let original = vec![Event::Text("a".to_owned()), key_event(Key::A, CTRL, true)];
    let mut events = original.clone();

    take_clipboard_shortcuts(&mut events, Some("clipboard"));

    assert_eq!(events, original);
}

#[test]
fn the_clipboard_is_read_only_for_a_paste() {
    assert!(wants_paste(&[key_event(Key::V, CTRL, true)]));
    assert!(!wants_paste(&[key_event(Key::V, CTRL, false)]), "a release");
    assert!(!wants_paste(&[
        key_event(Key::C, CTRL, true),
        Event::Text("v".to_owned())
    ]));
}

#[test]
fn copies_and_links_are_taken_from_the_output() {
    let url = "https://github.com/tienduc120/cuakey";
    let mut commands = vec![
        OutputCommand::CopyText("chữ".to_owned()),
        OutputCommand::TextSelectionSettled("chữ".to_owned()),
        OutputCommand::OpenUrl(OpenUrl::new_tab(url)),
    ];

    let taken = take_system_commands(&mut commands);

    assert_eq!(
        taken,
        [
            SystemCommand::Copy("chữ".to_owned()),
            SystemCommand::Open(url.to_owned())
        ]
    );
    assert!(
        matches!(commands.as_slice(), [OutputCommand::TextSelectionSettled(_)]),
        "what is not carried out here stays for eframe"
    );
}
