use alloc::vec;
use alloc::vec::Vec;

use super::{OutputContext, OutputOp, plan_output};
use crate::OnOff::On;
use crate::{CodeTable, Edit};

fn replace(delete: usize, insert: &str) -> Edit {
    Edit::Replace {
        delete,
        insert: insert.into(),
        resend_key: false,
    }
}

fn plan(edit: &Edit, word: &str, ctx: OutputContext) -> Vec<OutputOp> {
    plan_output(edit, word, ctx)
}

#[test]
fn unicode_replace() {
    let ops = plan(&replace(1, "â"), "a", OutputContext::new(CodeTable::Unicode));

    assert_eq!(ops, [OutputOp::Backspace, OutputOp::Unit(0xe2)]);
}

#[test]
fn browser_fix() {
    let ctx = OutputContext::new(CodeTable::Unicode).with_browser_fix(On);

    let ops = plan(&replace(1, "â"), "a", ctx);

    assert_eq!(
        ops,
        [
            OutputOp::Unit(0x202f),
            OutputOp::Backspace,
            OutputOp::Backspace,
            OutputOp::Unit(0xe2)
        ]
    );
}

#[test]
fn browser_fix_waits_for_a_deletion() {
    let ctx = OutputContext::new(CodeTable::Unicode).with_browser_fix(On);

    let ops = plan(&replace(0, "a"), "", ctx);

    assert_eq!(ops, [OutputOp::Unit(0x61)]);
}

#[test]
fn chromium_fix() {
    let ctx = OutputContext::new(CodeTable::Unicode)
        .with_chromium_fix(On)
        .with_chromium_app(On);

    let ops = plan(&replace(1, "â"), "a", ctx);

    assert_eq!(ops, [OutputOp::SelectPreviousChar, OutputOp::Unit(0xe2)]);
}

#[test]
fn chromium_fix_keeps_the_other_backspaces() {
    let ctx = OutputContext::new(CodeTable::Unicode)
        .with_chromium_fix(On)
        .with_chromium_app(On);

    let ops = plan(&replace(2, "ơ"), "ươ", ctx);

    assert_eq!(
        ops,
        [OutputOp::SelectPreviousChar, OutputOp::Backspace, OutputOp::Unit(0x1a1)]
    );
}

#[test]
fn chromium_fix_needs_a_chromium_app() {
    let ctx = OutputContext::new(CodeTable::Unicode).with_chromium_fix(On);

    let ops = plan(&replace(1, "â"), "a", ctx);

    assert_eq!(ops, [OutputOp::Backspace, OutputOp::Unit(0xe2)]);
}

#[test]
fn compound_two_units() {
    let ops = plan(&replace(1, "a"), "á", OutputContext::new(CodeTable::UnicodeCompound));

    assert_eq!(ops, [OutputOp::Backspace, OutputOp::Backspace, OutputOp::Unit(0x61)]);
}

#[test]
fn vni_backspace_deletes_every_unit() {
    let ctx = OutputContext::new(CodeTable::VniWindows);
    let two_unit_letter = "ề";

    let replaced = plan(&replace(1, ""), two_unit_letter, ctx);
    let passed = plan(&Edit::PassThrough, two_unit_letter, ctx);

    assert_eq!(replaced, [OutputOp::Backspace, OutputOp::Backspace]);
    assert_eq!(passed, []);
}

#[test]
fn clipboard_paste() {
    let ctx = OutputContext::new(CodeTable::Unicode).with_clipboard(On);

    let ops = plan(&replace(1, "ất"), "a", ctx);

    assert_eq!(ops, vec![OutputOp::Backspace, OutputOp::Paste(vec![0x1ea5, 0x74])]);
}

#[test]
fn resend_key_last() {
    let edit = Edit::Replace {
        delete: 1,
        insert: "â".into(),
        resend_key: true,
    };

    let ops = plan(&edit, "a", OutputContext::new(CodeTable::Unicode));

    assert_eq!(ops, [OutputOp::Backspace, OutputOp::Unit(0xe2), OutputOp::ResendKey]);
}

#[test]
fn pass_through_plans_nothing() {
    let ctx = OutputContext::new(CodeTable::Unicode).with_browser_fix(On);

    assert_eq!(plan(&Edit::PassThrough, "a", ctx), []);
}

#[test]
fn deletes_the_last_characters_of_the_word() {
    let ops = plan(
        &replace(2, "x"),
        "áb\u{1EA5}",
        OutputContext::new(CodeTable::UnicodeCompound),
    );

    assert_eq!(
        ops,
        [
            OutputOp::Backspace,
            OutputOp::Backspace,
            OutputOp::Backspace,
            OutputOp::Unit(0x78)
        ]
    );
}
