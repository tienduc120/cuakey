//! Fixture: an assert panics in production.
pub fn checked(x: u8) -> u8 {
    assert!(x > 0);
    x
}
