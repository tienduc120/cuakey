# Third-party material

Only MIT, BSD, and Apache-2.0 material is brought in. For each item keep the upstream license text in the repository, record the upstream and version below, and prefer a dependency to a copy. A new dependency needs an ADR.

## Vendored or adapted

- The rules, gates, and skills come from rust-base, the owner's own MIT-licensed work, under the same copyright holder as this repository.
- The icons in `icon/` are the project's own artwork, under the project license.

Nothing is taken from OpenKey: its sources are fetched and built locally by the comparison tool and never committed.

## Crates (used as dependencies, not copied)

Their versions are pinned once in the root `Cargo.toml` and their licenses are checked by `cargo deny check licenses` against the allowlist in `deny.toml`. `anyhow`, `clap`, `syn`, `cargo_metadata`, `toml_edit`, and `walkdir` belong to `xtask` only.

## Developer tools (installed, not copied)

cargo-nextest, cargo-deny, cargo-shear, typos, lefthook, zizmor. Each keeps its own license; `cargo xtask doctor` checks their versions.

The release workflow also runs `actions/upload-artifact` (MIT) and SignPath's `submit-signing-request` action, whose repository states no license. Neither is copied.
