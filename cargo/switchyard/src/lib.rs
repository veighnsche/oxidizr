#![deny(unsafe_code)]
#![doc = include_str!("../README.md")]
/* ---- unwrap/expect policy ---- */
// Warn everywhere (incl. tests), but deny in non-test builds.
#![warn(clippy::unwrap_used, clippy::expect_used)]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]
/* ---- dev defaults: useful warnings, not blocking ---- */
#![warn(
    // Rustc groups
    future_incompatible,
    rust_2018_idioms,

    // API hygiene
    unreachable_pub,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,

    // Docs quality
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links
)]
// Clippy general & hardening (warn by default during dev)
#![warn(clippy::all, clippy::cargo, clippy::pedantic)]
#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::todo,
        clippy::unimplemented,
        clippy::unreachable,
        clippy::allow_attributes_without_reason,
        clippy::must_use_candidate,
        clippy::missing_const_for_fn,
        clippy::suspicious_open_options,
        clippy::uninlined_format_args,
        clippy::missing_errors_doc,
        clippy::doc_markdown,
        clippy::too_many_lines,
        clippy::ref_option,
        clippy::used_underscore_binding,
        clippy::manual_let_else,
        clippy::implicit_clone,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )
)]
#![allow(
    clippy::multiple_crate_versions,
    reason = "deps resolved by Cargo, not in our control"
)]
/* ---- PROD MODE: turn key warnings into hard errors ----
   Triggers when either:
   - feature "prod" is enabled, or
   - building in release (not(debug_assertions)) — optional, keep if you like.
*/
#![cfg_attr(
    all(not(test), any(feature = "prod", not(debug_assertions))),
    deny(
        // rustc / rustdoc
        future_incompatible,
        rust_2018_idioms,
        unreachable_pub,
        trivial_casts,
        trivial_numeric_casts,
        unused_import_braces,
        unused_qualifications,
        rustdoc::broken_intra_doc_links,
        rustdoc::private_intra_doc_links,
        missing_debug_implementations
    )
)]
#![cfg_attr(
    all(not(test), any(feature = "prod", not(debug_assertions))),
    deny(
        // clippy: panic sources & API hygiene become errors in prod
        clippy::panic,
        clippy::panic_in_result_fn,
        clippy::todo,
        clippy::unimplemented,
        clippy::dbg_macro,
        clippy::indexing_slicing,
        clippy::allow_attributes_without_reason,
        clippy::cast_lossless,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )
)]

//! Switchyard: safe, atomic, reversible filesystem swaps.
//!
//! Safety model highlights:
//! - All mutations follow a TOCTOU-safe sequence using directory handles (open parent `O_DIRECTORY|O_NOFOLLOW` → *at on final component → renameat → fsync(parent)).
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
