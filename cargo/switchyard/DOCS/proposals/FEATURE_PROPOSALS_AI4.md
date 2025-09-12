# Feature Proposals — AI 4

Generated: 2025-09-12 16:34 CET
Author: AI 4

## Feature 1: Enforce SafePath in Core FS Paths with Deprecation Plan (Implement Now)

- Problem statement: Core filesystem operations currently accept raw `&Path` inputs, risking path traversal and TOCTOU vulnerabilities by bypassing `SafePath` safety checks. Identified as S2 Priority 3 in EDGE_CASES_AND_BEHAVIOR.md and CLI_INTEGRATION_GUIDE.md Round 3 assessments.
- User story(s):
  - As a security-conscious developer, I want all filesystem operations to enforce `SafePath` validation, so that my application is protected from path traversal attacks.
- Design overview:
  - APIs (new/changed, signatures): Modify core FS functions in `src/fs/` (e.g., `swap.rs`, `restore.rs`, `backup.rs`) to accept `SafePath` instead of `&Path` for mutating operations.
  - Behavior (normal, error, degraded): Normal operation validates paths via `SafePath`, rejecting traversal attempts with `E_TRAVERSAL`. Error cases return detailed messages. Degraded mode not applicable as safety is non-negotiable.
  - Telemetry/facts (new fields/events): Add `safepath_validation=success|failure` to `apply.attempt` facts in `src/api/apply/mod.rs`.
  - Policy flags/defaults: No policy flag; `SafePath` enforcement is mandatory with deprecation of raw `Path` variants.
  - Docs changes (where and what): Update `CLI_INTEGRATION_GUIDE.md` with `SafePath` usage examples and deprecation notice for raw `Path` APIs; update `SPEC §3 Public Interfaces` to mandate `SafePath`.
- Scope (files/functions):
  - `src/fs/swap.rs`, `src/fs/restore.rs`, `src/fs/backup.rs` - Core mutating functions.
  - `src/api/apply/handlers.rs` - API boundary conversion from raw input to `SafePath`.
- Tests:
  - Unit: Test each FS function rejects invalid `SafePath` inputs with `E_TRAVERSAL`.
  - Integration: Simulate CLI usage with invalid paths, verifying error propagation.
  - Compile-fail: Ensure deprecated raw `Path` variants trigger warnings.
- Feasibility: Medium
- Complexity: 4
- Effort: M
- Risks and mitigations:
  - Risk: Breaking change for existing library users. Mitigation: Deprecate raw `Path` variants with a one-minor-release window, providing clear migration guidance in docs.
- Dependencies:
  - SPEC Proposal: Enhanced CLI Integration Guidance in SPEC for SafePath Usage (Proposal 3 in SPEC_CHANGES_AI4.md).
- Rollout plan:
  - Branching: Implement in a feature branch with deprecation warnings first.
  - Flags: No flags; enforcement is mandatory post-deprecation.
  - Deprecations: Mark raw `Path` functions as `#[deprecated]` in current release; remove in next minor.
  - Dual-support windows: One minor release with dual support (warnings on raw `Path`).
- Acceptance criteria:
  - All core FS mutating functions accept only `SafePath` or validate raw inputs early.
  - Deprecated raw `Path` variants emit compile-time warnings with migration guidance.
  - Tests cover traversal attempts and error handling.
  - Documentation reflects `SafePath` enforcement and deprecation timeline.
- Evidence:
  - Code/spec citations: `src/fs/swap.rs`, `src/fs/restore.rs`, `src/fs/backup.rs` (accept raw `&Path`), `src/types/safepath.rs` (SafePath definition).
  - Analysis: DOCS/analysis/EDGE_CASES_AND_BEHAVIOR.md, DOCS/analysis/CLI_INTEGRATION_GUIDE.md Round 3 assessments.

## Feature 2: Implement SUID/SGID Check and Policy Knob with STOP Preflight Row (Implement Now)

- Problem statement: Modifying SUID/SGID binaries without explicit checks risks privilege escalation, a significant security concern. Identified as S3 Priority 2 in CORE_FEATURES_FOR_EDGE_CASES.md Round 3 assessment.
- User story(s):
  - As a system administrator, I want operations on SUID/SGID binaries to be blocked by default, so that accidental privilege escalation is prevented.
- Design overview:
  - APIs (new/changed, signatures): Add `check_suid_sgid_risk()` in `src/preflight/checks.rs`.
  - Behavior (normal, error, degraded): Normal behavior detects SUID/SGID bits via `stat()` and emits STOP row if policy disallows. Error case is detection failure (logged as warning). Degraded mode not applicable.
  - Telemetry/facts (new fields/events): Add `suid_sgid_risk=true|false` to preflight row facts in `src/api/preflight/mod.rs`.
  - Policy flags/defaults: Add `allow_suid_sgid_mutation` (default `false` in `production_preset` and `coreutils_switch_preset`) in `src/policy/config.rs`.
  - Docs changes (where and what): Update `CORE_FEATURES_FOR_EDGE_CASES.md` to document SUID/SGID protection; add policy usage in `SPEC §2.5 Safety and Conservatism`.
- Scope (files/functions):
  - `src/preflight/checks.rs` - New check function.
  - `src/policy/config.rs` - Policy flag.
  - `src/api/preflight/mod.rs` - Integrate into preflight pipeline.
- Tests:
  - Unit: Test detection of SUID/SGID bits on mock files in `src/preflight/checks.rs`.
  - Integration: Simulate operation on SUID binary, ensuring preflight fails unless policy allows.
- Feasibility: High
- Complexity: 1
- Effort: S
- Risks and mitigations:
  - Risk: False positives blocking legitimate operations. Mitigation: Provide policy override and clear error messages.
- Dependencies:
  - SPEC Proposal: SUID/SGID Preflight Gate with Default Deny and Override Flag (Proposal 1 in SPEC_CHANGES_AI4.md).
- Rollout plan:
  - Branching: Implement in main branch as additive feature.
  - Flags: Policy flag `allow_suid_sgid_mutation` for opt-in.
  - Deprecations: None.
  - Dual-support windows: Not needed; immediate rollout.
- Acceptance criteria:
  - SUID/SGID check implemented and integrated into preflight.
  - Policy flag defaults to `false` in production presets.
  - STOP row emitted when SUID/SGID detected and policy disallows.
  - Tests verify detection and policy behavior.
- Evidence:
  - Code/spec citations: `src/preflight/checks.rs` (current checks lack SUID/SGID logic).
  - Analysis: DOCS/analysis/CORE_FEATURES_FOR_EDGE_CASES.md Round 3 assessment.

## Feature 3: Backup Sidecar Integrity with Payload Hash Verification

- Problem statement: Lack of integrity checks for backup sidecars allows tampering, undermining restore reliability and security. Identified as S3 Priority 2 in EDGE_CASES_AND_BEHAVIOR.md Round 3 assessment.
- User story(s):
  - As an operator, I want backup integrity verified during restore, so that tampered data is not applied to my system.
- Design overview:
  - APIs (new/changed, signatures): Extend `BackupSidecar` struct in `src/fs/backup.rs` with `payload_hash` field.
  - Behavior (normal, error, degraded): Compute hash during backup creation; verify during restore. Fail with `E_BACKUP_TAMPERED` if hash mismatches and policy requires integrity. Degraded mode skips verification if policy allows.
  - Telemetry/facts (new fields/events): Add `sidecar_integrity_verified=true|false` to restore facts in `src/api/apply/mod.rs`.
  - Policy flags/defaults: Add `require_sidecar_integrity` (default `true` in `production_preset`) in `src/policy/config.rs`.
  - Docs changes (where and what): Update `EDGE_CASES_AND_BEHAVIOR.md` for tampering scenarios; document policy in `SPEC §2.6 Backup and Restore`.
- Scope (files/functions):
  - `src/fs/backup.rs::BackupSidecar` and `write_sidecar()` - Add hash field and computation.
  - `src/fs/restore.rs::restore_file()` - Verify hash before restore.
  - `src/policy/config.rs` - Policy flag.
- Tests:
  - Unit: Test hash computation/storage in backup; simulate tampering in restore with failure.
  - Integration: Full backup-restore cycle with integrity check.
- Feasibility: Medium
- Complexity: 3
- Effort: M
- Risks and mitigations:
  - Risk: Performance overhead for large backups. Mitigation: Use fast hash (SHA-256); make optional via policy.
- Dependencies:
  - SPEC Proposal: Sidecar Integrity with Payload Hash and Restore Verification (Proposal 2 in SPEC_CHANGES_AI4.md).
- Rollout plan:
  - Branching: Feature branch for schema v2 implementation.
  - Flags: Policy flag for optional integrity check.
  - Deprecations: None.
  - Dual-support windows: v1 and v2 sidecars coexist; v2 includes hash.
- Acceptance criteria:
  - `BackupSidecar` v2 includes `payload_hash`.
  - Backup computes and stores hash.
  - Restore verifies hash, failing if policy requires integrity.
  - Policy defaults to `true` in production.
  - Tests cover tampering and policy scenarios.
- Evidence:
  - Code/spec citations: `src/fs/backup.rs::BackupSidecar` (lines 244–252), `src/fs/restore.rs::restore_file()` (lines 96–107).
  - Analysis: DOCS/analysis/EDGE_CASES_AND_BEHAVIOR.md Round 3 assessment.

## Feature 4: Hardlink Breakage Preflight Check and Policy Control

- Problem statement: Silent hardlink breakage during operations can cause data duplication and break backup systems, violating user expectations. Identified as S3 Priority 2 in EDGE_CASES_AND_BEHAVIOR.md Round 3 assessment.
- User story(s):
  - As a system administrator, I want to be warned about hardlink breakage risks, so that I can prevent unintended data duplication or backup issues.
- Design overview:
  - APIs (new/changed, signatures): Add `check_hardlink_hazard()` in `src/preflight/checks.rs`.
  - Behavior (normal, error, degraded): Check if target has `nlink > 1` via `stat()`; emit STOP row if policy disallows breakage. Error case logs warning if check fails. Degraded mode not applicable.
  - Telemetry/facts (new fields/events): Add `hardlink_risk=true|false` to preflight row facts in `src/api/preflight/mod.rs`.
  - Policy flags/defaults: Add `allow_hardlink_breakage` (default `false` in `production_preset`) in `src/policy/config.rs`.
  - Docs changes (where and what): Update `EDGE_CASES_AND_BEHAVIOR.md` to document hardlink protection; add policy in `SPEC §2.5 Safety and Conservatism`.
- Scope (files/functions):
  - `src/preflight/checks.rs` - New check function.
  - `src/policy/config.rs` - Policy flag.
  - `src/api/preflight/mod.rs` - Integrate into preflight pipeline.
- Tests:
  - Unit: Test detection of hardlinks on mock files in `src/preflight/checks.rs`.
  - Integration: Simulate operation on hardlinked file, ensuring preflight fails unless policy allows.
- Feasibility: High
- Complexity: 2
- Effort: S
- Risks and mitigations:
  - Risk: False positives blocking valid operations. Mitigation: Policy override and clear error messaging.
- Dependencies:
  - None.
- Rollout plan:
  - Branching: Implement in main branch as additive feature.
  - Flags: Policy flag `allow_hardlink_breakage` for opt-in.
  - Deprecations: None.
  - Dual-support windows: Not needed; immediate rollout.
- Acceptance criteria:
  - Hardlink check implemented and integrated into preflight.
  - Policy flag defaults to `false` in production presets.
  - STOP row emitted when hardlink detected and policy disallows.
  - Tests verify detection and policy behavior.
- Evidence:
  - Code/spec citations: `src/preflight/checks.rs`, `src/fs/restore.rs` (uses `renameat` creating new inodes).
  - Analysis: DOCS/analysis/EDGE_CASES_AND_BEHAVIOR.md Round 3 assessment.

---

Proposals authored by AI 4 on 2025-09-12 16:34 CET
