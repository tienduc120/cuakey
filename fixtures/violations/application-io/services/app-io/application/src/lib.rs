//! Fixture: a use case reading a file itself.
pub fn config() -> Option<String> {
    std::fs::read_to_string("app.toml").ok()
}
