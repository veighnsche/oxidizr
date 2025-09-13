use crate::constants::DEFAULT_BACKUP_TAG;
use crate::constants::RESCUE_MIN_COUNT as DEFAULT_RESCUE_MIN_COUNT;
use std::path::PathBuf;

use super::types::{
    ApplyFlow, Backup, Durability, ExdevPolicy, Governance, LockingPolicy, PreservationPolicy,
    Rescue, RiskLevel, Risks, Scope, SmokePolicy, SourceTrustPolicy,
};

/// Policy governs preflight gates, apply behavior, and production hardening for Switchyard.
///
/// Grouped fields provide clearer ownership and ergonomics.
#[derive(Clone, Debug)]
pub struct Policy {
    pub scope: Scope,
    pub rescue: Rescue,
    pub risks: Risks,
    pub durability: Durability,
    pub apply: ApplyFlow,
    pub governance: Governance,
    pub backup: Backup,
    // Retention knobs remain top-level for prune API
    pub retention_count_limit: Option<usize>,
    pub retention_age_limit: Option<std::time::Duration>,
    // Advanced toggles not yet grouped
    pub allow_unreliable_immutable_check: bool,
    pub preservation_tier: PreservationTier,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            scope: Scope::default(),
            rescue: Rescue { require: false, exec_check: false, min_count: DEFAULT_RESCUE_MIN_COUNT },
            risks: Risks { suid_sgid: RiskLevel::Stop, hardlinks: RiskLevel::Stop, source_trust: SourceTrustPolicy::RequireTrusted, ownership_strict: false },
            durability: Durability { backup_durability: true, sidecar_integrity: true, preservation: PreservationPolicy::Off },
            apply: ApplyFlow { exdev: ExdevPolicy::Fail, override_preflight: false, best_effort_restore: false, extra_mount_checks: Vec::new(), capture_restore_snapshot: true },
            governance: Governance { locking: LockingPolicy::Optional, smoke: SmokePolicy::Off, allow_unlocked_commit: false },
            backup: Backup { tag: DEFAULT_BACKUP_TAG.to_string() },
            retention_count_limit: None,
            retention_age_limit: None,
            allow_unreliable_immutable_check: false,
            preservation_tier: PreservationTier::Basic,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PreservationTier {
    Basic,
    Extended,
    Full,
}

impl Policy {
    /// Construct a Policy configured with recommended **production defaults**.
    ///
    /// Enables (hardened-by-default):
    /// - `require_rescue = true` (+ `rescue_exec_check = true`)
    /// - `require_lock_manager = true`
    /// - `require_smoke_in_commit = true`
    ///
    /// Notes:
    /// - Other flags (e.g., `allow_degraded_fs`) remain at their defaults and should be set
    ///   explicitly per environment.
    /// - In Commit mode, absence of a `LockManager` yields an early `apply.attempt` failure
    ///   with `error_id=E_LOCKING` (`exit_code=30`).
    /// - Missing smoke runner when `require_smoke_in_commit=true` yields `E_SMOKE`
    ///   and triggers auto-rollback unless disabled by policy.
    ///
    /// # Example
    /// ```rust
    /// use switchyard::policy::Policy;
    /// use switchyard::{Switchyard, logging::JsonlSink};
    /// // Optional adapters used in production
    /// use switchyard::adapters::FileLockManager;
    /// use switchyard::adapters::DefaultSmokeRunner;
    ///
    /// let policy = Policy::production_preset();
    /// let api = Switchyard::new(JsonlSink::default(), JsonlSink::default(), policy)
    ///     .with_lock_manager(Box::new(FileLockManager::new(std::path::PathBuf::from("/tmp/lock"))))
    ///     .with_smoke_runner(Box::new(DefaultSmokeRunner::default()));
    /// # let _ = api; // avoid unused warning
    /// ```
    #[must_use]
    pub fn production_preset() -> Self {
        let mut p = Self::default();
        p.rescue.require = true;
        p.rescue.exec_check = true;
        p.governance.locking = LockingPolicy::Required;
        p.governance.smoke = SmokePolicy::Require { auto_rollback: true };
        p
    }

    /// Mutate this Policy to apply the recommended **production defaults**.
    pub const fn apply_production_preset(&mut self) -> &mut Self {
        self.rescue.require = true;
        self.rescue.exec_check = true;
        self.governance.locking = LockingPolicy::Required;
        self.governance.smoke = SmokePolicy::Require { auto_rollback: true };
        self
    }

    /// Construct a Policy tailored for **switching system coreutils to uutils-coreutils**.
    ///
    /// Builds on [`production_preset`](#method.production_preset) and tightens gates:
    /// - `allow_degraded_fs = false` (fail on EXDEV; no degraded fallback)
    /// - `strict_ownership = true` (requires `OwnershipOracle`)
    /// - `require_preservation = true` (STOP if basic preservation not supported)
    /// - `override_preflight = false` (fail-closed)
    /// - `force_untrusted_source = false`
    /// - `force_restore_best_effort = false` (missing backup → error)
    /// - `backup_tag = "coreutils"`
    ///
    /// Additionally, for safer toolchain swaps:
    /// - `extra_mount_checks` defaults to common tool mount points (`/usr`, `/bin`, etc.)
    /// - `forbid_paths` blocks virtual/volatile filesystems (`/proc`, `/sys`, `/dev`, `/run`, `/tmp`)
    ///
    /// **Caller must still scope the operation** by setting `allow_roots` to the exact tree
    /// being switched (e.g., `<root>/usr/bin`). Everything else remains blocked.
    ///
    /// # Example
    /// ```rust
    /// use switchyard::policy::Policy;
    /// # let root = std::path::PathBuf::from("/tmp/fakeroot");
    /// let mut policy = Policy::coreutils_switch_preset();
    /// policy.scope.allow_roots.push(root.join("usr/bin")); // narrow the blast radius
    /// // Optionally tighten expectations on rescue tool count:
    /// // policy.rescue.min_count = policy.rescue.min_count.max(6);
    /// ```
    #[must_use]
    pub fn coreutils_switch_preset() -> Self {
        let mut p = Self::production_preset();

        p.apply.exdev = ExdevPolicy::Fail;
        p.risks.ownership_strict = true;
        p.durability.preservation = PreservationPolicy::RequireBasic;
        p.apply.override_preflight = false;
        p.risks.source_trust = SourceTrustPolicy::RequireTrusted;
        p.apply.best_effort_restore = false;
        p.backup.tag = "coreutils".to_string();

        p.apply.extra_mount_checks = vec![
            PathBuf::from("/usr"),
            PathBuf::from("/bin"),
            PathBuf::from("/sbin"),
            PathBuf::from("/usr/bin"),
            PathBuf::from("/usr/sbin"),
        ];

        p.scope.forbid_paths = vec![
            PathBuf::from("/proc"),
            PathBuf::from("/sys"),
            PathBuf::from("/dev"),
            PathBuf::from("/run"),
            PathBuf::from("/tmp"),
        ];
        p
    }

    /// Mutate this Policy to apply the **coreutils switch** preset; see `coreutils_switch_preset()`.
    pub fn apply_coreutils_switch_preset(&mut self) -> &mut Self {
        self.apply_production_preset();
        self.apply.exdev = ExdevPolicy::Fail;
        self.risks.ownership_strict = true;
        self.durability.preservation = PreservationPolicy::RequireBasic;
        self.apply.override_preflight = false;
        self.risks.source_trust = SourceTrustPolicy::RequireTrusted;
        self.apply.best_effort_restore = false;
        self.backup.tag = "coreutils".to_string();
        self.apply.extra_mount_checks = vec![
            PathBuf::from("/usr"),
            PathBuf::from("/bin"),
            PathBuf::from("/sbin"),
            PathBuf::from("/usr/bin"),
            PathBuf::from("/usr/sbin"),
        ];
        self.scope.forbid_paths = vec![
            PathBuf::from("/proc"),
            PathBuf::from("/sys"),
            PathBuf::from("/dev"),
            PathBuf::from("/run"),
            PathBuf::from("/tmp"),
        ];
        self
    }
}
