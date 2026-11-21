# openkey-compare

Types the same keys into CuaKey's typing engine and into [OpenKey](https://github.com/tuyenvm/OpenKey)'s, and reports every place where the text on screen differs.

## Run

```sh
cargo xtask compare-openkey                       # the whole corpus
cargo xtask compare-openkey --filter vni          # one corpus file
cargo xtask compare-openkey --random 5000 --seed 7
```

The command fetches OpenKey's sources at a pinned commit into `target/openkey/` (again when the pin moves), builds them with this tool (MSVC, C++17), and writes `target/openkey-compare/report.md`. It exits 0 only when every difference it finds is accepted in `accepted-differences.toml` and no entry there has gone stale. The tool's own tests run with `cargo nextest run --manifest-path tools/openkey-compare/Cargo.toml` once the command has fetched the sources.

An accepted entry pins one divergence: the corpus file (or `random`), the keys, the setting tokens it occurs under, the key number, and both texts, with the reason. The header of `accepted-differences.toml` describes the fields, and each report entry prints what an entry needs. An entry that matches nothing in a run that typed what it names is reported as stale.

## What it compares

Each line of `corpus/` is typed into an empty text box, then a Space that ends its last word, under every combination of the file's input methods, the five code tables, and spell check, restore, modern tones, quick Telex, quick first consonant, quick last consonant, and capitals on and off: 640 combinations per input method. Macros, CuaKey's keep-`w`, and the compatibility fixes are off. After every key, the text OpenKey's Windows host would leave is compared with the text CuaKey's edit leaves once encoded in the code table, each deleted character taking as many backspaces as it has code units there. A run stops at its first difference, and the report groups the runs of a line that end the same way.

| File | Typed in |
| --- | --- |
| `telex.txt` | Telex |
| `vni.txt` | VNI |
| `simple_telex.txt` | Simple Telex |
| `english.txt` | all three |
| `sentences.txt` | Telex, Simple Telex |

A full run takes about seven seconds, so no combination is left out. Random mode types seeded sequences of letters, mark and tone keys, Space, Backspace, punctuation, clicks, and Ctrl shortcuts, each ended by a Space and typed under random settings.

Corpus notation: a printable character is the key that types it on a US layout, an upper-case letter or shifted symbol being that key with Shift. `<bs>` is Backspace, `<enter>`, `<tab>`, and `<esc>` those keys, `<click>` a mouse click (the caret stays at the end), `<caps>` turns Caps Lock on or off for the keys after it (the press itself is not sent), `<ctrl-x>` the letter x with Ctrl held, and `<lt>` the `<` key. A line starting with `#` is a comment.

## Clean room

CuaKey's typing engine is written from scratch. Only the author of this tool reads OpenKey's source, to write `shim/shim.cpp` and to apply OpenKey's output the way its Windows host does. Nothing of OpenKey is copied into this repository: the shim calls OpenKey's public header, and OpenKey itself is fetched and compiled on your machine, never committed. Work on CuaKey's engine reads the report, never OpenKey's source or the tool's OpenKey side: `shim/`, `src/oracle.rs`, and `src/openkey_screen.rs`.

## Contributing

Add corpus lines, inputs only, to the file for their input method. A difference CuaKey should not reproduce, such as one that comes from how OpenKey's host applies its output, or a random sequence no person would type, goes in `accepted-differences.toml` with its reason in terms of behaviour.

## License

MIT, like the rest of the repository. OpenKey is GPL-3.0; this tool only builds it locally and never distributes it.
