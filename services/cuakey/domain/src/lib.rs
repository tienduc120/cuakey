//! This service's domain: the rules that keep its values valid. `no_std` keeps I/O, threads, and the clock out; time
//! arrives as a value (ADR-0006).
#![no_std]

extern crate alloc;

mod app_name;
mod charset;
mod choices;
mod error;
mod hotkey;
mod key_code;
mod on_off;
mod preferences;
mod profile_id;
mod setting_change;
mod settings;
mod typing;

pub use app_name::AppName;
pub use charset::{BaseLetter, Letter, LetterCase, Tone, decode, encode, units_in};
pub use choices::{CodeTable, InputMethod, Language};
pub use error::PreferencesError;
pub use hotkey::{Hotkey, Modifier, ModifierSet};
pub use key_code::KeyCode;
pub use on_off::OnOff;
pub use preferences::{AppState, Preferences};
pub use profile_id::ProfileId;
pub use setting_change::SettingChange;
pub use settings::{
    BehaviourSettings, CompatibilitySettings, ConvertSettings, MacroSettings, Settings, SwitchSettings, SystemSettings,
    TypingSettings,
};
pub use typing::{Composer, ComposerSettings, Edit, KeyMods, KeyPress};
