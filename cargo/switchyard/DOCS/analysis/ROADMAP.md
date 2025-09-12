# Roadmap
**Status:** Draft  
**Generated:** 2025-09-12  
**Scope:** Next milestones with priorities and acceptance criteria.  
**Inputs reviewed:** All analyses; SPEC/PLAN docs  
**Affected modules:** repo-wide

## Round 1 Peer Review (AI 3, 2025-09-12 15:14 CEST)

**Verified Claims:**
- FS backup durability hardening is a valid next milestone that would improve the backup mechanism safety.
- Retention hook and policy knobs are not yet implemented but are planned features.
- Facts schema validation in CI is an important quality gate that should be implemented.
- Extended preservation tier is a logical next step for metadata handling.
- Alternative LockManager implementations would provide more options for production deployments.
- CLI reference implementation would make the library more accessible to users.

**Citations:**
- `src/fs/backup.rs` - Current backup implementation
- `src/fs/restore.rs` - Current restore implementation
- `src/fs/atomic.rs` - Current atomic operations implementation
- `src/policy/config.rs` - Policy configuration structure
- `src/adapters/lock/file.rs` - Current FileLockManager implementation
- `SPEC/audit_event.schema.json` - Audit event schema for validation

**Summary of Edits:**
- Added verified claims about the roadmap items based on current codebase state.
- Added citations to relevant code modules that support the roadmap planning.
- Added a Round 1 Peer Review section with verification details.

Reviewed and updated in Round 1 by AI 3 on 2025-09-12 15:14 CEST

## Milestones
1. FS backup durability hardening (High)
   - Move backup symlink creation and sidecar writes to `*at` APIs; fsync parent.
   - Accept: All backup paths use dirfd-based ops; new tests added.

2. Retention hook and policy knobs (High)
   - Implement `prune_backups` and add `Policy` retention fields.
   - Accept: Unit tests reclaim expected bytes and preserve newest pair.

3. Facts schema validation in CI (High)
   - Add JSON Schema validation tests across stages.
   - Accept: All emitted facts validate against `SPEC/audit_event.schema.json`.

4. Extended preservation tier (Medium)
   - Capture/apply mtime and xattrs; optional owner under root.
   - Accept: Round-trip tests on tmpfs.

5. Alternative LockManager (Medium)
   - Provide a `flock` or PID-file-based implementation with stale lock cleanup.
   - Accept: Integration test demonstrates bounded wait and cleanup.

6. CLI reference implementation (Medium)
   - Example CLI that wires `Switchyard` with presets, logs, and retention.
   - Accept: README walkthrough succeeds and passes e2e demo.

## References
- See corresponding analysis docs for details.
