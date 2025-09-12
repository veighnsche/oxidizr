# Switchyard Behaviors Inventory

Date: 2025-09-12

This document enumerates the observable behaviors of the Switchyard library across API stages, filesystem mechanisms, policy gating, adapters, logging, and types. Each item references the canonical implementation paths for traceability.

## API surface and stages

- __Plan (build deterministic plan)__
  - File: `src/api/plan.rs`
  - Behavior:
    - Builds `Plan` from `PlanInput` with actions `EnsureSymlink` and `RestoreFromBackup`.
    - Sorts actions deterministically by kind and target rel path.
    - Emits per-action `plan` facts with `action_id`, `path` (via `logging/audit.rs::emit_plan_fact`).

- __Preflight (policy gating, probes, per-action rows, summary)__
  - File: `src/api/preflight.rs`
  - Behavior:
    - Per action: emits `preflight` facts including `current_kind`, `planned_kind`, optional `policy_ok`, `provenance`, `notes`, `preservation` and `preservation_supported`.
    - Runs policy checks: `extra_mount_checks`, target `ensure_mount_rw_exec`, `check_immutable`, `check_source_trust` (warn or stop per `force_untrusted_source`), `strict_ownership` (requires `OwnershipOracle`), `allow_roots` and `forbid_paths` scope checks.
    - Global rescue probe via `policy::rescue::verify_rescue_tools_with_exec_min` when `require_rescue`.
    - Summary: emits `preflight` summary with `rescue_profile` and on failure includes `error_id=E_POLICY`, `exit_code=10`. Timestamp is zero (`TS_ZERO`) and events are redacted.

- __Apply (locking, gating, handlers, rollback, smoke, attestation, summary)__
  - File: `src/api/apply.rs`
  - Behavior:
    - Locking: acquires process lock via `LockManager` when present; else enforces policy (`require_lock_manager` or disallow unlocked commit in Commit). Emits `apply.attempt` with `E_LOCKING/30` on failure and returns. Stage parity: also emits `apply.result` failure with `E_LOCKING/30`.
    - Emits success `apply.attempt` with `lock_wait_ms` when locking path proceeds.
    - Policy gating: if not `override_preflight` and in Commit, computes `policy::gating::gating_errors(...)`. If non-empty, emits per-action and summary `apply.result` failures with `E_POLICY/10` and returns.
    - Handlers: per action delegate to `apply/handlers.rs`:
      - `EnsureSymlink`: coordinates backup + atomic replace with `fs::swap::replace_file_with_symlink`, collects before/after hashes, degraded EXDEV signal and `fsync_ms` duration, emits per-action `apply.attempt` (success) and `apply.result` (success/failure with error ID mapping `E_ATOMIC_SWAP` or `E_EXDEV` on failure).
      - `RestoreFromBackup`: optionally `create_snapshot()` when `capture_restore_snapshot`, then `restore_file_prev()` or `restore_file()`. Emits per-action `apply.attempt` (success) and `apply.result` (success; or failure with `E_BACKUP_MISSING` or `E_RESTORE_FAILED`).
    - On first failure, attempts rollback of already executed actions in reverse order via `fs::restore_file`; emits `rollback` step facts.
    - Smoke tests (Commit): if `require_smoke_in_commit` and runner missing, fails with `E_SMOKE/80`. If runner present and returns error, fails with `E_SMOKE/80` and (unless `disable_auto_rollback`) auto-rolls back.
    - Attestation (Commit success): if `Attestor` provided, emits `attestation` bundle fields inside final `apply.result`.
    - Final summary `apply.result`: includes `lock_wait_ms`. On failure where not smoke, defaults `error_id/exit_code` to `E_POLICY/10` (best-effort summary mapping).

- __Rollback (inverse planning)__
  - File: `src/api/rollback.rs`
  - Behavior:
    - Inverts executed actions from an `ApplyReport` into a new `Plan`.
    - For `RestoreFromBackup`: if `capture_restore_snapshot` true, invert to another `RestoreFromBackup`; else skip (unknown prior state).

## Logging and redaction

- __Audit helpers and envelope__
  - File: `src/logging/audit.rs`
  - Behavior:
    - Ensures `schema_version`, `ts`, `plan_id`, `path`, and `dry_run` are present on all facts; includes provenance with `env_sanitized=true`.
    - Emits events: `plan`, `preflight` (rows), `apply.attempt`, `apply.result`, `rollback`.

- __Redaction and timestamps__
  - File: `src/logging/redact.rs`
  - Behavior:
    - `ts_for_mode`: DryRun -> `TS_ZERO`, Commit -> RFC3339 now.
    - `redact_event`: zeroes `ts`; removes `duration_ms`, `lock_wait_ms`, `severity`, `degraded`, and content hashes; masks `provenance.helper` and `attestation.signature/bundle_hash/public_key_id`.

## Filesystem mechanisms

- __Atomic swap and degraded fallback__
  - Files: `src/fs/atomic.rs`, `src/fs/swap.rs`
  - Behavior:
    - TOCTOU-safe symlink swap with `open_dir_nofollow`, `symlinkat`, `renameat`, and `fsync` parent.
    - Env knob `SWITCHYARD_FORCE_EXDEV=1` simulates EXDEV to exercise degraded fallback; `allow_degraded_fs` controls fallback.

- __Backups and sidecar schema__
  - File: `src/fs/backup.rs`
  - Behavior:
    - `create_snapshot` for file/symlink/none topologies creates timestamped payload and sidecar (`BackupSidecar` with `prior_kind`, optional `prior_dest`, `mode`).
    - `backup_path_with_tag` naming: `.{name}.{tag}.{timestamp}.bak` (and `.meta.json`).
    - `find_latest_backup_and_sidecar` / `find_previous_backup_and_sidecar`; `has_backup_artifacts`.

- __Restore logic__
  - File: `src/fs/restore.rs`
  - Behavior:
    - `restore_file` (latest), `restore_file_prev` (second newest). Uses sidecar to restore topology; idempotent short-circuit when current state matches.
    - Legacy rename fallback when sidecar missing and payload present.
    - Preserves mode when available in sidecar; fsync parent directory.

- __Metadata/probes__
  - File: `src/fs/meta.rs`
  - Behavior:
    - `kind_of`, `resolve_symlink_target`, `sha256_hex_of`, `detect_preservation_capabilities` (owner/mode/timestamps/xattrs/acls/caps).

- __Mount inspection__
  - File: `src/fs/mount.rs`
  - Behavior:
    - `ProcStatfsInspector` parses `/proc/self/mounts`; `ensure_rw_exec` enforces rw+exec capability.

## Policy and gating

- __Policy config and presets__
  - File: `src/policy/config.rs`
  - Behavior:
    - Gating surface: `allow_roots`, `forbid_paths`, `strict_ownership`, `force_untrusted_source`, `override_preflight`, `require_preservation`, `require_rescue`, `rescue_exec_check`, `rescue_min_count`, `extra_mount_checks`.
    - Apply surface: `require_lock_manager`, `allow_unlocked_commit`, `require_smoke_in_commit`, `disable_auto_rollback`.
    - FS behavior: `allow_degraded_fs`, `force_restore_best_effort`, `backup_tag`, `capture_restore_snapshot`.

- __Gating engine__
  - File: `src/policy/gating.rs`
  - Behavior:
    - Computes `gating_errors` reasons mirroring preflight checks; used by apply-stage gating to emit `E_POLICY` failures.

- __Rescue verification__
  - File: `src/policy/rescue.rs`
  - Behavior:
    - Profiles: BusyBox or GNU subset (`RESCUE_MUST_HAVE` with minimum count). Env knob `SWITCHYARD_FORCE_RESCUE_OK=1|0` for tests.

## Adapters

- __Locking__
  - Files: `src/adapters/lock/{mod.rs,file.rs}`
  - Behavior:
    - `FileLockManager` implements file-based process lock with timeout and polling; returns `Result<Box<dyn LockGuard>>`.

- __Ownership__
  - Files: `src/adapters/ownership/{mod.rs,fs.rs}`
  - Behavior:
    - `OwnershipOracle` trait returning `OwnershipInfo { uid, gid, pkg }`; FS implementation consults system metadata (and possibly package DB).

- __Smoke__
  - File: `src/adapters/smoke.rs`
  - Behavior:
    - `SmokeTestRunner` trait; default minimal runner validates symlink targets resolve to sources.

- __Attestation__
  - File: `src/adapters/attest.rs`
  - Behavior:
    - `Attestor` trait; `sign(bundle)` -> `Signature`, with `key_id()` and `algorithm()`; used by apply summary on success.

## Types and determinism

- __Plan/Actions/ApplyMode__ — `src/types/plan.rs`.
- __IDs and determinism__ — `src/types/ids.rs` (UUIDv5 plan_id/action_id stable across runs).
- __Errors and exit codes__ — `src/api/errors.rs`: `E_POLICY=10`, `E_OWNERSHIP=20`, `E_LOCKING=30`, `E_ATOMIC_SWAP=40`, `E_EXDEV=50`, `E_BACKUP_MISSING=60`, `E_RESTORE_FAILED=70`, `E_SMOKE=80`, `E_GENERIC=1`.

## Environment knobs and constants (test hooks)

- `SWITCHYARD_FORCE_EXDEV=1` (simulate EXDEV during rename).
- `SWITCHYARD_FORCE_RESCUE_OK=1|0` (force rescue availability).
- `FSYNC_WARN_MS` threshold for severity=warn annotation (see `apply/audit_fields.rs`).

