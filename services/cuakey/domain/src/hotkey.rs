use crate::{KeyCode, PreferencesError};

/// One of the four modifier keys a hotkey can hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Modifier {
    Ctrl,
    Alt,
    Win,
    Shift,
}

impl Modifier {
    const fn bit(self) -> u8 {
        match self {
            Self::Ctrl => 1,
            Self::Alt => 2,
            Self::Win => 4,
            Self::Shift => 8,
        }
    }
}

/// A set of modifier keys, kept as bits so it needs no flag fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ModifierSet(u8);

impl ModifierSet {
    pub const fn empty() -> Self {
        Self(0)
    }

    /// The set with `modifier` added.
    #[must_use]
    pub const fn with(self, modifier: Modifier) -> Self {
        Self(self.0 | modifier.bit())
    }

    pub const fn contains(self, modifier: Modifier) -> bool {
        self.0 & modifier.bit() != 0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

/// Modifiers plus an optional key. A hotkey with neither would fire on nothing, so it cannot be built.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hotkey {
    pub(crate) modifiers: ModifierSet,
    pub(crate) key: Option<KeyCode>,
}

impl Hotkey {
    /// # Errors
    ///
    /// Refuses an empty modifier set with no key.
    pub const fn new(modifiers: ModifierSet, key: Option<KeyCode>) -> Result<Self, PreferencesError> {
        if modifiers.is_empty() && key.is_none() {
            return Err(PreferencesError::EmptyHotkey);
        }
        Ok(Self { modifiers, key })
    }

    pub const fn modifiers(self) -> ModifierSet {
        self.modifiers
    }

    pub const fn key(self) -> Option<KeyCode> {
        self.key
    }
}
