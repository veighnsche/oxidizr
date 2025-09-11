use super::plan::Action;

#[derive(Clone, Debug, Default)]
pub struct PreflightReport {
    pub ok: bool,
    pub warnings: Vec<String>,
    pub stops: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct ApplyReport {
    pub executed: Vec<Action>,
    pub duration_ms: u64,
    pub errors: Vec<String>,
}
