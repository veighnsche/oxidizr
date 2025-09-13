//! Data-only rescue status and error types used across the crate.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RescueStatus {
    BusyBox { path: String },
    GNU { found: usize, min: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RescueError {
    Unavailable,
}
