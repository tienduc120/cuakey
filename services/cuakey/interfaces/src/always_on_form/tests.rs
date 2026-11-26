use cuakey_application::PreferencesView;

use super::AlwaysOnForm;
use crate::UiAction;

fn form_with(apps: &[&str]) -> AlwaysOnForm {
    let view = PreferencesView {
        always_on: apps.iter().map(|app| (*app).to_owned()).collect(),
        ..PreferencesView::default()
    };
    AlwaysOnForm::new(view)
}

fn type_app(form: &mut AlwaysOnForm, text: &str) {
    text.clone_into(form.input_mut());
}

fn saved_list(action: Option<UiAction>) -> Vec<String> {
    match action {
        Some(UiAction::Save(view)) => view.always_on,
        other => panic!("expected a save, got {other:?}"),
    }
}

#[test]
fn always_on_add_normalises_and_dedupes() {
    let mut form = form_with(&["notes.exe"]);

    type_app(&mut form, r"C:\Games\Game.EXE");
    assert_eq!(saved_list(form.add()), ["game.exe", "notes.exe"]);
    assert_eq!(form.input_mut(), "");

    type_app(&mut form, "GAME.exe");
    assert_eq!(form.add(), None, "a repeat after normalising is ignored");
    assert_eq!(form.view().always_on, ["game.exe", "notes.exe"]);
    assert_eq!(form.error(), None);
}

#[test]
fn blank_input_is_refused_with_a_message() {
    let mut form = form_with(&[]);
    type_app(&mut form, "   ");

    assert_eq!(form.add(), None);

    assert_eq!(form.error(), Some("The app name cannot be empty"));
    assert_eq!(form.view().always_on, Vec::<String>::new());
}

#[test]
fn a_good_add_clears_an_earlier_message() {
    let mut form = form_with(&[]);
    type_app(&mut form, "");
    assert_eq!(form.add(), None);
    assert!(form.error().is_some());

    type_app(&mut form, "game.exe");
    assert!(form.add().is_some());

    assert_eq!(form.error(), None);
}

#[test]
fn removing_the_selected_app_saves_the_rest() {
    let mut form = form_with(&["a.exe", "b.exe"]);
    form.select(Some(0));

    assert_eq!(saved_list(form.remove()), ["b.exe"]);
    assert_eq!(form.selected(), None);
}

#[test]
fn removing_with_nothing_selected_does_nothing() {
    let mut form = form_with(&["a.exe"]);

    assert_eq!(form.remove(), None);
}

#[test]
fn close_closes_the_window() {
    assert_eq!(AlwaysOnForm::close(), UiAction::Close);
}

#[test]
fn set_preferences_replaces_the_list_and_the_selection() {
    let mut form = form_with(&["a.exe"]);
    form.select(Some(0));
    let other = PreferencesView {
        always_on: vec!["z.exe".to_owned()],
        ..PreferencesView::default()
    };

    form.set_preferences(other);

    assert_eq!(form.view().always_on, ["z.exe"]);
    assert_eq!(form.selected(), None);
}
