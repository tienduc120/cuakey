use cuakey_application::{CodeTableValue, InputMethodValue, LanguageValue, PreferencesView, Toggle};

use super::{MenuEffect, MenuEntry, MenuId, apply, model};
use crate::WindowKind;

fn labels(entries: &[MenuEntry]) -> Vec<&str> {
    entries.iter().map(|entry| entry.label.as_str()).collect()
}

fn find<'a>(entries: &'a [MenuEntry], label: &str) -> &'a MenuEntry {
    entries
        .iter()
        .find(|entry| entry.label == label)
        .unwrap_or_else(|| panic!("no entry {label}"))
}

fn always_on_item(entries: &[MenuEntry]) -> &MenuEntry {
    entries
        .iter()
        .find(|entry| matches!(entry.id, MenuId::AlwaysOnApp(_)))
        .expect("the item is shown")
}

#[test]
fn menu_reflects_defaults() {
    let prefs = PreferencesView::default();
    let entries = model(&prefs, None);

    assert_eq!(
        labels(&entries),
        [
            "Bật Tiếng Việt",
            "",
            "Bật kiểm tra chính tả",
            "Bật loại trừ ứng dụng thông minh",
            "Bật gõ tắt",
            "",
            "Cấu hình gõ tắt...",
            "Công cụ chuyển mã...",
            "Chuyển mã nhanh",
            "",
            "Kiểu gõ Telex",
            "Kiểu gõ VNI",
            "Kiểu gõ Simple Telex",
            "",
            "Unicode dựng sẵn",
            "TCVN3 (ABC)",
            "VNI Windows",
            "Bảng mã khác",
            "",
            "Bảng điều khiển...",
            "Giới thiệu CuaKey",
            "",
            "Thoát",
        ]
    );
    assert_eq!(find(&entries, "Bật Tiếng Việt").checked, Some(true));
    assert_eq!(find(&entries, "Kiểu gõ Telex").checked, Some(true));
    assert_eq!(find(&entries, "Kiểu gõ VNI").checked, Some(false));
    assert_eq!(find(&entries, "Unicode dựng sẵn").checked, Some(true));
    assert_eq!(find(&entries, "TCVN3 (ABC)").checked, Some(false));
    assert_eq!(find(&entries, "Cấu hình gõ tắt...").checked, None);
    assert_eq!(find(&entries, "Thoát").checked, None);
}

#[test]
fn separators_are_entries_without_label_or_state() {
    let entries = model(&PreferencesView::default(), None);

    let separators: Vec<&MenuEntry> = entries.iter().filter(|entry| entry.id == MenuId::Separator).collect();

    assert_eq!(separators.len(), 6);
    for entry in separators {
        assert_eq!(entry.label, "");
        assert_eq!(entry.checked, None);
        assert_eq!(entry.children, Vec::<MenuEntry>::new());
    }
}

#[test]
fn other_code_tables_are_children_of_one_entry() {
    let mut prefs = PreferencesView::default();
    prefs.typing.code_table = CodeTableValue::Cp1258;
    let entries = model(&prefs, None);

    let other = find(&entries, "Bảng mã khác");

    assert_eq!(labels(&other.children), ["Unicode tổ hợp", "Vietnamese locale CP 1258"]);
    assert_eq!(other.children[0].checked, Some(false));
    assert_eq!(other.children[1].checked, Some(true));
    assert_eq!(find(&entries, "Unicode dựng sẵn").checked, Some(false));
}

#[test]
fn always_on_item_names_app() {
    let entries = model(&PreferencesView::default(), Some("game.exe"));

    let item = always_on_item(&entries);

    assert_eq!(item.label, "★ Luôn bật tiếng Việt cho game.exe");
    assert_eq!(item.id, MenuId::AlwaysOnApp("game.exe".to_owned()));
    assert_eq!(item.checked, Some(false));
    assert_eq!(entries.len(), 25, "the item and one more separator");
    assert_eq!(entries[5].id, MenuId::Separator);
    assert_eq!(entries[6].id, item.id);
    assert_eq!(entries[7].id, MenuId::Separator);
}

#[test]
fn always_on_item_hidden_without_a_last_app() {
    let entries = model(&PreferencesView::default(), None);

    assert!(entries.iter().all(|entry| !matches!(entry.id, MenuId::AlwaysOnApp(_))));
    assert_eq!(entries.len(), 23);
}

#[test]
fn always_on_item_is_ticked_for_an_app_already_listed() {
    let prefs = PreferencesView {
        always_on: vec!["game.exe".to_owned()],
        ..PreferencesView::default()
    };

    let entries = model(&prefs, Some(r"C:\Games\Game.EXE"));

    assert_eq!(always_on_item(&entries).checked, Some(true));
}

#[test]
fn menu_apply_each_toggle() {
    let mut prefs = PreferencesView::default();

    assert_eq!(apply(&mut prefs, MenuId::Vietnamese), MenuEffect::Save);
    assert_eq!(prefs.language, LanguageValue::English);
    assert_eq!(apply(&mut prefs, MenuId::Vietnamese), MenuEffect::Save);
    assert_eq!(prefs.language, LanguageValue::Vietnamese);

    let spell = prefs.typing.spell_check;
    assert_eq!(apply(&mut prefs, MenuId::SpellCheck), MenuEffect::Save);
    assert_ne!(prefs.typing.spell_check, spell);

    let smart = prefs.switching.smart_switch;
    assert_eq!(apply(&mut prefs, MenuId::SmartSwitch), MenuEffect::Save);
    assert_ne!(prefs.switching.smart_switch, smart);

    let macros = prefs.macros.use_macros;
    assert_eq!(apply(&mut prefs, MenuId::UseMacros), MenuEffect::Save);
    assert_ne!(prefs.macros.use_macros, macros);
    assert_eq!(apply(&mut prefs, MenuId::UseMacros), MenuEffect::Save);
    assert_eq!(prefs.macros.use_macros, macros);
}

#[test]
fn menu_apply_radio_choices() {
    let mut prefs = PreferencesView::default();

    assert_eq!(
        apply(&mut prefs, MenuId::InputMethod(InputMethodValue::Vni)),
        MenuEffect::Save
    );
    assert_eq!(prefs.typing.input_method, InputMethodValue::Vni);
    assert_eq!(
        apply(&mut prefs, MenuId::CodeTable(CodeTableValue::Tcvn3)),
        MenuEffect::Save
    );
    assert_eq!(prefs.typing.code_table, CodeTableValue::Tcvn3);
}

#[test]
fn menu_apply_toggles_the_always_on_app() {
    let mut prefs = PreferencesView::default();
    let id = MenuId::AlwaysOnApp(r"C:\Games\Game.EXE".to_owned());

    assert_eq!(apply(&mut prefs, id.clone()), MenuEffect::Save);
    assert_eq!(prefs.always_on, ["game.exe"]);
    assert_eq!(apply(&mut prefs, id), MenuEffect::Save);
    assert_eq!(prefs.always_on, Vec::<String>::new());
}

#[test]
fn menu_apply_ignores_a_blank_always_on_app() {
    let mut prefs = PreferencesView::default();

    assert_eq!(
        apply(&mut prefs, MenuId::AlwaysOnApp("  ".to_owned())),
        MenuEffect::None
    );
    assert_eq!(prefs, PreferencesView::default());
}

#[test]
fn menu_apply_opens_windows_and_runs_commands_without_touching_preferences() {
    let mut prefs = PreferencesView::default();

    for (id, effect) in [
        (MenuId::ConfigureMacros, MenuEffect::Open(WindowKind::Macros)),
        (MenuId::ConvertTool, MenuEffect::Open(WindowKind::Convert)),
        (MenuId::ControlPanel, MenuEffect::Open(WindowKind::Settings)),
        (MenuId::About, MenuEffect::Open(WindowKind::About)),
        (MenuId::QuickConvert, MenuEffect::QuickConvert),
        (MenuId::Exit, MenuEffect::Exit),
        (MenuId::OtherCodeTables, MenuEffect::None),
        (MenuId::Separator, MenuEffect::None),
    ] {
        assert_eq!(apply(&mut prefs, id.clone()), effect, "{id:?}");
    }
    assert_eq!(prefs, PreferencesView::default());
}

#[test]
fn toggle_entries_show_their_state() {
    let mut prefs = PreferencesView::default();
    prefs.typing.spell_check = Toggle::On;
    prefs.macros.use_macros = Toggle::Off;

    let entries = model(&prefs, None);

    assert_eq!(find(&entries, "Bật kiểm tra chính tả").checked, Some(true));
    assert_eq!(find(&entries, "Bật gõ tắt").checked, Some(false));
}
