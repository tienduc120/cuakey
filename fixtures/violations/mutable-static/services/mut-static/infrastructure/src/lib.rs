//! Fixture: a counter every caller shares.
use std::sync::atomic::{AtomicU32, Ordering};

static COUNT: AtomicU32 = AtomicU32::new(0);

pub fn next() -> u32 {
    COUNT.fetch_add(1, Ordering::Relaxed)
}
