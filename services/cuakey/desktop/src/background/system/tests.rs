use cuakey_application::{InterfaceLanguageValue, SystemPrefs, Toggle};
use platform_windows::StartupMode;

use super::{SystemChanges, startup_mode, system_changes};

fn system() -> SystemPrefs {
    SystemPrefs {
        start_with_windows: Toggle::On,
        show_settings_at_startup: Toggle::On,
        run_as_administrator: Toggle::Off,
        desktop_shortcut: Toggle::Off,
        interface_language: InterfaceLanguageValue::English,
    }
}

#[test]
fn startup_follows_start_with_windows_and_run_as_administrator() {
    let off = SystemPrefs {
        start_with_windows: Toggle::Off,
        run_as_administrator: Toggle::On,
        ..system()
    };
    let elevated = SystemPrefs {
        run_as_administrator: Toggle::On,
        ..system()
    };

    assert_eq!(startup_mode(&off), StartupMode::Off);
    assert_eq!(startup_mode(&system()), StartupMode::Normal);
    assert_eq!(startup_mode(&elevated), StartupMode::Elevated);
}

#[test]
fn nothing_is_applied_when_nothing_changed() {
    let shown = SystemPrefs {
        show_settings_at_startup: Toggle::Off,
        ..system()
    };

    assert_eq!(system_changes(&system(), &system()), SystemChanges::default());
    assert_eq!(
        system_changes(&system(), &shown),
        SystemChanges::default(),
        "showing the settings at start changes nothing in Windows"
    );
}

#[test]
fn a_change_to_how_cuakey_starts_applies_the_new_mode() {
    let elevated = SystemPrefs {
        run_as_administrator: Toggle::On,
        ..system()
    };
    let off = SystemPrefs {
        start_with_windows: Toggle::Off,
        ..system()
    };

    assert_eq!(
        system_changes(&system(), &elevated).startup,
        Some(StartupMode::Elevated)
    );
    assert_eq!(system_changes(&system(), &off).startup, Some(StartupMode::Off));
}

#[test]
fn a_change_to_the_shortcut_puts_it_on_or_takes_it_off() {
    let with_shortcut = SystemPrefs {
        desktop_shortcut: Toggle::On,
        ..system()
    };

    assert_eq!(
        system_changes(&system(), &with_shortcut),
        SystemChanges {
            startup: None,
            shortcut: Some(Toggle::On),
        }
    );
    assert_eq!(system_changes(&with_shortcut, &system()).shortcut, Some(Toggle::Off));
}
