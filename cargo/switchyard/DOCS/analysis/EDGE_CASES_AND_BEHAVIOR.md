# Operational Edge Cases and Behavior

Generated: 2025-09-12

This document enumerates notable user-behavior edge cases and how Switchyard behaves, with references to source files. It also provides recommendations and optional improvements.

## Multiple experiments with different policies

- Design: Policy is owned by the `Switchyard` instance (`src/api.rs`, struct `Switchyard` field `policy`).
- Behavior: Each experiment can construct its own `Switchyard` with a distinct `Policy` (including different `backup_tag`, gating, rescue, etc.).
- Recommendation:
  - Create one `Switchyard` per experiment (or per plan family) to scope `Policy` independently.
  - Use unique `Policy.backup_tag` per experiment to segregate backup artifact namespaces.
  - If a single CLI orchestrates multiple experiments sequentially, instantiate separate `Switchyard` values or clone with modified `Policy` per run.

## Package manager updates that overwrite targets

- Scenario: A package manager updates binaries under `usr/bin` while Switchyard has enabled a symlink topology (or plans to restore).
- Relevant code:
  - Backup/sidecar creation: `src/fs/backup.rs::create_snapshot()` produces `.bak` payload + `.bak.meta.json` sidecar.
  - Restore: `src/fs/restore.rs::{restore_file, restore_file_prev}` use sidecar to drive idempotent restore.
  - Apply swap: `src/fs/swap.rs::replace_file_with_symlink()` snapshots then swaps.
- Behavior:
  - If the PM overwrites the symlink with a regular file, the sidecar remains. `restore_file()` checks `prior_kind` and will restore symlink topology to `prior_dest` (using `atomic_symlink_swap`).
  - If the PM deletes backup payloads, restore may fail with `NotFound` unless `Policy.force_restore_best_effort=true`.
  - If sidecar JSON is missing or corrupted, restore falls back to a legacy rename when a payload exists; otherwise `NotFound` (honors `force_restore_best_effort`).
- Recommendations:
  - For critical paths, keep `Policy.force_restore_best_effort=false` to fail closed on missing artifacts.
  - Consider taking a fresh snapshot before risky ops (or enable `Policy.capture_restore_snapshot`) so `restore_file_prev()` can roll back the last step.

## CLI removed/reinstalled, then re-apply or remove experiments

- Situation: User uninstalls the CLI that orchestrates Switchyard, then reinstalls and applies/removes experiments again.
- Behavior:
  - Artifacts: Backups and sidecars are plain files in the target directory; they persist across CLI re-installs.
  - Idempotence: `restore_file()` short-circuits if the current state already matches the sidecar’s `prior_kind` (and for symlinks, the destination), so repeated restores are no-ops.
  - Removal/Re-enable flows work without a central registry; the filesystem is the source of truth.
- Caveats:
  - Old artifacts will accumulate without a retention policy; consider periodic cleanup (see “Retention” below).

## Concurrency and locking

- Concurrency within Switchyard processes: `Switchyard::apply()` can (optionally) use a `LockManager` to serialize operations; enforced via policy:
  - `Policy.require_lock_manager` and `Policy.allow_unlocked_commit` (
    `src/policy/config.rs`)
  - Locking errors map to `E_LOCKING` with exit code 30 (`src/api/apply/mod.rs`).
- Cross-process concurrency (e.g., package manager running concurrently):
  - File operations use TOCTOU-safe sequences with directory handles: `open_dir_nofollow` + `*at` + `renameat` + `fsync` (`src/fs/atomic.rs`, `src/fs/backup.rs`, `src/fs/restore.rs`, `src/fs/swap.rs`).
  - There is no global cross-process lock; concurrent external modifications can still race. Policy gating plus operational discipline is recommended for critical switches.

## Cross-filesystem moves (EXDEV)

- Behavior:
  - If `Policy.allow_degraded_fs=true`, cross-FS swaps degrade to unlink+symlink (non-atomic). Return indicates degraded path was used.
  - If `false`, the operation fails with `E_EXDEV` and no mutation occurs.
- Files:
  - `src/fs/swap.rs::replace_file_with_symlink()`
  - Error mapping in `src/api/apply/handlers.rs`.
- Recommendation: For core system paths, prefer `allow_degraded_fs=false` to avoid windows of inconsistency.

## Mount flags, immutability, and ownership

- Mounts: `preflight::ensure_mount_rw_exec` (via `src/preflight/checks.rs`) verifies rw+exec; ambiguity fails closed.
- Immutability: `preflight::check_immutable` checks `lsattr -d` for `i` flag and STOPs with guidance to `chattr -i`.
- Ownership (optional): if `Policy.strict_ownership=true`, requires an `OwnershipOracle`; failing that, preflight STOPs.
- Forbid/allow: `Policy.forbid_paths` and `Policy.allow_roots` scope the mutation blast radius.

## Sidecar issues and fallbacks

- Missing/corrupted sidecar: `read_sidecar().ok()` treats errors as absent sidecar; restore then attempts legacy rename when payload exists.
- Missing payload:
  - `restore_file()` and `restore_file_prev()` require payload for `prior_kind="file"`; otherwise `NotFound` unless `force_restore_best_effort=true`.
- Symlink equality and normalization:
  - For idempotence check, restore canonicalizes relative symlink destinations against the target’s parent to compare robustly.

## Parent directories and path safety

- Safety: `fs::paths::is_safe_path` guards source/target strings.
- Parent handling:
  - Swaps create parent dirs as needed (`create_dir_all`) and use dir handles for unlink/rename.
  - Restores operate within the target’s parent; if the parent is missing, some restore variants will error.

## Permissions and preservation

- Mode: Backup preserves file mode and restores it (`fchmod`).
- Owner/group/timestamps/xattrs/ACLs:
  - Not fully restored by `restore_file()`; preservation capability is probed in preflight (`detect_preservation_capabilities`), but default restore does not change uid/gid/mtime.
  - Policy can require preservation support and STOP when unsupported, but actual restore is presently mode-only.

## Retention (artifact accumulation)

- Backups are timestamped, never pruned by the library.
- Risk: Unbounded growth in directories with frequent operations.
- Recommendation: Implement a retention policy in the orchestrating CLI (e.g., keep last N backups per target/tag; prune older ones).

## Filenames with NUL bytes

- `CString::new(...)` rejects names containing NUL; code maps this to `InvalidInput`.
- These are extremely rare in practice for filesystem component names; if encountered, operations will fail safely with a clear error.

## Restore invertibility

- `Policy.capture_restore_snapshot=true` (default) makes `RestoreFromBackup` invertible via `restore_file_prev()`.
- Without it, a restore’s inverse is not well-defined; `apply` logs that rollback of `RestoreFromBackup` is not supported and records an informational error.

## Permission denied and dry-run

- Without sufficient privileges, operations will fail (e.g., writing under `/usr/bin`).
- Preflight/gating mitigate some issues (ro/noexec/immutable), but not permissions.
- Dry-run (`ApplyMode::DryRun`) exercises paths without mutation; useful for validating policies and blast radius.

## Additional recommendations

- Per-experiment policy separation: construct separate `Switchyard` instances with tailored `Policy`.
- Unique `backup_tag` per experiment to avoid artifact collisions.
- Consider adding:
  - Retention and GC helper utilities for backups.
  - Optional extended preservation on restore (uid/gid/mtime/xattrs) when supported.
  - A `validate`/`doctor` command in the orchestrating CLI to scan for orphaned sidecars/payloads and offer remediation.
  - Metrics emission for restore durations (parity with apply’s FSYNC warn path).

## Quick references (files)

- `src/policy/config.rs` — Policy fields (rescue, lock, preservation, degraded fs, backups, preflight override, roots/forbid, etc.)
- `src/fs/backup.rs` — create_snapshot, sidecar IO, latest/previous discovery
- `src/fs/restore.rs` — restore_file, restore_file_prev, idempotence checks
- `src/fs/swap.rs` — replace_file_with_symlink (snapshots then atomic swap / EXDEV fallback)
- `src/preflight/checks.rs` — ensure_mount_rw_exec, check_immutable, check_source_trust
- `src/api/apply/mod.rs` — locking, policy gating, smoke tests, rollback behavior
- `src/api/preflight/mod.rs` — per-action gating rows and summary facts
