use std::path::{Path, PathBuf};
use crate::fs::backup::index::{find_latest_backup_and_sidecar, find_previous_backup_and_sidecar};

/// Return (`backup_path_if_present`, `sidecar_path`) for the latest timestamped pair.
#[must_use]
pub fn latest(target: &Path, tag: &str) -> Option<(Option<PathBuf>, PathBuf)> {
    find_latest_backup_and_sidecar(target, tag)
}

/// Return (`backup_path_if_present`, `sidecar_path`) for the previous (second newest) pair.
#[must_use]
pub fn previous(target: &Path, tag: &str) -> Option<(Option<PathBuf>, PathBuf)> {
    find_previous_backup_and_sidecar(target, tag)
}
