/// A letter without its tone: a Vietnamese vowel, d or đ, or another ASCII consonant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BaseLetter {
    A,
    /// ă
    ABreve,
    /// â
    ACircumflex,
    E,
    /// ê
    ECircumflex,
    I,
    O,
    /// ô
    OCircumflex,
    /// ơ
    OHorn,
    U,
    /// ư
    UHorn,
    Y,
    D,
    /// đ
    DStroke,
    B,
    C,
    F,
    G,
    H,
    J,
    K,
    L,
    M,
    N,
    P,
    Q,
    R,
    S,
    T,
    V,
    W,
    X,
    Z,
}

/// The six Vietnamese tones, by their Vietnamese names; every tone but `Ngang` is written as a mark on the vowel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tone {
    /// No mark.
    Ngang,
    /// Acute: á.
    Sac,
    /// Grave: à.
    Huyen,
    /// Hook above: ả.
    Hoi,
    /// Tilde: ã.
    Nga,
    /// Dot below: ạ.
    Nang,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LetterCase {
    Lower,
    Upper,
}

/// One letter split into what typing changes separately: its base, its tone, and its case. Every one of the 134
/// Vietnamese letters and every ASCII letter is one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Letter {
    /// Kept beside its parts, so `to_char` needs no lookup that could fail.
    character: char,
    base: BaseLetter,
    tone: Tone,
    case: LetterCase,
}

/// The vowels in table order: every grid of toned vowels, here and in the code tables, has its rows in this order.
const VOWELS: [BaseLetter; 12] = [
    BaseLetter::A,
    BaseLetter::ABreve,
    BaseLetter::ACircumflex,
    BaseLetter::E,
    BaseLetter::ECircumflex,
    BaseLetter::I,
    BaseLetter::O,
    BaseLetter::OCircumflex,
    BaseLetter::OHorn,
    BaseLetter::U,
    BaseLetter::UHorn,
    BaseLetter::Y,
];

/// The tones in table order: every grid of toned vowels has its columns in this order.
const TONES: [Tone; 6] = [Tone::Ngang, Tone::Sac, Tone::Huyen, Tone::Hoi, Tone::Nga, Tone::Nang];

/// Every lower-case vowel with every tone, rows in `VOWELS` order and columns in `TONES` order.
pub(super) const LOWER_VOWELS: [[char; 6]; 12] = [
    ['a', 'á', 'à', 'ả', 'ã', 'ạ'],
    ['ă', 'ắ', 'ằ', 'ẳ', 'ẵ', 'ặ'],
    ['â', 'ấ', 'ầ', 'ẩ', 'ẫ', 'ậ'],
    ['e', 'é', 'è', 'ẻ', 'ẽ', 'ẹ'],
    ['ê', 'ế', 'ề', 'ể', 'ễ', 'ệ'],
    ['i', 'í', 'ì', 'ỉ', 'ĩ', 'ị'],
    ['o', 'ó', 'ò', 'ỏ', 'õ', 'ọ'],
    ['ô', 'ố', 'ồ', 'ổ', 'ỗ', 'ộ'],
    ['ơ', 'ớ', 'ờ', 'ở', 'ỡ', 'ợ'],
    ['u', 'ú', 'ù', 'ủ', 'ũ', 'ụ'],
    ['ư', 'ứ', 'ừ', 'ử', 'ữ', 'ự'],
    ['y', 'ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ'],
];

/// `LOWER_VOWELS` in capitals.
pub(super) const UPPER_VOWELS: [[char; 6]; 12] = [
    ['A', 'Á', 'À', 'Ả', 'Ã', 'Ạ'],
    ['Ă', 'Ắ', 'Ằ', 'Ẳ', 'Ẵ', 'Ặ'],
    ['Â', 'Ấ', 'Ầ', 'Ẩ', 'Ẫ', 'Ậ'],
    ['E', 'É', 'È', 'Ẻ', 'Ẽ', 'Ẹ'],
    ['Ê', 'Ế', 'Ề', 'Ể', 'Ễ', 'Ệ'],
    ['I', 'Í', 'Ì', 'Ỉ', 'Ĩ', 'Ị'],
    ['O', 'Ó', 'Ò', 'Ỏ', 'Õ', 'Ọ'],
    ['Ô', 'Ố', 'Ồ', 'Ổ', 'Ỗ', 'Ộ'],
    ['Ơ', 'Ớ', 'Ờ', 'Ở', 'Ỡ', 'Ợ'],
    ['U', 'Ú', 'Ù', 'Ủ', 'Ũ', 'Ụ'],
    ['Ư', 'Ứ', 'Ừ', 'Ử', 'Ữ', 'Ự'],
    ['Y', 'Ý', 'Ỳ', 'Ỷ', 'Ỹ', 'Ỵ'],
];

/// Every consonant, Vietnamese or ASCII, with its lower-case and capital form.
const CONSONANTS: [(BaseLetter, char, char); 21] = [
    (BaseLetter::B, 'b', 'B'),
    (BaseLetter::C, 'c', 'C'),
    (BaseLetter::D, 'd', 'D'),
    (BaseLetter::DStroke, 'đ', 'Đ'),
    (BaseLetter::F, 'f', 'F'),
    (BaseLetter::G, 'g', 'G'),
    (BaseLetter::H, 'h', 'H'),
    (BaseLetter::J, 'j', 'J'),
    (BaseLetter::K, 'k', 'K'),
    (BaseLetter::L, 'l', 'L'),
    (BaseLetter::M, 'm', 'M'),
    (BaseLetter::N, 'n', 'N'),
    (BaseLetter::P, 'p', 'P'),
    (BaseLetter::Q, 'q', 'Q'),
    (BaseLetter::R, 'r', 'R'),
    (BaseLetter::S, 's', 'S'),
    (BaseLetter::T, 't', 'T'),
    (BaseLetter::V, 'v', 'V'),
    (BaseLetter::W, 'w', 'W'),
    (BaseLetter::X, 'x', 'X'),
    (BaseLetter::Z, 'z', 'Z'),
];

impl Letter {
    /// The letter `c` is, or `None` when `c` is neither a Vietnamese nor an ASCII letter.
    pub fn from_char(c: char) -> Option<Self> {
        Self::all().find(|letter| letter.character == c)
    }

    pub const fn to_char(self) -> char {
        self.character
    }

    pub const fn base(self) -> BaseLetter {
        self.base
    }

    pub const fn tone(self) -> Tone {
        self.tone
    }

    pub const fn case(self) -> LetterCase {
        self.case
    }

    /// The letter made of these parts, or `None` for a consonant with a tone.
    pub fn from_parts(base: BaseLetter, tone: Tone, case: LetterCase) -> Option<Self> {
        Self::all().find(|letter| letter.base == base && letter.tone == tone && letter.case == case)
    }

    /// The same letter carrying `tone` instead, or `None` when it is a consonant, which carries no tone.
    pub fn with_tone(self, tone: Tone) -> Option<Self> {
        Self::all().find(|letter| letter.base == self.base && letter.case == self.case && letter.tone == tone)
    }

    /// Every letter there is: the toned vowels, lower case first, then the consonants.
    fn all() -> impl Iterator<Item = Self> {
        let vowels = vowel_grid(LetterCase::Lower, &LOWER_VOWELS).chain(vowel_grid(LetterCase::Upper, &UPPER_VOWELS));
        let consonants = CONSONANTS.into_iter().flat_map(|(base, lower, upper)| {
            [(LetterCase::Lower, lower), (LetterCase::Upper, upper)].map(|(case, character)| Self {
                character,
                base,
                tone: Tone::Ngang,
                case,
            })
        });
        vowels.chain(consonants)
    }
}

fn vowel_grid(case: LetterCase, grid: &'static [[char; 6]; 12]) -> impl Iterator<Item = Letter> {
    VOWELS.into_iter().zip(grid).flat_map(move |(base, row)| {
        TONES.into_iter().zip(*row).map(move |(tone, character)| Letter {
            character,
            base,
            tone,
            case,
        })
    })
}
