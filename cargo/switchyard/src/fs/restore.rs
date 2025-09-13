/// Thin wrapper delegating to the new directory module implementation.

#[path = "restore/engine.rs"]
mod engine;
pub use engine::*;
