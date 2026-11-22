use super::{FocusState, should_convert};
use crate::OnOff::{Off, On};

#[test]
fn password_never_converts() {
    assert!(!should_convert(FocusState::Password, Off, On));
    assert!(!should_convert(FocusState::Password, Off, Off));
}

#[test]
fn pending_focus_passes() {
    assert!(!should_convert(FocusState::Pending, Off, On));
    assert!(!should_convert(FocusState::Pending, Off, Off));
}

#[test]
fn text_always_converts() {
    assert!(should_convert(FocusState::Text, On, Off));
    assert!(should_convert(FocusState::Text, Off, Off));
}

#[test]
fn not_text_converts_only_when_allowed() {
    assert!(
        !should_convert(FocusState::NotText, On, Off),
        "text boxes only, app not always on"
    );
    assert!(should_convert(FocusState::NotText, On, On), "app always on");
    assert!(should_convert(FocusState::NotText, Off, Off), "text boxes only is off");
}

#[test]
fn unknown_converts_only_when_allowed() {
    assert!(!should_convert(FocusState::Unknown, On, Off));
    assert!(should_convert(FocusState::Unknown, On, On));
    assert!(should_convert(FocusState::Unknown, Off, Off));
}
