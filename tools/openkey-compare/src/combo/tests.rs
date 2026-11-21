use cuakey_domain::{CodeTable, InputMethod, OnOff};

use super::{Combo, Toggle};

#[test]
fn every_code_table_meets_every_setting_of_the_seven_toggles() {
    let combos: Vec<Combo> = Combo::all_for(InputMethod::Vni).collect();

    assert_eq!(combos.len(), 5 * 128);
    assert!(combos.contains(&Combo::default_for(InputMethod::Vni)));
}

#[test]
fn the_label_names_the_method_the_table_and_each_toggle_that_is_on() {
    let combo = Combo {
        table: CodeTable::VniWindows,
        toggles: Combo::default_for(InputMethod::Telex)
            .toggles
            .with(Toggle::Restore, OnOff::Off)
            .with(Toggle::QuickTelex, OnOff::On),
        ..Combo::default_for(InputMethod::SimpleTelex)
    };

    assert_eq!(
        Combo::default_for(InputMethod::Telex).to_string(),
        "telex unicode spell-check restore"
    );
    assert_eq!(combo.to_string(), "simple-telex vni-windows spell-check quick-telex");
    assert_eq!(
        combo.differences(),
        ["Simple Telex", "VNI Windows", "restore off", "quick-telex on"]
    );
}
