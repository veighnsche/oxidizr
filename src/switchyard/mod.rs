//! Switchyard internal module (phase 1 - scaffold)
//!
//! This internal module provides a minimal, compiling façade API for the
//! switchyard safety package. In subsequent steps, existing symlink and
//! filesystem safety code will be migrated behind this façade.

pub mod api;
pub mod adapters;

// Re-export primary façade types for convenience during migration.
pub use api::*;
