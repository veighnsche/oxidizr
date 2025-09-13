use super::plan::Action;
use uuid::Uuid;

// Typed representation of a preflight report.
/// Centralized under `crate::types` for cross-layer reuse.
#[must_use]
#[derive(Clone, Debug, Default)]
pub struct PreflightReport {
    /// Overall status of the preflight check
    pub ok: bool,
    /// List of warnings encountered during preflight
    pub warnings: Vec<String>,
    /// List of reasons why the operation should stop
    pub stops: Vec<String>,
    /// List of detailed diff rows for the preflight report
    pub rows: Vec<serde_json::Value>,
}

// Typed representation of an apply report.
/// Centralized under `crate::types` for cross-layer reuse.
#[must_use]
#[derive(Clone, Debug, Default)]
pub struct ApplyReport {
    /// List of actions that were executed
    pub executed: Vec<Action>,
    /// Duration of the apply operation in milliseconds
    pub duration_ms: u64,
    /// List of errors encountered during apply
    pub errors: Vec<String>,
    /// UUID of the plan that was applied
    pub plan_uuid: Option<Uuid>,
    /// Whether the operation was rolled back
    pub rolled_back: bool,
    /// List of errors encountered during rollback
    pub rollback_errors: Vec<String>,
}

/// Typed representation of a prune result.
/// Centralized under `crate::types` for cross-layer reuse.
#[must_use]
#[derive(Clone, Copy, Debug, Default)]
pub struct PruneResult {
    /// Number of items that were pruned
    pub pruned_count: usize,
    /// Number of items that were retained
    pub retained_count: usize,
}
