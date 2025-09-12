# Retention & GC Strategy

**Status:** Draft  
**Generated:** 2025-09-12  
**Scope:** Define default retention controls for backups/sidecars (per target/tag keep N, max age, total size caps). Propose library hooks and CLI integration for pruning.  
**Inputs reviewed:** SPEC §2 (conservatism, recovery), SPEC §5 (facts), PLAN/55-operational-bounds.md, CODE: `src/fs/backup.rs`, `src/fs/restore.rs`, `src/constants.rs`  
**Affected modules:** `fs/backup.rs`, `fs/restore.rs`, `policy/config.rs`

## Round 1 Peer Review (AI 3, 2025-09-12 15:13 CEST)

**Verified Claims:**
- Backups and sidecars accumulate per target in the parent directory with tag names `.<name>.<tag>.<millis>.bak` and sidecar `...meta.json`.
- Discovery helpers `find_latest_backup_and_sidecar` and `find_previous_backup_and_sidecar` already enumerate by timestamp prefix.
- No retention enforcement exists today; artifacts are only pruned implicitly on restore.
- The naming scheme is implemented in `backup_path_with_tag` function.

**Citations:**
- `src/fs/backup.rs:L11-L23` - `backup_path_with_tag` implementation
- `src/fs/backup.rs:L277-L316` - `find_latest_backup_and_sidecar` implementation
- `src/fs/backup.rs:L25-L65` - `find_previous_backup_and_sidecar` implementation
- `src/constants.rs:L8-L12` - `TMP_SUFFIX` constant
- `src/fs/backup.rs:L244-L252` - `BackupSidecar` struct

**Summary of Edits:**
- Added verified claims about backup naming and discovery mechanisms.
- Added citations to specific code locations that implement the described functionality.
- Added a Round 1 Peer Review section with verification details.

Reviewed and updated in Round 1 by AI 3 on 2025-09-12 15:13 CEST

## Summary

- Backups and sidecars accumulate per target in the parent directory with tag names `.<name>.<tag>.<millis>.bak` and sidecar `...meta.json`.
- Provide a retention policy that can be enforced by libraries or a CLI: keep last N per `(target, tag)`, enforce max age, and optionally total-size caps per directory.
- Implement pruning as a best-effort, non-destructive operation guarded by SafePath checks and capability-style `open_dir_nofollow` handles.

## Inventory / Findings

- Naming: `backup_path_with_tag(target, tag)` produces timestamped artifacts (monotonic increasing millis within a directory).
- Discovery helpers: `find_latest_backup_and_sidecar`, `find_previous_backup_and_sidecar` already enumerate by timestamp prefix.
- No retention enforcement exists today; artifacts are only pruned implicitly on restore (payload removal under some paths).

## Recommendations

1. Policy knobs
   - Add optional retention knobs on `Policy`:
     - `retention_keep_last_n: Option<usize>` (default None)
     - `retention_max_age_days: Option<u64>` (default None)
     - `retention_max_total_bytes: Option<u64>` (default None)
   - These are advisory; pruning is safe to run anytime and should never delete the most recent pair.

2. Library hook
   - Add `fs::backup::prune_backups(target: &Path, tag: &str, policy: &Policy) -> io::Result<PruneReport>` that:
     - Lists all pairs for `(target, tag)` sorted by timestamp.
     - Preserves the newest pair unconditionally.
     - Deletes older pairs beyond `keep_last_n` using `unlinkat`/`remove_file` and removes sidecars.
     - Applies `max_age_days` and `max_total_bytes` thresholds conservatively.
     - Returns counts and reclaimed bytes in `PruneReport`.

3. CLI helper (downstream)
   - Provide `switchyard prune --root <R> --tag <T> [--keep N] [--max-age DAYS] [--max-size BYTES]` that calls the library hook for all known targets under `<R>`.

4. Safety
   - Use `open_dir_nofollow(parent)` and operate on names relative to the directory handle.
   - Validate that candidates match `.<name>.<tag>.<millis>.bak` pattern before deleting.

## Risks & Trade-offs

- Aggressive pruning could remove operator-needed artifacts; mitigate by defaulting to keep at least one and requiring explicit flags for size/age limits.

## Spec/Docs deltas

- Add a section to SPEC §9 (Operational) documenting retention knobs and safe pruning semantics.

## Acceptance Criteria

- `prune_backups` implemented and tested; never deletes the newest pair.
- Unit tests cover `keep_last_n`, `max_age_days`, and `max_total_bytes` limit behavior.

## References

- SPEC: §9 Operational Bounds
- PLAN: 55-operational-bounds.md
- CODE: `src/fs/backup.rs`, `src/fs/restore.rs`
