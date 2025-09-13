use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub(crate) struct PerfAgg {
    pub hash: u64,
    pub backup: u64,
    pub swap: u64,
}
