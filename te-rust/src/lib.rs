//! Core library for `te-rust`, a clean-room reimplementation of `trec_eval`.
//!
//! This crate exposes the evaluation engine (parsing, alignment, and metric
//! computation) so it can be consumed by the CLI binary and, in the future, by
//! Python bindings. See `docs/` for the design.

pub mod io;
pub mod eval;
pub mod metrics;
