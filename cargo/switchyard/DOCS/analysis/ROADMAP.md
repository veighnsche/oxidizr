# Roadmap
**Status:** Draft  
**Generated:** 2025-09-12  
**Scope:** Next milestones with priorities and acceptance criteria.  
**Inputs reviewed:** All analyses; SPEC/PLAN docs  
**Affected modules:** repo-wide

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
