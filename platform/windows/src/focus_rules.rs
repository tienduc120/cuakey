//! Whether the control with the keyboard focus takes typing, judged from what UI Automation says about it.

use crate::FocusClass;

/// What UI Automation and the control's window say about the focused control: everything [`classify`] judges it by.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ElementProps {
    /// The UI Automation control type id, such as 50004 for an edit box.
    pub control_type: i32,
    pub flags: ElementFlags,
    /// What the Value pattern says, or `None` for a control without one.
    pub value_read_only: Option<bool>,
    /// The class name UI Automation reports, which is the window class for a classic control.
    pub class_name: String,
    /// The lower-cased executable file name of the app whose top-level window holds the focus, such as `excel.exe`.
    pub process: String,
    /// A classic edit window with the `ES_PASSWORD` style, which hides what is typed.
    pub es_password: bool,
}

/// The yes-or-no properties UI Automation reports for the focused control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ElementFlags {
    /// UI Automation's `IsPassword`.
    pub password: bool,
    pub enabled: bool,
    /// The control offers the text-edit pattern.
    pub text_edit: bool,
}

// UI Automation control type ids.
const EDIT: i32 = 50004;
const DATA_ITEM: i32 = 50029;
const DOCUMENT: i32 = 50030;

const CONSOLE_CLASS: &str = "ConsoleWindowClass";
const EXCEL: &str = "excel.exe";
const TERMINALS: [&str; 2] = ["windowsterminal.exe", "openconsole.exe"];

/// What the focused control is. A password box is checked first, so no later rule can let typing into one; a disabled
/// control takes none; the built-in rules come before the generic ones because those apps' text areas are not edit
/// boxes or documents to UI Automation.
pub fn classify(props: &ElementProps) -> FocusClass {
    if props.flags.password || props.es_password {
        return FocusClass::Password;
    }
    if !props.flags.enabled {
        return FocusClass::NotText;
    }
    if built_in_text(props) {
        return FocusClass::Text;
    }
    let editable = match props.control_type {
        EDIT => props.value_read_only != Some(true),
        // A web page's body is a document too, so a document must show it takes typing: a writable value, or the
        // text-edit pattern, which a rich editor offers even when its value says read-only.
        DOCUMENT => props.value_read_only == Some(false) || props.flags.text_edit,
        _ => false,
    };
    if editable {
        FocusClass::Text
    } else {
        FocusClass::NotText
    }
}

/// Text areas UI Automation describes as something else: an Excel cell is a data item that typing edits, and a
/// console's or Windows Terminal's text area is a plain window or a custom control.
fn built_in_text(props: &ElementProps) -> bool {
    let process = props.process.as_str();
    (process == EXCEL && props.control_type == DATA_ITEM)
        || props.class_name == CONSOLE_CLASS
        || TERMINALS.contains(&process)
}

/// `ES_PASSWORD`, the edit window style that hides what is typed.
#[cfg(any(windows, test))]
const ES_PASSWORD: isize = 0x20;

/// Whether a window `style` hides what is typed. The bit means `ES_PASSWORD` only on an edit window; on other classes
/// it is a different style.
#[cfg(any(windows, test))]
pub(crate) const fn style_says_password(control_type: i32, style: isize) -> bool {
    control_type == EDIT && style & ES_PASSWORD != 0
}

#[cfg(test)]
mod tests;
