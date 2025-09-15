# Switchyard Bug Log

## apply-enospc-backup-restore — Test environment limitation (Deferred)

- Date: 2025-09-14
- Status: ⏸ Deferred (harness limitation)
- Test: apply::enospc_backup_restore::e2e_apply_014_enospc_during_backup_restore_path (tests/apply/enospc_backup_restore.rs:62)
- Repro: `cargo test -p switchyard --all-features -- --exact apply::enospc_backup_restore::e2e_apply_014_enospc_during_backup_restore_path`
- Failure: Cannot simulate ENOSPC (no space left on device) in test environment
- Suspected Root Cause: The test requires special filesystem setup to simulate disk full conditions which is not available in the test environment
- Blocked Because: requires test infrastructure changes to simulate ENOSPC conditions
- Files Likely Involved:
  - cargo/switchyard/tests/apply/enospc_backup_restore.rs
- Recommended Action (test-only):
  - Create a mock adapter or feature-gated path to simulate ENOSPC conditions; otherwise keep scenario as documentation-only.
- Evidence:
  - SPEC/DOCS refs: SPEC/SPEC.md §2.8 Error Handling (REQ-EH1, REQ-EH2)
  - Code cites: tests/apply/enospc_backup_restore.rs:82-84

Updated: 2025-09-16
