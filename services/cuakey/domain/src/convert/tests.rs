use super::{ConvertCase, ConvertOptions, convert};
use crate::{CodeTable, OnOff};

fn options(from: CodeTable, to: CodeTable) -> ConvertOptions {
    ConvertOptions::new(from, to)
}

fn unicode_with(case: ConvertCase) -> ConvertOptions {
    options(CodeTable::Unicode, CodeTable::Unicode).with_case(case)
}

#[test]
fn convert_unicode_to_tcvn3_and_back() {
    let text = "Tiếng Việt, được người";

    let in_tcvn3 = convert(text, options(CodeTable::Unicode, CodeTable::Tcvn3));
    let back = convert(&in_tcvn3, options(CodeTable::Tcvn3, CodeTable::Unicode));

    assert_ne!(in_tcvn3, text, "the bytes are not the Unicode letters");
    assert_eq!(
        in_tcvn3.chars().count(),
        22,
        "one char per unit, and a letter is one unit in TCVN3"
    );
    assert_eq!(back, text, "the round trip");
}

#[test]
fn convert_writes_the_units_the_table_defines() {
    let in_compound = convert("á", options(CodeTable::Unicode, CodeTable::UnicodeCompound));

    assert_eq!(in_compound, "a\u{301}", "a letter and its combining acute, two units");
}

#[test]
fn convert_reads_compound_text_back_to_precomposed_letters() {
    let text = convert("Việt", options(CodeTable::Unicode, CodeTable::UnicodeCompound));

    let back = convert(&text, options(CodeTable::UnicodeCompound, CodeTable::Unicode));

    assert_eq!(back, "Việt", "precomposed again");
}

#[test]
fn convert_between_two_byte_tables() {
    let in_vni = convert("Việt Nam", options(CodeTable::Unicode, CodeTable::VniWindows));
    let in_cp1258 = convert(&in_vni, options(CodeTable::VniWindows, CodeTable::Cp1258));

    let back = convert(&in_cp1258, options(CodeTable::Cp1258, CodeTable::Unicode));

    assert_eq!(back, "Việt Nam", "through VNI Windows and CP1258");
}

#[test]
fn convert_upper_lower_sentence_each_word() {
    let text = "xin chào. tôi là đức! mình ở hà nội";

    assert_eq!(
        convert(text, unicode_with(ConvertCase::Upper)),
        "XIN CHÀO. TÔI LÀ ĐỨC! MÌNH Ở HÀ NỘI",
        "upper"
    );
    assert_eq!(
        convert("XIN CHÀO", unicode_with(ConvertCase::Lower)),
        "xin chào",
        "lower"
    );
    assert_eq!(
        convert(text, unicode_with(ConvertCase::SentenceStart)),
        "Xin chào. Tôi là đức! Mình ở hà nội",
        "sentence start"
    );
    assert_eq!(
        convert(text, unicode_with(ConvertCase::EachWord)),
        "Xin Chào. Tôi Là Đức! Mình Ở Hà Nội",
        "each word"
    );
    assert_eq!(convert(text, unicode_with(ConvertCase::Keep)), text, "keep");
}

#[test]
fn sentence_start_leaves_the_other_letters_alone() {
    assert_eq!(
        convert("hELLO wORLD", unicode_with(ConvertCase::SentenceStart)),
        "HELLO wORLD",
        "sentence start only raises, so names keep their capitals"
    );
}

#[test]
fn each_word_writes_every_word_with_one_capital() {
    for (text, cased) in [
        ("hELLO wORLD", "Hello World"),
        ("TIẾNG VIỆT", "Tiếng Việt"),
        (
            "tie\u{302}\u{301}ng vie\u{323}\u{302}t",
            "Tie\u{302}\u{301}ng Vie\u{323}\u{302}t",
        ),
    ] {
        assert_eq!(convert(text, unicode_with(ConvertCase::EachWord)), cased, "{text:?}");
    }
}

#[test]
fn each_word_raises_only_the_first_letter_after_a_space() {
    for (text, cased) in [
        ("gửi nam@example.com nhé", "Gửi Nam@example.com Nhé"),
        (r"a\b và c/d", r"A\b Và C/d"),
        ("giá (3 cái)", "Giá (3 Cái)"),
    ] {
        assert_eq!(convert(text, unicode_with(ConvertCase::EachWord)), cased, "{text:?}");
    }
}

#[test]
fn a_stop_ends_a_sentence_only_before_a_space() {
    for (text, cased) in [
        ("www.example.com", "Www.example.com"),
        ("giá 100.000 đồng. còn", "Giá 100.000 đồng. Còn"),
        ("\"xin chào.\" tôi", "\"Xin chào.\" Tôi"),
    ] {
        assert_eq!(
            convert(text, unicode_with(ConvertCase::SentenceStart)),
            cased,
            "{text:?}"
        );
    }
}

#[test]
fn a_sentence_after_a_quote_or_line_break_starts_with_a_capital() {
    assert_eq!(
        convert("a. \"b\"\nc 3 d", unicode_with(ConvertCase::SentenceStart)),
        "A. \"B\"\nC 3 d",
        "after a stop and quote, after a line break, not after a digit"
    );
}

#[test]
fn case_applies_to_the_converted_table_text() {
    let options = options(CodeTable::Tcvn3, CodeTable::Unicode).with_case(ConvertCase::Upper);
    let in_tcvn3 = convert("tiếng", ConvertOptions::new(CodeTable::Unicode, CodeTable::Tcvn3));

    assert_eq!(convert(&in_tcvn3, options), "TIẾNG", "decoded first, then raised");
}

#[test]
fn remove_accents() {
    let options = options(CodeTable::Unicode, CodeTable::Unicode).with_remove_accents(OnOff::On);

    assert_eq!(convert("Tiếng Việt", options), "Tieng Viet", "tones and marks go");
    assert_eq!(
        convert("Đường ĐƯỢC ơ ă â ô", options),
        "Duong DUOC o a a o",
        "đ and the horned letters too"
    );
}

#[test]
fn remove_accents_drops_combining_marks() {
    let options = options(CodeTable::Unicode, CodeTable::Unicode).with_remove_accents(OnOff::On);

    assert_eq!(convert("Vie\u{302}\u{323}t", options), "Viet", "a decomposed ệ");
}

#[test]
fn remove_accents_then_convert_to_a_byte_table() {
    let options = options(CodeTable::Unicode, CodeTable::Tcvn3).with_remove_accents(OnOff::On);

    assert_eq!(convert("Việt", options), "Viet", "plain ASCII is the same in TCVN3");
}

#[test]
fn emoji_survive_conversion() {
    let text = "xin chào 😀 世界 👍🏽";

    for table in [
        CodeTable::Tcvn3,
        CodeTable::VniWindows,
        CodeTable::Cp1258,
        CodeTable::UnicodeCompound,
    ] {
        let there = convert(text, options(CodeTable::Unicode, table));
        let back = convert(&there, options(table, CodeTable::Unicode));

        assert!(there.contains('😀'), "the emoji is kept in {table:?}");
        assert_eq!(back, text, "the round trip through {table:?}");
    }
}

#[test]
fn emoji_survive_case_and_accent_changes() {
    let options = unicode_with(ConvertCase::Upper).with_remove_accents(OnOff::On);

    assert_eq!(convert("việt 😀", options), "VIET 😀", "the emoji stays");
}

#[test]
fn empty_text_stays_empty() {
    assert_eq!(
        convert("", options(CodeTable::Unicode, CodeTable::Tcvn3)),
        "",
        "nothing in, nothing out"
    );
}
