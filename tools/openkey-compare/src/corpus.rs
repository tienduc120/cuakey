//! The key sequences the engines are compared on, and the notation they are written in.
//!
//! A line is typed key by key: a printable character is the key that types it on a US layout (an upper-case letter or
//! a shifted symbol is that key with Shift), and these stand for other keys: `<bs>` Backspace, `<enter>`, `<tab>`,
//! `<esc>`, `<click>` a mouse click, `<caps>` turns Caps Lock on or off for the keys after it (the press itself is not
//! sent), `<ctrl-x>` the letter x with Ctrl held, `<lt>` the `<` character. A line starting with `#` is a comment.

use std::fmt::{self, Write as _};
use std::fs;
use std::path::Path;

use cuakey_domain::{InputMethod, KeyCode, KeyMods, KeyPress, Language, OnOff};

use crate::combo::{Combo, Toggle};

/// One thing the person does: press a key, or click.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stroke {
    Key(KeyPress),
    Click,
}

/// A corpus file, the input methods and languages its lines are typed in, and the toggles each setting of which they
/// are typed under; the other toggles stay at their defaults.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CorpusFile {
    pub name: &'static str,
    pub methods: &'static [InputMethod],
    pub languages: &'static [Language],
    pub toggles: &'static [Toggle],
    /// Its lines are typed with the macro table of `macro_table.txt`; the others with no macros.
    pub macros: bool,
}

impl CorpusFile {
    /// Every combination its lines are typed under.
    pub fn combos(self) -> impl Iterator<Item = Combo> {
        self.methods
            .iter()
            .flat_map(move |method| Combo::all_for(*method, self.toggles, self.languages))
    }

    pub const fn combos_per_method(self) -> usize {
        Combo::count_for(self.toggles, self.languages)
    }
}

const fn typing_file(name: &'static str, methods: &'static [InputMethod]) -> CorpusFile {
    CorpusFile {
        name,
        methods,
        languages: &[Language::Vietnamese],
        toggles: &Toggle::TYPING,
        macros: false,
    }
}

pub const CORPUS_FILES: [CorpusFile; 6] = [
    typing_file("telex", &[InputMethod::Telex]),
    typing_file("vni", &[InputMethod::Vni]),
    typing_file("simple_telex", &[InputMethod::SimpleTelex]),
    typing_file(
        "english",
        &[InputMethod::Telex, InputMethod::Vni, InputMethod::SimpleTelex],
    ),
    typing_file("sentences", &[InputMethod::Telex, InputMethod::SimpleTelex]),
    CorpusFile {
        name: "macros",
        methods: &[InputMethod::Telex],
        languages: &[Language::Vietnamese, Language::English],
        toggles: &Toggle::MACROS,
        macros: true,
    },
];

/// One line of a corpus file: its text and the strokes it stands for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub text: String,
    pub strokes: Vec<Stroke>,
}

impl Line {
    /// The strokes as the comparison types them: the line, then a Space that ends its last word, since an invalid
    /// word is restored and a quick consonant expanded only when the word ends.
    pub fn ended(&self) -> Vec<Stroke> {
        let space = Stroke::Key(KeyPress::new(KeyCode::Space, KeyMods::none()));
        self.strokes.iter().copied().chain([space]).collect()
    }
}

/// The lines of `dir/<file>.txt`, comments and blank lines left out.
pub fn read_file(dir: &Path, file: CorpusFile) -> Result<Vec<Line>, String> {
    let path = dir.join(format!("{}.txt", file.name));
    let text = fs::read_to_string(&path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    text.lines()
        .enumerate()
        .filter(|(_, line)| !line.is_empty() && !line.starts_with('#'))
        .map(|(at, line)| {
            let strokes = parse(line).map_err(|e| format!("{}:{}: {e}", path.display(), at + 1))?;
            Ok(Line {
                text: line.to_owned(),
                strokes,
            })
        })
        .collect()
}

/// The macro table in `dir/macro_table.txt`.
pub fn read_macro_table(dir: &Path) -> Result<Vec<(String, String)>, String> {
    let path = dir.join("macro_table.txt");
    let text = fs::read_to_string(&path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    parse_macro_table(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// The macros of a file in the macro file format: one `shortcut:expansion` per line, split at the first colon; a
/// line starting with `;` is a comment.
pub fn parse_macro_table(text: &str) -> Result<Vec<(String, String)>, String> {
    text.lines()
        .filter(|line| !line.is_empty() && !line.starts_with(';'))
        .map(|line| {
            line.split_once(':')
                .map(|(shortcut, expansion)| (shortcut.to_owned(), expansion.to_owned()))
                .ok_or_else(|| format!("`{line}` has no colon between the shortcut and its expansion"))
        })
        .collect()
}

/// The strokes `line` stands for.
pub fn parse(line: &str) -> Result<Vec<Stroke>, String> {
    let mut strokes = Vec::new();
    let mut caps = OnOff::Off;
    let mut rest = line;
    while let Some(c) = rest.chars().next() {
        if c == '<' {
            let end = rest
                .find('>')
                .ok_or_else(|| format!("`{rest}` opens a key name it never closes"))?;
            let name = rest.get(1..end).unwrap_or_default();
            rest = rest.get(end + 1..).unwrap_or_default();
            if name == "caps" {
                caps = toggled(caps);
                continue;
            }
            strokes.push(named(name, caps).ok_or_else(|| format!("`<{name}>` is no key this notation knows"))?);
        } else {
            rest = rest.get(c.len_utf8()..).unwrap_or_default();
            let (code, shift) = key_typing(c).ok_or_else(|| format!("`{c}` is no key on a US layout"))?;
            strokes.push(press(code, shift, caps, OnOff::Off));
        }
    }
    Ok(strokes)
}

/// The notation for `strokes`; parsing it gives the same strokes back.
pub fn notation(strokes: &[Stroke]) -> String {
    let mut text = String::new();
    let mut caps = OnOff::Off;
    for stroke in strokes {
        if let Stroke::Key(key) = stroke
            && key.mods().caps_lock() != caps
        {
            caps = key.mods().caps_lock();
            text.push_str("<caps>");
        }
        // Writing to a String cannot fail.
        write!(text, "{}", Notation(*stroke)).unwrap_or_default();
    }
    text
}

/// One stroke in the notation, without the Caps Lock state.
#[derive(Debug, Clone, Copy)]
pub struct Notation(pub Stroke);

impl fmt::Display for Notation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let key = match self.0 {
            Stroke::Click => return f.write_str("<click>"),
            Stroke::Key(key) => key,
        };
        if key.mods().ctrl_or_alt().is_on() {
            let letter = char_typed(key.code(), OnOff::Off).unwrap_or('?');
            return write!(f, "<ctrl-{letter}>");
        }
        match (key.code(), char_typed(key.code(), key.mods().shift())) {
            (KeyCode::Backspace, _) => f.write_str("<bs>"),
            (KeyCode::Enter, _) => f.write_str("<enter>"),
            (KeyCode::Tab, _) => f.write_str("<tab>"),
            (KeyCode::Escape, _) => f.write_str("<esc>"),
            (_, Some('<')) => f.write_str("<lt>"),
            (_, Some(c)) => f.write_char(c),
            (_, None) => f.write_str("<?>"),
        }
    }
}

fn named(name: &str, caps: OnOff) -> Option<Stroke> {
    let code = match name {
        "bs" => KeyCode::Backspace,
        "enter" => KeyCode::Enter,
        "tab" => KeyCode::Tab,
        "esc" => KeyCode::Escape,
        "click" => return Some(Stroke::Click),
        "lt" => return Some(press(KeyCode::Comma, OnOff::On, caps, OnOff::Off)),
        _ => {
            let letter = name.strip_prefix("ctrl-")?;
            let mut chars = letter.chars();
            let (code, _) = chars.next().filter(char::is_ascii_lowercase).and_then(key_typing)?;
            return chars.next().is_none().then(|| press(code, OnOff::Off, caps, OnOff::On));
        }
    };
    Some(press(code, OnOff::Off, caps, OnOff::Off))
}

const fn toggled(on: OnOff) -> OnOff {
    match on {
        OnOff::On => OnOff::Off,
        OnOff::Off => OnOff::On,
    }
}

fn press(code: KeyCode, shift: OnOff, caps: OnOff, ctrl: OnOff) -> Stroke {
    Stroke::Key(KeyPress::new(
        code,
        KeyMods::none()
            .with_shift(shift)
            .with_caps_lock(caps)
            .with_ctrl_or_alt(ctrl),
    ))
}

/// The keys of a US layout that type a character: the key, what it types, and what it types with Shift.
const PRINTABLE: [(KeyCode, char, char); 48] = [
    (KeyCode::A, 'a', 'A'),
    (KeyCode::B, 'b', 'B'),
    (KeyCode::C, 'c', 'C'),
    (KeyCode::D, 'd', 'D'),
    (KeyCode::E, 'e', 'E'),
    (KeyCode::F, 'f', 'F'),
    (KeyCode::G, 'g', 'G'),
    (KeyCode::H, 'h', 'H'),
    (KeyCode::I, 'i', 'I'),
    (KeyCode::J, 'j', 'J'),
    (KeyCode::K, 'k', 'K'),
    (KeyCode::L, 'l', 'L'),
    (KeyCode::M, 'm', 'M'),
    (KeyCode::N, 'n', 'N'),
    (KeyCode::O, 'o', 'O'),
    (KeyCode::P, 'p', 'P'),
    (KeyCode::Q, 'q', 'Q'),
    (KeyCode::R, 'r', 'R'),
    (KeyCode::S, 's', 'S'),
    (KeyCode::T, 't', 'T'),
    (KeyCode::U, 'u', 'U'),
    (KeyCode::V, 'v', 'V'),
    (KeyCode::W, 'w', 'W'),
    (KeyCode::X, 'x', 'X'),
    (KeyCode::Y, 'y', 'Y'),
    (KeyCode::Z, 'z', 'Z'),
    (KeyCode::Digit0, '0', ')'),
    (KeyCode::Digit1, '1', '!'),
    (KeyCode::Digit2, '2', '@'),
    (KeyCode::Digit3, '3', '#'),
    (KeyCode::Digit4, '4', '$'),
    (KeyCode::Digit5, '5', '%'),
    (KeyCode::Digit6, '6', '^'),
    (KeyCode::Digit7, '7', '&'),
    (KeyCode::Digit8, '8', '*'),
    (KeyCode::Digit9, '9', '('),
    (KeyCode::Space, ' ', ' '),
    (KeyCode::Minus, '-', '_'),
    (KeyCode::Equal, '=', '+'),
    (KeyCode::LeftBracket, '[', '{'),
    (KeyCode::RightBracket, ']', '}'),
    (KeyCode::Backslash, '\\', '|'),
    (KeyCode::Semicolon, ';', ':'),
    (KeyCode::Quote, '\'', '"'),
    (KeyCode::Comma, ',', '<'),
    (KeyCode::Period, '.', '>'),
    (KeyCode::Slash, '/', '?'),
    (KeyCode::Backquote, '`', '~'),
];

/// The key that types `c` on a US layout, and whether it needs Shift.
fn key_typing(c: char) -> Option<(KeyCode, OnOff)> {
    PRINTABLE.iter().find_map(|(code, plain, shifted)| {
        if c == *plain {
            Some((*code, OnOff::Off))
        } else if c == *shifted {
            Some((*code, OnOff::On))
        } else {
            None
        }
    })
}

/// The character `code` types on a US layout, with or without Shift. Caps Lock is the caller's to apply.
pub fn char_typed(code: KeyCode, shift: OnOff) -> Option<char> {
    PRINTABLE
        .iter()
        .find(|(key, ..)| *key == code)
        .map(|(_, plain, shifted)| if shift.is_on() { *shifted } else { *plain })
}

#[cfg(test)]
mod tests;
