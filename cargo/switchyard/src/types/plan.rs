use super::safepath::SafePath;

/// Mode for executing an apply plan.
///
/// - `DryRun`: perform analysis and emit facts with redaction; do not mutate.
/// - `Commit`: perform mutations, emit full facts, and run optional smoke checks.
#[derive(Debug, Copy, Clone, Default)]
pub enum ApplyMode {
    #[default]
    DryRun,
    Commit,
}

/// Request to ensure a symlink from `source` to `target`.
#[derive(Clone, Debug)]
pub struct LinkRequest {
    pub source: SafePath,
    pub target: SafePath,
}

/// Request to restore a target from previously captured backups.
#[derive(Clone, Debug)]
pub struct RestoreRequest {
    pub target: SafePath,
}

/// Input for planning. Combine link and restore requests into a plan.
#[derive(Clone, Debug, Default)]
pub struct PlanInput {
    pub link: Vec<LinkRequest>,
    pub restore: Vec<RestoreRequest>,
}

/// Concrete actions the engine can execute.
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    EnsureSymlink { source: SafePath, target: SafePath },
    RestoreFromBackup { target: SafePath },
}

/// Planned sequence of actions with stable ordering.
#[derive(Clone, Debug, Default)]
pub struct Plan {
    pub actions: Vec<Action>,
}
