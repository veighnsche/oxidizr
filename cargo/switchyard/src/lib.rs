#![forbid(unsafe_code)]
//! Switchyard: safe, atomic, reversible filesystem swaps.
//!
//! Safety model highlights:
//! - All mutations follow a TOCTOU-safe sequence using directory handles (open parent O_DIRECTORY|O_NOFOLLOW → *at on final component → renameat → fsync(parent)).
//! - Public mutating APIs operate on `SafePath` only; internal FS code uses capability-style directory handles.
//! - This crate forbids `unsafe` and uses `rustix` for syscalls.

pub mod constants;
pub mod adapters;
pub mod api;
pub mod fs;
pub mod logging;
pub mod policy;
pub mod preflight;
pub mod rescue;
pub mod types;

pub use api::*;
