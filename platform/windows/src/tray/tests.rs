use super::{
    ChangeCall, MF_CHECKED, MF_GRAYED, MF_POPUP, MF_SEPARATOR, MenuItem, NIN_KEYSELECT, Presence, TIP_UNITS, TrayEvent,
    WM_CONTEXTMENU, WM_LBUTTONDBLCLK, callback_event, menu_flags as flags_of, tooltip_units,
};

/// `NIN_SELECT`: a single click on the icon, which also comes around a double click.
const NIN_SELECT: u32 = 0x0400;
/// `WM_MOUSEMOVE`: the pointer passed over the icon.
const WM_MOUSEMOVE: u32 = 0x0200;
/// `WM_LBUTTONUP`: the left button came up, just before `NIN_SELECT`.
const WM_LBUTTONUP: u32 = 0x0202;
/// `NIN_POPUPOPEN`: the pointer rests on the icon, for a popup the icon does not draw.
const NIN_POPUPOPEN: u32 = 0x0406;

/// A version-4 callback's `lParam`: the event in the low word and the icon's id, 1, in the high word.
fn callback(event: u32) -> isize {
    isize::try_from((1 << 16) | event).unwrap()
}

fn command(checked: Option<bool>, enabled: bool) -> MenuItem {
    MenuItem::Command {
        id: 7,
        label: "Bật tiếng Việt".to_owned(),
        checked,
        enabled,
    }
}

/// The tooltip as the notification area shows it: the units before the first NUL, which must be valid UTF-16.
fn shown(units: &[u16; TIP_UNITS]) -> String {
    let end = units.iter().position(|&unit| unit == 0).unwrap();
    String::from_utf16(&units[..end]).unwrap()
}

#[test]
fn callback_events_decode() {
    let decoded = [
        WM_LBUTTONDBLCLK,
        NIN_KEYSELECT,
        WM_CONTEXTMENU,
        NIN_SELECT,
        WM_MOUSEMOVE,
        WM_LBUTTONUP,
        NIN_POPUPOPEN,
    ]
    .map(|event| callback_event(callback(event)));

    assert_eq!(
        decoded,
        [
            Some(TrayEvent::Activate),
            Some(TrayEvent::Activate),
            Some(TrayEvent::MenuRequested),
            None,
            None,
            None,
            None,
        ],
        "a double click or a key select activates, a context-menu request asks for the menu, and a single click and \
         the rest are ignored"
    );
}

#[test]
fn menu_flags() {
    let checked = flags_of(&command(Some(true), true));
    let unchecked = flags_of(&command(Some(false), true));
    let no_mark = flags_of(&command(None, true));
    let disabled = flags_of(&command(Some(true), false));
    let separator = flags_of(&MenuItem::Separator);
    let submenu = flags_of(&MenuItem::Submenu {
        label: "Bảng mã".to_owned(),
        items: vec![command(None, true)],
    });

    assert_eq!(checked, MF_CHECKED, "a checked entry has a check mark");
    assert_eq!(unchecked, 0, "an unchecked entry is a plain string");
    assert_eq!(no_mark, 0, "an entry without a check state is a plain string");
    assert_eq!(
        disabled,
        MF_CHECKED | MF_GRAYED,
        "a disabled entry is dimmed and keeps its mark"
    );
    assert_eq!(separator, MF_SEPARATOR, "a separator is a dividing line");
    assert_eq!(submenu, MF_POPUP, "a submenu opens a popup");
}

#[test]
fn tooltip_truncates_on_a_char_boundary() {
    let long = "CuaKey: bộ gõ tiếng Việt, đang bật kiểu gõ Telex và bảng mã Unicode dựng sẵn. ".repeat(3);
    let pair_at_the_end = format!("{}\u{1f600}", "ệ".repeat(TIP_UNITS - 2));

    let long_units = tooltip_units(&long);
    let pair_units = tooltip_units(&pair_at_the_end);

    let long_shown = shown(&long_units);
    assert_eq!(
        long_shown.encode_utf16().count(),
        TIP_UNITS - 1,
        "a long tooltip fills every unit but the last"
    );
    assert!(long.starts_with(&long_shown), "the tooltip is the start of the text");
    assert_eq!(long_units[TIP_UNITS - 1], 0, "the last unit is the terminating NUL");
    assert_eq!(
        shown(&pair_units),
        "ệ".repeat(TIP_UNITS - 2),
        "a character whose surrogate pair would not fit before the NUL is left out whole"
    );
}

#[test]
fn the_icon_waits_for_the_taskbar_and_takes_the_changes_made_meanwhile() {
    let added = Presence::after_offer(|| true, || panic!("an icon that was added needs no update"));
    let kept = Presence::after_offer(|| false, || true);
    let no_taskbar = Presence::after_offer(|| false, || false);

    assert_eq!(added, Presence::Shown, "an icon the notification area took is shown");
    assert_eq!(
        kept,
        Presence::Shown,
        "an icon the notification area still had is shown"
    );
    assert_eq!(
        no_taskbar,
        Presence::Pending,
        "with no notification area yet, the icon waits for the taskbar"
    );
    assert_eq!(
        Presence::Shown.change_call(),
        ChangeCall::Update,
        "a change to a shown icon updates it"
    );
    assert_eq!(
        Presence::Pending.change_call(),
        ChangeCall::Offer,
        "a change to a pending icon offers it again, carrying the change"
    );
}

#[cfg(windows)]
#[test]
fn the_codes_match_the_windows_bindings() {
    use windows::Win32::UI::Shell::{self as shell, NOTIFYICONDATAW};
    use windows::Win32::UI::WindowsAndMessaging as wm;

    let ours = [
        WM_LBUTTONDBLCLK,
        NIN_SELECT,
        NIN_KEYSELECT,
        WM_CONTEXTMENU,
        MF_GRAYED,
        MF_CHECKED,
        MF_POPUP,
        MF_SEPARATOR,
    ];
    // The headers define the key select as the select with the keyboard flag.
    let theirs = [
        wm::WM_LBUTTONDBLCLK,
        shell::NIN_SELECT,
        shell::NIN_SELECT | shell::NINF_KEY,
        wm::WM_CONTEXTMENU,
        wm::MF_GRAYED.0,
        wm::MF_CHECKED.0,
        wm::MF_POPUP.0,
        wm::MF_SEPARATOR.0,
    ];

    assert_eq!(ours, theirs, "the codes written out here are the ones Windows uses");
    assert_eq!(
        TIP_UNITS,
        NOTIFYICONDATAW::default().szTip.len(),
        "the tooltip room is the size of szTip"
    );
}

#[cfg(windows)]
mod live {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::super::shell::{Tray, TrayError, notification_area_has};
    use super::{NIN_KEYSELECT, NIN_SELECT, WM_CONTEXTMENU, WM_LBUTTONDBLCLK, WM_MOUSEMOVE};
    use crate::{AppMessage, MessageWindow, TrayEvent, TrayIcon, post_to};

    #[test]
    #[ignore = "adds an icon to the notification area, which needs a signed-in desktop with Explorer running"]
    fn the_icon_comes_back_after_explorer_restarts_and_goes_on_drop() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let sink = Rc::clone(&events);
        // An assertion that fails below drops the tray as it unwinds, which removes the icon.
        let mut tray = Tray::add_stock("CuaKey: thử biểu tượng", move |event| {
            sink.borrow_mut().push(event);
        })
        .unwrap();
        let class = format!("CuaKeyTest.tray.{}", std::process::id());
        let window = MessageWindow::new(&class, |_| {}).unwrap();

        let shown_when_added = (notification_area_has(tray.hwnd()), tray.is_shown());
        tray.set_tooltip(&"Bộ gõ tiếng Việt CuaKey ".repeat(8)).unwrap();
        let missing_icon = tray.set_icon(TrayIcon(1));
        tray.act_out_explorer_restart().unwrap();
        for event in [
            NIN_SELECT,
            WM_LBUTTONDBLCLK,
            WM_MOUSEMOVE,
            NIN_KEYSELECT,
            WM_CONTEXTMENU,
        ] {
            tray.post_callback(event).unwrap();
        }
        post_to(&class, AppMessage::Quit).unwrap();
        window.run_loop();
        let shown_after_restart = (notification_area_has(tray.hwnd()), tray.is_shown());
        let hwnd = tray.hwnd();
        drop(tray);
        let shown_after_drop = notification_area_has(hwnd);

        assert_eq!(
            shown_when_added,
            (true, true),
            "adding the tray shows its icon, and the tray knows it"
        );
        assert!(
            matches!(missing_icon, Err(TrayError::LoadIcon(_))),
            "an icon the executable does not carry is refused: {missing_icon:?}"
        );
        assert_eq!(
            shown_after_restart,
            (true, true),
            "the tray adds its icon again when Explorer says it made the taskbar"
        );
        assert_eq!(
            *events.borrow(),
            [TrayEvent::Activate, TrayEvent::Activate, TrayEvent::MenuRequested],
            "the message loop of the thread hands the icon's events to the handler"
        );
        assert!(!shown_after_drop, "dropping the tray removes its icon");
    }
}

#[cfg(windows)]
mod live_menu {
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_DOWN, VK_ESCAPE, VK_RETURN};

    use super::super::shell::Tray;
    use super::command;
    use crate::MenuItem;

    fn items() -> Vec<MenuItem> {
        vec![
            MenuItem::Command {
                id: 11,
                label: "Bật tiếng Việt".to_owned(),
                checked: Some(true),
                enabled: true,
            },
            MenuItem::Separator,
            MenuItem::Submenu {
                label: "Bảng mã".to_owned(),
                items: vec![command(Some(false), true), MenuItem::Separator, command(None, false)],
            },
            command(None, false),
        ]
    }

    #[test]
    #[ignore = "opens the menu on the desktop and answers it with keys queued for its loop"]
    fn the_menu_returns_the_entry_chosen_or_none() {
        let tray = Tray::add_stock("CuaKey: thử menu", |_| {}).unwrap();

        tray.post_key(VK_DOWN.0).unwrap();
        tray.post_key(VK_RETURN.0).unwrap();
        let chosen = tray.show_menu(&items()).unwrap();
        tray.post_key(VK_ESCAPE.0).unwrap();
        let dismissed = tray.show_menu(&items()).unwrap();

        assert_eq!(chosen, Some(11), "the entry picked is the one returned");
        assert_eq!(dismissed, None, "a dismissed menu returns nothing");
    }
}
