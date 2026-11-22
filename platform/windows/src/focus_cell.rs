//! What the control with the keyboard focus is, shared without a lock between the thread that works it out and the
//! keyboard hook that reads it on every key.

use std::sync::atomic::{AtomicU64, Ordering};

/// What is known about the control that has the keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusClass {
    /// The focus moved and the new control is not classified yet.
    Pending,
    Text,
    Password,
    NotText,
    Unknown,
}

impl FocusClass {
    const fn bits(self) -> u64 {
        match self {
            Self::Pending => 0,
            Self::Text => 1,
            Self::Password => 2,
            Self::NotText => 3,
            Self::Unknown => 4,
        }
    }

    const fn from_bits(bits: u64) -> Self {
        match bits {
            0 => Self::Pending,
            1 => Self::Text,
            2 => Self::Password,
            3 => Self::NotText,
            _ => Self::Unknown,
        }
    }
}

/// The focus class, tagged with the invalidation it answers. Each [`FocusCell::invalidate`] starts a new generation,
/// and a class worked out for an older one is dropped, so a slow answer about the control the focus left never lands
/// on the new one.
#[derive(Debug)]
pub struct FocusCell {
    /// The generation above the low [`CLASS_BITS`], the class in them, so both change in one atomic step.
    state: AtomicU64,
}

impl FocusCell {
    /// A cell at the first generation, waiting for its first class.
    pub const fn new() -> Self {
        Self {
            state: AtomicU64::new(pack(0, FocusClass::Pending)),
        }
    }

    pub fn get(&self) -> FocusClass {
        FocusClass::from_bits(self.state.load(Ordering::Acquire) & CLASS_MASK)
    }

    /// Records `class` as the answer for `generation`, unless the focus has moved again since; says whether it did.
    pub fn set(&self, class: FocusClass, generation: u64) -> bool {
        self.state
            .try_update(Ordering::AcqRel, Ordering::Acquire, |state| {
                (generation_of(state) == generation).then_some(pack(generation, class))
            })
            .is_ok()
    }

    /// The focus moved: the class goes back to pending, and the generation the new answer must carry is returned.
    pub fn invalidate(&self) -> u64 {
        let next = |state| pack(generation_of(state).wrapping_add(1), FocusClass::Pending);
        let previous = self.state.update(Ordering::AcqRel, Ordering::Acquire, next);
        generation_of(next(previous))
    }
}

impl Default for FocusCell {
    fn default() -> Self {
        Self::new()
    }
}

/// How many low bits of the state hold the class.
const CLASS_BITS: u32 = 3;
const CLASS_MASK: u64 = (1 << CLASS_BITS) - 1;

const fn pack(generation: u64, class: FocusClass) -> u64 {
    (generation << CLASS_BITS) | class.bits()
}

const fn generation_of(state: u64) -> u64 {
    state >> CLASS_BITS
}

const TAB: u16 = 0x09;
const ENTER: u16 = 0x0d;
const ESCAPE: u16 = 0x1b;
const F6: u16 = 0x75;

/// Whether `vk` usually moves the keyboard focus: Tab, Enter, Esc, F6 (the browser's address bar), or any key pressed
/// with Ctrl, Alt or Win. The focus is invalidated after such a key is handled, so the key itself is judged by the
/// control it was typed into.
pub const fn focus_moving_key(vk: u16, ctrl_alt_win_held: bool) -> bool {
    ctrl_alt_win_held || matches!(vk, TAB | ENTER | ESCAPE | F6)
}

#[cfg(test)]
mod tests;
