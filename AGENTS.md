# CuaKey

CuaKey is a Vietnamese typing app for Windows, written in Rust and built on these rules. It has one service, `services/cuakey`.

The decisions behind it: [docs/adr/](docs/adr/README.md). What is deliberately not built yet, and what triggers it: [docs/deferred.md](docs/deferred.md). Do not report a rule as enforced unless a tool in this repo checks it.

## How to work

- **Red, then green.** Write the failing test, run that crate's tests and watch it go red for the right reason, make it pass, run them again: `cargo nextest run -p <crate>`. Between steps that is the only check you run.
- **The hooks run the gates.** The pre-commit hook runs `cargo xtask verify`, the pre-push hook and CI run `--full`, and a rejected commit names exactly what to fix. Run `verify` by hand only to read a failure in full (the logs are in `.xtask/logs`).
- **Write Rust-shaped code**, idiomatic to Rust rather than ported line by line from another language. For an idiom you are unsure of, read the [Microsoft Pragmatic Rust Guidelines](https://microsoft.github.io/rust-guidelines/agents/all.txt).

## Commands

Run from the repository root. There is no Make; `cargo xtask` is the command surface.

| Need | Command |
| --- | --- |
| Tests (the TDD loop) | `cargo nextest run -p <crate>` (never `cargo test`: it has no per-test timeout; nextest kills a test after two minutes) |
| Run the gates (hooks and CI do this) | `cargo xtask verify`; `--full` adds cargo-deny, unused dependencies, docs, and the rules proof |
| Prove every rule fires on its fixture | `cargo xtask rules` |
| Check the toolchain and pinned tools | `cargo xtask doctor` (prints the install command for anything missing) |
| Install the git hooks (once per clone) | `lefthook install` |
| Start a new service | `cargo xtask new-service <name> --aggregate <Name>` (skill: `new-service`) |
| Check a commit message | `cargo xtask commitmsg <file>` (the commit-msg hook calls it) |
| Format | `cargo fmt --all` |
| Make the skills visible to Claude Code (once per clone) | Windows: `cmd /c mklink /J .claude\skills .agents\skills`; Unix: `ln -s ../.agents/skills .claude/skills` |

## Hard rules

- **The typing engine is written from scratch.** Never read or copy OpenKey's source; compare behaviour only through `cargo xtask compare-openkey`.
- **Test first.** No production code without a test that failed first, for the right reason. Unit tests live in a sibling file (`#[cfg(test)] mod tests;` next to `<module>/tests.rs`), never inline. Follow the `writing-rust-tests` skill.
- **Start a service with the generator.** `cargo xtask new-service` copies the witness `services/example` under a new name, so it starts green. Change `services/example` only when every future service should change, and keep it passing every gate.
- **Errors are typed values owned by the layer that raises them.** `Result<T, E>` with a `thiserror` enum per operation; never panic for an expected failure; keep the cause as `#[source]`; only `interfaces` maps an outcome to a status, with an exhaustive `match`. `anyhow` is for the tool only. Follow the `handling-errors` skill (ADR-0011).
- **Concurrency only through `platform-task`, serving only through `platform-http`, time only through `platform-clock`.** clippy refuses `tokio::spawn`, `std::thread::spawn`, `std::thread::sleep`, `axum::serve`, `SystemTime::now`, `CancellationToken::new`, `std::env::var`, and `std::process::exit` everywhere else (ADR-0009).
- **Suppress a lint only with `#[expect(lint, reason = "...")]` on the one item that needs it.** `#[allow]` and crate-wide `#![allow]` are refused. A suppression is debt: say so to the user.
- **A new dependency is a decision, not an edit.** Versions live once in the root `Cargo.toml` (`[workspace.dependencies]`), which is protected; a member writes `{ workspace = true }`. Propose it with an ADR, and credit it in the README.
- **Never weaken a gate to make a change pass.** `Cargo.toml`, `clippy.toml`, `architecture.toml`, `deny.toml`, the other root configs, `xtask/src/gates`, and `fixtures/` are the rules that judge you: change them only with the user's explicit approval. Fix the cause instead.
- **Every rule has a fixture.** A new or changed rule gets a violation fixture in `fixtures/violations/<id>/` with a `rule.toml` naming its enforcer (`clippy` or `gates`) and the text it must report, and `cargo xtask rules` must stay green. The clean twin in `fixtures/clean/` must stay free of findings.
- **Never bypass git hooks** (`--no-verify`, `LEFTHOOK=0`). `lefthook.yml` runs `verify` before each commit, `commitmsg` on the message, and `verify --full` before a push.
- **A `refactor:` commit may only add tests**, never change an existing test line (the commit-msg hook enforces it): a changed expectation means behaviour changed, so it is a `fix:` or `feat:`.
- **Evidence, not claims.** "Tests pass" means you ran them and saw the result. Name every step you skipped and why.
- **Comments say why**, never what, and stay short. Conventional Commits: `type(scope): subject`, imperative, lowercase, at most 72 characters.
- **Credit what you reference.** A design, config, or code from a public source goes in the README "Credits" in the same change; copied or adapted material also needs its license and a `NOTICE.md` entry. Only MIT, BSD, and Apache-2.0 material comes in.
- **Do not commit `.local/`** (gitignored notes) and do not install software system-wide without asking.

## Architecture

One bounded context per service under `services/<name>/`, one crate per layer. A dependency that crosses a layer the wrong way is a missing Cargo edge, so it does not compile; `cargo xtask verify` also checks every edge against `architecture.toml`:

| Crate | May depend on | External crates |
| --- | --- | --- |
| `<svc>-domain` | nothing of ours | only `thiserror`, `jiff`; the crate is `#![no_std]`, so the compiler keeps I/O, threads, env, and the clock out (ADR-0006) |
| `<svc>-application` | its domain, `platform-clock` | only `thiserror`, `tracing`, `jiff` |
| `<svc>-infrastructure` | its domain and application, any platform | not `axum`, `tower-http`, `hyper`, `anyhow` |
| `<svc>-interfaces` | its application, any platform | not `sqlx`, `anyhow` |
| `<svc>-api` | its own layers, any platform | not `anyhow` |
| `platform-<name>` | other platform crates, never a service | not `anyhow` |

- A service never depends on another service's crates. Nothing depends on `xtask`.
- Ports are traits declared in `application`, with an explicit `-> impl Future<Output = ...> + Send`. Adapters are chosen in the composition root; use cases are generic over their ports (ADR-0007).
- An aggregate root carries `agent:aggregate` in its doc comment, has a field `id` of a type its crate defines, `new` (checks invariants), and `restore` (rebuilds a stored value). Only infrastructure calls `restore` outside tests (ADR-0007).
- A use case takes a command of plain values and returns its own result and error types; delivery never handles a domain type. Fakes of a crate's ports live in its `testing` module behind the `test-support` feature, so other layers' tests can use them.

Enforced, each with a fixture:

- **Layer rules** (`cargo xtask verify`, `architecture.toml`): the table above, `#![no_std]` in the domain, and every crate under `services/<svc>/<layer>`, `platform/<name>`, or `xtask`.
- **Manifest policy**: `[lints] workspace = true`, package fields inherited from the workspace, the package named after where it lives, external versions only from the workspace.
- **Source rules** (syn): aggregate shape, `restore` only in infrastructure, no `extern crate std` in a `no_std` crate, no inline test modules, no crate-wide `#![allow]`.
- **Rules from published guides** (`cargo xtask verify`, ADR-0013): no `assert!` outside tests (`debug_assert!` is fine); domain values with private fields and fallible construction (no `pub` field, `Deref`, or `From<primitive>`); no mutable statics in services; no direct file, network, process, or environment access in the application layer; no `#[path]`; error messages lowercase with no trailing period; log values as fields, not inside the message; no glob re-exports; overflow checks on in release and never turned off; no build flags from the environment (`RUSTFLAGS`, `RUSTC_WRAPPER`).
- **Lints** (`Cargo.toml`, `clippy.toml`): clippy `all` and `pedantic` at deny, plus picked restriction lints: no `unwrap`/`expect`/`panic`/`todo`/indexing outside tests, no `println!`, no `mod.rs`, no wildcard arm on an enum, no `#[allow]`, no `unsafe`, functions under 80 lines, no lock guard across an `.await`, every public type implements `Debug`, no `.ok();` that drops an error, no `catch_unwind`, no `Box::leak` or `ManuallyDrop`, no `as _`, public items linted like private ones, and the bans listed under Hard rules.
- **Supply chain** (`deny.toml`, in `verify --full`): permissive licenses only, advisories, crates.io only, and banned crates (`anyhow` outside the tool, `async-trait`, `lazy_static`, `openssl`, deprecated YAML crates, and `ctor`/`inventory`/`linkme`, since no code runs before `main`).
- **Tests**: nextest fails a test after two minutes and fails one that leaks a child process.

## Server-only rules

This base targets backend services, and a few rules exist only for that. Each carries a `server-only` comment where it lives. When the project is a desktop app (Windows or otherwise), replace exactly these, with an ADR, and keep every other rule:

| Rule | Where | For a desktop app |
| --- | --- | --- |
| `platform-http` and the `axum::serve` ban | `platform/http`, `clippy.toml` | delete both: there is no server |
| The `api` kind, an HTTP composition root | `architecture.toml`, `services/example/api` | a desktop root kind instead: it opens the window and runs the message loop, with `#![windows_subsystem = "windows"]` in `main.rs` |
| Web crates denied in infrastructure | `architecture.toml` (`axum`, `tower-http`, `hyper`) | drop the line |
| `interfaces` maps outcomes to HTTP status | `services/example/interfaces` | map them to UI state and messages; keep the exhaustive `match` |
| `unsafe_code = "forbid"` everywhere | `Cargo.toml` | Win32 and COM calls are `unsafe`: allow it in one `platform-windows` crate only, with every block documented (`undocumented_unsafe_blocks`) |
| `platform-task` on tokio, and the spawn bans | `platform/task`, `clippy.toml` | keep them for background work; the UI toolkit's own event loop runs on the main thread, started from the desktop root |
| CI on Linux only | `.github/workflows/ci.yml` | add a `windows-latest` job |

## Agent guard

In Claude Code, `.claude/settings.json` runs `cargo xtask hook` before every edit and shell command.

It asks the user, in Claude Code's permission prompt, before:

- changing a committed rule file (the root configs, `.github/`, `.claude/settings.json`, `xtask/src/gates`, `fixtures/`, `LICENSE`), by an edit or through the shell. New files are always allowed, so adding a fixture or a gate is fine; protection starts once a file is committed.

It refuses outright, with a reason you can act on:

- skipping the git hooks: `--no-verify`, `git commit -n`, `LEFTHOOK=0`, `core.hooksPath`, `lefthook uninstall`.
- commands that never return or move too much at once: `cargo test` (use nextest), `cargo run` of an `-api` or `-worker` binary in the foreground, `cargo watch`, a bare `cargo update`, `docker compose up` without `-d`, following logs, `tail -f`, `Get-Content -Wait`.

The prompt is used only in the permission modes where Claude Code documents that it reaches the user (default, plan, auto). In `acceptEdits`, `bypassPermissions`, and `dontAsk` a protected change is refused instead; there, or for a session of many rule changes, the user starts the session with `XTASK_ALLOW_PROTECTED=1`, which lifts only the protected-file rule (ADR-0012).

When you are refused, fix the cause; do not look for another way to make the same change. Change a rule only when the user asked for it. The guard is a guard rail, not a security boundary: `CODEOWNERS`, the git hooks, and CI are the boundary, and other agents do not run these hooks (ADR-0005).

## Conventions not enforced by a tool

- Markdown: one paragraph or list item per line; never hard-wrap prose.
- A name says what the thing is for in the domain's words; a function does one thing at one level.

## Things that will surprise you

Grow this by correction: when any tool gets something wrong, add the line.

- **The guard judges paths from the session's directory.** `cd elsewhere && rm Cargo.toml` is judged as if `Cargo.toml` were the repository's, which blocks more, never less. Shell writes to a protected path are refused even before it is committed; write a gate you are still building with the edit tools.
- **Every public type must implement `Debug`** (`missing_debug_implementations`). A type that holds a secret writes its `Debug` by hand and leaves the secret out.
- **clippy never flags `pub use x::*`**, whatever its settings; the `glob-reexport` gate does.
- **clippy reports the resolved path of a banned call** (`tokio::task::spawn`, not `tokio::spawn`). A fixture's expected text must name the resolved path.
- **An `async fn` impl with nothing to await is refused** (`unused_async_trait_impl`). Return `std::future::ready(..)` from a plain `fn` instead.
- **A test in one layer cannot name another layer's types it may not depend on.** Use the fakes from that layer's `testing` module (`features = ["test-support"]` on the dev-dependency).
- **`doc_markdown` wants backticks** around identifiers in doc comments, such as `no_std` or `CreateNote`.
- **Cargo treats a `members` glob that matches nothing as a literal path** and fails to load the workspace.
- **On Windows, a tool installed mid-session is not on the shell's PATH.** In PowerShell, reload it: `$env:Path = [Environment]::GetEnvironmentVariable('Path','User') + ';' + [Environment]::GetEnvironmentVariable('Path','Machine')`. In Git Bash use `/c/Users/...` paths, not `$USERPROFILE`.
- **`XTASK_ALLOW_PROTECTED=1` must reach the process that starts Claude Code.** In VS Code, close every window first: `code` run while VS Code is open hands the folder to the running instance, and the variable never arrives. Check it in the session (`$env:XTASK_ALLOW_PROTECTED` prints `1`) before relying on it.
