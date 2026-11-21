//! Fixture: everything in a module re-exported at once.
mod inner {
    pub const fn one() -> u8 {
        1
    }
}

pub use inner::*;
