use super::plan::Action;
use uuid::Uuid;

#[must_use]
#[derive(Clone, Debug, Default)]
pub struct PreflightReport {
    pub ok: bool,
    pub warnings: Vec<String>,
    pub stops: Vec<String>,
    pub rows: Vec<serde_json::Value>,
}

#[must_use]
#[derive(Clone, Debug, Default)]
pub struct ApplyReport {
    pub executed: Vec<Action>,
    pub duration_ms: u64,
    pub errors: Vec<String>,
    pub plan_uuid: Option<Uuid>,
    pub rolled_back: bool,
    pub rollback_errors: Vec<String>,
}

#[must_use]
#[derive(Clone, Debug, Default)]
pub struct PruneResult {
    pub pruned_count: usize,
    pub retained_count: usize,
}
