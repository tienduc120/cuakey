use std::time::Duration;

use cuakey_application::{LanguageValue, PreferencesView};
use cuakey_interfaces::{MenuEffect, MenuId, apply};
use platform_clock::Timestamp;
use platform_windows::TrayEvent;

use super::{TrayClicks, TrayResponse};

const DOUBLE_CLICK: Duration = Duration::from_millis(500);

fn switch() -> TrayResponse {
    TrayResponse::Choose(MenuId::Vietnamese)
}

/// What the events do, each given as the event and how many milliseconds after the start it came.
fn run(events: &[(TrayEvent, i64)]) -> Vec<TrayResponse> {
    let mut clicks = TrayClicks::default();
    events
        .iter()
        .flat_map(|&(event, millis)| clicks.respond(event, Timestamp::from_millisecond(millis).unwrap(), DOUBLE_CLICK))
        .collect()
}

#[test]
fn a_tray_click_switches_language_like_the_menu() {
    let responses = run(&[(TrayEvent::Click, 0)]);

    assert_eq!(responses, [switch()], "a click chooses the menu's language entry once");
    let mut prefs = PreferencesView::default();
    assert_eq!(prefs.language, LanguageValue::English, "the app starts in English");
    assert_eq!(
        apply(&mut prefs, MenuId::Vietnamese),
        MenuEffect::Save,
        "choosing it saves"
    );
    assert_eq!(prefs.language, LanguageValue::Vietnamese, "and switches the language");
}

#[test]
fn a_right_click_shows_the_menu() {
    assert_eq!(run(&[(TrayEvent::MenuRequested, 0)]), [TrayResponse::ShowMenu]);
}

#[test]
fn a_double_click_switches_back_and_opens_the_settings() {
    let responses = run(&[(TrayEvent::Click, 0), (TrayEvent::DoubleClick, 200)]);

    assert_eq!(responses, [switch(), switch(), TrayResponse::OpenSettings]);
}

#[test]
fn the_click_that_follows_a_double_click_is_ignored() {
    let responses = run(&[
        (TrayEvent::Click, 0),
        (TrayEvent::DoubleClick, 200),
        (TrayEvent::Click, 250),
    ]);

    assert_eq!(
        responses,
        [switch(), switch(), TrayResponse::OpenSettings],
        "two switches cancel out, and the settings open once"
    );
}

#[test]
fn a_click_after_the_time_is_a_click_of_its_own() {
    let responses = run(&[(TrayEvent::Click, 0), (TrayEvent::Click, 501)]);

    assert_eq!(responses, [switch(), switch()]);
}

#[test]
fn a_double_click_with_no_click_before_it_only_opens_the_settings() {
    assert_eq!(run(&[(TrayEvent::DoubleClick, 0)]), [TrayResponse::OpenSettings]);
}

#[test]
fn a_double_click_long_after_a_click_does_not_switch_back() {
    let responses = run(&[(TrayEvent::Click, 0), (TrayEvent::DoubleClick, 900)]);

    assert_eq!(responses, [switch(), TrayResponse::OpenSettings]);
}

#[test]
fn a_click_long_after_a_double_click_switches() {
    let responses = run(&[(TrayEvent::DoubleClick, 0), (TrayEvent::Click, 900)]);

    assert_eq!(responses, [TrayResponse::OpenSettings, switch()]);
}

#[test]
fn the_keyboard_opens_the_settings_and_undoes_nothing() {
    let responses = run(&[(TrayEvent::Click, 0), (TrayEvent::Activate, 100)]);

    assert_eq!(responses, [switch(), TrayResponse::OpenSettings]);
}
