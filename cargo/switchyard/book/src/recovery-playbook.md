# Recovery Playbook

When an apply fails, Switchyard automatically rolls back prior actions. If rollback also encounters issues (e.g., missing backup payload), use this playbook.

> Note: Reference materials: `INVENTORY/15_FS_Restore_and_Rollback.md`, `INVENTORY/10_FS_Backup_and_Sidecar.md`, `INVENTORY/30_Infra_Rescue_Profile_Verification.md`, `SPEC/SPEC.md` (§2.3, §2.9).

## 1) Pause Further Mutations

- Ensure only one apply is running. If not already configured, acquire the process lock via your LockManager tool before manual steps.

## 2) Inspect Facts

- Check `apply.result`, `rollback`, and `rollback.summary` facts for:
  - `summary_error_ids` (e.g., `["E_RESTORE_FAILED","E_BACKUP_MISSING"]`).
  - `backup_durable`, `sidecar_integrity_verified` flags.
  - `path`, `before_hash`, `after_hash` on mutated entries.

## 3) Verify Rescue Profile

- Confirm a fallback toolset exists on PATH (BusyBox or GNU subset). If missing, install or stage a static BusyBox as a last resort.

## 4) Locate Backup Artifacts

- For each affected target:
  - Find the latest backup payload and sidecar (tagged by `policy.backup.tag`).
  - Validate sidecar integrity: if `payload_hash` is present and policy requires, verify hash.

## 5) Attempt Library-Guided Restore

- Prefer `plan_rollback_of(apply_report)` and `apply(rollback_plan, Commit)` to leverage idempotent restoration and event emission.

## 6) Manual Restore (Break-Glass)

When library restore cannot proceed:

- Ensure parent directories exist with correct ownership and permissions.
- Recreate the symlink topology:
  - For a single symlink replacement, `ln -sT TARGET LINK` (atomic via rename is preferred; at minimum ensure no broken window is visible between unlink and symlink creation).
- Restore ownership/mode/xattrs/ACLs/caps if applicable and known.

> Warning: Manual steps will not emit audit facts. Document actions and capture hashes where possible.

## 7) Validate With Smoke

- Run the minimal smoke suite locally (ls, cp, mv, rm, ln, stat, readlink, sha256sum, sort, date) to verify health before resuming.

## 8) Prune & Retain

- Once stable, consider pruning backups using retention policy to keep the newest and remove older artifacts safely. See How-To: Prune Backups.

## 9) File an Incident Record

- Attach emitted facts, policies used, environment notes (mount options, filesystem), and a remediation summary.

## Decision Aids

- Proceed with degraded EXDEV? Only if `allow_degraded_fs=true` and the impact is understood. Facts should include `degraded=true` with a reason.
- Retry window: Ensure `lock_timeout_ms` and retry backoff are reasonable to avoid thrashing.
