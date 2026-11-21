//! Windows itself: the Win32 and COM calls the app needs, behind safe functions. This is the one crate allowed `unsafe`
//! (ADR-0014). The crate root says so with `#![expect(unsafe_code, reason = "...")]`, written together with the first
//! `unsafe` block, since an expectation with nothing to meet fails the build. Every block holds one unsafe operation
//! and a safety comment that says why it is sound; the gates refuse `unsafe_code` lowered in any other crate.

/// The `dwExtraInfo` tag on every input the app sends ("CUKY" in ASCII), so its own keyboard hook can tell that input
/// from the user's and let it through untouched.
pub const INJECTED_TAG: usize = 0x4355_4b59;

/// Whether an input event carries [`INJECTED_TAG`], that is, whether the app sent it.
pub const fn is_injected(extra_info: usize) -> bool {
    extra_info == INJECTED_TAG
}

#[cfg(test)]
mod tests;
