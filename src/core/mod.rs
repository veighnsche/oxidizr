//! Placeholder module for potential per-command abstractions.
//! For oxidizr-like switching, prefer using `worker` + `experiment` modules.

/// Marker enum for future extensions.
#[allow(dead_code)]
pub enum CoreUtilsImpl {
    Gnu,
    Uutils,
}

/// Minimal trait kept for backward compatibility with earlier scaffolding.
#[allow(dead_code)]
pub trait CoreUtil: Send + Sync {
    fn name(&self) -> &'static str;
}
