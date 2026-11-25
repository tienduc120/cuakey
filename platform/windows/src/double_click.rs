//! The system's double-click time.

use std::time::Duration;

use windows::Win32::UI::Input::KeyboardAndMouse::GetDoubleClickTime;

/// How long may pass between the two clicks of a double click, as the user set it in Windows.
pub fn double_click_time() -> Duration {
    // SAFETY: the call takes nothing and reads a system setting.
    let millis = unsafe { GetDoubleClickTime() };
    Duration::from_millis(u64::from(millis))
}

#[cfg(test)]
mod tests;
