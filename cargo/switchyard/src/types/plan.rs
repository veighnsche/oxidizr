use super::safepath::SafePath;

#[derive(Clone, Debug)]
pub enum ApplyMode {
    DryRun,
    Commit,
}

impl Default for ApplyMode {
    fn default() -> Self { ApplyMode::DryRun }
}

#[derive(Clone, Debug)]
pub struct LinkRequest {
    pub source: SafePath,
    pub target: SafePath,
}

#[derive(Clone, Debug)]
pub struct RestoreRequest {
    pub target: SafePath,
}

#[derive(Clone, Debug, Default)]
pub struct PlanInput {
    pub link: Vec<LinkRequest>,
    pub restore: Vec<RestoreRequest>,
}

#[derive(Clone, Debug)]
pub enum Action {
    EnsureSymlink { source: SafePath, target: SafePath },
    RestoreFromBackup { target: SafePath },
}

#[derive(Clone, Debug, Default)]
pub struct Plan {
    pub actions: Vec<Action>,
}
