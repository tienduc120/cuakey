use super::{Button, UiAction, WindowKind};

#[test]
fn buttons_map_to_actions() {
    assert_eq!(Button::ExitApp.action(), UiAction::Exit);
    assert_eq!(Button::ResetToDefaults.action(), UiAction::Reset);
    assert_eq!(Button::CloseWindow.action(), UiAction::Close);
    assert_eq!(Button::OpenMacros.action(), UiAction::Open(WindowKind::Macros));
    assert_eq!(Button::OpenAlwaysOn.action(), UiAction::Open(WindowKind::AlwaysOn));
}
