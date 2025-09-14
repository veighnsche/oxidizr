# Switchyard Bug Log

## provenance-completeness — Needs multi-file/architectural fix

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
