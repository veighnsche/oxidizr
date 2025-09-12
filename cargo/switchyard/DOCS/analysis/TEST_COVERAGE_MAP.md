# Test Coverage Map
**Status:** Draft  
**Generated:** 2025-09-12  
**Scope:** Map unit/integration tests to features; identify gaps and propose a prioritized test backlog.  
**Inputs reviewed:** CODE tests under `src/**`; SPEC §8 Acceptance Tests; PLAN/80-testing-mapping.md  
**Affected modules:** `fs/*`, `api/*`, `logging/*`, `policy/*`, `preflight/*`

## Round 1 Peer Review (AI 3, 2025-09-12 15:13 CEST)

**Verified Claims:**
- Core operations are tested with in-module unit tests: swap round-trip, restore idempotence, mount checks, redaction.
- The existing tests cover the basic functionality described in the document.
- Test functions are properly named and organized in the codebase.
- The fsync timing bounds are tested as mentioned in PLAN/55-operational-bounds.md.

**Citations:**
- `src/fs/swap.rs:L149-L167` - `atomic_swap_creates_symlink_pointing_to_source` test
- `src/fs/swap.rs:L169-L200` - `replace_and_restore_roundtrip` test
- `src/fs/restore.rs:L234-L253` - `symlink_topology_restore_roundtrip` test
- `src/fs/restore.rs:L255-L265` - `none_topology_restore_removes_target` test
- `src/fs/restore.rs:L267-L277` - `idempotent_restore_file_twice_is_noop` test
- `src/fs/backup.rs:L76-L86` - `snapshot_file_creates_payload_and_sidecar` test
- `src/fs/backup.rs:L88-L101` - `snapshot_symlink_creates_symlink_backup_and_sidecar` test
- `src/fs/backup.rs:L103-L111` - `snapshot_none_creates_tombstone_and_sidecar` test
- `src/fs/mount.rs:L95-L104` - `ensure_rw_exec_*` tests
- `src/logging/redact.rs:L40-L53` - `redact_masks_and_removes_expected_fields` test
- `src/api.rs:L173-L192` - `emits_minimal_facts_for_plan_preflight_apply` test
- `src/api.rs:L194-L217` - `rollback_reverts_first_action_on_second_failure` test

**Summary of Edits:**
- Added verified claims about existing test coverage.
- Added citations to specific test functions in the codebase.
- Added a Round 1 Peer Review section with verification details.

Reviewed and updated in Round 1 by AI 3 on 2025-09-12 15:13 CEST

## Summary
- Core operations are tested with in-module unit tests: swap round-trip, restore idempotence, mount checks, redaction.
- Gaps remain for EXDEV degraded path, sidecar corruption handling, immutable bit detection, and schema validation of facts.

## Inventory / Findings
- Existing tests (examples)
  - `fs/swap.rs`:
    - `atomic_swap_creates_symlink_pointing_to_source`
    - `replace_and_restore_roundtrip`
  - `fs/restore.rs`:
    - `symlink_topology_restore_roundtrip`
    - `none_topology_restore_removes_target`
    - `idempotent_restore_file_twice_is_noop`
  - `fs/backup.rs`:
    - `snapshot_file_creates_payload_and_sidecar`
    - `snapshot_symlink_creates_symlink_backup_and_sidecar`
    - `snapshot_none_creates_tombstone_and_sidecar`
  - `fs/mount.rs`:
    - `ensure_rw_exec_*` trio
  - `logging/redact.rs`:
    - `redact_masks_and_removes_expected_fields`
  - `api.rs` (facade):
    - `emits_minimal_facts_for_plan_preflight_apply`
    - `rollback_reverts_first_action_on_second_failure`

- Gaps
  - EXDEV degraded path: Use `SWITCHYARD_FORCE_EXDEV=1` to simulate and assert `degraded=true` with `degraded_reason="exdev_fallback"` when allowed; assert failure with `error_id=E_EXDEV` when disallowed.
  - Immutable bit: Preflight `check_immutable` path depends on `lsattr`; add test that stubs command or marks xfail in environments without `lsattr`.
  - Sidecar corruption: Malformed/missing sidecar paths already covered partially; add tests for malformed JSON and mismatch handling.
  - Facts schema validation: Validate emitted facts against `SPEC/audit_event.schema.json`.
  - Rescue tooling: Add tests around `SWITCHYARD_FORCE_RESCUE_OK` to ensure gating behavior.
  - Ownership gating: Add tests verifying `strict_ownership` behavior with a mock `OwnershipOracle` returning errors.
  - Retention: Tests for future `prune_backups` hook (see RETENTION_STRATEGY.md).

## Recommendations (Backlog)
1. Add EXDEV degraded-mode tests (allowed vs disallowed).
2. Add JSON Schema validation for facts across all stages.
3. Add sidecar corruption tests for file and symlink prior_kind.
4. Add rescue profile gating tests using env overrides.
5. Add ownership gating tests with a mock oracle.
6. Add retention pruning tests once implemented.

## Acceptance Criteria
- New tests added; CI runs them deterministically (use tmpfs where possible and feature-gated env overrides).
- Facts schema validation passes for all emitted events in unit tests.

## References
- SPEC: §8 Acceptance Tests; `SPEC/audit_event.schema.json`
- PLAN: 80-testing-mapping.md
- CODE: tests listed above
