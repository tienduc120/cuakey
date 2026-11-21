# CuaKey

Vietnamese typing for Windows, written in Rust.

## Licence

MIT, Copyright (c) 2026 Luu Tien Duc. See [LICENSE](LICENSE).

## Credits

This project stands on public work. Material that is copied or adapted is listed in [NOTICE.md](NOTICE.md) with its license; this section thanks everyone whose work shaped a decision, whether or not any text was copied.

**Used**

- [rust-base](https://github.com/tienduc120/rust-base): the project standard whose rules, gates, and skills this repository follows.
- The Rust project: the language, cargo, rustfmt, clippy, and rustdoc.
- The character charts the code tables are written from: Unicode's [CP1258 mapping](https://www.unicode.org/Public/MAPPINGS/VENDORS/MICSFT/WINDOWS/CP1258.TXT), the [TCVN 5712 (VSCII) chart](https://en.wikipedia.org/wiki/VSCII), and the [VNI Windows chart](https://vietunicode.sourceforge.net/charset/) of the Vietnamese Unicode FAQs.
- Crates we depend on: [tokio](https://github.com/tokio-rs/tokio), tokio-util, [serde](https://github.com/serde-rs/serde), [thiserror](https://github.com/dtolnay/thiserror), [anyhow](https://github.com/dtolnay/anyhow), [tracing](https://github.com/tokio-rs/tracing), [jiff](https://github.com/BurntSushi/jiff), [clap](https://github.com/clap-rs/clap), [syn](https://github.com/dtolnay/syn), [cargo_metadata](https://github.com/oli-obk/cargo_metadata), [toml and toml_edit](https://github.com/toml-rs/toml), [walkdir](https://github.com/BurntSushi/walkdir), [serde_json](https://github.com/serde-rs/json).
- Developer tools we run, not copy: [cargo-nextest](https://github.com/nextest-rs/nextest), [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) (Embark Studios), [cargo-shear](https://github.com/Boshen/cargo-shear), [typos](https://github.com/crate-ci/typos), [lefthook](https://github.com/evilmartians/lefthook) (Evil Martians), [cargo-binstall](https://github.com/cargo-bins/cargo-binstall), [zizmor](https://github.com/zizmorcore/zizmor), [taiki-e/install-action](https://github.com/taiki-e/install-action), [Swatinem/rust-cache](https://github.com/Swatinem/rust-cache).

**Referenced for design** (no code copied)

| Source                                                                                                                   | What shaped us                                                                                                        |
| ------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------- |
| [go-base](https://github.com/tienduc120/go-base)                                                                          | the whole shape: every rule proven by a fixture, one command surface, the agent guard, the layer rules                |
| [matklad&#39;s cargo xtask](https://github.com/matklad/cargo-xtask)                                                       | tooling written in Rust and run with `cargo xtask` instead of Make                                                  |
| [zed](https://github.com/zed-industries/zed)                                                                              | forbidden crate edges checked from the dependency graph; no `mod.rs` files                                          |
| [openai/codex](https://github.com/openai/codex)                                                                           | `unwrap_used`/`expect_used` at deny with test exemptions; cargo-deny `wrappers`; the clean-worktree check in CI |
| [astral-sh/uv](https://github.com/astral-sh/uv) and [ruff](https://github.com/astral-sh/ruff)                              | `CARGO_BUILD_WARNINGS=deny`; nextest timeouts; agent rules on panics, `#[expect]`, and exhaustive matches         |
| [rust-analyzer](https://github.com/rust-lang/rust-analyzer)                                                               | tidy checks as tests in xtask; new dependencies and public items as architectural changes                             |
| [Microsoft Pragmatic Rust Guidelines](https://microsoft.github.io/rust-guidelines/)                                       | lint posture, `#[expect]` over `#[allow]`, errors per layer, Rust-shaped code from agents                         |
| [How To Code It: hexagonal architecture in Rust](https://www.howtocodeit.com/articles/master-hexagonal-architecture-rust) | ports as traits with explicit `Send` futures, generics over `dyn`                                                 |
| [AGENTS.md](https://agents.md)                                                                                            | the shared agent instruction format                                                                                   |
| [Google Rust Style Guide](https://google.github.io/styleguide/rustguide.html) | no `catch_unwind`, no code before `main`, lowercase hex literals |
| [Chromium Rust documents](https://chromium.googlesource.com/chromium/src/+/main/docs/rust/) | unsafe kept to one opted-in crate, each block justified |
| [Android lint configuration](https://android.googlesource.com/platform/build/soong/+/HEAD/rust/config/lints.go) | `undocumented_unsafe_blocks` and `unsafe_op_in_unsafe_fn` at deny; overflow checks on |
| [Fuchsia Rust API rubric](https://fuchsia.dev/fuchsia-src/development/api/rust) | `Debug` on every public type; unsafe operations in their own blocks |
| [Linux kernel Rust coding guidelines](https://docs.kernel.org/rust/coding-guidelines.html) | `SAFETY:` comments, named cast targets, the unsafe lint set |
| [ANSSI Secure Rust Guidelines](https://anssi-fr.github.io/rust-guide/) | no assert in production, no deliberate leaks, overflow checks never off, no build flags from the environment |
| [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) and [Rust Style Guide](https://doc.rust-lang.org/style-guide/) | lowercase error messages, acronyms as one word, no `#[path]` |
| [Rust Foundation safety-critical coding guidelines](https://coding-guidelines.arewesafetycriticalyet.org/) (draft) | no `as _` casts |
| [Canonical Rust best practices](https://canonical.github.io/rust-best-practices/) | lowercase hex literals, unsafe discipline |
