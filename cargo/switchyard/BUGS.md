# Switchyard Bug Log

## apply-attestation-fields-present — Needs multi-file/architectural fix

- Date: 2025-09-14
- Test: apply::attestation_apply_success::attestation_fields_present_on_success_and_masked_after_redaction (tests/apply/attestation_apply_success.rs:71)
- Repro: `cargo test -p switchyard --all-features -- --exact apply::attestation_apply_success::attestation_fields_present_on_success_and_masked_after_redaction`
- Failure: attestation fields missing from apply.result success event
- Suspected Root Cause: The apply summary logic is not properly including attestation fields in successful apply events when an attestor is configured
- Blocked Because: requires changes across api/apply/summary.rs and possibly api/apply/mod.rs to properly emit attestation fields
- Files Likely Involved:
  - cargo/switchyard/src/api/apply/summary.rs
  - cargo/switchyard/src/api/apply/mod.rs
  - cargo/switchyard/src/adapters/attest.rs
- Potential Direction (non-binding):
  - Option A: Ensure the attestation method in ApplySummary is called for successful commits
  - Option B: Add attestation emission directly in the apply result logic
- Evidence:
  - SPEC/DOCS refs: SPEC/SPEC.md §2.4 Observability & Audit (REQ-O4)
  - Code cites: src/api/apply/summary.rs:68-91, src/adapters/attest.rs:37-53

## apply-attestation-error-tolerated — Needs multi-file/architectural fix

- Date: 2025-09-14
- Test: apply::attestation_error_tolerated::attestation_error_is_tolerated_and_omitted (tests/apply/attestation_error_tolerated.rs:68)
- Repro: `cargo test -p switchyard --all-features -- --exact apply::attestation_error_tolerated::attestation_error_is_tolerated_and_omitted`
- Failure: apply should succeed when attestation signing fails (error tolerance)
- Suspected Root Cause: The apply logic is not properly handling attestation errors as optional failures that should not stop execution
- Blocked Because: requires changes across api/apply/summary.rs and api/apply/mod.rs to properly handle attestation errors
- Files Likely Involved:
  - cargo/switchyard/src/api/apply/summary.rs
  - cargo/switchyard/src/api/apply/mod.rs
  - cargo/switchyard/src/adapters/attest.rs
- Potential Direction (non-binding):
  - Option A: Make attestation building error handling more robust
  - Option B: Ensure attestation failures don't propagate to apply result errors
- Evidence:
  - SPEC/DOCS refs: SPEC/SPEC.md §2.4 Observability & Audit (REQ-O4)
  - Code cites: src/api/apply/summary.rs:68-91, src/adapters/attest.rs:39-42

## apply-enospc-backup-restore — Test environment limitation

- Date: 2025-09-14
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

## apply-exdev-fallback-disallowed — Test environment limitation

- Date: 2025-09-14

## sprint-acceptance-schema-validation — Needs investigation

- Date: 2025-09-14

## trybuild-compile-fail — Test environment limitation

- Date: 2025-09-14
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
- Test: sprint_acceptance-0001::golden_two_action_plan_preflight_apply (tests/sprint_acceptance-0001.rs:226)
- Repro: `cargo test -p switchyard --all-features -- --exact sprint_acceptance-0001::golden_two_action_plan_preflight_apply`
- Failure: JSON schema validation fails for preflight events - missing required properties (path, current_kind, planned_kind)
- Suspected Root Cause: The preflight event emission is missing required fields according to the audit schema
- Blocked Because: requires investigation of audit event schema compliance
- Files Likely Involved:
  - cargo/switchyard/tests/sprint_acceptance-0001.rs
  - cargo/switchyard/src/preflight/mod.rs
  - cargo/switchyard/SPEC/audit_event.v2.schema.json
- Recommended Action:
  - Check that all required fields are properly included in preflight audit events
  - Update event emission to comply with the JSON schema
- Evidence:
  - Error messages show missing "path", "current_kind", and "planned_kind" properties
  - SPEC/DOCS refs: SPEC/audit_event.v2.schema.json

- Date: 2025-09-14
- Test: apply::error_exdev::ensure_symlink_emits_e_exdev_when_fallback_disallowed (tests/apply/error_exdev.rs:54)
- Repro: `cargo test -p switchyard --all-features -- --exact apply::error_exdev::ensure_symlink_emits_e_exdev_when_fallback_disallowed`
- Failure: Cannot properly simulate EXDEV (cross-filesystem link) error in test environment
- Suspected Root Cause: The SWITCHYARD_FORCE_EXDEV environment variable is not being properly handled or the test environment doesn't support cross-filesystem operations
- Blocked Because: requires test infrastructure changes to properly simulate EXDEV conditions
- Files Likely Involved:
  - cargo/switchyard/tests/apply/error_exdev.rs
  - cargo/switchyard/src/fs/swap.rs
- Recommended Action (test-only):
  - Add proper EXDEV simulation in test environments
  - Create mock filesystem operations that can trigger EXDEV errors
- Evidence:
  - SPEC/DOCS refs: SPEC/SPEC.md §2.7 Cross-Filesystem Operations (REQ-F2)
  - Code cites: tests/apply/error_exdev.rs:53-55

- Date: 2025-09-14
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

## Recently Resolved (2025-09-14)

- provenance-completeness — passing: `requirements::provenance_completeness::req_o7_provenance_completeness`
- preflight-rescue-verification — passing: `preflight::baseline_ok::e2e_preflight_004_rescue_not_required_ok`
- preflight-backup-tag-handling — passing: `preflight::baseline_ok::e2e_preflight_009_empty_backup_tag_ok`
- preflight-exec-check-handling — passing: `preflight::baseline_ok::e2e_preflight_010_exec_check_disabled_ok`
- preflight-coreutils-tag-handling — passing: `preflight::baseline_ok::e2e_preflight_011_coreutils_tag_ok`
- preflight-mount-check-notes — passing: `preflight::extra_mount_checks_five::e2e_preflight_006_extra_mount_checks_five`, `preflight::extra_mount_checks_many::extra_mount_checks_many_emit_notes`
- lockmanager-required-production — passing: `requirements::lockmanager_required_production::req_l4_lockmanager_required_production`
- partial-restoration-facts — passing: `requirements::partial_restoration_facts::req_r5_partial_restoration_facts`
- smoke-invariants — passing under Option A (facts-only): `oracles::smoke_invariants::smoke_invariants`

## provenance-completeness — Needs multi-file/architectural fix

- Status: Resolved on 2025-09-14 — Verified passing: `requirements::provenance_completeness::req_o7_provenance_completeness` (entry was stale).
- Date: 2025-09-14
- Repro: `cargo test -p switchyard --all-features -- --exact requirements::provenance_completeness::req_o7_provenance_completeness`
- Failure: provenance should include uid field
- Suspected Root Cause: The FsOwnershipOracle sets pkg to an empty string, but the test expects actual package information. This requires integration with package managers which is an architectural change.
- Blocked Because: requires changes across adapters/ownership and integration with package manager APIs
- Files Likely Involved:
  - cargo/switchyard/src/adapters/ownership/fs.rs
  - cargo/switchyard/src/adapters/ownership/mod.rs
  - cargo/switchyard/src/types/ownership.rs
- Potential Direction (non-binding):
  - Option A: Integrate with package manager APIs (pacman, apt, etc.) to determine actual package ownership
  - Option B: Create a mock package information provider for testing purposes
- Evidence:
  - SPEC/DOCS refs: SPEC/SPEC.md §2.4 Observability & Audit (REQ-O7)
  - SPEC/DOCS refs: SPEC/audit_event.v2.schema.json provenance definition
  - Code cites: src/adapters/ownership/fs.rs:23

## preflight-rescue-verification — Needs multi-file/architectural fix

- Status: Resolved on 2025-09-14 — Verified passing: `preflight::baseline_ok::e2e_preflight_004_rescue_not_required_ok`.
- Date: 2025-09-14
- Repro: `cargo test -p switchyard --all-features -- --exact preflight::baseline_ok::e2e_preflight_004_rescue_not_required_ok`
- Failure: preflight should succeed when rescue not required
- Suspected Root Cause: The rescue verification logic is not properly handling the case when rescue is not required by policy
- Blocked Because: requires changes across policy/rescue.rs and api/preflight/mod.rs
- Files Likely Involved:
  - cargo/switchyard/src/policy/rescue.rs
  - cargo/switchyard/src/api/preflight/mod.rs
- Potential Direction (non-binding):
  - Option A: Modify rescue verification to properly handle the not-required case
  - Option B: Update preflight logic to skip rescue checks when not required
- Evidence:
  - SPEC/DOCS refs: SPEC/SPEC.md §2.6 Rescue (REQ-RC1, REQ-RC2, REQ-RC3)
  - Code cites: src/policy/rescue.rs:44-82, src/api/preflight/mod.rs:48-55

## preflight-backup-tag-handling — Needs multi-file/architectural fix

- Status: Resolved on 2025-09-14 — Verified passing: `preflight::baseline_ok::e2e_preflight_009_empty_backup_tag_ok`.
- Date: 2025-09-14
- Repro: `cargo test -p switchyard --all-features -- --exact preflight::baseline_ok::e2e_preflight_009_empty_backup_tag_ok`
- Failure: preflight should succeed with empty backup tag
- Suspected Root Cause: The preflight logic doesn't properly handle empty backup tags
- Blocked Because: requires changes across policy/backup.rs and api/preflight/mod.rs
- Files Likely Involved:
  - cargo/switchyard/src/policy/backup.rs
  - cargo/switchyard/src/api/preflight/mod.rs
- Potential Direction (non-binding):
  - Option A: Modify backup tag handling to properly handle empty tags
  - Option B: Update preflight logic to skip backup tag validation when empty
- Evidence:
  - SPEC/DOCS refs: SPEC/SPEC.md §2.3 Safety Preconditions (REQ-S1 through REQ-S6)
  - Code cites: src/api/preflight/mod.rs:128-131

## preflight-exec-check-handling — Needs multi-file/architectural fix

- Status: Resolved on 2025-09-14 — Verified passing: `preflight::baseline_ok::e2e_preflight_010_exec_check_disabled_ok`.
- Date: 2025-09-14
- Repro: `cargo test -p switchyard --all-features -- --exact preflight::baseline_ok::e2e_preflight_010_exec_check_disabled_ok`
- Failure: preflight should succeed with exec_check disabled
- Suspected Root Cause: The preflight logic doesn't properly handle disabled exec checks
- Blocked Because: requires changes across policy/rescue.rs and api/preflight/mod.rs
- Files Likely Involved:
  - cargo/switchyard/src/policy/rescue.rs
  - cargo/switchyard/src/api/preflight/mod.rs
- Potential Direction (non-binding):
  - Option A: Modify rescue verification to properly handle disabled exec checks
  - Option B: Update preflight logic to skip exec check validation when disabled
- Evidence:
  - SPEC/DOCS refs: SPEC/SPEC.md §2.6 Rescue (REQ-RC1, REQ-RC2, REQ-RC3)
  - Code cites: src/policy/rescue.rs:44-82, src/api/preflight/mod.rs:48-55

## preflight-coreutils-tag-handling — Needs multi-file/architectural fix

- Status: Resolved on 2025-09-14 — Verified passing: `preflight::baseline_ok::e2e_preflight_011_coreutils_tag_ok`.
- Date: 2025-09-14
- Repro: `cargo test -p switchyard --all-features -- --exact preflight::baseline_ok::e2e_preflight_011_coreutils_tag_ok`
- Failure: preflight should succeed with coreutils tag
- Suspected Root Cause: The preflight logic doesn't properly handle coreutils backup tags
- Blocked Because: requires changes across policy/backup.rs and api/preflight/mod.rs
- Files Likely Involved:
  - cargo/switchyard/src/policy/backup.rs
  - cargo/switchyard/src/api/preflight/mod.rs
- Potential Direction (non-binding):
  - Option A: Modify backup tag handling to properly handle coreutils tags
  - Option B: Update preflight logic to properly validate coreutils tags
- Evidence:
  - SPEC/DOCS refs: SPEC/SPEC.md §2.3 Safety Preconditions (REQ-S1 through REQ-S6)
  - Code cites: src/api/preflight/mod.rs:130

## preflight-mount-check-notes — Needs multi-file/architectural fix

- Status: Resolved on 2025-09-14 — Verified passing: `preflight::extra_mount_checks_five::e2e_preflight_006_extra_mount_checks_five` and `preflight::extra_mount_checks_many::extra_mount_checks_many_emit_notes`.
- Date: 2025-09-14
- Repro: `cargo test -p switchyard --all-features -- --exact preflight::extra_mount_checks_five::e2e_preflight_006_extra_mount_checks_five`
- Failure: preflight rows should contain mount check notes
- Suspected Root Cause: The preflight mount check logic is not properly emitting notes
- Blocked Because: requires changes across policy/mount.rs and api/preflight/row_emitter.rs
- Files Likely Involved:
  - cargo/switchyard/src/policy/mount.rs
  - cargo/switchyard/src/api/preflight/row_emitter.rs
- Potential Direction (non-binding):
  - Option A: Modify mount check logic to properly emit notes
  - Option B: Update row emitter to include mount check notes
- Evidence:
  - SPEC/DOCS refs: SPEC/SPEC.md §2.3 Safety Preconditions (REQ-S2)
  - Code cites: src/api/preflight/row_emitter.rs

## lockmanager-required-production — Needs multi-file/architectural fix

- Status: Resolved on 2025-09-14 — Verified passing: `requirements::lockmanager_required_production::req_l4_lockmanager_required_production`.
- Date: 2025-09-14
- Test Ignored: requirements::lockmanager_required_production::req_l4_lockmanager_required_production (tests/requirements/lockmanager_required_production.rs:28)
- Repro: `cargo test -p switchyard --all-features -- --exact requirements::lockmanager_required_production::req_l4_lockmanager_required_production`
- Failure: commit should fail when locking is required but no manager is configured
- Suspected Root Cause: The apply logic is not properly enforcing the lock manager requirement in production mode
- Blocked Because: requires changes across api/apply/lock.rs and policy/locking.rs
- Files Likely Involved:
  - cargo/switchyard/src/api/apply/lock.rs
  - cargo/switchyard/src/policy/locking.rs
- Potential Direction (non-binding):
  - Option A: Modify lock acquisition logic to fail when required but not configured
  - Option B: Update policy enforcement to properly check for lock manager in production
- Evidence:
  - SPEC/DOCS refs: SPEC/SPEC.md §2.5 Locking (REQ-L4)
  - Code cites: src/api/apply/lock.rs

## partial-restoration-facts — Needs multi-file/architectural fix

- Status: Resolved on 2025-09-14 — Verified passing: `requirements::partial_restoration_facts::req_r5_partial_restoration_facts`.
- Date: 2025-09-14
- Test Ignored: requirements::partial_restoration_facts::req_r5_partial_restoration_facts (tests/requirements/partial_restoration_facts.rs:27)
- Repro: `cargo test -p switchyard --all-features -- --exact requirements::partial_restoration_facts::req_r5_partial_restoration_facts`
- Failure: rollback operations should emit facts
- Suspected Root Cause: The rollback logic is not properly emitting facts when partial restoration occurs
- Blocked Because: requires changes across api/apply/rollback.rs and api/apply/summary.rs
- Files Likely Involved:
  - cargo/switchyard/src/api/apply/rollback.rs
  - cargo/switchyard/src/api/apply/summary.rs
- Potential Direction (non-binding):
  - Option A: Modify rollback logic to emit facts on partial restoration
  - Option B: Update summary emission to include partial restoration information
- Evidence:
  - SPEC/DOCS refs: SPEC/SPEC.md §2.2 Rollback (REQ-R5)
  - Code cites: src/api/apply/rollback.rs, src/api/apply/summary.rs

## smoke-invariants — Needs multi-file/architectural fix

- Status: Resolved on 2025-09-14 — Adopted Option A (facts-only) behavior. Verified passing: `oracles::smoke_invariants::smoke_invariants`.
- Date: 2025-09-14
- Test Ignored: oracles::smoke_invariants::smoke_invariants (tests/oracles/smoke_invariants.rs:39)
- Repro: `cargo test -p switchyard --all-features -- --exact oracles::smoke_invariants::smoke_invariants`
- Failure: apply should fail when smoke test fails
- Suspected Root Cause: The smoke test integration is not properly triggering failures
- Blocked Because: requires changes across adapters/smoke.rs and api/apply/mod.rs
- Files Likely Involved:
  - cargo/switchyard/src/adapters/smoke.rs
  - cargo/switchyard/src/api/apply/mod.rs
- Potential Direction (non-binding):
  - Option A: Modify smoke test adapter to properly report failures
  - Option B: Update apply logic to properly handle smoke test failures
- Evidence:
  - SPEC/DOCS refs: SPEC/SPEC.md §2.9 Health Verification (REQ-H1, REQ-H2, REQ-H3)
  - Code cites: src/adapters/smoke.rs, src/api/apply/mod.rs:134-164

## environment-base4-runner-long-path — Test harness limitation (ENAMETOOLONG)

- Date: 2025-09-14
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
