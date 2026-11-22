use super::{AppMessage, WindowKind};

const ALL_KINDS: [WindowKind; 5] = [
    WindowKind::Settings,
    WindowKind::Macros,
    WindowKind::Convert,
    WindowKind::AlwaysOn,
    WindowKind::About,
];

#[test]
fn app_message_round_trip_encoding() {
    let mut messages = vec![AppMessage::Reload, AppMessage::Quit, AppMessage::Wake];
    messages.extend(ALL_KINDS.map(AppMessage::ShowWindow));

    for message in messages {
        assert_eq!(
            AppMessage::decode(message.encode()),
            Some(message),
            "{message:?} survives its code"
        );
    }
}

#[test]
fn unknown_code_is_ignored() {
    assert_eq!(AppMessage::decode(0), None, "zero is no message");
    assert_eq!(
        AppMessage::decode(usize::MAX),
        None,
        "a code from another version is not ours"
    );
}

#[test]
fn codes_are_distinct() {
    let mut codes: Vec<usize> = ALL_KINDS
        .iter()
        .map(|kind| AppMessage::ShowWindow(*kind).encode())
        .collect();
    codes.extend([AppMessage::Reload, AppMessage::Quit, AppMessage::Wake].map(AppMessage::encode));
    codes.sort_unstable();
    codes.dedup();

    assert_eq!(codes.len(), ALL_KINDS.len() + 3, "no two messages share a code");
}

#[test]
fn wake_travels_clear_of_the_window_kinds() {
    assert_eq!(AppMessage::Wake.encode(), 3, "the plain messages count up from one");
}
