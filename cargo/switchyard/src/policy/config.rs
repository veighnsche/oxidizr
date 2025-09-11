use std::path::PathBuf;
use crate::constants::DEFAULT_BACKUP_TAG;

/// Policy governs preflight gates, apply behavior, and production hardening for Switchyard.
///
/// Highlights:
/// - Production commits should enable: `require_lock_manager`, `require_smoke_in_commit`,
///   and `require_rescue` (with `rescue_exec_check`) for safe rollouts.
/// - Use `allow_degraded_fs` to permit EXDEV degraded fallback (unlink+symlink) on cross-FS symlink
///   swaps; set to `false` for critical system paths to fail closed on `E_EXDEV`.
/// - Scope mutations via `allow_roots` and block sensitive paths with `forbid_paths`.
/// - See presets: [`Policy::production_preset`], [`Policy::coreutils_switch_preset`].
#[derive(Clone, Debug)]
pub struct Policy {
    /// Paths under which mutations are allowed. Targets must start with one of these roots
    /// to pass preflight/apply gating. See checks in `src/api/preflight.rs` and `src/api/apply.rs`.
    pub allow_roots: Vec<PathBuf>,
    /// Paths that are explicitly forbidden. If a target starts with any of these prefixes,
    /// preflight/apply will STOP. See checks in `src/api/preflight.rs` and `src/api/apply.rs`.
    pub forbid_paths: Vec<PathBuf>,
    /// When true, require a valid `OwnershipOracle` and enforce target ownership constraints
    /// during preflight (`strict_ownership` checks in `src/api/preflight.rs`).
    pub strict_ownership: bool,
    /// When true, allow untrusted sources (e.g., world-writable or non-root owned) to pass
    /// preflight source trust checks as warnings instead of STOP. See `check_source_trust` usage
    /// in `src/api/preflight.rs` and `src/api/apply.rs`.
    pub force_untrusted_source: bool,
    /// When true, `restore_file()` is allowed to succeed without a backup payload present
    /// (best-effort). When false, missing backup yields `E_BACKUP_MISSING` and stops.
    /// Propagated from here to `apply.rs` → `fs::restore_file()`.
    pub force_restore_best_effort: bool,
    /// When true, allow degraded, non-atomic fallback on EXDEV during symlink replacement
    /// (implemented as unlink+symlink within the parent). When false, cross-filesystem swaps
    /// fail with `E_EXDEV` and no mutation is performed. Leave false for critical system paths
    /// (e.g., coreutils) to avoid windows of inconsistency.
    pub allow_degraded_fs: bool,
    /// When true, auto-rollback on post-apply smoke failure is disabled. See rollback flow
    /// in `src/api/apply.rs` (smoke failure path).
    pub disable_auto_rollback: bool,
    /// Logical tag used for naming backup artifacts and sidecar files. See `backup_path_with_tag()`
    /// and sidecar creation in `src/fs/symlink.rs`.
    pub backup_tag: String,
    /// When true, apply() proceeds even if preflight has policy_ok=false rows.
    /// Default is false (fail-closed). See gating in `src/api/apply.rs` and PLAN/45-preflight.md.
    pub override_preflight: bool,
    /// Require filesystem preservation capabilities (owner/mode/timestamps/xattrs/acl/caps).
    /// When true and the target path lacks support, preflight MUST STOP (unless override_preflight).
    /// See `detect_preservation_capabilities()` in `src/api/fs_meta.rs` and checks in preflight.
    pub require_preservation: bool,
    /// Require a rescue profile and toolset to be available (e.g., BusyBox or GNU core tools on PATH).
    /// When true and unavailable, preflight/apply MUST STOP (unless override_preflight).
    /// Verified via `rescue::verify_rescue_tools_with_exec()` in `src/api/preflight.rs` and gated in apply.
    pub require_rescue: bool,
    /// Require a LockManager to be present in Commit mode. When true and no lock manager is
    /// configured, apply() MUST fail early with E_LOCKING (exit code 30) without mutating state.
    /// Enforced at the top of `src/api/apply.rs`.
    pub require_lock_manager: bool,
    /// Allow Commit mode to proceed without a LockManager. Defaults to true for development
    /// ergonomics; set to false in hardened environments to fail-closed unless a lock manager
    /// is configured. If both `require_lock_manager` and `allow_unlocked_commit` are set, the
    /// requirement takes precedence and missing lock will fail.
    pub allow_unlocked_commit: bool,
    /// In Commit mode, require that a SmokeTestRunner is configured and passes. When true and
    /// no runner is configured, apply() MUST fail with E_SMOKE and auto-rollback unless disabled.
    /// Enforced in the post-apply smoke section of `src/api/apply.rs`.
    pub require_smoke_in_commit: bool,
    /// When verifying rescue profile/tooling, also attempt an executability check (e.g., X_OK
    /// or spawning "--help" with a very small timeout). Typically enabled in production only.
    /// See `rescue::verify_rescue_tools_with_exec(exec_check)`.
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
            backup_tag: DEFAULT_BACKUP_TAG.to_string(),
            override_preflight: false,
            require_preservation: false,
            require_rescue: false,
            require_lock_manager: false,
            require_smoke_in_commit: false,
            rescue_exec_check: false,
            allow_unlocked_commit: true,
        }
    }
}

impl Policy {
    /// Construct a Policy configured with recommended production defaults.
    ///
    /// Enables:
    /// - `require_rescue = true` (with `rescue_exec_check = true`)
    /// - `require_lock_manager = true`
    /// - `require_smoke_in_commit = true`
    ///
    /// Notes:
    /// - Other flags like `allow_degraded_fs` remain at their defaults and should be set
    ///   explicitly based on your environment.
    /// - In Commit mode, absence of a LockManager yields an early `apply.attempt` failure
    ///   with `error_id=E_LOCKING` and `exit_code=30`.
    /// - Missing smoke runner when `require_smoke_in_commit=true` yields `E_SMOKE` and
    ///   triggers auto-rollback unless disabled by policy.
    ///
    /// # Example
    /// ```rust
    /// use switchyard::policy::Policy;
    /// let policy = Policy::production_preset();
    /// // Optionally customize
    /// // policy.allow_degraded_fs = true;
    /// ```
    pub fn production_preset() -> Self {
        let mut p = Self::default();
        p.require_rescue = true;
        p.rescue_exec_check = true;
        p.require_lock_manager = true;
        p.require_smoke_in_commit = true;
        p
    }

    /// Mutate this Policy to apply the recommended production defaults.
    pub fn apply_production_preset(&mut self) -> &mut Self {
        self.require_rescue = true;
        self.rescue_exec_check = true;
        self.require_lock_manager = true;
        self.require_smoke_in_commit = true;
        self
    }

    /// Construct a Policy tailored for switching system coreutils to uutils-coreutils.
    ///
    /// Builds on [`production_preset`](#method.production_preset) and tightens gates:
    /// - `allow_degraded_fs = false` (fail on EXDEV; no degraded fallback)
    /// - `strict_ownership = true` (requires `OwnershipOracle`)
    /// - `require_preservation = true` (STOP if basic preservation not supported)
    /// - `override_preflight = false` (fail-closed)
    /// - `force_untrusted_source = false`
    /// - `force_restore_best_effort = false` (missing backup -> error)
    /// - `backup_tag = "coreutils"`
    ///
    /// You should still scope changes via `allow_roots` (e.g., `<root>/usr/bin`).
    ///
    /// # Example
    /// ```rust
    /// use switchyard::policy::Policy;
    /// # let root = std::path::PathBuf::from("/tmp/fakeroot");
    /// let mut policy = Policy::coreutils_switch_preset();
    /// policy.allow_roots.push(root.join("usr/bin"));
    /// ```
    pub fn coreutils_switch_preset() -> Self {
        let mut p = Self::production_preset();
        p.allow_degraded_fs = false;
        p.strict_ownership = true;
        p.require_preservation = true;
        p.override_preflight = false;
        p.force_untrusted_source = false;
        p.force_restore_best_effort = false;
        p.backup_tag = "coreutils".to_string();
        p
    }

    /// Mutate this Policy to apply the coreutils switch preset; see `coreutils_switch_preset()`.
    pub fn apply_coreutils_switch_preset(&mut self) -> &mut Self {
        self.apply_production_preset();
        self.allow_degraded_fs = false;
        self.strict_ownership = true;
        self.require_preservation = true;
        self.override_preflight = false;
        self.force_untrusted_source = false;
        self.force_restore_best_effort = false;
        self.backup_tag = "coreutils".to_string();
        self
    }
}
