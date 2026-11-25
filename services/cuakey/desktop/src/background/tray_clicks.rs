//! What the icon does for each click on it. Windows reports the first click of a double click like a click of its own,
//! so the language is switched at once, and switched back when the double click turns out to be one.

use std::time::Duration;

use cuakey_interfaces::MenuId;
use platform_clock::Timestamp;
use platform_windows::TrayEvent;

/// What the background does for an event on the icon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TrayResponse {
    OpenSettings,
    ShowMenu,
    /// The same as choosing this entry of the menu.
    Choose(MenuId),
}

/// The clicks the icon last saw, to tell a double click from two clicks.
#[derive(Debug, Default)]
pub(crate) struct TrayClicks {
    /// The last click that switched the language and has not been taken back.
    click: Option<Timestamp>,
    last_double_click: Option<Timestamp>,
}

impl TrayClicks {
    /// What to do for `event`, which arrived at `now`, when two clicks within `double_click_time` make a double click.
    pub(crate) fn respond(
        &mut self,
        event: TrayEvent,
        now: Timestamp,
        double_click_time: Duration,
    ) -> Vec<TrayResponse> {
        match event {
            TrayEvent::Click => {
                // Some versions of Windows report the click that ends a double click as well.
                if within(self.last_double_click, now, double_click_time) {
                    return Vec::new();
                }
                self.click = Some(now);
                vec![switch_language()]
            }
            TrayEvent::DoubleClick => {
                let undo = within(self.click.take(), now, double_click_time);
                self.last_double_click = Some(now);
                let mut responses = Vec::new();
                if undo {
                    responses.push(switch_language());
                }
                responses.push(TrayResponse::OpenSettings);
                responses
            }
            TrayEvent::Activate => vec![TrayResponse::OpenSettings],
            TrayEvent::MenuRequested => vec![TrayResponse::ShowMenu],
        }
    }
}

/// The menu's language entry, which flips the language.
const fn switch_language() -> TrayResponse {
    TrayResponse::Choose(MenuId::Vietnamese)
}

/// Whether `earlier` is an instant at most `span` before `now`.
fn within(earlier: Option<Timestamp>, now: Timestamp, span: Duration) -> bool {
    earlier.is_some_and(|earlier| Duration::try_from(now.duration_since(earlier)).is_ok_and(|elapsed| elapsed <= span))
}

#[cfg(test)]
mod tests;
