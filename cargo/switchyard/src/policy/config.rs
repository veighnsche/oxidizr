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
    /// Require a LockManager to be present in Commit mode. When true and no lock manager is
    /// configured, apply() MUST fail early with E_LOCKING (exit code 30) without mutating state.
    pub require_lock_manager: bool,
    /// In Commit mode, require that a SmokeTestRunner is configured and passes. When true and
    /// no runner is configured, apply() MUST fail with E_SMOKE and auto-rollback unless disabled.
    pub require_smoke_in_commit: bool,
    /// When verifying rescue profile/tooling, also attempt an executability check (e.g., X_OK
    /// or spawning "--help" with a very small timeout). Typically enabled in production only.
    pub rescue_exec_check: bool,
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
            require_lock_manager: false,
            require_smoke_in_commit: false,
            rescue_exec_check: false,
        }
    }
}
