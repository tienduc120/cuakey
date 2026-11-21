//! This service over HTTP. Only this layer maps an outcome to a status code (ADR-0011).
//!
//! server-only: in a desktop app this layer maps each outcome to UI state instead, still with an exhaustive `match`
//! (AGENTS.md, "Server-only rules").

mod rest;

pub use rest::router;
