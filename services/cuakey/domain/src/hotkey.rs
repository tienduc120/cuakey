use crate::{KeyCode, OnOff, PreferencesError, Settings};

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

    const fn without(self, modifier: Modifier) -> Self {
        Self(self.0 & !modifier.bit())
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
        if matches!(key, Some(KeyCode::Other)) {
            return Err(PreferencesError::UnknownHotkeyKey);
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

/// What a hotkey does when it fires.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyAction {
    SwitchLanguage,
    OpenSettings,
    QuickConvert,
}

/// Whether a key went down or came up. An auto-repeat arrives as another `Down`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyDirection {
    Down,
    Up,
}

/// A key as the matcher sees it: modifiers apart from other keys, so a modifier never reaches the typing engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyKey {
    Modifier(Modifier),
    Key(KeyCode),
}

/// The hotkeys in force.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HotkeyBindings {
    switch: Hotkey,
    settings: Hotkey,
    quick_convert: Option<Hotkey>,
}

impl HotkeyBindings {
    pub const fn from_settings(settings: &Settings) -> Self {
        Self {
            switch: settings.switching().hotkey(),
            settings: settings.behaviours().settings_hotkey(),
            quick_convert: settings.convert().quick_convert_hotkey(),
        }
    }
}

/// What the keyboard hook does with one event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HotkeyOutcome {
    swallow: bool,
    fire: Option<HotkeyAction>,
}

impl HotkeyOutcome {
    pub const fn new(swallow: bool, fire: Option<HotkeyAction>) -> Self {
        Self { swallow, fire }
    }

    /// Whether the app must not see the event.
    pub const fn swallow(self) -> bool {
        self.swallow
    }

    pub const fn fire(self) -> Option<HotkeyAction> {
        self.fire
    }
}

impl HotkeyBindings {
    /// The action of a modifier-only hotkey whose modifiers are exactly `chord`.
    fn modifier_only(self, chord: ModifierSet) -> Option<HotkeyAction> {
        self.each()
            .find_map(|(hotkey, action)| (hotkey.key.is_none() && hotkey.modifiers == chord).then_some(action))
    }

    /// The action of a hotkey on `key` whose modifiers are exactly `chord`.
    fn with_key(self, chord: ModifierSet, key: KeyCode) -> Option<HotkeyAction> {
        self.each()
            .find_map(|(hotkey, action)| (hotkey.key == Some(key) && hotkey.modifiers == chord).then_some(action))
    }

    fn each(self) -> impl Iterator<Item = (Hotkey, HotkeyAction)> {
        [
            Some((self.switch, HotkeyAction::SwitchLanguage)),
            Some((self.settings, HotkeyAction::OpenSettings)),
            self.quick_convert.map(|hotkey| (hotkey, HotkeyAction::QuickConvert)),
        ]
        .into_iter()
        .flatten()
    }
}

/// Follows the keyboard to tell when a hotkey is released.
///
/// The chord is every modifier held since all were last released. A hotkey fires on release: a modifier-only one when
/// the first modifier comes up with no other key pressed meanwhile, one with a key when that key comes up.
#[derive(Debug, Clone)]
pub struct HotkeyMatcher {
    held: ModifierSet,
    chord: ModifierSet,
    /// Whether a key or a release already spoiled this chord for a modifier-only hotkey.
    spent: OnOff,
    /// The key whose press was swallowed, and what it fires on release.
    pending: Option<(KeyCode, HotkeyAction)>,
}

impl Default for HotkeyMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl HotkeyMatcher {
    pub const fn new() -> Self {
        Self {
            held: ModifierSet::empty(),
            chord: ModifierSet::empty(),
            spent: OnOff::Off,
            pending: None,
        }
    }

    /// Forgets the chord and any swallowed key, for when the focus moves and a key-up may have been lost.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn on_key(
        &mut self,
        key: HotkeyKey,
        direction: KeyDirection,
        bindings: HotkeyBindings,
        password_box: OnOff,
    ) -> HotkeyOutcome {
        match (key, direction) {
            (HotkeyKey::Modifier(modifier), KeyDirection::Down) => self.modifier_down(modifier),
            (HotkeyKey::Modifier(modifier), KeyDirection::Up) => return self.modifier_up(modifier, bindings),
            (HotkeyKey::Key(code), KeyDirection::Down) => return self.key_down(code, bindings, password_box),
            (HotkeyKey::Key(code), KeyDirection::Up) => return self.key_up(code),
        }
        HotkeyOutcome::new(false, None)
    }

    fn modifier_down(&mut self, modifier: Modifier) {
        if self.held.is_empty() {
            self.chord = ModifierSet::empty();
            self.spent = OnOff::Off;
        }
        self.held = self.held.with(modifier);
        self.chord = self.chord.with(modifier);
    }

    fn modifier_up(&mut self, modifier: Modifier, bindings: HotkeyBindings) -> HotkeyOutcome {
        if !self.held.contains(modifier) {
            return HotkeyOutcome::new(false, None);
        }
        self.held = self.held.without(modifier);
        let fire = if self.spent.is_on() {
            None
        } else {
            bindings.modifier_only(self.chord)
        };
        self.spent = OnOff::On;
        HotkeyOutcome::new(false, fire)
    }

    fn key_down(&mut self, code: KeyCode, bindings: HotkeyBindings, password_box: OnOff) -> HotkeyOutcome {
        self.spent = OnOff::On;
        if self.pending.is_some_and(|(pending, _)| pending == code) {
            return HotkeyOutcome::new(true, None);
        }
        self.pending = if password_box.is_on() {
            None
        } else {
            bindings.with_key(self.chord, code).map(|action| (code, action))
        };
        HotkeyOutcome::new(self.pending.is_some(), None)
    }

    fn key_up(&mut self, code: KeyCode) -> HotkeyOutcome {
        match self.pending {
            Some((pending, action)) if pending == code => {
                self.pending = None;
                HotkeyOutcome::new(true, Some(action))
            }
            Some(_) | None => HotkeyOutcome::new(false, None),
        }
    }
}

#[cfg(test)]
mod tests;
