/// How typed keys become Vietnamese letters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputMethod {
    Telex,
    Vni,
    /// Telex where a lone `w` never becomes `ư` and the brackets do not type `ơ` or `ư`.
    SimpleTelex,
}

/// The character encoding the typed text is produced in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CodeTable {
    Unicode,
    Tcvn3,
    VniWindows,
    UnicodeCompound,
    /// The Vietnamese Windows code page 1258.
    Cp1258,
}

/// The language the keyboard is typing in right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Language {
    Vietnamese,
    English,
}
