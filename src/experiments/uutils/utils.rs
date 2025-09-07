use crate::utils::worker::Worker;
use std::path::{Path, PathBuf};

/// Resolves the target path for a given filename, using relative paths in test mode.
pub fn resolve_target<W: Worker>(worker: &W, filename: &str) -> PathBuf {
    if let Ok(Some(found)) = worker.which(filename) {
        return found;
    }
    if cfg!(test) {
        Path::new("bin").join(filename)
    } else {
        Path::new("/usr/bin").join(filename)
    }
}
