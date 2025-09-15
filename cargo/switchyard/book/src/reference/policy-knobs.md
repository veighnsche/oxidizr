# Policy Knobs

Switchyard groups policy controls into cohesive sections to make intent clear.

Core groups:

- Scope
  - `scope.allow_roots: Vec<PathBuf>` — roots under which `SafePath` mutations are allowed.
  - `scope.forbid_paths: Vec<PathBuf>` — absolute paths to block outright.
- Rescue
  - `rescue.require: bool` — require a rescue profile before mutation.
  - `rescue.exec_check: bool` — verify runner executability (x bits) on PATH.
  - `rescue.min_count: usize` — minimum number of rescue tools to consider acceptable.
- Risks
  - `risks.ownership_strict: bool` — require an `OwnershipOracle` and stop when not owned.
  - `risks.source_trust: SourceTrustPolicy` — trust model for sources (require/warn/allow).
  - `risks.suid_sgid: RiskLevel` — suid/sgid risk handling (stop/warn/allow).
  - `risks.hardlinks: RiskLevel` — hardlink hazard handling (stop/warn/allow).
- Durability
  - `durability.backup_durability: bool` — fsync parent for backup/sidecar.
  - `durability.sidecar_integrity: bool` — verify sidecar payload hash when present.
  - `durability.preservation: PreservationPolicy` — preservation requirement (off/require basic).
- Apply Flow
  - `apply.exdev: ExdevPolicy` — cross‑filesystem behavior (fail or degraded fallback).
  - `apply.override_preflight: bool` — ignore preflight STOPs (not recommended in production).
  - `apply.best_effort_restore: bool` — continue on missing backup (emit error) vs hard stop.
  - `apply.extra_mount_checks: Vec<PathBuf>` — additional mount points to verify.
  - `apply.capture_restore_snapshot: bool` — capture state when restoring.
- Governance
  - `governance.locking: LockingPolicy` — require a lock manager in Commit.
  - `governance.smoke: SmokePolicy` — require smoke in Commit and auto‑rollback policy.
  - `governance.allow_unlocked_commit: bool` — development override (do not use in prod).
- Backup
  - `backup.tag: String` — tag used for backup payload and sidecar filenames.
- Retention
  - `retention_count_limit: Option<usize>` — keep at most N backups (per target + tag).
  - `retention_age_limit: Option<Duration>` — keep backups not older than the given age.
- Advanced
  - `allow_unreliable_immutable_check: bool` — relax immutable attribute probing.
  - `preservation_tier: PreservationTier` — tiered preservation expectations (basic/extended/full).

Citations:
- `cargo/switchyard/src/policy/config.rs`
- `cargo/switchyard/src/policy/types.rs`
