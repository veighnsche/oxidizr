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
    /// When true, apply() proceeds even if preflight has policy_ok=false rows.
    /// Default is false (fail-closed). See PLAN/45-preflight.md.
    pub override_preflight: bool,
    /// Require filesystem preservation capabilities (owner/mode/timestamps/xattrs/acl/caps).
    /// When true and the target path lacks support, preflight MUST STOP (unless override_preflight).
    pub require_preservation: bool,
    /// Require a rescue profile and toolset to be available (e.g., BusyBox or GNU core tools on PATH).
    /// When true and unavailable, preflight/apply MUST STOP (unless override_preflight).
    pub require_rescue: bool,
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
            override_preflight: false,
            require_preservation: false,
            require_rescue: false,
        }
    }
}
