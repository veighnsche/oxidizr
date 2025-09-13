//! Filesystem primitives used by Switchyard.
//!
//! This module provides low-level, TOCTOU-safe filesystem operations used by the
//! higher-level API stages. Consumers should prefer calling public API stages in
//! `switchyard::api` rather than these atoms. Low-level atoms are crate-private
//! and are not re-exported at the module root.

pub mod atomic;
pub mod backup;
pub mod meta;
pub mod mount;
pub mod paths;
pub mod restore;
pub mod swap;
