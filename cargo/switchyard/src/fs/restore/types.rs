//! Types for restore subsystem

#[derive(Clone, Copy, Debug, Default)]
pub struct RestoreStats {
    pub fsync_ms: u64,
}
