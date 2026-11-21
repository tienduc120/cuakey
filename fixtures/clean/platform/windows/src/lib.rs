//! Clean twin: the one crate that may hold unsafe code says so at its root, and every block says why it is sound.
#![expect(unsafe_code, reason = "Win32 and COM calls are unsafe")]

pub fn first(bytes: &[u8]) -> Option<u8> {
    if bytes.is_empty() {
        return None;
    }
    // SAFETY: the slice is not empty, so index 0 is in bounds.
    Some(unsafe { *bytes.get_unchecked(0) })
}
