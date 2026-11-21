/// Why a preferences value or one of its parts was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PreferencesError {
    #[error("the id must not be empty")]
    EmptyId,
    #[error("the app name must not be empty")]
    EmptyAppName,
    #[error("a hotkey needs a modifier or a key")]
    EmptyHotkey,
    #[error("the switch hotkey and the settings hotkey must differ")]
    SwitchEqualsSettings,
    #[error("the quick convert hotkey must differ from the switch and settings hotkeys")]
    QuickConvertClashes,
}
