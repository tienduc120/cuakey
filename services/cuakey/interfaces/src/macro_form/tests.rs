use std::path::PathBuf;

use cuakey_application::{ImportMode, MacroView, PreferencesView, Toggle};

use super::MacroForm;
use crate::UiAction;

fn row(shortcut: &str, expansion: &str) -> MacroView {
    MacroView {
        shortcut: shortcut.to_owned(),
        expansion: expansion.to_owned(),
    }
}

fn shortcuts(form: &MacroForm) -> Vec<&str> {
    form.macros().iter().map(|view| view.shortcut.as_str()).collect()
}

fn form() -> MacroForm {
    MacroForm::new(PreferencesView::default(), vec![row("tks", "thanks")])
}

fn type_row(form: &mut MacroForm, shortcut: &str, expansion: &str) {
    shortcut.clone_into(form.shortcut_mut());
    expansion.clone_into(form.expansion_mut());
}

#[test]
fn macro_rows_in_order() {
    let form = MacroForm::new(
        PreferencesView::default(),
        vec![
            row("btw", "by the way"),
            row("Zed", "z"),
            row("Apple", "a"),
            row("brb", "be right back"),
        ],
    );

    assert_eq!(shortcuts(&form), ["Apple", "brb", "btw", "Zed"]);
}

#[test]
fn adding_a_row_saves_the_sorted_table_and_clears_the_fields() {
    let mut form = form();
    type_row(&mut form, "brb", "be right back");

    let action = form.add();

    assert_eq!(
        action,
        Some(UiAction::SaveMacros(vec![
            row("brb", "be right back"),
            row("tks", "thanks")
        ]))
    );
    assert_eq!(form.shortcut_mut(), "");
    assert_eq!(form.expansion_mut(), "");
}

#[test]
fn adding_a_shortcut_again_replaces_its_expansion() {
    let mut form = form();
    type_row(&mut form, "tks", "thank you");

    let action = form.add();

    assert_eq!(action, Some(UiAction::SaveMacros(vec![row("tks", "thank you")])));
}

#[test]
fn a_blank_shortcut_adds_nothing() {
    let mut form = form();
    type_row(&mut form, "  ", "text");

    assert_eq!(form.add(), None);
    assert_eq!(form.macros().len(), 1);
}

#[test]
fn removing_the_selected_row_saves_the_rest() {
    let mut form = MacroForm::new(PreferencesView::default(), vec![row("a", "1"), row("b", "2")]);
    form.select(Some(0));

    assert_eq!(form.remove(), Some(UiAction::SaveMacros(vec![row("b", "2")])));
    assert_eq!(form.selected(), None);
}

#[test]
fn removing_with_nothing_selected_does_nothing() {
    let mut form = form();

    assert_eq!(form.remove(), None);
}

#[test]
fn a_loaded_table_replaces_the_rows_and_the_selection() {
    let mut form = form();
    form.select(Some(0));

    form.set_macros(vec![row("z", "1"), row("a", "2")]);

    assert_eq!(shortcuts(&form), ["a", "z"]);
    assert_eq!(form.selected(), None);
}

#[test]
fn the_import_choice_returns_the_right_mode() {
    for mode in [ImportMode::Append, ImportMode::Replace] {
        let mut form = form();
        form.choose_import_file(PathBuf::from("macros.txt"));
        assert!(form.import_pending());

        let action = form.choose_import_mode(mode);

        assert_eq!(
            action,
            Some(UiAction::ImportMacros {
                path: PathBuf::from("macros.txt"),
                mode
            })
        );
        assert!(!form.import_pending());
    }
}

#[test]
fn cancelling_the_import_choice_does_nothing() {
    let mut form = form();
    form.choose_import_file(PathBuf::from("macros.txt"));

    form.cancel_import();

    assert!(!form.import_pending());
    assert_eq!(form.choose_import_mode(ImportMode::Append), None);
}

#[test]
fn the_export_file_returns_the_export_action() {
    assert_eq!(
        MacroForm::export_to(PathBuf::from("out.txt")),
        UiAction::ExportMacros {
            path: PathBuf::from("out.txt")
        }
    );
}

#[test]
fn match_capitals_is_a_preference_that_saves_the_view() {
    let mut form = form();

    let action = form.set_match_case(true);

    let mut expected = PreferencesView::default();
    expected.macros.match_case = Toggle::On;
    assert_eq!(action, UiAction::Save(expected.clone()));
    assert_eq!(form.view(), &expected);
}

#[test]
fn an_error_shows_until_cleared() {
    let mut form = form();

    form.show_error(Some("lỗi"));
    assert_eq!(form.error(), Some("lỗi"));
    form.show_error(None);
    assert_eq!(form.error(), None);
}
