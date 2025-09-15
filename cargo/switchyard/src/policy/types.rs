use std::path::PathBuf;

/// Risk handling level for potentially dangerous conditions (e.g., SUID/SGID bits, hardlinks).
#[derive(Clone, Copy, Debug)]
pub enum RiskLevel {
    Stop,
    Warn,
    Allow,
}

/// Cross‑filesystem behavior policy for atomic rename failures (EXDEV).
#[derive(Clone, Copy, Debug, Default)]
pub enum ExdevPolicy {
    #[default]
    Fail,
    DegradedFallback,
}

/// Locking policy for serialize‑mutations requirement in Commit mode.
#[derive(Clone, Copy, Debug)]
pub enum LockingPolicy {
    Required,
    Optional,
}

/// Preservation requirement policy for metadata dimensions.
#[derive(Clone, Copy, Debug)]
pub enum PreservationPolicy {
    Off,
    RequireBasic,
}

/// Source trust policy for evaluating whether a source path is acceptable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceTrustPolicy {
    RequireTrusted,
    WarnOnUntrusted,
    AllowUntrusted,
}

/// Smoke testing policy for Commit mode.
#[derive(Clone, Copy, Debug)]
pub enum SmokePolicy {
    Off,
    Require { auto_rollback: bool },
}

/// Scope policy restricting allowed roots and forbidding specific absolute paths.
#[derive(Clone, Debug, Default)]
pub struct Scope {
    pub allow_roots: Vec<PathBuf>,
    pub forbid_paths: Vec<PathBuf>,
}

/// Rescue expectations for production safety.
#[derive(Debug, Copy, Clone, Default)]
pub struct Rescue {
    pub require: bool,
    pub exec_check: bool,
    pub min_count: usize,
}

/// Risk controls toggles.
#[derive(Debug, Copy, Clone)]
pub struct Risks {
    pub suid_sgid: RiskLevel,
    pub hardlinks: RiskLevel,
    pub source_trust: SourceTrustPolicy,
    pub ownership_strict: bool,
}

impl Default for Risks {
    fn default() -> Self {
        Self {
            suid_sgid: RiskLevel::Stop,
            hardlinks: RiskLevel::Stop,
            source_trust: SourceTrustPolicy::RequireTrusted,
            ownership_strict: false,
        }
    }
}

/// Durability requirements for backups and preservation.
#[derive(Debug, Copy, Clone)]
pub struct Durability {
    pub backup_durability: bool,
    pub sidecar_integrity: bool,
    pub preservation: PreservationPolicy,
}

impl Default for Durability {
    fn default() -> Self {
        Self {
            backup_durability: true,
            sidecar_integrity: true,
            preservation: PreservationPolicy::Off,
        }
    }
}

/// Apply stage policy affecting degraded paths and preflight parity.
#[derive(Clone, Debug, Default)]
pub struct ApplyFlow {
    pub exdev: ExdevPolicy,
    pub override_preflight: bool,
    pub best_effort_restore: bool,
    pub extra_mount_checks: Vec<PathBuf>,
    pub capture_restore_snapshot: bool,
}

/// Governance policy defining required adapters and allowances.
#[derive(Debug, Copy, Clone)]
pub struct Governance {
    pub locking: LockingPolicy,
    pub smoke: SmokePolicy,
    pub allow_unlocked_commit: bool,
}

impl Default for Governance {
    fn default() -> Self {
        Self {
            locking: LockingPolicy::Optional,
            smoke: SmokePolicy::Off,
            allow_unlocked_commit: true,
        }
    }
}

/// Backup configuration.
#[derive(Clone, Debug, Default)]
pub struct Backup {
    pub tag: String,
}
