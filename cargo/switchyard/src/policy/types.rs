use std::path::PathBuf;

#[derive(Clone, Copy, Debug)]
pub enum RiskLevel {
    Stop,
    Warn,
    Allow,
}

#[derive(Clone, Copy, Debug)]
pub enum ExdevPolicy {
    Fail,
    DegradedFallback,
}

impl Default for ExdevPolicy {
    fn default() -> Self { ExdevPolicy::Fail }
}

#[derive(Clone, Copy, Debug)]
pub enum LockingPolicy {
    Required,
    Optional,
}

#[derive(Clone, Copy, Debug)]
pub enum PreservationPolicy {
    Off,
    RequireBasic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceTrustPolicy {
    RequireTrusted,
    WarnOnUntrusted,
    AllowUntrusted,
}

#[derive(Clone, Copy, Debug)]
pub enum SmokePolicy {
    Off,
    Require { auto_rollback: bool },
}

#[derive(Clone, Debug, Default)]
pub struct Scope {
    pub allow_roots: Vec<PathBuf>,
    pub forbid_paths: Vec<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct Rescue {
    pub require: bool,
    pub exec_check: bool,
    pub min_count: usize,
}

impl Default for Rescue {
    fn default() -> Self {
        Self { require: false, exec_check: false, min_count: 0 }
    }
}

#[derive(Clone, Debug)]
pub struct Risks {
    pub suid_sgid: RiskLevel,
    pub hardlinks: RiskLevel,
    pub source_trust: SourceTrustPolicy,
    pub ownership_strict: bool,
}

impl Default for Risks {
    fn default() -> Self {
        Self { suid_sgid: RiskLevel::Stop, hardlinks: RiskLevel::Stop, source_trust: SourceTrustPolicy::RequireTrusted, ownership_strict: false }
    }
}

#[derive(Clone, Debug)]
pub struct Durability {
    pub backup_durability: bool,
    pub sidecar_integrity: bool,
    pub preservation: PreservationPolicy,
}

impl Default for Durability {
    fn default() -> Self {
        Self { backup_durability: true, sidecar_integrity: true, preservation: PreservationPolicy::Off }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ApplyFlow {
    pub exdev: ExdevPolicy,
    pub override_preflight: bool,
    pub best_effort_restore: bool,
    pub extra_mount_checks: Vec<PathBuf>,
    pub capture_restore_snapshot: bool,
}

#[derive(Clone, Debug)]
pub struct Governance {
    pub locking: LockingPolicy,
    pub smoke: SmokePolicy,
    pub allow_unlocked_commit: bool,
}

impl Default for Governance {
    fn default() -> Self {
        Self { locking: LockingPolicy::Optional, smoke: SmokePolicy::Off, allow_unlocked_commit: true }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Backup {
    pub tag: String,
}
