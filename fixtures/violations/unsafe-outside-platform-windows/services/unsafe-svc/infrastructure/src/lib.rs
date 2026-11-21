//! Fixture: a service crate that lets itself hold unsafe code.
#![expect(unsafe_code, reason = "reads the first byte without a bounds check")]

pub fn first(bytes: &[u8]) -> Option<u8> {
    if bytes.is_empty() {
        return None;
    }
    // SAFETY: the slice is not empty, so index 0 is in bounds.
    Some(unsafe { *bytes.get_unchecked(0) })
}
