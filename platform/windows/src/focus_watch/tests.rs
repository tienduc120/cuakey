use std::cell::Cell;

use platform_clock::Timestamp;

use super::{FocusChange, app_name, app_window, log_line, next_change, request_recheck};
use crate::{ElementFlags, ElementProps, FocusCell, FocusClass};

#[test]
fn app_changed_compares_with_the_last_app() {
    let first = next_change(None, "notepad.exe".to_owned(), FocusClass::Text);
    let same = next_change(Some("notepad.exe"), "notepad.exe".to_owned(), FocusClass::NotText);
    let other = next_change(Some("notepad.exe"), "chrome.exe".to_owned(), FocusClass::Password);

    assert_eq!(
        first,
        FocusChange {
            app: "notepad.exe".to_owned(),
            app_changed: true,
            class: FocusClass::Text,
        },
        "the first change brings an app to the front"
    );
    assert!(
        !same.app_changed,
        "another control in the same app is not an app change"
    );
    assert!(other.app_changed, "a control in another app is");
}

#[test]
fn app_names_are_lower_cased_file_names() {
    let names = [
        r"C:\Program Files\Google\Chrome\Application\Chrome.EXE",
        r"C:\Games\game.exe",
        "",
    ]
    .map(app_name);

    assert_eq!(
        names,
        ["chrome.exe", "game.exe", ""],
        "an app is named by its executable's file name in lower case, so the always-on list and per-app memory match it"
    );
}

#[test]
fn log_line_names_every_property() {
    let time: Timestamp = "2026-11-22T08:30:00Z".parse().unwrap();
    let props = ElementProps {
        control_type: 50004,
        flags: ElementFlags {
            password: false,
            enabled: true,
            text_edit: true,
        },
        value_read_only: Some(false),
        class_name: "Edit".to_owned(),
        process: "notepad.exe".to_owned(),
        es_password: false,
    };

    let line = log_line(time, "notepad.exe", Ok::<_, &str>(&props), FocusClass::Text);

    assert_eq!(
        line,
        "2026-11-22T08:30:00Z process=\"notepad.exe\" class_name=\"Edit\" control_type=50004 password=false \
         enabled=true text_edit=true value_read_only=Some(false) es_password=false result=Text\n",
        "one line per classification: the time, the app, what UI Automation said, and the result"
    );
}

#[test]
fn log_line_reports_a_failed_read() {
    let time: Timestamp = "2026-11-22T08:30:00Z".parse().unwrap();

    let line = log_line(
        time,
        "chrome.exe",
        Err::<&ElementProps, _>("timed out"),
        FocusClass::Unknown,
    );

    assert_eq!(
        line, "2026-11-22T08:30:00Z process=\"chrome.exe\" error=\"timed out\" result=Unknown\n",
        "a failed read is logged with its error in place of the properties"
    );
}

#[cfg(windows)]
#[test]
#[ignore = "drives the real UI Automation client, which needs a signed-in desktop"]
fn the_watch_answers_a_recheck() {
    use std::sync::{Arc, mpsc};
    use std::time::Duration;

    use platform_clock::FixedClock;

    use super::FocusWatch;

    let cell = Arc::new(FocusCell::new());
    let (changes, received) = mpsc::channel();
    let watch = FocusWatch::start(
        Arc::clone(&cell),
        move |change| {
            let _sent = changes.send(change);
        },
        None,
        FixedClock(Timestamp::UNIX_EPOCH),
    )
    .unwrap();

    watch.recheck();
    let change = received.recv_timeout(Duration::from_secs(10)).unwrap();
    watch.stop().unwrap();

    assert_ne!(
        change.class,
        FocusClass::Pending,
        "the watch answers a recheck with a class"
    );
    assert_ne!(cell.get(), FocusClass::Pending, "and records it in the cell");
}

#[test]
fn recheck_marks_the_focus_pending_before_queueing() {
    let cell = FocusCell::new();
    let answered = cell.invalidate();
    cell.set(FocusClass::Text, answered);
    let mut queued = None;

    let posted = request_recheck(&cell, |generation| {
        queued = Some((cell.get(), generation));
        Ok::<(), &str>(())
    });
    let (class_when_queued, generation) = queued.unwrap();

    assert_eq!(posted, Ok(()), "the recheck reports how queueing went");
    assert_eq!(
        class_when_queued,
        FocusClass::Pending,
        "the keyboard hook sees pending before the watch thread hears of the recheck"
    );
    assert!(
        cell.set(FocusClass::Password, generation),
        "the watch thread answers with the generation it was sent"
    );
    assert!(
        !cell.set(FocusClass::Text, answered),
        "the answer about the control before the recheck no longer lands"
    );
}

#[test]
fn the_app_comes_from_the_window_with_the_focus() {
    let from_event = app_window(Some("event"), || Some("focus"), || Some("front"));
    let from_focus = app_window(None, || Some("focus"), || Some("front"));
    let from_front = app_window(None, || None, || Some("front"));
    let from_nothing = app_window::<&str>(None, || None, || None);

    assert_eq!(
        from_event,
        Some("event"),
        "a focus change names the window it happened in, even before that window's app is in front"
    );
    assert_eq!(
        from_focus,
        Some("focus"),
        "a recheck asks the thread in front which window has the focus"
    );
    assert_eq!(
        from_front,
        Some("front"),
        "and takes the window in front when that thread names none"
    );
    assert_eq!(from_nothing, None, "with no window at all, the app goes unnamed");
}

#[test]
fn an_event_window_needs_no_further_lookup() {
    let lookups = Cell::new(0);
    let look = |window| {
        lookups.set(lookups.get() + 1);
        Some(window)
    };

    let found = app_window(Some(1), || look(2), || look(3));

    assert_eq!(
        (found, lookups.get()),
        (Some(1), 0),
        "the system calls that find the focus in front are made only for a recheck"
    );
}
