//! Fixture.
pub fn forever() -> &'static u8 {
    Box::leak(Box::new(1))
}
