# 0014 - A Windows desktop app, not a server

- **Status**: accepted
- **Date**: 2026-11-21

## Context

CuaKey adopted a base written for backend services. A few of its rules exist only for a server: an HTTP server crate and the ban that sends serving through it, an HTTP composition root, web crates denied in infrastructure, outcomes mapped to HTTP statuses, `unsafe` forbidden everywhere, and CI on Linux only. CuaKey is a Windows desktop app: a background process with a keyboard hook and a tray icon, and windows drawn by a UI toolkit. Win32 and COM calls are `unsafe`, a Win32 message loop must own its thread, and the program ships for Windows. Every other rule of the base fits it unchanged. CuaKey also checks its typing against another program's with a tool that is its own Cargo project.

## Decision

1. **There is no HTTP server.** `platform-http`, the clippy ban on `axum::serve`, and its fixture are deleted.
2. **The composition root kind is `desktop`, not `api`** (`architecture.toml`). `services/<svc>/desktop` may depend on its own service's layers and on any platform crate, and nothing depends on it (fixture `arch-desktop-root`). It wires the adapters and starts the UI toolkit's event loop on the main thread; its `main.rs` starts with `#![windows_subsystem = "windows"]`, so the app opens no console window. The witness `services/example/desktop` wires the in-memory adapter and runs one use case without a window, so the generator copies a root that builds and passes every gate.
3. **Infrastructure no longer denies the web crates** `axum`, `tower-http`, and `hyper`; `anyhow` stays denied.
4. **`interfaces` maps each outcome to UI state**: the message a window shows, decided in an exhaustive `match` (the rule of ADR-0011 decision 4, with a message in place of a status). An internal failure shows a generic message and logs its cause.
5. **`unsafe_code` is `deny`, and only `platform-windows` may lower it.** That crate says so once, at its crate root, with `#![expect(unsafe_code, reason = "...")]`, written together with its first `unsafe` block, because an expectation with nothing to meet fails the build. The source rule `unsafe-outside-platform-windows` refuses an `allow`, `expect`, or `warn` of `unsafe_code` (also behind `cfg_attr`) in any other crate, so an `unsafe` block anywhere else stays a compile error (fixtures `unsafe-code` and `unsafe-outside-platform-windows`). `undocumented_unsafe_blocks`, `multiple_unsafe_ops_per_block`, `unnecessary_safety_comment`, and `unnecessary_safety_doc` stay at deny and now bind that crate: every block holds one unsafe operation and a safety comment that says why it is sound. The fixture `undocumented-unsafe` proves the first of them, and the clean twin holds a documented block in a `platform-windows` crate.
6. **`platform-task` stays the only place concurrency starts, and gains `spawn_thread`** for work that must own an OS thread, such as a Win32 message loop. It names the thread, and `ThreadHandle::join` returns a panic as `TaskError::Panicked`. The spawn bans in `clippy.toml` stay and name it; the UI toolkit's own event loop runs on the main thread, started from the desktop root.
7. **CI runs `cargo xtask verify --full` on `windows-latest`** in a job of its own, next to the Linux job.
8. **`tools/` is outside the workspace**, like `fixtures/`: a tool there is its own Cargo project, and the gates do not judge it. `cargo xtask compare-openkey` is the command that runs the comparison tool there.

## Consequences

- Positive: the rules fit the program being built, each change is proven by a fixture or a test, and every other rule of the base still applies as written.
- Negative: an `unsafe` block is a compile error everywhere but one crate, so Windows code that needs one moves into `platform-windows` even when it serves a single service. The Windows job roughly doubles CI time. The generator now produces a desktop root; a server would need a record of its own to bring the `api` kind back.
- Neutral: `#![windows_subsystem]` is ignored off Windows, so the Linux job builds and tests the same crates. A desktop root has no console, so whatever it logs goes to a file or nowhere.

## Alternatives considered

- **A lint table of its own for `platform-windows`**: Cargo refuses to mix `workspace = true` with overrides (ADR-0004), and the manifest policy requires `workspace = true`.
- **`#![allow(unsafe_code)]` in `platform-windows`, exempted from the inner-allow rule**: present before the first `unsafe` block, but an allow never goes stale, and it would be the one crate-wide allow the gates accept.
- **`#[expect(unsafe_code)]` on each item that holds unsafe code**: the same gate would apply, but every block would carry a second attribute that says what the crate root already says.
- **One CI job with an operating-system matrix**: one definition for both systems, but it renames the existing `verify` check that branch protection may require.
