//! Fixture.
mod prelude {
    pub(crate) const fn one() -> u8 {
        1
    }
}

use prelude::*;

pub const fn two() -> u8 {
    one() + 1
}
