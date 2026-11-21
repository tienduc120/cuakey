//! Fixture: a message that reads badly inside another one.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Input was empty.")]
    Empty,
}
