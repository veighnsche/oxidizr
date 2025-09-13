#![forbid(unsafe_code)]

// Keep your strict stance on unwrap/expect outside tests.
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

// Broad, stable Rust warnings that catch future breakage & API footguns.
#![warn(
    // Rustc groups
    future_incompatible,
    rust_2018_idioms,

    // API hygiene
    unreachable_pub,                 // accidental public API
    missing_debug_implementations,   // make types debuggable
    missing_copy_implementations,    // highlight trivial Copy candidates
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,

    // Docs quality
    missing_docs,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links
)]

// Clippy: general quality + cargo + pedantic (you already had these)
#![warn(clippy::all, clippy::cargo, clippy::pedantic)]

// Clippy: production hardening (set to warn; you can dial up later)
#![warn(
    // Panic sources & placeholders
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::todo,
    clippy::unimplemented,
    clippy::dbg_macro,

    // Indexing safety (prefer get()/get_mut()/slice::get)
    clippy::indexing_slicing,

    // Error/documentation hygiene
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::missing_const_for_fn,

    // API & style hygiene
    clippy::wildcard_imports,
    clippy::wildcard_enum_match_arm,
    clippy::allow_attributes_without_reason,

    // Numeric safety (casts & loss)
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
//! Switchyard: safe, atomic, reversible filesystem swaps.
//!
//! Safety model highlights:
//! - All mutations follow a TOCTOU-safe sequence using directory handles (open parent O_DIRECTORY|O_NOFOLLOW → *at on final component → renameat → fsync(parent)).
//! - Public mutating APIs operate on `SafePath` only; internal FS code uses capability-style directory handles.
//! - This crate forbids `unsafe` and uses `rustix` for syscalls.
//!
//! Quickstart (builder is the default way to construct the API):
//! ```rust
//! use switchyard::api::Switchyard;
//! use switchyard::logging::JsonlSink;
//! use switchyard::policy::Policy;
//! let facts = JsonlSink::default();
//! let audit = JsonlSink::default();
//! let _api = Switchyard::builder(facts, audit, Policy::default()).build();
//! ```

pub mod adapters;
pub mod api;
pub mod constants;
pub mod fs;
pub mod logging;
pub mod policy;
pub mod preflight;
pub mod types;

pub use api::*;
