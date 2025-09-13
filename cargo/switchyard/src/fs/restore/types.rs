//! Types for restore subsystem

#[derive(Clone, Copy, Debug, Default)]
pub struct RestoreStats {
    pub fsync_ms: u64,
}

/// Which snapshot pair to select for restore
#[derive(Clone, Copy, Debug)]
pub enum SnapshotSel {
    Latest,
    Previous,
}

/// Options for restore behavior
#[derive(Clone, Debug)]
pub struct RestoreOptions {
    pub dry_run: bool,
    pub force_best_effort: bool,
    pub backup_tag: String,
}

/// Prior state kind as encoded in sidecar
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PriorKind {
    File,
    Symlink,
    None,
    Other(String),
}

impl PriorKind {
    pub fn from_str(s: &str) -> Self {
        match s {
            "file" => PriorKind::File,
            "symlink" => PriorKind::Symlink,
            "none" => PriorKind::None,
            other => PriorKind::Other(other.to_string()),
        }
    }
}
