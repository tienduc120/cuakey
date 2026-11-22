use std::cell::RefCell;
use std::rc::Rc;

use super::{MessageWindow, PostError, post_to};
use crate::{AppMessage, WindowKind};

fn unique_class(tag: &str) -> String {
    format!("CuaKeyTest.{tag}.{}", std::process::id())
}

#[test]
fn posted_messages_reach_the_handler_until_quit() {
    let class = unique_class("loop");
    let seen = Rc::new(RefCell::new(Vec::new()));
    let sink = Rc::clone(&seen);
    let window = MessageWindow::new(&class, move |message| sink.borrow_mut().push(message)).unwrap();

    post_to(&class, AppMessage::Reload).unwrap();
    post_to(&class, AppMessage::ShowWindow(WindowKind::About)).unwrap();
    post_to(&class, AppMessage::Quit).unwrap();
    window.run_loop();

    assert_eq!(
        *seen.borrow(),
        [
            AppMessage::Reload,
            AppMessage::ShowWindow(WindowKind::About),
            AppMessage::Quit
        ],
        "each message arrives once, in order, and quit ends the loop"
    );
}

#[test]
fn posting_to_a_missing_window_says_not_running() {
    let result = post_to(&unique_class("missing"), AppMessage::Reload);

    assert!(matches!(result, Err(PostError::NotRunning)), "no window has that class");
}

#[test]
fn class_is_free_again_after_the_window_drops() {
    let class = unique_class("reuse");

    drop(MessageWindow::new(&class, |_| {}).unwrap());
    let again = MessageWindow::new(&class, |_| {});

    assert!(again.is_ok(), "dropping the window unregisters its class");
}
