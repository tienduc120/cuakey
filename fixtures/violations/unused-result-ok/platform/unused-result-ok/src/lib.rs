//! Fixture.
pub fn parse_quietly(raw: &str) {
    raw.parse::<u8>().ok();
}
