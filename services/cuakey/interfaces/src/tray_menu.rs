//! The notification-area menu as plain data: what it shows for the current preferences, and what a click changes.
//! It knows no Win32, so the process that draws the menu only translates entries to menu items and clicks to ids.

use cuakey_application::{CodeTableValue, InputMethodValue, LanguageValue, PreferencesView, Toggle, app_name};

use crate::settings_form::{is_on, toggle};
use crate::{WindowKind, labels};

/// What a menu item stands for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuId {
    Separator,
    Vietnamese,
    SpellCheck,
    SmartSwitch,
    UseMacros,
    /// Toggles this app in the always-on list.
    AlwaysOnApp(String),
    ConfigureMacros,
    ConvertTool,
    QuickConvert,
    InputMethod(InputMethodValue),
    CodeTable(CodeTableValue),
    /// The parent of the less common code tables; it does nothing by itself.
    OtherCodeTables,
    ControlPanel,
    About,
    Exit,
}

/// One item of the menu, or a separator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuEntry {
    pub id: MenuId,
    pub label: String,
    /// `Some` for an item that has a tick, ticked or not; `None` for an item that is only clicked.
    pub checked: Option<bool>,
    /// The items of a submenu.
    pub children: Vec<Self>,
}

/// What the background process does after a click.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuEffect {
    /// The click changed the preferences: store them and apply them.
    Save,
    Open(WindowKind),
    /// Convert the clipboard with the stored options.
    QuickConvert,
    Exit,
    None,
}

impl MenuEntry {
    fn item(id: MenuId, label: &str, checked: Option<bool>) -> Self {
        Self {
            id,
            label: label.to_owned(),
            checked,
            children: Vec::new(),
        }
    }

    fn separator() -> Self {
        Self::item(MenuId::Separator, "", None)
    }
}

const INPUT_METHODS: [(InputMethodValue, &str); 3] = [
    (InputMethodValue::Telex, labels::MENU_INPUT_METHOD_TELEX),
    (InputMethodValue::Vni, labels::MENU_INPUT_METHOD_VNI),
    (InputMethodValue::SimpleTelex, labels::MENU_INPUT_METHOD_SIMPLE_TELEX),
];

const MAIN_CODE_TABLES: [(CodeTableValue, &str); 3] = [
    (CodeTableValue::Unicode, labels::CODE_TABLE_UNICODE),
    (CodeTableValue::Tcvn3, labels::CODE_TABLE_TCVN3),
    (CodeTableValue::VniWindows, labels::CODE_TABLE_VNI_WINDOWS),
];

const OTHER_CODE_TABLES: [(CodeTableValue, &str); 2] = [
    (CodeTableValue::UnicodeCompound, labels::CODE_TABLE_UNICODE_COMPOUND),
    (CodeTableValue::Cp1258, labels::CODE_TABLE_CP1258),
];

fn code_table_entry(prefs: &PreferencesView, table: CodeTableValue, label: &str) -> MenuEntry {
    MenuEntry::item(MenuId::CodeTable(table), label, Some(prefs.typing.code_table == table))
}

fn switches(prefs: &PreferencesView) -> Vec<MenuEntry> {
    vec![
        MenuEntry::item(
            MenuId::Vietnamese,
            labels::MENU_VIETNAMESE,
            Some(prefs.language == LanguageValue::Vietnamese),
        ),
        MenuEntry::separator(),
        MenuEntry::item(
            MenuId::SpellCheck,
            labels::MENU_SPELL_CHECK,
            Some(is_on(prefs.typing.spell_check)),
        ),
        MenuEntry::item(
            MenuId::SmartSwitch,
            labels::MENU_SMART_SWITCH,
            Some(is_on(prefs.switching.smart_switch)),
        ),
        MenuEntry::item(
            MenuId::UseMacros,
            labels::MENU_USE_MACROS,
            Some(is_on(prefs.macros.use_macros)),
        ),
        MenuEntry::separator(),
    ]
}

/// The item for the app that had the focus before the menu opened. It is hidden when there is no such app.
fn always_on_item(prefs: &PreferencesView, last_app: Option<&str>) -> Vec<MenuEntry> {
    let Some(name) = last_app.and_then(|app| app_name(app).ok()) else {
        return Vec::new();
    };
    let listed = prefs.always_on.contains(&name);
    let label = format!("{}{name}", labels::MENU_ALWAYS_ON_PREFIX);
    vec![
        MenuEntry::item(MenuId::AlwaysOnApp(name), &label, Some(listed)),
        MenuEntry::separator(),
    ]
}

fn tools() -> Vec<MenuEntry> {
    vec![
        MenuEntry::item(MenuId::ConfigureMacros, labels::MENU_CONFIGURE_MACROS, None),
        MenuEntry::item(MenuId::ConvertTool, labels::MENU_CONVERT_TOOL, None),
        MenuEntry::item(MenuId::QuickConvert, labels::MENU_QUICK_CONVERT, None),
        MenuEntry::separator(),
    ]
}

fn input_methods(prefs: &PreferencesView) -> Vec<MenuEntry> {
    let mut entries: Vec<MenuEntry> = INPUT_METHODS
        .iter()
        .map(|(method, label)| {
            MenuEntry::item(
                MenuId::InputMethod(*method),
                label,
                Some(prefs.typing.input_method == *method),
            )
        })
        .collect();
    entries.push(MenuEntry::separator());
    entries
}

fn code_tables(prefs: &PreferencesView) -> Vec<MenuEntry> {
    let mut entries: Vec<MenuEntry> = MAIN_CODE_TABLES
        .iter()
        .map(|(table, label)| code_table_entry(prefs, *table, label))
        .collect();
    entries.push(MenuEntry {
        children: OTHER_CODE_TABLES
            .iter()
            .map(|(table, label)| code_table_entry(prefs, *table, label))
            .collect(),
        ..MenuEntry::item(MenuId::OtherCodeTables, labels::MENU_OTHER_CODE_TABLES, None)
    });
    entries.push(MenuEntry::separator());
    entries
}

fn windows_and_exit() -> Vec<MenuEntry> {
    vec![
        MenuEntry::item(MenuId::ControlPanel, labels::MENU_CONTROL_PANEL, None),
        MenuEntry::item(MenuId::About, labels::MENU_ABOUT, None),
        MenuEntry::separator(),
        MenuEntry::item(MenuId::Exit, labels::MENU_EXIT, None),
    ]
}

/// The menu for `prefs`. `last_app` is the app that had the focus before the menu opened, if any is known.
#[must_use]
pub fn model(prefs: &PreferencesView, last_app: Option<&str>) -> Vec<MenuEntry> {
    [
        switches(prefs),
        always_on_item(prefs, last_app),
        tools(),
        input_methods(prefs),
        code_tables(prefs),
        windows_and_exit(),
    ]
    .concat()
}

fn flip(setting: &mut Toggle) {
    *setting = toggle(!is_on(*setting));
}

/// Adds the app to the always-on list, or takes it out when it is there. Text with no file name changes nothing.
fn toggle_always_on(prefs: &mut PreferencesView, app: &str) -> MenuEffect {
    let Ok(name) = app_name(app) else {
        return MenuEffect::None;
    };
    if let Some(position) = prefs.always_on.iter().position(|listed| *listed == name) {
        prefs.always_on.remove(position);
    } else {
        prefs.always_on.push(name);
        prefs.always_on.sort();
    }
    MenuEffect::Save
}

/// What a click on `id` changes in `prefs`, and what the caller does next.
pub fn apply(prefs: &mut PreferencesView, id: MenuId) -> MenuEffect {
    match id {
        MenuId::Vietnamese => {
            prefs.language = match prefs.language {
                LanguageValue::Vietnamese => LanguageValue::English,
                LanguageValue::English => LanguageValue::Vietnamese,
            };
        }
        MenuId::SpellCheck => flip(&mut prefs.typing.spell_check),
        MenuId::SmartSwitch => flip(&mut prefs.switching.smart_switch),
        MenuId::UseMacros => flip(&mut prefs.macros.use_macros),
        MenuId::InputMethod(method) => prefs.typing.input_method = method,
        MenuId::CodeTable(table) => prefs.typing.code_table = table,
        MenuId::AlwaysOnApp(app) => return toggle_always_on(prefs, &app),
        MenuId::ConfigureMacros => return MenuEffect::Open(WindowKind::Macros),
        MenuId::ConvertTool => return MenuEffect::Open(WindowKind::Convert),
        MenuId::ControlPanel => return MenuEffect::Open(WindowKind::Settings),
        MenuId::About => return MenuEffect::Open(WindowKind::About),
        MenuId::QuickConvert => return MenuEffect::QuickConvert,
        MenuId::Exit => return MenuEffect::Exit,
        MenuId::Separator | MenuId::OtherCodeTables => return MenuEffect::None,
    }
    MenuEffect::Save
}

#[cfg(test)]
mod tests;
