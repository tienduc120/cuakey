use super::{ComposerSettings, KeyMods};
use crate::{InputMethod, Language, LetterCase, OnOff, SettingChange, Settings};

#[test]
fn shift_turns_caps_lock_around_for_letters() {
    let case = |shift, caps_lock| {
        KeyMods::none()
            .with_shift(shift)
            .with_caps_lock(caps_lock)
            .letter_case()
    };

    assert_eq!(case(OnOff::Off, OnOff::Off), LetterCase::Lower, "neither");
    assert_eq!(case(OnOff::On, OnOff::Off), LetterCase::Upper, "Shift alone");
    assert_eq!(case(OnOff::Off, OnOff::On), LetterCase::Upper, "Caps Lock alone");
    assert_eq!(case(OnOff::On, OnOff::On), LetterCase::Lower, "Shift with Caps Lock");
}

#[test]
fn composer_settings_take_the_language_and_the_setting_groups() {
    let settings = Settings::default()
        .with(SettingChange::InputMethod(InputMethod::Vni))
        .with(SettingChange::UseMacros(OnOff::Off))
        .with(SettingChange::KeepW(OnOff::Off));

    let composer = ComposerSettings::from_settings(&settings, Language::English);

    assert_eq!(composer.language(), Language::English, "the language passed in");
    assert_eq!(composer.typing(), settings.typing(), "the typing group");
    assert_eq!(composer.macros(), settings.macros(), "the macro group");
    assert_eq!(composer.keep_w(), OnOff::Off, "keep-w from the behaviours");
}
