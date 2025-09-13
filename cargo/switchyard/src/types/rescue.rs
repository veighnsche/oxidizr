//! Data-only rescue status and error types used across the crate.

/// Status types for rescue operations.
/// Centralized under `crate::types` for cross-layer reuse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RescueStatus {
    /// BusyBox rescue status with path information
    BusyBox { path: String },
    /// GNU rescue status with found and minimum counts
    GNU { found: usize, min: usize },
}

/// Error types for rescue operations.
/// Centralized under `crate::types` for cross-layer reuse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RescueError {
    /// Rescue functionality is unavailable
    Unavailable,
}
