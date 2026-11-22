use super::{ElementFlags, ElementProps, classify, style_says_password};
use crate::FocusClass;

// UI Automation control type ids.
const BUTTON: i32 = 50000;
const EDIT: i32 = 50004;
const LIST: i32 = 50008;
const TEXT: i32 = 50020;
const DATA_ITEM: i32 = 50029;
const DOCUMENT: i32 = 50030;
const PANE: i32 = 50033;

/// `ES_PASSWORD`, the edit window style that hides what is typed.
const ES_PASSWORD: isize = 0x20;

const ENABLED: ElementFlags = ElementFlags {
    password: false,
    enabled: true,
    text_edit: false,
};

/// An enabled control of `control_type` in `process`, with nothing else known about it.
fn element(control_type: i32, process: &str, class_name: &str) -> ElementProps {
    ElementProps {
        control_type,
        flags: ENABLED,
        value_read_only: None,
        class_name: class_name.to_owned(),
        process: process.to_owned(),
        es_password: false,
    }
}

#[test]
fn chrome_input() {
    let input = ElementProps {
        value_read_only: Some(false),
        ..element(EDIT, "chrome.exe", "")
    };

    assert_eq!(
        classify(&input),
        FocusClass::Text,
        "a web page's text field takes typing"
    );
}

#[test]
fn chrome_password() {
    let password = ElementProps {
        flags: ElementFlags {
            password: true,
            ..ENABLED
        },
        value_read_only: Some(false),
        ..element(EDIT, "chrome.exe", "")
    };

    assert_eq!(
        classify(&password),
        FocusClass::Password,
        "a web page's password field is never typed into"
    );
}

#[test]
fn chrome_page_body() {
    let page = ElementProps {
        value_read_only: Some(true),
        ..element(DOCUMENT, "chrome.exe", "Chrome_RenderWidgetHostHWND")
    };

    assert_eq!(
        classify(&page),
        FocusClass::NotText,
        "a page body whose value is read-only, with no text-edit pattern, takes no typing"
    );
}

#[test]
fn contenteditable() {
    let editor = ElementProps {
        flags: ElementFlags {
            text_edit: true,
            ..ENABLED
        },
        value_read_only: Some(true),
        ..element(DOCUMENT, "chrome.exe", "")
    };

    assert_eq!(
        classify(&editor),
        FocusClass::Text,
        "the text-edit pattern marks a document that takes typing, even when its value says read-only"
    );
}

#[test]
fn notepad_edit() {
    let classic = ElementProps {
        value_read_only: Some(false),
        ..element(EDIT, "notepad.exe", "Edit")
    };
    let document = ElementProps {
        flags: ElementFlags {
            text_edit: true,
            ..ENABLED
        },
        value_read_only: Some(false),
        ..element(DOCUMENT, "notepad.exe", "RichEditD2DPT")
    };

    assert_eq!(
        classify(&classic),
        FocusClass::Text,
        "a classic edit window takes typing"
    );
    assert_eq!(
        classify(&document),
        FocusClass::Text,
        "the new Notepad's document says it is writable and takes typing"
    );
}

#[test]
fn only_a_document_needs_evidence_of_editing() {
    let silent_document = element(DOCUMENT, "firefox.exe", "");
    let writable_document = ElementProps {
        value_read_only: Some(false),
        ..element(DOCUMENT, "winword.exe", "")
    };
    let silent_edit = element(EDIT, "notepad.exe", "Edit");

    assert_eq!(
        classify(&silent_document),
        FocusClass::NotText,
        "a document with no Value pattern and no text-edit pattern shows nothing that takes typing"
    );
    assert_eq!(
        classify(&writable_document),
        FocusClass::Text,
        "a document whose value is writable takes typing"
    );
    assert_eq!(
        classify(&silent_edit),
        FocusClass::Text,
        "an edit box takes typing unless it says it is read-only"
    );
}

#[test]
fn win32_password_edit() {
    let password = ElementProps {
        es_password: true,
        ..element(EDIT, "mstsc.exe", "Edit")
    };

    assert_eq!(
        classify(&password),
        FocusClass::Password,
        "a classic edit with the password style is a password box even when UI Automation does not say so"
    );
}

#[test]
fn excel_cell() {
    let cell = element(DATA_ITEM, "excel.exe", "");
    let elsewhere = element(DATA_ITEM, "explorer.exe", "");

    assert_eq!(
        classify(&cell),
        FocusClass::Text,
        "typing into a selected Excel cell edits it"
    );
    assert_eq!(
        classify(&elsewhere),
        FocusClass::NotText,
        "a data item outside Excel is a row in a list"
    );
}

#[test]
fn console() {
    let console = element(PANE, "cmd.exe", "ConsoleWindowClass");

    assert_eq!(classify(&console), FocusClass::Text, "a console window takes typing");
}

#[test]
fn windows_terminal() {
    let terminal = element(TEXT, "windowsterminal.exe", "TermControl");
    let open_console = element(PANE, "openconsole.exe", "");

    assert_eq!(classify(&terminal), FocusClass::Text, "Windows Terminal takes typing");
    assert_eq!(
        classify(&open_console),
        FocusClass::Text,
        "its console host takes typing"
    );
}

#[test]
fn desktop_list() {
    let desktop = element(LIST, "explorer.exe", "SysListView32");

    assert_eq!(
        classify(&desktop),
        FocusClass::NotText,
        "the desktop's icon list takes no typing"
    );
}

#[test]
fn disabled_edit() {
    let disabled = ElementProps {
        flags: ElementFlags {
            enabled: false,
            ..ENABLED
        },
        value_read_only: Some(false),
        ..element(EDIT, "notepad.exe", "Edit")
    };

    assert_eq!(
        classify(&disabled),
        FocusClass::NotText,
        "a disabled edit box takes no typing"
    );
}

#[test]
fn read_only_edit() {
    let read_only = ElementProps {
        value_read_only: Some(true),
        ..element(EDIT, "notepad.exe", "Edit")
    };

    assert_eq!(
        classify(&read_only),
        FocusClass::NotText,
        "a read-only edit box takes no typing"
    );
}

#[test]
fn password_beats_rules() {
    let password = ElementFlags {
        password: true,
        ..ENABLED
    };
    let disabled_password = ElementProps {
        flags: ElementFlags {
            enabled: false,
            ..password
        },
        ..element(EDIT, "chrome.exe", "")
    };
    let password_in_terminal = ElementProps {
        flags: password,
        ..element(TEXT, "windowsterminal.exe", "TermControl")
    };
    let password_style_in_excel = ElementProps {
        es_password: true,
        ..element(DATA_ITEM, "excel.exe", "")
    };

    assert_eq!(
        [&disabled_password, &password_in_terminal, &password_style_in_excel].map(classify),
        [FocusClass::Password; 3],
        "a password box is never typed into, whatever other rule matches it"
    );
}

#[test]
fn unknown_element_is_not_text() {
    let nothing_known = ElementProps::default();
    let enabled_unknown = element(0, "", "");

    assert_eq!(
        classify(&nothing_known),
        FocusClass::NotText,
        "a control nothing is known about takes no typing"
    );
    assert_eq!(
        classify(&enabled_unknown),
        FocusClass::NotText,
        "an enabled control of an unknown type takes no typing"
    );
}

#[test]
fn password_style_counts_only_on_edits() {
    assert!(
        style_says_password(EDIT, ES_PASSWORD | 0x1),
        "an edit window with the style hides what is typed"
    );
    assert!(
        !style_says_password(EDIT, 0x1),
        "an edit window without it shows what is typed"
    );
    assert!(
        !style_says_password(BUTTON, ES_PASSWORD),
        "on another kind of window the same bit is a different style"
    );
}
