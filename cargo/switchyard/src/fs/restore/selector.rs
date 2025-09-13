use std::path::{Path, PathBuf};

/// Return (backup_path_if_present, sidecar_path) for the latest timestamped pair.
pub fn latest(target: &Path, tag: &str) -> Option<(Option<PathBuf>, PathBuf)> {
    crate::fs::backup::find_latest_backup_and_sidecar(target, tag)
}

/// Return (backup_path_if_present, sidecar_path) for the previous (second newest) pair.
pub fn previous(target: &Path, tag: &str) -> Option<(Option<PathBuf>, PathBuf)> {
    crate::fs::backup::find_previous_backup_and_sidecar(target, tag)
}
