//! Fixture: a module pointed at a file of its own choosing.
#[path = "other.rs"]
mod inner;

pub const fn one() -> u8 {
    inner::ONE
}
