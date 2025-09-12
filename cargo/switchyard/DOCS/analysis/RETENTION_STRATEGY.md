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

## Round 2 Gap Analysis (AI 2, 2025-09-12 15:23 CEST)

- **Invariant:** Backup availability for rollback operations
- **Assumption (from doc):** Consumers expect backups to remain available for rollback indefinitely until explicitly pruned
- **Reality (evidence):** No retention enforcement exists at `src/fs/backup.rs`; backups accumulate until manual cleanup; discovery helpers at `src/fs/backup.rs:L25-L316` find latest/previous backups by timestamp but don't manage lifecycle
- **Gap:** Unmanaged backup accumulation could fill filesystems; consumers have no predictable retention behavior or automated cleanup
- **Mitigations:** Implement proposed `prune_backups` function with policy knobs; add default retention warnings when backup count exceeds thresholds
- **Impacted users:** Long-running services and automation that perform frequent switching operations
- **Follow-ups:** Implement retention policy fields in Policy struct; add filesystem usage monitoring

- **Invariant:** Consistent backup discovery across operations
- **Assumption (from doc):** Latest and previous backup discovery provides reliable restore targets
- **Reality (evidence):** Discovery helpers `find_latest_backup_and_sidecar` and `find_previous_backup_and_sidecar` implemented with timestamp-based ordering; however, no validation ensures sidecar-payload pairs remain intact during retention operations
- **Gap:** Retention operations could orphan sidecars or payloads if not implemented atomically; consumers may encounter inconsistent backup state
- **Mitigations:** Implement atomic pair cleanup in retention operations; add integrity validation before backup operations
- **Impacted users:** Operations teams relying on backup integrity for disaster recovery
- **Follow-ups:** Add backup integrity validation; implement atomic pair operations in prune_backups

- **Invariant:** Safe retention operations respect filesystem boundaries
- **Assumption (from doc):** Proposed retention uses `open_dir_nofollow` and validates candidates before deletion
- **Reality (evidence):** Safety approach documented using `open_dir_nofollow(parent)` and pattern validation; however, no implementation exists yet to validate these safety claims
- **Gap:** Without implementation, safety claims remain unverified; consumers may encounter race conditions or path traversal issues
- **Mitigations:** Implement retention with documented safety patterns; add comprehensive testing for edge cases and concurrent operations
- **Impacted users:** Security-conscious environments requiring verified path safety
- **Follow-ups:** Implement prune_backups with safety patterns; add security-focused testing

Gap analysis in Round 2 by AI 2 on 2025-09-12 15:23 CEST

## Round 3 Severity Assessment (AI 1, 2025-09-12 15:44 +02:00)

- Title: No retention enforcement → risk of disk fill and degraded performance
  - Category: Missing Feature
  - Impact: 4  Likelihood: 3  Confidence: 4  → Priority: 3  Severity: S2
  - Disposition: Implement  LHF: No
  - Feasibility: High  Complexity: 3
  - Why update vs why not: Unbounded backup accumulation can exhaust storage and slow discovery; consumers expect predictable retention controls.
  - Evidence: No retention logic in `src/fs/backup.rs`; helpers only discover: `find_latest_backup_and_sidecar` (L277–316), `find_previous_backup_and_sidecar` (L25–65).
  - Next step: Implement `prune_backups(target, tag, policy)` with `Policy` knobs `retention_keep_last_n`, `retention_max_age_days`, `retention_max_total_bytes`; add unit tests and docs.

- Title: Pruning may orphan payload/sidecar pairs without atomic pair operations
  - Category: Bug/Defect
  - Impact: 3  Likelihood: 2  Confidence: 3  → Priority: 2  Severity: S3
  - Disposition: Implement  LHF: No
  - Feasibility: High  Complexity: 3
  - Why update vs why not: Orphaned artifacts lead to restore failures; pairwise deletion prevents inconsistent state.
  - Evidence: Pair discovery split across files; no prune implementation guaranteeing atomic pair handling.
  - Next step: In `prune_backups`, validate both payload and sidecar exist before deletion; delete both via dirfd + `unlinkat` with parent `fsync`; add integrity check tests.

- Title: Safety invariants for retention operations not enforced
  - Category: Missing Feature
  - Impact: 3  Likelihood: 2  Confidence: 4  → Priority: 2  Severity: S3
  - Disposition: Implement  LHF: Yes
  - Feasibility: High  Complexity: 2
  - Why update vs why not: Operating via directory handles and pattern validation is necessary to preserve TOCTOU safety during pruning.
  - Evidence: Safety guidance listed in this doc; no code exists yet.
  - Next step: Implement retention using `open_dir_nofollow(parent)` and `unlinkat` with strict filename pattern validation; add tests for traversal attempts.

Severity assessed in Round 3 by AI 1 on 2025-09-12 15:44 +02:00
