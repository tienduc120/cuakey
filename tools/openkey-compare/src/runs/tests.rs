use cuakey_domain::{CodeTable, InputMethod, MacroTable, OnOff};

use super::{Flow, RunCheck, Texts, compare_line, compare_sequences};
use crate::accepted::Acceptance;
use crate::combo::{Combo, Toggle};
use crate::corpus::parse;
use crate::openkey_screen::Family;
use crate::oracle::OpenKey;
use crate::report::Divergence;

fn accept_none(_: &Divergence) -> Option<usize> {
    None
}

fn texts<'a>(cuakey: &'a [u16], corrected: &'a [u16], faithful: &'a [u16]) -> Texts<'a> {
    Texts {
        cuakey,
        corrected,
        faithful,
    }
}

#[test]
fn a_host_defect_early_does_not_hide_an_engine_divergence_later() {
    let mut check = RunCheck::default();

    let flows = [
        check.step(0, texts(&[1], &[1], &[1]), None, &mut accept_none),
        check.step(
            1,
            texts(&[1, 2], &[1, 2], &[1, 9]),
            Some(Family::WholeLetter),
            &mut accept_none,
        ),
        check.step(
            2,
            texts(&[1, 2, 3], &[1, 2, 4], &[1, 9, 4]),
            Some(Family::WholeLetter),
            &mut accept_none,
        ),
    ];

    assert_eq!(flows, [Flow::Next, Flow::Next, Flow::Stop]);
    let result = check.finish();
    assert_eq!(
        result.open.map(|d| d.step),
        Some(2),
        "the engine divergence after the host defect"
    );
    assert_eq!(
        result.host_defect.map(|defect| (defect.family, defect.step)),
        Some((Family::WholeLetter, 1))
    );
}

#[test]
fn an_accepted_divergence_resyncs_the_screens_and_the_run_goes_on_to_the_next_one() {
    let mut check = RunCheck::default();
    let mut accept_step_one = |divergence: &Divergence| (divergence.step == 1).then_some(0);

    let flows = [
        check.step(1, texts(&[1, 2], &[1, 3], &[1, 3]), None, &mut accept_step_one),
        check.step(2, texts(&[1, 2, 5], &[1, 2, 6], &[1, 3, 6]), None, &mut accept_step_one),
    ];

    assert_eq!(flows, [Flow::Resync, Flow::Stop]);
    let result = check.finish();
    let accepted: Vec<(usize, usize)> = result.accepted.iter().map(|(d, at)| (d.step, *at)).collect();
    assert_eq!(accepted, [(1, 0)]);
    assert_eq!(result.open.map(|d| d.step), Some(2));
}

fn cp1258() -> Combo {
    Combo {
        table: CodeTable::Cp1258,
        ..Combo::default_for(InputMethod::Telex)
    }
}

#[test]
fn cp1258_ass_matches_under_the_corrected_host_and_is_a_host_defect() {
    let mut engine = OpenKey::take().unwrap();
    engine.set_macros(&[]);
    engine.configure(cp1258());

    let result = compare_line(
        &mut engine,
        cp1258(),
        (&cp1258().cuakey_settings(), &MacroTable::default()),
        &parse("ass ").unwrap(),
        &mut accept_none,
    );

    assert_eq!(result.open, None);
    assert_eq!(
        result.host_defect.map(|defect| (defect.family, defect.step)),
        Some((Family::WholeLetter, 2))
    );
}

#[test]
fn a_random_run_whose_only_divergences_are_host_defects_passes() {
    let mut engine = OpenKey::take().unwrap();
    let mut acceptance = Acceptance::new(Vec::new());

    let outcome = compare_sequences(&mut engine, vec![(cp1258(), parse("ass ").unwrap())], &mut acceptance);

    assert!(outcome.findings.is_empty(), "{:?}", outcome.findings);
    assert_eq!(outcome.host_defects.count(Family::WholeLetter), 1);
    assert!(outcome.passed(&acceptance));
}

#[test]
fn an_accepted_divergence_in_vni_windows_leaves_no_stale_letter_lengths_for_a_later_backspace() {
    let mut engine = OpenKey::take().unwrap();
    let combo = Combo {
        table: CodeTable::VniWindows,
        ..Combo::default_for(InputMethod::Telex)
    };
    engine.set_macros(&[]);
    engine.configure(combo);
    // CuaKey puts the tone of `hoaf` on the a (modern tones), OpenKey on the o: a divergence at key 4, accepted here.
    let modern = Combo {
        toggles: combo.toggles.with(Toggle::ModernTones, OnOff::On),
        ..combo
    };

    let result = compare_line(
        &mut engine,
        combo,
        (&modern.cuakey_settings(), &MacroTable::default()),
        &parse("hoaf<bs>").unwrap(),
        &mut |_| Some(0),
    );

    let accepted: Vec<usize> = result.accepted.iter().map(|(divergence, _)| divergence.step).collect();
    assert_eq!(accepted, [3]);
    assert_eq!(
        result.open, None,
        "the Backspace deletes the whole of à from CuaKey's text"
    );
}
