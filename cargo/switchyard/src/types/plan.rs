use super::safepath::SafePath;

#[derive(Debug, Copy, Clone, Default)]
pub enum ApplyMode {
    #[default]
    DryRun,
    Commit,
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

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    EnsureSymlink { source: SafePath, target: SafePath },
    RestoreFromBackup { target: SafePath },
}

#[derive(Clone, Debug, Default)]
pub struct Plan {
    pub actions: Vec<Action>,
}
