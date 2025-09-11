use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Policy {
    pub allow_roots: Vec<PathBuf>,
    pub forbid_paths: Vec<PathBuf>,
    pub strict_ownership: bool,
    pub force_untrusted_source: bool,
    pub force_restore_best_effort: bool,
    pub allow_degraded_fs: bool,
    pub disable_auto_rollback: bool,
    pub backup_tag: String,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            allow_roots: Vec::new(),
            forbid_paths: Vec::new(),
            strict_ownership: false,
            force_untrusted_source: false,
            force_restore_best_effort: false,
            allow_degraded_fs: false,
            disable_auto_rollback: false,
            backup_tag: "switchyard".to_string(),
        }
    }
}
