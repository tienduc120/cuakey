use super::{INJECTED_TAG, is_injected};

#[test]
fn only_input_carrying_the_tag_is_ours() {
    assert!(is_injected(INJECTED_TAG), "input we sent carries the tag");
    assert!(!is_injected(0), "the user's own keys carry no tag");
    assert!(!is_injected(INJECTED_TAG + 1), "another program's tag is not ours");
}
