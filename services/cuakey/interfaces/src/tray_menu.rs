//! The notification-area menu as plain data: what it shows for the current preferences, in their interface language,
//! and what a click changes. It knows no Win32, so the process that draws the menu only translates entries to menu
//! items and clicks to ids.

use cuakey_application::{CodeTableValue, InputMethodValue, LanguageValue, PreferencesView, Toggle, app_name};

use crate::WindowKind;
use crate::labels::{Labels, labels_for};
use crate::settings_form::{is_on, toggle};

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

const fn input_method_items(text: &Labels) -> [(InputMethodValue, &'static str); 3] {
    [
        (InputMethodValue::Telex, text.menu_input_method_telex),
        (InputMethodValue::Vni, text.menu_input_method_vni),
        (InputMethodValue::SimpleTelex, text.menu_input_method_simple_telex),
    ]
}

const fn main_code_tables(text: &Labels) -> [(CodeTableValue, &'static str); 3] {
    [
        (CodeTableValue::Unicode, text.code_table_unicode),
        (CodeTableValue::Tcvn3, text.code_table_tcvn3),
        (CodeTableValue::VniWindows, text.code_table_vni_windows),
    ]
}

const fn other_code_tables(text: &Labels) -> [(CodeTableValue, &'static str); 2] {
    [
        (CodeTableValue::UnicodeCompound, text.code_table_unicode_compound),
        (CodeTableValue::Cp1258, text.code_table_cp1258),
    ]
}

fn code_table_entry(prefs: &PreferencesView, table: CodeTableValue, label: &str) -> MenuEntry {
    MenuEntry::item(MenuId::CodeTable(table), label, Some(prefs.typing.code_table == table))
}

fn switches(prefs: &PreferencesView, text: &Labels) -> Vec<MenuEntry> {
    vec![
        MenuEntry::item(
            MenuId::Vietnamese,
            text.menu_vietnamese,
            Some(prefs.language == LanguageValue::Vietnamese),
        ),
        MenuEntry::separator(),
        MenuEntry::item(
            MenuId::SpellCheck,
            text.menu_spell_check,
            Some(is_on(prefs.typing.spell_check)),
        ),
        MenuEntry::item(
            MenuId::SmartSwitch,
            text.menu_smart_switch,
            Some(is_on(prefs.switching.smart_switch)),
        ),
        MenuEntry::item(
            MenuId::UseMacros,
            text.menu_use_macros,
            Some(is_on(prefs.macros.use_macros)),
        ),
        MenuEntry::separator(),
    ]
}

/// The item for the app that had the focus before the menu opened. It is hidden when there is no such app.
fn always_on_item(prefs: &PreferencesView, text: &Labels, last_app: Option<&str>) -> Vec<MenuEntry> {
    let Some(name) = last_app.and_then(|app| app_name(app).ok()) else {
        return Vec::new();
    };
    let listed = prefs.always_on.contains(&name);
    let label = format!("{}{name}", text.menu_always_on_prefix);
    vec![
        MenuEntry::item(MenuId::AlwaysOnApp(name), &label, Some(listed)),
        MenuEntry::separator(),
    ]
}

fn tools(text: &Labels) -> Vec<MenuEntry> {
    vec![
        MenuEntry::item(MenuId::ConfigureMacros, text.menu_configure_macros, None),
        MenuEntry::item(MenuId::ConvertTool, text.menu_convert_tool, None),
        MenuEntry::item(MenuId::QuickConvert, text.menu_quick_convert, None),
        MenuEntry::separator(),
    ]
}

fn input_methods(prefs: &PreferencesView, text: &Labels) -> Vec<MenuEntry> {
    let mut entries: Vec<MenuEntry> = input_method_items(text)
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

fn code_tables(prefs: &PreferencesView, text: &Labels) -> Vec<MenuEntry> {
    let mut entries: Vec<MenuEntry> = main_code_tables(text)
        .iter()
        .map(|(table, label)| code_table_entry(prefs, *table, label))
        .collect();
    entries.push(MenuEntry {
        children: other_code_tables(text)
            .iter()
            .map(|(table, label)| code_table_entry(prefs, *table, label))
            .collect(),
        ..MenuEntry::item(MenuId::OtherCodeTables, text.menu_other_code_tables, None)
    });
    entries.push(MenuEntry::separator());
    entries
}

fn windows_and_exit(text: &Labels) -> Vec<MenuEntry> {
    vec![
        MenuEntry::item(MenuId::ControlPanel, text.menu_control_panel, None),
        MenuEntry::item(MenuId::About, text.menu_about, None),
        MenuEntry::separator(),
        MenuEntry::item(MenuId::Exit, text.menu_exit, None),
    ]
}

/// The menu for `prefs`, in their interface language. `last_app` is the app that had the focus before the menu opened,
/// if any is known.
#[must_use]
pub fn model(prefs: &PreferencesView, last_app: Option<&str>) -> Vec<MenuEntry> {
    let text = labels_for(prefs);
    [
        switches(prefs, text),
        always_on_item(prefs, text, last_app),
        tools(text),
        input_methods(prefs, text),
        code_tables(prefs, text),
        windows_and_exit(text),
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
