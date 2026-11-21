# 0015 - Dependencies of the desktop app

- **Status**: accepted
- **Date**: 2026-11-21

## Context

A Windows desktop app needs crates the base does not carry: bindings to Win32 and COM, a UI toolkit, a tray icon, file dialogs, and resources written into the executable. A dependency is code that runs with ours and a licence we accept (ADR-0008). `deny.toml` allows only permissive licences, and it is never loosened to let a crate in. The HTTP server and its crates are gone (ADR-0014).

## Decision

1. **These crates are accepted**, each for the reason and under the licence given:

   | Crate | For | Licence | Limits |
   | --- | --- | --- | --- |
   | `windows` | Win32 and COM: the keyboard hook, `SendInput`, UI Automation, the clipboard, the registry, the message window, the notification-area icon and its menu | MIT OR Apache-2.0 | `platform-windows` only; each `Win32_*` feature named |
   | `eframe` | opens the windows and runs egui in them | MIT OR Apache-2.0 | `default-features = false` with `glow`; never `default_fonts` |
   | `egui` | the UI the windows are drawn with | MIT OR Apache-2.0 | `default-features = false`; never `default_fonts` |
   | `rfd` | the open and save dialogs | MIT | |
   | `winresource` | writes the icon, version information, and manifest into the executable | MIT | a build dependency of the desktop root only |
   | `serde_json` | the preferences file (already in the workspace) | MIT OR Apache-2.0 | |
   | `cc` | compiles the comparison tool's C++ shim | MIT OR Apache-2.0 | the tool in `tools/` only, outside the workspace |

2. **The notification-area icon and its menu are written on Win32 in `platform-windows`**, through the `windows` crate: `Shell_NotifyIconW` shows and updates the icon, and a popup menu opens from it. No tray crate is taken.
3. **egui's bundled fonts are not used.** `default_fonts` brings fonts under the Ubuntu Font Licence and the SIL Open Font License, which `deny.toml` refuses; the windows load a system font that covers Vietnamese.
4. **Each crate enters the root `Cargo.toml` in the change that first uses it**, with an exact version, because cargo-shear refuses a workspace dependency that no member uses. The same holds for the `platform-windows` path entry.
5. **Each is credited in the README** when it enters.
6. **A transitive crate whose licence `deny.toml` refuses is replaced, or the feature that brings it is turned off.** `deny.toml` is not loosened.
7. **`axum`, `tower`, `tower-http`, and `http-body-util` leave the workspace** with the HTTP server.

## Consequences

- Positive: every crate the app needs is decided once, with its licence, before code depends on it. With these features, `cargo deny check licenses bans` passes for every target, as `deny.toml` judges them (checked on 2026-11-21 against `windows` 0.62.2, `eframe` and `egui` 0.37.0, `rfd` 0.18.0, and `winresource` 0.1.34).
- Negative: the tray icon and its menu are code of our own: more Win32 calls, each an `unsafe` block to document and review, with no shared crate behind them. eframe and egui are large and lengthen a clean build.
- Neutral: `cc` and anything else the comparison tool uses are outside the workspace, so the workspace's `cargo deny` does not check them; the tool's own manifest lists them.

## Alternatives considered

- **`windows-sys`**: raw bindings without COM interfaces; UI Automation is COM, so the `windows` crate.
- **Win32 dialogs written by hand** instead of a UI toolkit: more `unsafe` code and no layout.
- **`tray-icon`**: a tested tray crate under MIT OR Apache-2.0, but on Linux and macOS it depends on `dirs`, whose `option-ext` is MPL-2.0, which `deny.toml` refuses. `cargo deny` judges every target and `deny.toml` is not loosened, and making the crate a Windows-only dependency does not change what is judged.
- **Slint**: its royalty-free licence is not on `deny.toml`'s list.
- **egui's bundled fonts**: refused licences, as decision 3 says.
