use super::{read_clipboard_text, write_clipboard_text};

#[test]
fn text_written_to_the_clipboard_reads_back() {
    let text = "Tiếng Việt 😀 gõ nhanh";

    write_clipboard_text(text).unwrap();
    let read = read_clipboard_text().unwrap();

    assert_eq!(
        read.as_deref(),
        Some(text),
        "what was written is what is read, emoji included"
    );
}
