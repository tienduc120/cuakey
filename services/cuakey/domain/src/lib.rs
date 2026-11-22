//! This service's domain: the rules that keep its values valid. `no_std` keeps I/O, threads, and the clock out; time
//! arrives as a value (ADR-0006).
#![no_std]

extern crate alloc;

mod app_name;
mod charset;
mod choices;
mod convert;
mod error;
mod focus;
mod hotkey;
mod key_code;
mod macros;
mod on_off;
mod output;
mod preferences;
mod profile_id;
mod setting_change;
mod settings;
mod typing;

pub use app_name::AppName;
pub use charset::{BaseLetter, Letter, LetterCase, Tone, decode, encode, units_in};
pub use choices::{CodeTable, InputMethod, Language};
pub use convert::{ConvertCase, ConvertOptions, convert};
pub use error::PreferencesError;
pub use focus::{FocusState, should_convert};
pub use hotkey::{
    Hotkey, HotkeyAction, HotkeyBindings, HotkeyKey, HotkeyMatcher, HotkeyOutcome, KeyDirection, Modifier, ModifierSet,
};
pub use key_code::KeyCode;
pub use macros::{MacroEntry, MacroError, MacroTable};
pub use on_off::OnOff;
pub use output::{OutputContext, OutputOp, plan_output};
pub use preferences::{AppState, Preferences};
pub use profile_id::ProfileId;
pub use setting_change::SettingChange;
pub use settings::{
    BehaviourSettings, CompatibilitySettings, ConvertSettings, MacroSettings, Settings, SwitchSettings, SystemSettings,
    TypingSettings,
};
pub use typing::{Composer, ComposerSettings, Edit, KeyMods, KeyPress};
