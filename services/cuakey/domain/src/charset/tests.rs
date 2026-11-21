use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use super::{BaseLetter, Letter, LetterCase, Tone, decode, encode, units_in};
use crate::CodeTable;

/// The 67 lower-case Vietnamese letters that are not plain ASCII, grouped by vowel.
const LOWER_LETTERS: &str = "áàảãạ ăắằẳẵặ âấầẩẫậ éèẻẽẹ êếềểễệ íìỉĩị óòỏõọ ôốồổỗộ ơớờởỡợ úùủũụ ưứừửữự ýỳỷỹỵ đ";

/// The same letters in capitals.
const UPPER_LETTERS: &str = "ÁÀẢÃẠ ĂẮẰẲẴẶ ÂẤẦẨẪẬ ÉÈẺẼẸ ÊẾỀỂỄỆ ÍÌỈĨỊ ÓÒỎÕỌ ÔỐỒỔỖỘ ƠỚỜỞỠỢ ÚÙỦŨỤ ƯỨỪỬỮỰ ÝỲỶỸỴ Đ";

/// The only capitals TCVN3 has a byte for.
const TCVN3_CAPITALS: &str = "ĂÂÊÔƠƯĐ";

const SAMPLE: &str = "Tiếng Việt có dấu: ẮẰẲẴẶ ắằẳẵặ Đđ ƯỨỪỬỮỰ ơớờởỡợ";

const TABLES: [CodeTable; 5] = [
    CodeTable::Unicode,
    CodeTable::Tcvn3,
    CodeTable::VniWindows,
    CodeTable::UnicodeCompound,
    CodeTable::Cp1258,
];

fn vietnamese_letters() -> Vec<char> {
    LOWER_LETTERS
        .chars()
        .chain(UPPER_LETTERS.chars())
        .filter(|c| !c.is_whitespace())
        .collect()
}

fn letter(c: char) -> Letter {
    Letter::from_char(c).unwrap()
}

/// What TCVN3 reads back for `c`: a capital it has no byte for comes back lower case.
fn through_tcvn3(c: char) -> String {
    if c.is_uppercase() && !TCVN3_CAPITALS.contains(c) {
        c.to_lowercase().to_string()
    } else {
        c.to_string()
    }
}

#[test]
fn all_134_letters_round_trip_through_letter() {
    let letters = vietnamese_letters();
    let distinct: BTreeSet<char> = letters.iter().copied().collect();

    let lost: Vec<char> = letters
        .iter()
        .copied()
        .chain('a'..='z')
        .chain('A'..='Z')
        .filter(|c| Letter::from_char(*c).map(Letter::to_char) != Some(*c))
        .collect();

    assert_eq!(distinct.len(), 134, "the test list holds 134 different letters");
    assert!(
        letters.iter().all(|c| !c.is_ascii()),
        "the test list holds no plain ASCII letter"
    );
    assert_eq!(lost, Vec::<char>::new(), "every letter reads back as itself");
}

#[test]
fn a_letter_knows_its_base_tone_and_case() {
    let parts = |c| Letter::from_char(c).map(|l| (l.base(), l.tone(), l.case()));

    assert_eq!(
        parts('Ặ'),
        Some((BaseLetter::ABreve, Tone::Nang, LetterCase::Upper)),
        "Ặ"
    );
    assert_eq!(parts('ữ'), Some((BaseLetter::UHorn, Tone::Nga, LetterCase::Lower)), "ữ");
    assert_eq!(parts('ở'), Some((BaseLetter::OHorn, Tone::Hoi, LetterCase::Lower)), "ở");
    assert_eq!(
        parts('Ề'),
        Some((BaseLetter::ECircumflex, Tone::Huyen, LetterCase::Upper)),
        "Ề"
    );
    assert_eq!(parts('ý'), Some((BaseLetter::Y, Tone::Sac, LetterCase::Lower)), "ý");
    assert_eq!(
        parts('đ'),
        Some((BaseLetter::DStroke, Tone::Ngang, LetterCase::Lower)),
        "đ"
    );
    assert_eq!(parts('G'), Some((BaseLetter::G, Tone::Ngang, LetterCase::Upper)), "G");
    assert_eq!(parts('7'), None, "a digit is not a letter");
    assert_eq!(parts('ß'), None, "a letter Vietnamese and ASCII lack is not one");
}

#[test]
fn a_vowel_takes_any_tone_and_a_consonant_none() {
    assert_eq!(letter('ê').with_tone(Tone::Sac), Some(letter('ế')), "ê with sắc");
    assert_eq!(
        letter('Ợ').with_tone(Tone::Ngang),
        Some(letter('Ơ')),
        "Ợ loses its tone"
    );
    assert_eq!(letter('a').with_tone(Tone::Nga), Some(letter('ã')), "a with ngã");
    assert_eq!(
        letter('đ').with_tone(Tone::Ngang),
        Some(letter('đ')),
        "đ without a tone"
    );
    assert_eq!(letter('đ').with_tone(Tone::Sac), None, "a consonant takes no tone");
}

#[test]
fn round_trip_every_table() {
    for table in [
        CodeTable::Unicode,
        CodeTable::UnicodeCompound,
        CodeTable::VniWindows,
        CodeTable::Cp1258,
    ] {
        assert_eq!(decode(&encode(SAMPLE, table), table), SAMPLE, "{table:?}");
    }

    assert_eq!(
        decode(&encode(SAMPLE, CodeTable::Tcvn3), CodeTable::Tcvn3),
        "Tiếng Việt có dấu: ắằẳẵặ ắằẳẵặ Đđ Ưứừửữự ơớờởỡợ",
        "TCVN3 reads its missing capitals back lower case"
    );
}

#[test]
fn every_letter_round_trips_in_every_table() {
    for table in TABLES {
        for c in vietnamese_letters() {
            let expected = if table == CodeTable::Tcvn3 {
                through_tcvn3(c)
            } else {
                c.to_string()
            };

            let read_back = decode(&encode(&c.to_string(), table), table);

            assert_eq!(read_back, expected, "{c} in {table:?}");
        }
    }
}

#[test]
fn compound_uses_combining_tone() {
    assert_eq!(encode("á", CodeTable::UnicodeCompound), [0x61, 0x0301], "a + acute");
    assert_eq!(encode("ấ", CodeTable::UnicodeCompound), [0xe2, 0x0301], "â + acute");
    assert_eq!(
        encode("Ợ", CodeTable::UnicodeCompound),
        [0x01a0, 0x0323],
        "Ơ + dot below"
    );
    assert_eq!(encode("ơ", CodeTable::UnicodeCompound), [0x01a1], "no tone, no mark");
}

#[test]
fn cp1258_spot_checks() {
    // CP1258.TXT row `0xE3 0x0103 LATIN SMALL LETTER A WITH BREVE`.
    assert_eq!(encode("ă", CodeTable::Cp1258), [0xe3], "ă");
    // CP1258.TXT row `0xD0 0x0110 LATIN CAPITAL LETTER D WITH STROKE`.
    assert_eq!(encode("Đ", CodeTable::Cp1258), [0xd0], "Đ");
    // The code page has no ố: row `0xF4 0x00F4 ... O WITH CIRCUMFLEX`, then row `0xEC 0x0301 COMBINING ACUTE ACCENT`.
    assert_eq!(encode("ố", CodeTable::Cp1258), [0xf4, 0xec], "ố");
}

#[test]
fn cp1258_reads_a_letter_whole_or_with_a_tone_byte() {
    assert_eq!(decode(&[0xe1], CodeTable::Cp1258), "á", "á whole");
    assert_eq!(decode(&[0x61, 0xec], CodeTable::Cp1258), "á", "a, then the acute byte");
    assert_eq!(
        decode(&[0xf5, 0xf2], CodeTable::Cp1258),
        "ợ",
        "ơ, then the dot-below byte"
    );
    assert_eq!(
        decode(&[0x62, 0xec], CodeTable::Cp1258),
        "b\u{301}",
        "a consonant keeps the bare mark"
    );
}

#[test]
fn tcvn3_spot_checks() {
    // VSCII chart, VSCII-3 cells: row Ax column 8 is U+0103 ă.
    assert_eq!(encode("ă", CodeTable::Tcvn3), [0xa8], "ă");
    // Row Cx column A is U+1EA5 ấ.
    assert_eq!(encode("ấ", CodeTable::Tcvn3), [0xca], "ấ");
    // Row Fx column 9 is U+1EF1 ự.
    assert_eq!(encode("ự", CodeTable::Tcvn3), [0xf9], "ự");
    // VSCII-3 has no Ấ: the capital is written with the ấ byte.
    assert_eq!(encode("Ấ", CodeTable::Tcvn3), [0xca], "Ấ");
}

#[test]
fn vni_spot_checks() {
    // Vietnamese Unicode FAQs chart, VNI column: row U+00E1 á is `61 F9`.
    assert_eq!(encode("á", CodeTable::VniWindows), [0x61, 0xf9], "á");
    // Row U+01AF Ư is `D6`.
    assert_eq!(encode("Ư", CodeTable::VniWindows), [0xd6], "Ư");
    // Row U+1EE3 ợ is `F4 EF`.
    assert_eq!(encode("ợ", CodeTable::VniWindows), [0xf4, 0xef], "ợ");
}

#[test]
fn units_match_encode() {
    let others = ['a', 'Z', '7', ' ', '漢', '😀'];

    for table in TABLES {
        for c in vietnamese_letters().into_iter().chain(others) {
            let encoded = encode(&c.to_string(), table);

            assert_eq!(units_in(c, table), encoded.len(), "{c} in {table:?}");
        }
    }
}

#[test]
fn non_vietnamese_passes_through() {
    let text = "😀漢";
    let utf16: Vec<u16> = text.encode_utf16().collect();

    for table in TABLES {
        assert_eq!(encode(text, table), utf16, "{table:?} writes them as UTF-16");
        assert_eq!(decode(&utf16, table), text, "{table:?} reads them back");
    }
}

#[test]
fn ascii_is_the_same_in_every_table() {
    let text = "Plain ASCII, 123 + xyz!";
    let utf16: Vec<u16> = text.encode_utf16().collect();

    for table in TABLES {
        assert_eq!(encode(text, table), utf16, "{table:?} writes ASCII as itself");
        assert_eq!(decode(&utf16, table), text, "{table:?} reads ASCII as itself");
    }
}
