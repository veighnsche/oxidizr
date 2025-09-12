# Filesystem Operations Safety Audit

**Status:** Draft  
**Generated:** 2025-09-12  
**Scope:** Review of all mutating filesystem operations for TOCTOU safety, correct open*at/renameat/fsync ordering, EXDEV degraded paths, and remaining gaps.  
**Inputs reviewed:** SPEC §2.1 (Atomicity), §2.3 (Safety Preconditions), §2.10 (Filesystems & Degraded), PLAN/60-rollback-exdev.md, CODE: `src/fs/{atomic,swap,restore,backup,meta,mount,paths}.rs`, `src/api/apply/*`, `src/preflight/checks.rs`  
**Affected modules:** `src/fs/**`, `src/api/apply/**`, `src/preflight/checks.rs`

## Summary

- The core swap path follows the correct sequence: `open_dir_nofollow(parent) → symlinkat(tmp) → renameat(tmp→final) → fsync(parent)`. EXDEV fallback also calls `fsync_parent_dir()`.
- Remove/cleanup of existing targets uses `unlinkat(dirfd, name)` via a capability-style directory handle, avoiding path traversal.
- Restore paths use `renameat` with `open_dir_nofollow(parent)` and `fchmod` to restore mode; directory syncs are applied after rename.
- Gaps: symlink-backup creation and sidecar writes use path-based APIs without a final `fsync(parent)`; recommend migrating to *at forms and fsyncing the backup parent dir.

## Inventory / Findings

- Core atomic swap (`src/fs/atomic.rs`)
  - `open_dir_nofollow(parent)` uses `rustix::fs::openat(CWD, …, OFlags::DIRECTORY|NOFOLLOW|CLOEXEC)`.
  - `symlinkat(src, dirfd, tmp)` creates a temporary link.
  - `renameat(dirfd,tmp → dirfd,fname)` performs the atomic replacement.
  - `fsync_parent_dir(target)` is called post-rename (both normal and EXDEV degraded paths).
  - EXDEV simulation guard via `SWITCHYARD_FORCE_EXDEV` present for tests.

- Swap orchestration (`src/fs/swap.rs`)
  - Validates `is_safe_path` for `source` and `target` prior to mutation.
  - On existing symlink targets, resolves current dest for change detection; then snapshots and removes via `unlinkat(dirfd,fname)` and calls `atomic_symlink_swap`.
  - On regular file targets, snapshots current state, removes via `unlinkat` with a pre-opened parent dirfd, then calls `atomic_symlink_swap`.
  - Ensures parent directories exist (`create_dir_all(parent)`) for new paths.

- Backup and sidecar (`src/fs/backup.rs`)
  - File snapshot path uses `open_dir_nofollow(parent)` and *at syscalls (`openat`, `fchmod`), followed by copying bytes; however, it does not explicitly `fsync(parent)` after creating the backup file or sidecar.
  - Symlink snapshot path creates a symlink backup using `std::os::unix::fs::symlink(curr, &backup)` (path-based) and writes the sidecar with `std::fs::File::create(sc_path)` (path-based). Parent directory is not `fsync`'d afterward.
  - Tombstone backups (`prior_kind=none`) are created via path-based create without a `fsync(parent)`.

- Restore logic (`src/fs/restore.rs`)
  - Reads sidecar; short-circuits idempotently when the current state already matches the prior topology (symlink/file/none).
  - With sidecar
    - `prior_kind=file`: uses `renameat(dirfd, backup → dirfd, target)` and then `fchmod` if mode present; `fsync_parent_dir(target)` called.
    - `prior_kind=symlink`: uses `atomic_symlink_swap(src, target, allow_degraded=true)` and `fsync_parent_dir(target)`.
    - `prior_kind=none`: uses `unlinkat(dirfd,fname)` or `remove_file` and `fsync_parent_dir(target)`; removes payload if present.
  - No sidecar: legacy `renameat` path with parent `fsync` performed.

- Mount inspection and preflight checks
  - `fs/mount.rs` provides `ProcStatfsInspector` and `ensure_rw_exec()`. `preflight/checks.rs` wraps this and adds immutable and source trust checks.

## Recommendations

1. Backup symlink path hardening
   - Replace `std::os::unix::fs::symlink(curr, &backup)` with `symlinkat(curr, dirfd, name)` using a `dirfd` from `open_dir_nofollow(parent)`.
   - After creating the backup payload and the sidecar, call `fsync(dirfd)` or reuse `fsync_parent_dir(backup)`.

2. Sidecar writes durability
   - For all sidecar writes (`write_sidecar()`), follow with `fsync_parent_dir(backup)` in addition to file `sync_all()` if desired. Consider writing to a temp and `renameat` into place for atomic sidecar updates.

3. Consistent *at usage in backup paths
   - Ensure all backup/tombstone create/remove operations use `openat`/`unlinkat`/`renameat` against a pre-opened parent dirfd; eliminate remaining path-based fallbacks.

4. Provide capability wrappers
   - Introduce a small internal helper module (e.g., `fs/cap.rs`) with:
     - `open_parent_nofollow(path) -> OwnedFd`
     - `unlink_name(dirfd, name)`
     - `symlink_name(dirfd, name, src)`
     - `rename_name(dirfd, old, new)`
     - `fsync_dirfd(dirfd)`
   - Migrate `backup.rs` and `restore.rs` call sites to these helpers for uniformity.

5. Green list (safe swap checklist)
   - Inputs normalized to `SafePath` and validated with `is_safe_path()`.
   - Pre-open parent with `O_DIRECTORY|O_NOFOLLOW` into a capability handle.
   - Perform all mutations via `*at` calls relative to the capability handle.
   - `renameat` for atomic visibility; on EXDEV allowed by policy, use a documented degraded path and emit telemetry.
   - `fsync(parent)` after each visibility change (rename/create/unlink).
   - Idempotent short-circuits when no topology change is needed.

## Risks & Trade-offs

- Adding parent `fsync` for backups may introduce small latency; acceptable for safety and consistency.
- Using `symlinkat` in more places slightly increases complexity; mitigated by helper wrappers.

## Spec/Docs deltas

- SPEC §2.1/§2.10: Clarify that backup payload and sidecar creation are also followed by a parent directory fsync and should use `*at` forms to avoid TOCTOU.
- PLAN/60-rollback-exdev.md: Add note that backups/sidecars follow the same atomicity/durability rules.

## Acceptance Criteria

- All backup and sidecar creation paths use `open_dir_nofollow` + `*at` syscalls and fsync the parent directory.
- Unit tests demonstrate durability for backups (e.g., crash-after-backup scenario simulated by immediate process exit still leaves valid artifacts).
- No remaining path-based mutations in `fs/backup.rs`.

## References

- SPEC: §2.1 Atomicity; §2.3 Safety Preconditions; §2.10 Filesystems & Degraded
- PLAN: 60-rollback-exdev.md; 10-types-traits.md
- CODE: `src/fs/atomic.rs`, `src/fs/swap.rs`, `src/fs/restore.rs`, `src/fs/backup.rs`, `src/fs/meta.rs`, `src/fs/mount.rs`

## Actions/PR checklist

- [ ] Migrate symlink backup creation to `symlinkat` with `open_dir_nofollow(parent)`.
- [ ] Add `fsync_parent_dir(backup)` after writing backup and sidecar.
- [ ] Introduce `fs/cap.rs` helpers and refactor `backup.rs` usages.
- [ ] Add unit tests asserting directory fsync after backups (time-bound checks may be coarse).

## Round 1 Peer Review (AI 1, 2025-09-12 15:09 +02:00)

- Claims verified
  - Atomic swap sequence uses directory handles and *at syscalls, then fsyncs parent.
    - Proof: `src/fs/atomic.rs::atomic_symlink_swap()` calls `open_dir_nofollow()`, `symlinkat()`, `renameat()`, then `fsync_parent_dir()` lines 56–96.
  - Existing target cleanup uses `unlinkat(dirfd,name)` via capability handle.
    - Proof: `src/fs/swap.rs::replace_file_with_symlink()` uses `unlinkat` at lines 70–81 and 125–133 after `open_dir_nofollow(parent)`.
  - Restore paths use `renameat` with pre-opened parent, restore mode via `fchmod`, and fsync parent directory.
    - Proof: `src/fs/restore.rs::restore_file()` uses `renameat` lines 126–127, `fchmod` lines 134–137, and `fsync_parent_dir()` lines 139–140; similar in other branches (e.g., lines 171–174, 223–225, 259–261).
  - EXDEV degraded path emits fsync and is gated by policy/env.
    - Proof: `src/fs/atomic.rs::atomic_symlink_swap()` EXDEV branch at lines 86–93 calls `fsync_parent_dir(target)`; test knob `SWITCHYARD_FORCE_EXDEV` at lines 74–76.
  - Backup symlink creation and sidecar writes currently use path-based APIs without parent fsync.
    - Proof: `src/fs/backup.rs::create_snapshot()` symlink backups use `std::os::unix::fs::symlink` (lines 137–139); `write_sidecar()` creates file path-based (lines 262–270) and no `fsync_parent_dir` call; tombstone path uses `File::create` (lines 218–231) with no fsync.

- Key citations
  - `src/fs/atomic.rs::{open_dir_nofollow, fsync_parent_dir, atomic_symlink_swap}`
  - `src/fs/swap.rs::replace_file_with_symlink`
  - `src/fs/restore.rs::restore_file`
  - `src/fs/backup.rs::{create_snapshot, write_sidecar}`

- Summary of edits
  - Added precise code citations for each claim and confirmed behavior against current implementation. Highlighted backup/sidecar durability gap and kept recommendations aligned with findings.

Reviewed and updated in Round 1 by AI 1 on 2025-09-12 15:09 +02:00
