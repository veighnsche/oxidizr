# Switchyard Bug Log

## apply-enospc-backup-restore — Test environment limitation

- Date: 2025-09-14
- Status: ⬜ Needs work
- Test: apply::enospc_backup_restore::e2e_apply_014_enospc_during_backup_restore_path (tests/apply/enospc_backup_restore.rs:62)
- Repro: `cargo test -p switchyard --all-features -- --exact apply::enospc_backup_restore::e2e_apply_014_enospc_during_backup_restore_path`
- Failure: Cannot simulate ENOSPC (no space left on device) in test environment
- Suspected Root Cause: The test requires special filesystem setup to simulate disk full conditions which is not available in the test environment
- Blocked Because: requires test infrastructure changes to simulate ENOSPC conditions
- Files Likely Involved:
  - cargo/switchyard/tests/apply/enospc_backup_restore.rs
- Recommended Action (test-only):
  - Add conditional compilation or feature flag to skip ENOSPC simulation in test environments
  - Create a mock filesystem adapter that can simulate ENOSPC conditions
  - Use a test helper to inject disk space errors
- Evidence:
  - SPEC/DOCS refs: SPEC/SPEC.md §2.8 Error Handling (REQ-EH1, REQ-EH2)
  - Code cites: tests/apply/enospc_backup_restore.rs:82-84

## apply-ownership-strict-with-oracle — Needs multi-file/architectural fix

- Date: 2025-09-14
- Status: ⬜ Needs work

## trybuild-compile-fail — Test environment limitation

- Date: 2025-09-14
- Status: ⬜ Needs work
- Test: trybuild::compile_fail_on_atom_imports (tests/trybuild.rs:2)
- Repro: `cargo test -p switchyard --all-features -- --exact trybuild::compile_fail_on_atom_imports`
- Failure: trybuild test error messages don't exactly match expectations
- Suspected Root Cause: The compiler error message format has changed slightly but the functionality is working correctly
- Blocked Because: requires updating expected error messages in trybuild tests
- Files Likely Involved:
  - cargo/switchyard/tests/trybuild.rs
  - cargo/switchyard/tests/trybuild/mutate_with_raw_path.rs
- Recommended Action (test-only):
  - Update expected error messages to match current compiler output
- Evidence:
  - The test correctly fails compilation when using raw paths instead of SafePath
  - Error message format differs slightly from expected but functionality is correct

## sprint-acceptance-schema-validation — Needs investigation

- Date: 2025-09-14
- Status: ⬜ Needs work
- Test: apply::ownership_strict_with_oracle::e2e_apply_017_ownership_strict_with_oracle_present (tests/apply/ownership_strict_with_oracle.rs:64)
- Repro: `cargo test -p switchyard --all-features -- --exact apply::ownership_strict_with_oracle::e2e_apply_017_ownership_strict_with_oracle_present`
- Failure: provenance information missing from apply.result success event
- Suspected Root Cause: The FsOwnershipOracle sets pkg to an empty string, but the test expects actual package information. This requires integration with package managers which is an architectural change.
- Blocked Because: requires changes across adapters/ownership/fs.rs and integration with package manager APIs
- Files Likely Involved:
  - cargo/switchyard/src/adapters/ownership/fs.rs
  - cargo/switchyard/src/types/ownership.rs
  - cargo/switchyard/src/api/apply/summary.rs
- Potential Direction (non-binding):
  - Option A: Integrate with package manager APIs (pacman, apt, etc.) to determine actual package ownership
  - Option B: Create a mock package information provider for testing purposes
- Evidence:
  - SPEC/DOCS refs: SPEC/SPEC.md §2.4 Observability & Audit (REQ-O7)
  - SPEC/DOCS refs: SPEC/SPEC.md §2.3 Safety Preconditions (REQ-S4)
  - Code cites: src/adapters/ownership/fs.rs:23

## environment-base4-runner-long-path — Test harness limitation (ENAMETOOLONG)

- Date: 2025-09-14
- Status: ⬜ Needs work
- Repro: `cargo test -p switchyard -- --exact environment::base4_runner::envrunner_base4_weekly_platinum`
- Failure: `Os { code: 36, kind: InvalidFilename, message: "File name too long" }`
- Suspected Root Cause: The test constructs an extremely long path segment (~2000 chars). Even in DryRun mode, any attempt to actually create such a path on the host filesystem will fail with `ENAMETOOLONG`. This is a test harness limitation, not a runtime Switchyard defect.
- Files Likely Involved:
  - cargo/switchyard/tests/environment/base4_runner.rs
- Recommended Action (test-only):
  - Avoid real filesystem creation for overlong segments in this test.
  - Use `ApplyMode::DryRun` exclusively and assert on planned actions and emitted facts instead of touching the filesystem.
  - If path existence is needed, shorten the generated segment to within platform limits (e.g., <=255 bytes per component) or mock path probing.
  - Optionally guard long-path branches behind a feature flag for CI environments with stricter limits.
-
  - Evidence:
    - POSIX and common Linux filesystems limit filename components to 255 bytes.
    - The current test creates a single component exceeding this bound, guaranteeing ENAMETOOLONG.