use crate::OnOff;

/// What is known about the control that has the keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusState {
    /// Focus just moved and the control is still being classified.
    Pending,
    Text,
    Password,
    NotText,
    Unknown,
}

/// Whether typing in the focused control is converted. A password box and a control still being classified never
/// are; a control that is not known to be a text box is only when the app is always on or text boxes are not required.
pub const fn should_convert(focus: FocusState, only_text_boxes: OnOff, app_always_on: OnOff) -> bool {
    match focus {
        FocusState::Pending | FocusState::Password => false,
        FocusState::Text => true,
        FocusState::NotText | FocusState::Unknown => !only_text_boxes.is_on() || app_always_on.is_on(),
    }
}

#[cfg(test)]
mod tests;
