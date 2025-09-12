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

## Link drift detection and RELINK

- Scenarios:
  - `target` symlink is overwritten by a package manager or manual changes (becomes a regular file or different symlink).
  - `source` moved or deleted leaving `target` a dangling symlink.
- Current behavior:
  - Idempotent checks in `src/fs/restore.rs::restore_file()` short-circuit when the on-disk state already matches `prior_kind` and `prior_dest` from the sidecar.
  - There is no background integrity checker; drift is only corrected when `RestoreFromBackup` or `EnsureSymlink` is executed.
- UX proposal:
  - Add a CLI subcommand `doctor links` to scan a tree, using `src/fs/meta.rs::{kind_of, resolve_symlink_target}` to validate that "symlink → expected dest" still holds. Emit actionable JSON and human summaries.
  - Add `relink` capability: when drift is detected and the desired topology is known (e.g., from an experiment plan or policy), execute `Action::EnsureSymlink` to re-point the link without touching historical backups; otherwise, fallback to `RestoreFromBackup` driven by the sidecar.
  - Optional systemd timer (Link Guardian): run `doctor links --fix=relink|restore|warn` periodically on critical paths. Surface results to audit via `apply.result`-like facts.
  - Policy idea (future): `link_health = Off|Warn|Fix(ReLink)|Restore` to choose behavior on drift.

## Directory targets (unsupported)

- Issue:
  - `src/fs/backup.rs::create_snapshot()` assumes file-or-symlink. If `target` is a directory, the `openat(..., OFlags::RDONLY)` path will fail (`EISDIR`).
- Behavior:
  - Snapshot returns an error; replace/restore paths aren’t designed for directories.
- Recommendation:
  - Treat directory targets as invalid at planning/preflight time. Use `src/fs/meta.rs::kind_of()` and reject `kind == "dir"` for `Action::EnsureSymlink` and `Action::RestoreFromBackup` in `src/policy/gating.rs`.
  - CLI: produce a clear error "directories are not supported targets" and suggest selecting a file within the directory.

## Hardlink semantics are not preserved

- Issue:
  - When restoring a regular file, `src/fs/restore.rs` performs a `renameat` of the backup payload to the target, creating a new inode and breaking any prior hardlink relationships.
- Recommendation:
  - Preflight note: detect `nlink > 1` (via `std::os::unix::fs::MetadataExt::nlink`) and STOP unless `--force`. Emit a clear advisory that hardlinks will be broken.
  - Document in CLI help and audit facts (preflight rows) when such a target is encountered.

## Sticky bit or restricted parent directories

- Issue:
  - In sticky directories (e.g., `/tmp` with `+t`), `unlinkat`/`renameat` can fail with `EACCES` unless the caller owns the file or directory.
- Behavior:
  - `src/fs/swap.rs` and `src/fs/restore.rs` will surface the I/O error.
- Recommendation:
  - Add a preflight probe for parent dir writability/ownership. If sticky and not owned, STOP with guidance.
  - Include this check in `policy::gating::gating_errors` alongside `check_immutable`.

## Missing source at apply time

- Behavior:
  - `src/preflight/checks.rs::check_source_trust()` reads metadata of `source` and fails if missing or untrusted (unless `force_untrusted_source=true`). This prevents creating a symlink to a nonexistent binary by default.
- Recommendation:
  - Keep default strictness. If users need to stage links ahead of source arrival, allow via `force_untrusted_source`, but audit heavily: add a preflight row and `apply.result` field noting `source_missing=true`.

## Symlink loops and very long paths

- Behavior:
  - Idempotence uses best-effort normalization: `canonicalize()` is attempted; on error (e.g., symlink loop or permission error), code compares unresolved paths.
  - `symlinkat` and `renameat` will fail with `ENAMETOOLONG` for extremely long names.
- Recommendation:
  - `doctor links` should flag and refuse to auto-fix looped symlinks.
  - Emit a targeted error when `ENAMETOOLONG` occurs, with a remediation hint to shorten path components.

## Overlay/union filesystems and container layers

- Behavior:
  - Cross-layer/union mounts frequently surface as `EXDEV`. `src/fs/atomic.rs::atomic_symlink_swap()` already degrades (unlink+symlink) when `allow_degraded_fs=true`.
- Recommendation:
  - For critical system paths keep `allow_degraded_fs=false` and fail fast with `E_EXDEV`.
  - For ephemeral environments (e.g., containers), allow degraded mode but run smoke tests post-apply and log `degraded_reason="exdev_fallback"` (already emitted by handlers).

## Backup timestamp collisions and clock skew

- Behavior:
  - Backup filenames use `SystemTime::now().as_millis()` in `backup_path_with_tag()`. On very fast successive operations or skewed clocks, collisions are unlikely but possible.
- Recommendation:
  - If collisions are observed in practice, consider adding a tie-breaker (PID or a monotonic counter) into the filename. Today, callers remove any preexisting backup of the same name before re-creating.

## Sidecar tampering/corruption

- Behavior:
  - Sidecar JSON is trusted when present; parse errors are treated as "no sidecar" and we fall back to legacy rename when a payload exists.
- Recommendation:
  - Strengthen provenance by embedding a hash of the payload (and recording it in the sidecar). On restore, verify the hash before trusting fields like `prior_kind` and `prior_dest`.
  - Optionally sign sidecars as part of an attestation flow; store verification results in audit facts.

## Quick references (files)

- `src/policy/config.rs` — Policy fields (rescue, lock, preservation, degraded fs, backups, preflight override, roots/forbid, etc.)
- `src/fs/backup.rs` — create_snapshot, sidecar IO, latest/previous discovery
- `src/fs/restore.rs` — restore_file, restore_file_prev, idempotence checks
- `src/fs/swap.rs` — replace_file_with_symlink (snapshots then atomic swap / EXDEV fallback)
- `src/preflight/checks.rs` — ensure_mount_rw_exec, check_immutable, check_source_trust
- `src/api/apply/mod.rs` — locking, policy gating, smoke tests, rollback behavior
- `src/api/preflight/mod.rs` — per-action gating rows and summary facts

## Round 1 Peer Review (AI 4, 2025-09-12 15:16 CET)

- **Claims Verified:**
  - Multiple experiments with different policies can be managed by creating separate `Switchyard` instances with distinct `Policy` configurations. Cited `src/policy/config.rs` for policy fields and presets.
  - Package manager updates overwriting targets are handled by `restore_file()` which checks `prior_kind` and restores symlink topology. Cited `src/fs/restore.rs: restore_file()`.
  - Concurrency within Switchyard processes can use `LockManager` for serialization. Cited `src/adapters/lock/file.rs` for `FileLockManager` implementation.
  - Cross-filesystem moves (EXDEV) behavior is controlled by `Policy.allow_degraded_fs`. Cited `src/policy/config.rs: allow_degraded_fs` and `src/fs/swap.rs: replace_file_with_symlink()`.
- **Key Citations:**
  - `src/policy/config.rs`: Policy structure and presets.
  - `src/fs/restore.rs: restore_file()`: Restore logic and idempotence checks.
  - `src/fs/swap.rs: replace_file_with_symlink()`: Atomic swap and EXDEV fallback.
  - `src/adapters/lock/file.rs`: LockManager implementation for concurrency control.
- **Summary of Edits:**
  - Added specific code citations to support claims about policy management, restore behavior, concurrency, and cross-filesystem operations.
  - Clarified the behavior of `restore_file()` with respect to missing backups and the `force_restore_best_effort` flag.
  - No major content changes were necessary as the claims aligned well with the codebase.

Reviewed and updated in Round 1 by AI 4 on 2025-09-12 15:16 CET

## Round 2 Gap Analysis (AI 3, 2025-09-12 15:33+02:00)

- Invariant: Hardlink relationships are not broken without warning.
- Assumption (from doc): The document recommends a preflight check to detect hardlinks (`nlink > 1`) and stop the operation unless forced, warning the user that the link will be broken (`EDGE_CASES_AND_BEHAVIOR.md:146-153`).
- Reality (evidence): A search for `nlink` usage within `cargo/switchyard/src/preflight/checks.rs` shows no such check is implemented. The current implementation will break hardlinks silently when restoring a file, as `src/fs/restore.rs` uses a `renameat` operation which creates a new inode.
- Gap: The recommended preflight check to prevent silent breaking of hardlinks is missing from the codebase.
- Mitigations: Implement the proposed preflight check in `src/preflight/checks.rs` to detect `nlink > 1` on a target file. Add a policy flag (e.g., `allow_hardlink_break`) to control the behavior and ensure the check is logged as a preflight row.
- Impacted users: System administrators and users who rely on hardlinks for space efficiency or file management, as their file relationships can be broken without their knowledge.
- Follow-ups: This gap should be flagged for severity scoring in Round 3 and an implementation plan should be created in Round 4.

- Invariant: Backup artifacts are tamper-resistant.
- Assumption (from doc): The document recommends strengthening sidecar provenance by embedding a hash of the backup payload and verifying it on restore to prevent tampering (`EDGE_CASES_AND_BEHAVIOR.md:196-202`).
- Reality (evidence): The `BackupSidecar` struct defined in `cargo/switchyard/src/fs/backup.rs:245` does not contain a field for a payload hash. The `read_sidecar` and `restore_file` functions in `cargo/switchyard/src/fs/restore.rs` trust the sidecar content if the file is present and parses correctly, with no integrity verification.
- Gap: The sidecar and its associated backup payload are not cryptographically verified, making them vulnerable to tampering. An attacker could modify the `prior_kind` or `prior_dest` in the sidecar to alter restore behavior.
- Mitigations: Add a `payload_hash` field (e.g., SHA256) to the `BackupSidecar` struct. Update `create_snapshot` in `backup.rs` to compute and store the hash of the payload. Update `restore_file` in `restore.rs` to verify the hash before trusting the sidecar's contents, failing with a new error type (e.g., `E_INTEGRITY`) if the hash mismatches.
- Impacted users: Users in security-sensitive environments where the integrity of system file backups and restore operations is critical.
- Follow-ups: Flag for severity scoring in Round 3. This is a security enhancement that should be prioritized in the implementation plan for Round 4.

Gap analysis in Round 2 by AI 3 on 2025-09-12 15:33+02:00

## Round 3 Severity Assessment (AI 2, 2025-09-12 15:45+02:00)

- **Title:** Missing hardlink breakage preflight check
- **Category:** Missing Feature
- **Impact:** 3  **Likelihood:** 3  **Confidence:** 5  → **Priority:** 2  **Severity:** S3
- **Disposition:** Implement  **LHF:** No
- **Feasibility:** High  **Complexity:** 2
- **Why update vs why not:** Silent hardlink breakage can cause data duplication, break backup systems, and violate user expectations. Low implementation complexity with clear user value. Cost of inaction is silent corruption of file management strategies.
- **Evidence:** `src/fs/restore.rs` uses `renameat` creating new inodes; no `nlink > 1` check in `src/preflight/checks.rs`
- **Next step:** Add `check_hardlink_hazard` to preflight checks with policy knob `allow_hardlink_breakage`

- **Title:** Backup sidecar tampering vulnerability  
- **Category:** Bug/Defect (Security)
- **Impact:** 4  **Likelihood:** 2  **Confidence:** 4  → **Priority:** 2  **Severity:** S3
- **Why update vs why not:** Tampering with sidecars could alter restore behavior, creating security risks in sensitive environments. However, requires local filesystem access. Integrity verification adds robust security layer with minimal performance cost.
- **Evidence:** `BackupSidecar` struct in `src/fs/backup.rs:245` lacks hash field; `restore_file` trusts sidecar content without verification
- **Disposition:** Implement  **LHF:** No
- **Feasibility:** Medium  **Complexity:** 3
- **Next step:** Add `payload_hash` field to sidecar schema and implement verification in restore logic

Severity assessed in Round 3 by AI 2 on 2025-09-12 15:45+02:00
