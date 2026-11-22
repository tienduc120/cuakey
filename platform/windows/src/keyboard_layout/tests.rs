use super::{is_english, may_have_ime};

const US_ENGLISH: isize = 0x0409_0409;
const UK_ENGLISH: isize = 0x0809_0809;
const VIETNAMESE: isize = 0x042a_042a;
const SIMPLIFIED_CHINESE: isize = 0x0804_0804;
const JAPANESE: isize = 0x0411_0411;
const KOREAN: isize = 0x0412_0412;

#[test]
fn english_layout_detection() {
    assert!(is_english(US_ENGLISH), "US English is English");
    assert!(!is_english(VIETNAMESE), "Vietnamese is not English");
}

#[test]
fn every_english_variant_is_english() {
    // A handle whose high word names a layout variant rather than repeating the language: US English with the Dvorak
    // layout, sign-extended as Windows hands it out.
    let us_dvorak = isize::try_from(0xf002_0409_u32.cast_signed()).unwrap();

    let english = [UK_ENGLISH, us_dvorak].map(is_english);

    assert_eq!(
        english, [true; 2],
        "the language, not the country or the layout, decides"
    );
}

#[test]
fn only_chinese_japanese_and_korean_may_have_an_ime() {
    let with_ime = [SIMPLIFIED_CHINESE, JAPANESE, KOREAN].map(may_have_ime);
    let without = [US_ENGLISH, VIETNAMESE].map(may_have_ime);

    assert_eq!(
        with_ime, [true; 3],
        "these languages are typed through an input method editor"
    );
    assert_eq!(without, [false; 2], "these are typed key by key");
}
