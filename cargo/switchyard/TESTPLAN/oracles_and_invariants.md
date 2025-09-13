# Oracles and Invariants (Switchyard Library)

Defines per-function observable oracles and invariants to assert. Oracles are deterministic and implementation-agnostic. Invariants express outcomes independent of internal structure.

## Common Oracles (all functions)

- Return types and error categories match expectations (`ApiError::*`, `ErrorId`, `exit_code` mapping in facts where applicable).
- Facts emission via `logging::StageLogger` includes minimal envelope fields and respects redaction rules in DryRun (`ts=TS_ZERO`, no volatile fields).
- No temporary files left after failure (`fs/atomic.rs::atomic_symlink_swap` ensures cleanup of tmp symlink name; restore steps ensure parent fsync and unlink on failure).

## Switchyard::plan(&self, PlanInput) -> Plan

- Oracles
  - Plans are sorted deterministically: by kind (`EnsureSymlink` before `RestoreFromBackup`) and then by `target.rel()` ascending (see `src/api/plan.rs`).
  - `StageLogger::plan()` emits one success event per action with `action_id` and `path` (TS_ZERO in DryRun context).
- Invariants
  - Pure function w.r.t. input: no filesystem mutations.
  - Stable `action_id` generation per SPEC: derived from `plan_id`, action content, and index (see `types::ids`).

## Switchyard::preflight(&self, &Plan) -> Result<PreflightReport, ApiError>

- Oracles
  - Returns `PreflightReport{ ok, warnings, stops, rows }` with stable ordering by `(path, action_id)` (see `api/preflight/mod.rs`).
  - Emits one `preflight` fact per action and a `preflight.summary` event. When failed, summary includes `error_id=E_POLICY`, `exit_code=10`, and best-effort `summary_error_ids` chain (ownership co-emission when mentioned).
- Invariants
  - When `policy.rescue.require=true` and tools insufficient, `ok=false` with a STOP reason ("rescue profile unavailable").
  - When `policy.durability.preservation=RequireBasic` and unsupported for path, `ok=false` and STOP includes preservation unsupported.
  - Ownership strictness without oracle yields STOP (policy gate) and does not proceed to apply unless `override_preflight=true`.

## Switchyard::apply(&self, &Plan, ApplyMode) -> Result<ApplyReport, ApiError>

- Oracles
  - Emits `apply.attempt` per action and one summary `apply.result` per function call. Summary includes `error_id` and `exit_code` on failure, and `summary_error_ids` best-effort chain (via `errors::infer_summary_error_ids`).
  - `ApplyReport{ executed, duration_ms, errors, plan_uuid, rolled_back, rollback_errors }` reflects performed work; `executed` equals executed subset of plan (prefix) when early exit occurs.
  - For EnsureSymlink success: `fs/atomic.rs::atomic_symlink_swap` makes `target` a symlink to `source`; degraded flag set when EXDEV fallback used; parent directory fsynced; before/after hashes computed.
  - For RestoreFromBackup success: target reflects prior state encoded in sidecar; if `payload_hash` present and integrity enabled, it must match; parent directory fsynced.
- Invariants
  - Locking: If `governance.locking=Required` and `mode=Commit` and no lock manager, apply fails early with `E_LOCKING` and no FS mutations.
  - Smoke: If `SmokePolicy::Require{..}` and runner absent/fails in Commit, summary `E_SMOKE`, and auto-rollback executed when enabled.
  - EXDEV: With `DegradedFallback`, EXDEV leads to non-atomic but durable replacement and `degraded=true`; with `Fail`, EXDEV maps to `E_EXDEV` failure.
  - Best-effort restore: When enabled or integrity disabled, missing payload/hash mismatch does not error (tolerated path).
  - Redaction: In DryRun, facts have `ts=TS_ZERO`, no `duration_ms`/`lock_wait_ms`/hash fields; comparisons use redacted form.

## Switchyard::plan_rollback_of(&self, &ApplyReport) -> Plan

- Oracles
  - Produces an inverse plan based on executed actions, reversed order (see `api/rollback.rs`).
  - If `policy.apply.capture_restore_snapshot=true`, restore actions invert to restore from previous snapshot; otherwise, may skip inversions that rely on unknown prior state.
- Invariants
  - No FS mutations; pure derivation from `ApplyReport` and `policy`.

## Switchyard::prune_backups(&self, &SafePath) -> Result<PruneResult, ApiError>

- Oracles
  - Never deletes the newest backup (see `fs/backup/prune.rs`).
  - Applies count and age policies independently; deletes entries violating either; sidecars deleted alongside payloads.
  - Emits `prune.result` fact including counts, tag, and retention knobs.
- Invariants
  - Directory containing entries is fsynced best-effort after deletions.
  - Non-UTF-8 filenames are skipped safely (no panic).

## SafePath::from_rooted(root, candidate)

- Oracles
  - Returns `Ok(SafePath)` only when `candidate` is relative or absolute inside `root` and contains no `..` or unsupported components; `rel` is normalized (no `.` components).
  - Returns `Error` for `candidate` outside root, with `..`, or unsupported components; panics if `root` is not absolute (assert).
- Invariants
  - `as_path()` always starts with `root`; `rel()` never contains `..`.

## Negative-Path Oracles (precision)

- Locking timeout → `E_LOCKING`, bounded wait ≤ `timeout_ms`.
- Missing backup when `best_effort_restore=false` → `E_BACKUP_MISSING`.
- Restore failure other than NotFound → `E_RESTORE_FAILED`.
- Atomic swap failure without EXDEV → `E_ATOMIC_SWAP`.
- Smoke failure → `E_SMOKE` and `rolled_back` matches policy.

## Determinism Notes

- DryRun: rely on redacted facts only; ignore volatile fields. Assert that `ts=TS_ZERO`, presence of `schema_version`, `plan_id`, `run_id`, and `event_id` format (UUID) without value matching.
- Commit: assert on filesystem end state and stable fact fields (exclude `event_id`, real `ts`).
