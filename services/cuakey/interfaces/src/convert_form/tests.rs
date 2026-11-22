use cuakey_application::{CodeTableValue, ConvertCaseValue, HotkeyView, KeyName, PreferencesView, Toggle};

use super::ConvertForm;
use crate::UiAction;

const CASES: [ConvertCaseValue; 4] = [
    ConvertCaseValue::Upper,
    ConvertCaseValue::Lower,
    ConvertCaseValue::SentenceStart,
    ConvertCaseValue::EachWord,
];

fn form() -> ConvertForm {
    ConvertForm::new(PreferencesView::default())
}

fn saved(action: UiAction) -> PreferencesView {
    let UiAction::Save(view) = action else {
        panic!("expected a save, got {action:?}");
    };
    view
}

fn ctrl_f6() -> HotkeyView {
    HotkeyView {
        ctrl: Toggle::On,
        alt: Toggle::Off,
        win: Toggle::Off,
        shift: Toggle::Off,
        key: Some(KeyName::F6),
    }
}

fn empty_hotkey() -> HotkeyView {
    HotkeyView {
        ctrl: Toggle::Off,
        alt: Toggle::Off,
        win: Toggle::Off,
        shift: Toggle::Off,
        key: None,
    }
}

#[test]
fn convert_view_round_trip() {
    let mut form = form();

    saved(form.set_notify_when_done(true));
    saved(form.set_remove_accents(true));
    saved(form.set_from(CodeTableValue::Tcvn3));
    saved(form.set_to(CodeTableValue::Cp1258));
    saved(form.set_case(ConvertCaseValue::EachWord, true));
    saved(form.set_quick_hotkey(ctrl_f6()));

    let mut expected = PreferencesView::default();
    expected.convert.notify_when_done = Toggle::On;
    expected.convert.remove_accents = Toggle::On;
    expected.convert.from = CodeTableValue::Tcvn3;
    expected.convert.to = CodeTableValue::Cp1258;
    expected.convert.case = ConvertCaseValue::EachWord;
    expected.convert.quick_convert_hotkey = Some(ctrl_f6());
    assert_eq!(form.view(), &expected);
}

#[test]
fn swap_button_swaps_tables() {
    let mut form = form();
    saved(form.set_from(CodeTableValue::Tcvn3));
    saved(form.set_to(CodeTableValue::VniWindows));

    let view = saved(form.swap_tables());

    assert_eq!(view.convert.from, CodeTableValue::VniWindows);
    assert_eq!(view.convert.to, CodeTableValue::Tcvn3);
}

#[test]
fn ticking_a_case_unticks_the_others() {
    for ticked in CASES {
        let mut form = form();
        for other in CASES {
            saved(form.set_case(other, true));
        }

        let view = saved(form.set_case(ticked, true));

        assert_eq!(view.convert.case, ticked);
        for other in CASES {
            assert_eq!(
                form.is_case_ticked(other),
                other == ticked,
                "{other:?} after {ticked:?}"
            );
        }
    }
}

#[test]
fn unticking_the_ticked_case_keeps_the_case() {
    let mut form = form();
    saved(form.set_case(ConvertCaseValue::Upper, true));

    let view = saved(form.set_case(ConvertCaseValue::Upper, false));

    assert_eq!(view.convert.case, ConvertCaseValue::Keep);
}

#[test]
fn unticking_another_case_changes_nothing() {
    let mut form = form();
    saved(form.set_case(ConvertCaseValue::Upper, true));

    let view = saved(form.set_case(ConvertCaseValue::Lower, false));

    assert_eq!(view.convert.case, ConvertCaseValue::Upper);
}

#[test]
fn an_empty_hotkey_clears_the_quick_convert_key() {
    let mut form = form();
    saved(form.set_quick_hotkey(ctrl_f6()));

    let view = saved(form.set_quick_hotkey(empty_hotkey()));

    assert_eq!(view.convert.quick_convert_hotkey, None);
}

#[test]
fn convert_carries_the_current_options() {
    let mut form = form();
    saved(form.set_case(ConvertCaseValue::Lower, true));
    saved(form.set_to(CodeTableValue::Tcvn3));

    let action = form.convert();

    let UiAction::Convert(view) = action else {
        panic!("expected a conversion, got {action:?}");
    };
    assert_eq!(&view, form.view());
    assert_eq!(view.convert.case, ConvertCaseValue::Lower);
    assert_eq!(view.convert.to, CodeTableValue::Tcvn3);
}

#[test]
fn close_closes_the_window() {
    assert_eq!(ConvertForm::close(), UiAction::Close);
}

#[test]
fn set_preferences_replaces_the_view() {
    let mut form = form();
    let mut other = PreferencesView::default();
    other.convert.remove_accents = Toggle::On;

    form.set_preferences(other.clone());

    assert_eq!(form.view(), &other);
}
