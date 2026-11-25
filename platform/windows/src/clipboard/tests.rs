use super::{GlobalMemory, read_clipboard_text, write_clipboard_text};

#[test]
#[ignore = "touches the real clipboard; run with --run-ignored"]
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

#[test]
fn text_larger_than_the_block_is_refused_not_written() {
    let memory = GlobalMemory::alloc(2).unwrap();
    // Far past any rounding the allocator does, so the block cannot hold it.
    let too_long = vec![1_u16; 4096];

    let filled = memory.fill(&too_long);

    assert!(filled.is_err(), "a write past the end of the block is refused");
}

#[test]
fn text_that_fits_the_block_is_written() {
    let memory = GlobalMemory::alloc(6).unwrap();

    memory.fill(&[104, 105, 0]).unwrap();
}
