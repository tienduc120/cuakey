# 0013 - Rules from published Rust guides

- **Status**: accepted; decision 6 in part superseded by 0014
- **Date**: 2026-11-14

## Context

The base's rules came from its own design and a survey of well-run repositories. The owner asked to strengthen them from legitimate published guides. On 2026-11-14 these were read in full: Microsoft's Pragmatic Rust Guidelines (2026.6), Google's Rust Style Guide, Chromium's Rust documents, Android's lint configuration (soong `rust/config/lints.go`), Fuchsia's Rust API rubric, the Linux kernel's Rust coding guidelines, ANSSI's Secure Rust Guidelines, the Rust Foundation's safety-critical coding guidelines (a draft), rust-lang's API Guidelines and Style Guide, and Canonical's Rust best practices. Amazon, Meta, Cloudflare, Discord, 1Password, and Dropbox publish no Rust style guide that could be found.

Most of what these guides ask, the base already enforced. A rule here still needs a tool and a fixture (ADR-0002); a guide's advice that no tool can check goes into a skill, not into the list of enforced rules.

## Decision

1. **Lints added** (`Cargo.toml`): `missing_debug_implementations`, `unsafe_op_in_unsafe_fn`, `trivial_numeric_casts`, `unused_lifetimes`, `redundant_imports` (rustc); `unused_result_ok`, `as_underscore`, and the unsafe discipline `undocumented_unsafe_blocks`, `multiple_unsafe_ops_per_block`, `unnecessary_safety_comment`, `unnecessary_safety_doc` (clippy). The dead `module_name_repetitions` allow goes: the lint left pedantic in Rust 1.84.
2. **Clippy configuration** (`clippy.toml`): public items are linted like private ones (`avoid-breaking-exported-api = false`; every crate is `publish = false`), acronyms are one word, no glob import even from a prelude, and new bans on `catch_unwind`, `Box::leak`, `Vec::leak`, `String::leak`, and `ManuallyDrop`.
3. **Supply chain** (`deny.toml`): `ctor`, `inventory`, and `linkme` are banned, since no code runs before `main`.
4. **Build**: `[profile.release] overflow-checks = true`, and rustfmt writes hex literals in lowercase.
5. **xtask rules** (`xtask/src/gates/hygiene.rs`): no `assert!` outside tests; domain values keep private fields and fallible construction; no mutable statics in services; the application layer reaches files, the network, processes, and the environment only through ports; no `#[path]`; error messages lowercase without a trailing period; log values as fields; no glob re-exports (clippy exempts `pub use x::*` whatever its settings); no profile turns overflow checks or debug assertions off; no `RUSTFLAGS`, `CARGO_ENCODED_RUSTFLAGS`, or `RUSTC_WRAPPER` in the environment.
6. **Proof**: every new rule has a violation fixture, except three kinds that cannot: the unsafe lints (no crate may hold unsafe, and a fixture cannot both use the workspace lints and allow it), the `deny.toml` bans (cargo-deny is not an enforcer `cargo xtask rules` runs), and the environment check (a fixture carries no environment; a unit test covers it).
7. **Review-only advice** goes into the skills: ownership over `Rc`/`Arc` and over a placating `.clone()`, an enum for an unclear bool parameter, named fields over tuples, `Path` for paths, secrets redacted in `Debug`, named constants for meaningful numbers, no test that cannot fail, and `#[expect(clippy::expect_used, reason)]` as the one sanctioned way to state an impossible state.

## Consequences

- Positive: the rules now trace to published guidance from organisations that run Rust at scale, and each new one is proven to fire.
- Negative: more for an agent to satisfy on its first try; a `Debug` derive on every public type; a type holding a secret needs a hand-written `Debug`.
- Neutral: the unsafe lints do nothing until the one crate allowed unsafe exists (AGENTS.md, "Server-only rules").

## Alternatives considered

Each was read and not adopted, for the reason given:

- **Warn instead of deny** (Microsoft, Chromium, Android): a warning an agent can ignore is not a rule (ADR-0004).
- **`#[non_exhaustive]` on public enums** (Google): it would force a wildcard arm in `interfaces` and break the exhaustive mapping of outcomes to status.
- **`missing_docs` and `# Errors`/`# Panics` sections everywhere** (Android, kernel, Microsoft): written for public libraries; in these layers it is noise against "comments say why".
- **`anyhow` in applications** (Microsoft allows it): typed errors per layer stay the rule (ADR-0011).
- **Inline `#[cfg(test)] mod tests`** (Fuchsia): sibling files keep the `refactor:` check possible.
- **`panic = "abort"`** (ANSSI suggests it): `platform-task` needs unwinding to report a panicking task.
- **Embark's lint list verbatim**: stale, with deprecated lints that would break the build.
- **The safety-critical drafts' bans on macros and procedural macros**: they would rule out `thiserror`, `tracing`, and `tokio`.
- **Left for later, each with its own ADR**: `max_width = 100` (Google, rust-lang), cargo-vet audits (Mozilla, ANSSI), a cargo-hack feature check, and an auditable, signed release binary.
