//! The notification-area icon and its menu, translated from the menu model to what the tray draws.

use cuakey_application::LanguageValue;
use cuakey_interfaces::{MenuEntry, MenuId};
use platform_windows::{MenuItem, TrayEvent, TrayIcon};

/// The executable's icon resources the tray shows, one per language.
const VIETNAMESE_ICON: TrayIcon = TrayIcon(2);
const ENGLISH_ICON: TrayIcon = TrayIcon(3);

/// Apps the menu never offers to keep always on: `CuaKey` itself, and the taskbar the menu opens from.
const NOT_OFFERED: [&str; 2] = ["cuakey.exe", "explorer.exe"];

/// The icon for the language being typed.
pub(crate) const fn tray_icon(language: LanguageValue) -> TrayIcon {
    match language {
        LanguageValue::Vietnamese => VIETNAMESE_ICON,
        LanguageValue::English => ENGLISH_ICON,
    }
}

/// What the background does for an event on the icon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TrayResponse {
    OpenSettings,
    ShowMenu,
    /// The same as choosing this entry of the menu.
    Choose(MenuId),
}

/// A click on the icon switches the language like the menu's language entry; a double click opens the settings, and a
/// right click shows the menu.
pub(crate) const fn tray_response(event: TrayEvent) -> TrayResponse {
    match event {
        TrayEvent::Click => TrayResponse::Choose(MenuId::Vietnamese),
        TrayEvent::Activate => TrayResponse::OpenSettings,
        TrayEvent::MenuRequested => TrayResponse::ShowMenu,
    }
}

/// The app a focus change names, if the menu may offer it as the last app in front.
pub(crate) fn menu_app(app: &str) -> Option<&str> {
    (!app.is_empty() && !NOT_OFFERED.contains(&app)).then_some(app)
}

/// The menu's items for the tray, and the entry each id stands for. Ids count from 1 in the order the entries appear,
/// submenus included, since the tray refuses 0.
#[derive(Debug)]
pub(crate) struct MenuMap {
    items: Vec<MenuItem>,
    ids: Vec<MenuId>,
}

impl MenuMap {
    pub(crate) fn new(entries: &[MenuEntry]) -> Self {
        let mut ids = Vec::new();
        let items = items(entries, &mut ids);
        Self { items, ids }
    }

    pub(crate) fn items(&self) -> &[MenuItem] {
        &self.items
    }

    /// The entry the tray answered with `id`, or `None` for an id the menu never had.
    pub(crate) fn chosen(&self, id: u32) -> Option<&MenuId> {
        let index = usize::try_from(id).ok()?.checked_sub(1)?;
        self.ids.get(index)
    }
}

fn items(entries: &[MenuEntry], ids: &mut Vec<MenuId>) -> Vec<MenuItem> {
    entries
        .iter()
        .map(|entry| {
            if entry.id == MenuId::Separator {
                MenuItem::Separator
            } else if entry.children.is_empty() {
                ids.push(entry.id.clone());
                MenuItem::Command {
                    id: u32::try_from(ids.len()).unwrap_or(u32::MAX),
                    label: entry.label.clone(),
                    checked: entry.checked,
                    enabled: true,
                }
            } else {
                MenuItem::Submenu {
                    label: entry.label.clone(),
                    items: items(&entry.children, ids),
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests;
