use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub(crate) struct PerfAgg {
    pub hash_ms: u64,
    pub backup_ms: u64,
    pub swap_ms: u64,
}
