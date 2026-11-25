use super::{Button, UiAction, WindowKind};

#[test]
fn buttons_map_to_actions() {
    assert_eq!(Button::ExitApp.action(), Some(UiAction::Exit));
    assert_eq!(Button::ResetToDefaults.action(), Some(UiAction::Reset));
    assert_eq!(Button::CloseWindow.action(), Some(UiAction::Close));
    assert_eq!(Button::OpenMacros.action(), Some(UiAction::Open(WindowKind::Macros)));
    assert_eq!(
        Button::OpenAlwaysOn.action(),
        Some(UiAction::Open(WindowKind::AlwaysOn))
    );
    assert_eq!(Button::ToggleExpanded.action(), None, "the window handles it itself");
}
