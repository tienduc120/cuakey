use cuakey_application::{CodeTableValue, LanguageValue, PreferencesView};
use cuakey_interfaces::{MenuEffect, MenuEntry, MenuId, apply, model};
use platform_windows::{MenuItem, TrayEvent, TrayIcon};

use super::{MenuMap, TrayResponse, menu_app, tray_icon, tray_response};

fn entry(id: MenuId, label: &str, checked: Option<bool>) -> MenuEntry {
    MenuEntry {
        id,
        label: label.to_owned(),
        checked,
        children: Vec::new(),
    }
}

fn command(id: u32, label: &str, checked: Option<bool>) -> MenuItem {
    MenuItem::Command {
        id,
        label: label.to_owned(),
        checked,
        enabled: true,
    }
}

#[test]
fn entries_become_items_with_ids_counted_from_one() {
    let entries = [
        entry(MenuId::Vietnamese, "Bật Tiếng Việt", Some(true)),
        entry(MenuId::Separator, "", None),
        MenuEntry {
            children: vec![entry(MenuId::CodeTable(CodeTableValue::Cp1258), "CP 1258", Some(false))],
            ..entry(MenuId::OtherCodeTables, "Bảng mã khác", None)
        },
        entry(MenuId::Exit, "Thoát", None),
    ];

    let map = MenuMap::new(&entries);

    assert_eq!(
        map.items(),
        [
            command(1, "Bật Tiếng Việt", Some(true)),
            MenuItem::Separator,
            MenuItem::Submenu {
                label: "Bảng mã khác".to_owned(),
                items: vec![command(2, "CP 1258", Some(false))],
            },
            command(3, "Thoát", None),
        ]
    );
    assert_eq!(map.chosen(1), Some(&MenuId::Vietnamese));
    assert_eq!(map.chosen(2), Some(&MenuId::CodeTable(CodeTableValue::Cp1258)));
    assert_eq!(map.chosen(3), Some(&MenuId::Exit));
}

#[test]
fn an_id_the_menu_never_had_chooses_nothing() {
    let map = MenuMap::new(&[entry(MenuId::Exit, "Thoát", None)]);

    assert_eq!(map.chosen(0), None, "zero is a dismissed menu");
    assert_eq!(map.chosen(2), None);
}

#[test]
fn every_entry_of_the_real_menu_is_chosen_by_its_own_id() {
    let entries = model(&PreferencesView::default(), Some("notepad.exe"));
    let map = MenuMap::new(&entries);

    let mut chosen = Vec::new();
    let mut id = 1;
    while let Some(entry) = map.chosen(id) {
        chosen.push(entry.clone());
        id += 1;
    }

    assert!(chosen.contains(&MenuId::AlwaysOnApp("notepad.exe".to_owned())));
    assert!(chosen.contains(&MenuId::CodeTable(CodeTableValue::UnicodeCompound)));
    assert!(chosen.contains(&MenuId::Exit));
    assert!(!chosen.contains(&MenuId::Separator), "a separator cannot be chosen");
}

#[test]
fn the_icon_shows_the_language() {
    assert_eq!(tray_icon(LanguageValue::Vietnamese), TrayIcon(2));
    assert_eq!(tray_icon(LanguageValue::English), TrayIcon(3));
}

#[test]
fn the_menu_offers_the_last_app_that_is_neither_cuakey_nor_the_desktop() {
    assert_eq!(menu_app("notepad.exe"), Some("notepad.exe"));
    assert_eq!(menu_app("cuakey.exe"), None, "the windows of CuaKey itself");
    assert_eq!(menu_app("explorer.exe"), None, "the taskbar the menu opens from");
    assert_eq!(menu_app(""), None, "an app whose name could not be read");
}

#[test]
fn a_tray_click_switches_language_like_the_menu() {
    let response = tray_response(TrayEvent::Click);

    assert_eq!(
        response,
        TrayResponse::Choose(MenuId::Vietnamese),
        "a click chooses the menu's language entry"
    );
    let mut prefs = PreferencesView::default();
    assert_eq!(prefs.language, LanguageValue::English, "the app starts in English");
    assert_eq!(
        apply(&mut prefs, MenuId::Vietnamese),
        MenuEffect::Save,
        "choosing it saves"
    );
    assert_eq!(prefs.language, LanguageValue::Vietnamese, "and switches the language");
    assert_eq!(
        tray_response(TrayEvent::Activate),
        TrayResponse::OpenSettings,
        "a double click still opens the settings"
    );
    assert_eq!(
        tray_response(TrayEvent::MenuRequested),
        TrayResponse::ShowMenu,
        "a right click still shows the menu"
    );
}
