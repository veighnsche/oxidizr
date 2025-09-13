//! Preflight checks and helpers.
//!
//! This module provides best-effort filesystem and policy gating checks used by the
//! higher-level API. It also exposes a small helper to render a `PreflightReport`
//! into a SPEC-aligned YAML sequence for fixtures and artifacts.

pub mod checks;
pub mod yaml;

// Re-export common helpers for convenience
pub use checks::{check_immutable, check_source_trust, ensure_mount_rw_exec};
pub use yaml::to_yaml;
