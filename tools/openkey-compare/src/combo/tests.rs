use cuakey_domain::{CodeTable, InputMethod, Language, OnOff};

use super::{Combo, Toggle};
use crate::corpus::{CORPUS_FILES, CorpusFile};

fn file(name: &str) -> CorpusFile {
    CORPUS_FILES.into_iter().find(|file| file.name == name).unwrap()
}

#[test]
fn a_typing_file_meets_every_code_table_with_every_setting_of_the_typing_toggles() {
    let vni = file("vni");

    let combos: Vec<Combo> = vni.combos().collect();

    assert_eq!(combos.len(), 5 * 128);
    assert_eq!(vni.combos_per_method(), 640);
    assert!(combos.contains(&Combo::default_for(InputMethod::Vni)));
    assert!(
        combos
            .iter()
            .all(|c| c.language == Language::Vietnamese && c.is_on(Toggle::Macros)),
        "the typing toggles vary; the language and the macro toggles stay at their defaults"
    );
}

#[test]
fn the_macro_file_also_meets_both_languages_and_every_setting_of_the_macro_toggles() {
    let macros = file("macros");

    let combos: Vec<Combo> = macros.combos().collect();

    assert_eq!(combos.len(), 5 * 2 * 64);
    assert!(
        combos
            .iter()
            .any(|c| c.language == Language::English && c.is_on(Toggle::MacrosInEnglish) && c.is_on(Toggle::MatchCase))
    );
    assert!(combos.iter().all(|c| !c.is_on(Toggle::ModernTones)));
}

#[test]
fn the_label_names_the_method_the_table_the_language_when_english_and_each_toggle_that_is_on() {
    let combo = Combo {
        table: CodeTable::VniWindows,
        toggles: Combo::default_for(InputMethod::Telex)
            .toggles
            .with(Toggle::Restore, OnOff::Off)
            .with(Toggle::QuickTelex, OnOff::On),
        ..Combo::default_for(InputMethod::SimpleTelex)
    };
    let english = Combo {
        language: Language::English,
        ..Combo::default_for(InputMethod::Telex)
    };

    assert_eq!(
        Combo::default_for(InputMethod::Telex).to_string(),
        "telex unicode spell-check restore macros"
    );
    assert_eq!(
        combo.to_string(),
        "simple-telex vni-windows spell-check quick-telex macros"
    );
    assert_eq!(
        combo.differences(),
        ["Simple Telex", "VNI Windows", "restore off", "quick-telex on"]
    );
    assert_eq!(english.to_string(), "telex unicode english spell-check restore macros");
    assert_eq!(english.differences(), ["English"]);
}

#[test]
fn tokens_name_the_method_the_table_the_language_and_every_toggle_on_or_off() {
    let tokens = Combo::default_for(InputMethod::Telex).tokens();

    assert_eq!(
        tokens,
        [
            "telex",
            "unicode",
            "vietnamese",
            "spell-check",
            "restore",
            "!modern-tones",
            "!quick-telex",
            "!quick-first",
            "!quick-last",
            "!capitals",
            "macros",
            "!macros-in-english",
            "!match-caps"
        ]
    );
    let known = Combo::known_tokens();
    assert!(known.iter().any(|t| t == "vni-windows") && known.iter().any(|t| t == "!restore"));
    assert!(known.iter().any(|t| t == "english") && known.iter().any(|t| t == "match-caps"));
}
