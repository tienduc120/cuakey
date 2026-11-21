//! Fixture.
#[derive(Debug)]
pub struct Kept(pub std::mem::ManuallyDrop<u8>);
