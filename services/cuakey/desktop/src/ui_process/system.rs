//! What the windows exchange with Windows itself: the clipboard and the browser. eframe's own clipboard and links are
//! not built in, so the clipboard shortcuts are turned into egui's events before a window draws, and the copies and
//! links it asks for are carried out after.

use eframe::egui::{Context, Event, Key, Modifiers, OutputCommand};
use platform_windows::{open_url, read_clipboard_text, write_clipboard_text};

/// What a clipboard shortcut asks of the field with the keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ClipboardShortcut {
    Copy,
    Cut,
    Paste,
}

/// What a window asked of the system in one pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SystemCommand {
    /// Put this text on the clipboard.
    Copy(String),
    /// Open this address in the browser.
    Open(String),
}

/// The clipboard shortcut a key press stands for, as Windows has them: Ctrl+C or Ctrl+Insert copies, Ctrl+X or
/// Shift+Delete cuts, Ctrl+V or Shift+Insert pastes.
pub(super) fn clipboard_shortcut(key: Key, modifiers: Modifiers) -> Option<ClipboardShortcut> {
    let command = modifiers.command_only();
    let shortcuts = [
        (ClipboardShortcut::Copy, Key::Copy, true),
        (ClipboardShortcut::Copy, Key::C, command),
        (ClipboardShortcut::Copy, Key::Insert, modifiers.ctrl),
        (ClipboardShortcut::Cut, Key::Cut, true),
        (ClipboardShortcut::Cut, Key::X, command),
        (ClipboardShortcut::Cut, Key::Delete, modifiers.shift),
        (ClipboardShortcut::Paste, Key::Paste, true),
        (ClipboardShortcut::Paste, Key::V, command),
        (ClipboardShortcut::Paste, Key::Insert, modifiers.shift),
    ];
    shortcuts
        .into_iter()
        .find_map(|(shortcut, shortcut_key, held)| (held && key == shortcut_key).then_some(shortcut))
}

fn shortcut_pressed(event: &Event) -> Option<ClipboardShortcut> {
    if let Event::Key {
        key,
        pressed: true,
        modifiers,
        ..
    } = event
    {
        clipboard_shortcut(*key, *modifiers)
    } else {
        None
    }
}

/// Whether `events` hold a paste, so the clipboard has to be read.
pub(super) fn wants_paste(events: &[Event]) -> bool {
    events
        .iter()
        .any(|event| shortcut_pressed(event) == Some(ClipboardShortcut::Paste))
}

/// Puts what egui does in place of each clipboard shortcut pressed in `events`: a copy, a cut, or a paste of
/// `clipboard` with its Windows line ends made plain. The shortcut's key goes, so a field never acts on both, and a
/// paste with no text to paste goes entirely.
pub(super) fn take_clipboard_shortcuts(events: &mut Vec<Event>, clipboard: Option<&str>) {
    let pasted = clipboard
        .filter(|text| !text.is_empty())
        .map(|text| text.replace("\r\n", "\n"));
    events.retain_mut(|event| match shortcut_pressed(event) {
        None => true,
        Some(ClipboardShortcut::Copy) => {
            *event = Event::Copy;
            true
        }
        Some(ClipboardShortcut::Cut) => {
            *event = Event::Cut;
            true
        }
        Some(ClipboardShortcut::Paste) => pasted.clone().is_some_and(|text| {
            *event = Event::Paste(text);
            true
        }),
    });
}

/// Takes out of `commands` the copies and links this process carries out itself, and leaves the rest for eframe.
pub(super) fn take_system_commands(commands: &mut Vec<OutputCommand>) -> Vec<SystemCommand> {
    let mut taken = Vec::new();
    commands.retain(|command| {
        let system = match command {
            OutputCommand::CopyText(text) => SystemCommand::Copy(text.clone()),
            OutputCommand::OpenUrl(open) => SystemCommand::Open(open.url.clone()),
            OutputCommand::TextSelectionSettled(_) | OutputCommand::CopyImage(_) => return true,
        };
        taken.push(system);
        false
    });
    taken
}

/// Turns this viewport's clipboard shortcuts into egui's events; call it before a window draws.
pub(super) fn before_drawing(ctx: &Context) {
    let clipboard = ctx
        .input(|input| wants_paste(&input.events))
        .then(read_clipboard)
        .flatten();
    ctx.input_mut(|input| take_clipboard_shortcuts(&mut input.events, clipboard.as_deref()));
}

/// Carries out the copies and links this viewport's window asked for; call it after the window drew.
pub(super) fn after_drawing(ctx: &Context) {
    for command in ctx.output_mut(|output| take_system_commands(&mut output.commands)) {
        match command {
            SystemCommand::Copy(text) => {
                // Another program holding the clipboard open loses this one copy; the user can copy again.
                if let Err(error) = write_clipboard_text(&text) {
                    tracing::warn!(error = ?error, "could not copy to the clipboard");
                }
            }
            SystemCommand::Open(url) => {
                if let Err(error) = open_url(&url) {
                    tracing::warn!(error = ?error, "could not open the address");
                }
            }
        }
    }
}

/// The text on the clipboard, or `None` when it holds none or another program holds it open.
fn read_clipboard() -> Option<String> {
    read_clipboard_text().unwrap_or_else(|error| {
        tracing::warn!(error = ?error, "could not read the clipboard");
        None
    })
}

#[cfg(test)]
mod tests;
