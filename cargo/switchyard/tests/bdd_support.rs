// Thin shim to re-export helpers from directory-based module at bdd_support/mod.rs
// This avoids module path conflicts while preserving the existing public API.

#[path = "bdd_support/mod.rs"]
mod bdd_support_impl;

pub use bdd_support_impl::*;
