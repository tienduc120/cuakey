//! Fixture.
pub fn survives() -> bool {
    std::panic::catch_unwind(|| 1).is_ok()
}
