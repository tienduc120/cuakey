# openkey-compare

Types the same keys into CuaKey's typing engine and into [OpenKey](https://github.com/tuyenvm/OpenKey)'s, converts the same texts with both, and reports every place where the results differ.

## Run

```sh
cargo xtask compare-openkey                       # the whole corpus
cargo xtask compare-openkey --filter vni          # one corpus file (or macros, or convert)
cargo xtask compare-openkey --random 5000 --seed 7
```

The command fetches OpenKey's sources at a pinned commit into `target/openkey/` (again when the pin moves), builds them with this tool (MSVC, C++17), and writes `target/openkey-compare/report.md`. It exits 0 only when every difference it finds is accepted in `accepted-differences.toml` and no entry there has gone stale. The tool's own tests run with `cargo nextest run --manifest-path tools/openkey-compare/Cargo.toml` once the command has fetched the sources.

An accepted entry pins one divergence: the corpus file (or `convert`, or `random`), the keys, the setting tokens it occurs under, the key number, and both texts, with the reason. The header of `accepted-differences.toml` describes the fields, and each report entry prints what an entry needs. An entry that matches nothing in a run that typed what it names is reported as stale.

## What it compares

Each line of a typing file in `corpus/` is typed into an empty text box, then a Space that ends its last word, under every combination of the file's input methods, the five code tables, and spell check, restore, modern tones, quick Telex, quick first consonant, quick last consonant, and capitals on and off: 640 combinations per input method. The macro file is typed with the macros of `macro_table.txt` (the macro file format: `shortcut:expansion`, split at the first colon), in Vietnamese and in English, with macros, macros in English, matching capitals, spell check, restore, and capitals on and off. CuaKey's keep-`w` and the compatibility fixes are off. After every key, the text CuaKey's edit leaves, once encoded in the code table with each deleted character taking as many backspaces as it has code units there, is compared with the text OpenKey's engine leaves when a corrected host applies its answers. A run stops at its first difference no entry accepts; after an accepted one, the corrected host's text (and its record of how many code units each letter takes) is rebuilt from CuaKey's and the run goes on. The report groups the runs of a line that end the same way.

The corrected host applies OpenKey's answers the way OpenKey's Windows host does, except for four behaviours of that host which CuaKey does not reproduce. A second pass applies the answers with those behaviours too, and the report counts the runs whose only difference comes from them, per behaviour, in a section that never fails the run:

- `cp1258-whole-letter`: in CP1258 OpenKey writes a toned letter as two code units but deletes it with one Backspace, so the letter's first unit stays on screen; the corrected host deletes the whole letter.
- `space-after-backspace`: in VNI Windows and Unicode compound, a Space typed right after a Backspace makes OpenKey's host delete the second unit of the last two-unit letter; the corrected host only types the Space.
- `english-macro-key`: typing English with macros on in English, OpenKey's host lets the key that ends a shortcut reach the text box before it deletes the shortcut and types the expansion, so the shortcut's first letter stays; the corrected host holds the key back until the expansion is typed.
- `retyped-key`: after restoring a word or expanding a quick consonant or a shortcut, OpenKey's host types the key that ended the word itself, and types the quote key as a backslash, the backslash key as a backquote, the backquote key as a quote, and, with Caps Lock on, a punctuation or digit key as its shifted character; the corrected host types what the key types.

| File | Typed in |
| --- | --- |
| `telex.txt` | Telex |
| `vni.txt` | VNI |
| `simple_telex.txt` | Simple Telex |
| `english.txt` | all three |
| `sentences.txt` | Telex, Simple Telex |
| `macros.txt` | Telex, in Vietnamese and English |

Each text of `convert.txt` (Unicode; `\n` a line break, `\\` a backslash, `\u{301}` a combining mark) is written in each code table and converted by both engines to every code table, with each case option (keep, upper, lower, sentence start, each word), with and without accents: 250 settings. The outputs are compared unit for unit, and a conversion that makes OpenKey stop with an error is reported as such.

A full run takes about eight seconds, so no combination is left out. Random mode types seeded sequences of letters, mark and tone keys, Space, Backspace, punctuation, clicks, and Ctrl shortcuts, each ended by a Space and typed under random settings.

Corpus notation: a printable character is the key that types it on a US layout, an upper-case letter or shifted symbol being that key with Shift. `<bs>` is Backspace, `<enter>`, `<tab>`, and `<esc>` those keys, `<click>` a mouse click (the caret stays at the end), `<caps>` turns Caps Lock on or off for the keys after it (the press itself is not sent), `<ctrl-x>` the letter x with Ctrl held, and `<lt>` the `<` key. A line starting with `#` is a comment.

## Clean room

CuaKey's typing engine is written from scratch. Only the author of this tool reads OpenKey's source, to write `shim/shim.cpp` and to apply OpenKey's output the way its Windows host does. Nothing of OpenKey is copied into this repository: the shim calls OpenKey's public header, and OpenKey itself is fetched and compiled on your machine, never committed. Work on CuaKey's engine reads the report, never OpenKey's source or the tool's OpenKey side: `shim/`, `src/oracle.rs`, and `src/openkey_screen.rs`.

## Contributing

Add corpus lines, inputs only, to the file for their input method, and texts to `convert.txt`. A difference CuaKey should not reproduce, such as one that comes from how OpenKey's host applies its output, or a random sequence no person would type, goes in `accepted-differences.toml` with its reason in terms of behaviour.

## License

MIT, like the rest of the repository. OpenKey is GPL-3.0; this tool only builds it locally and never distributes it.
