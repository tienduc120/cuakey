//! This service's use cases, and the ports their adapters implement (ADR-0007). A use case takes a command of plain
//! values and returns its own result and error types, so delivery never handles a domain type.

/// Declares an enum of plain values that mirrors a domain enum variant for variant, with a conversion each way, so a
/// caller picks a choice without naming the domain type.
macro_rules! mirror_enum {
    ($(#[$meta:meta])* $name:ident mirrors $domain:ident { $($variant:ident),* $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name {
            $($variant),*
        }

        impl From<$domain> for $name {
            fn from(value: $domain) -> Self {
                match value {
                    $($domain::$variant => Self::$variant),*
                }
            }
        }

        impl From<$name> for $domain {
            fn from(value: $name) -> Self {
                match value {
                    $($name::$variant => Self::$variant),*
                }
            }
        }
    };
}

mod macros;
mod ports;
mod preferences;
mod quick_convert;
#[cfg(any(test, feature = "test-support"))]
pub mod testing;
mod typing_session;

pub use macros::{
    ExportMacros, ExportMacrosCommand, ExportMacrosError, ImportMacros, ImportMacrosCommand, ImportMacrosError,
    InvalidMacro, LoadMacros, LoadMacrosError, MacroView, SaveMacros, SaveMacrosError,
};
pub use ports::{Clipboard, ClipboardError, ImportMode, MacroStore, PreferencesStore, StoreError};
pub use preferences::{
    AppMemoryView, BehaviourPrefs, CodeTableValue, CompatibilityPrefs, ConvertCaseValue, ConvertPrefs, HotkeyView,
    InputMethodValue, LanguageValue, LoadPreferences, LoadPreferencesError, MacroPrefs, PreferencesView,
    ResetPreferences, ResetPreferencesError, SavePreferences, SavePreferencesError, SwitchPrefs, SystemPrefs, Toggle,
    TypingPrefs, UpdateError,
};
pub use quick_convert::{QuickConvert, QuickConvertError, QuickConvertResult};
pub use typing_session::{
    AppSwitchResult, CommandValue, FocusValue, KeyCommand, KeyFlags, KeyName, KeyResult, LayoutFlags, OutputAction,
    SessionError, TypingSession,
};
