//! Fixture: anyone can build an Email that breaks its invariant.
#![no_std]

extern crate alloc;

#[derive(Debug)]
pub struct Email(pub alloc::string::String);
