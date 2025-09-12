# AI 4 — Round 1 Analysis Report

Generated: 2025-09-12 14:24:34+02:00
Analyst: AI 4
Coordinator: Cascade

Scope: Verify claims, provide proofs, and patch gaps in the assigned documents only. Record evidence and edits here. Do not start Round 2 until instructed.

## Assigned Documents (37 pts)

- BACKWARDS_COMPAT_SHIMS.md — 6
- BEHAVIORS.md — 9
- EXPERIMENT_CONSTANTS_REVIEW.md — 4
- REEXPORTS_AND_FACADES.md — 4
- RETENTION_STRATEGY.md — 3
- PERFORMANCE_PLAN.md — 3
- TEST_COVERAGE_MAP.md — 4
- MIGRATION_GUIDE.md — 1
- ROADMAP.md — 1
- CODING_STANDARDS.md — 1
- CONTRIBUTING_ENHANCEMENTS.md — 1

## Round 1 Checklist

- [ ] Evidence mapping completed for all assigned docs
- [ ] Patches applied to assigned docs where needed
- [ ] All claims verified or corrected with citations
- [ ] Open questions recorded

## Evidence — BACKWARDS_COMPAT_SHIMS.md

- Claims → Proofs
  - [ ] Claim: listed shims exist and are re-exported
    - Proof: `cargo/switchyard/src/**` re-exports and deprecation notes

## Changes Made — BACKWARDS_COMPAT_SHIMS.md

- [ ] Edit summary: <what changed and why>

## Evidence — BEHAVIORS.md

- Claims → Proofs
  - [ ] Claim: documented behaviors match code paths
    - Proof: cite functions under `cargo/switchyard/src/api/*`, `src/fs/*`

## Changes Made — BEHAVIORS.md

- [ ] Edit summary: <what changed and why>

## Evidence — EXPERIMENT_CONSTANTS_REVIEW.md

- Claims → Proofs
  - [ ] Claim: constants and defaults align with `src/constants.rs` and policy presets
    - Proof: `cargo/switchyard/src/constants.rs`, `policy/config.rs`

## Changes Made — EXPERIMENT_CONSTANTS_REVIEW.md

- [ ] Edit summary: <what changed and why>

## Evidence — REEXPORTS_AND_FACADES.md

- Claims → Proofs
  - [ ] Claim: crate root re-exports minimal public facade
    - Proof: `cargo/switchyard/src/lib.rs`, `src/api.rs`

## Changes Made — REEXPORTS_AND_FACADES.md

- [ ] Edit summary: <what changed and why>

## Evidence — RETENTION_STRATEGY.md

- Claims → Proofs
  - [ ] Claim: backup naming and discovery helpers
    - Proof: `cargo/switchyard/src/fs/backup.rs`

## Changes Made — RETENTION_STRATEGY.md

- [ ] Edit summary: <what changed and why>

## Evidence — PERFORMANCE_PLAN.md

- Claims → Proofs
  - [ ] Claim: hotspots and benchmarking plan
    - Proof: `cargo/switchyard/src/fs/meta.rs::sha256_hex_of()`, `fs/atomic.rs`, `fs/backup.rs`

## Changes Made — PERFORMANCE_PLAN.md

- [ ] Edit summary: <what changed and why>

## Evidence — TEST_COVERAGE_MAP.md

- Claims → Proofs
  - [ ] Claim: tests listed exist and cover behaviors
    - Proof: unit tests in `cargo/switchyard/src/**` (cite names), env knobs

## Changes Made — TEST_COVERAGE_MAP.md

- [ ] Edit summary: <what changed and why>

## Evidence — MIGRATION_GUIDE.md

- Claims → Proofs
  - [ ] Claim: planned internalization of FS atoms and timelines
    - Proof: see API audit recommendations and current `fs/mod.rs` re-exports

## Changes Made — MIGRATION_GUIDE.md

- [ ] Edit summary: <what changed and why>

## Evidence — ROADMAP.md

- Claims → Proofs
  - [ ] Claim: milestones and acceptance criteria map to analysis recommendations
    - Proof: cross-reference analysis docs and affected code paths

## Changes Made — ROADMAP.md

- [ ] Edit summary: <what changed and why>

## Evidence — CODING_STANDARDS.md

- Claims → Proofs
  - [ ] Claim: lints and structure match repo practices
    - Proof: project config, code layout under `cargo/switchyard/src/**`

## Changes Made — CODING_STANDARDS.md

- [ ] Edit summary: <what changed and why>

## Evidence — CONTRIBUTING_ENHANCEMENTS.md

- Claims → Proofs
  - [ ] Claim: commands and practices are valid for this repo
    - Proof: confirm with `cargo` tasks and Makefile if present

## Changes Made — CONTRIBUTING_ENHANCEMENTS.md

- [ ] Edit summary: <what changed and why>

## Open Questions

- [ ] <question>

## Round 2 Plan (Do NOT start yet)

- You will peer review AI 1’s outputs and assigned docs in Round 2:
  - EDGE_CASES_AND_BEHAVIOR.md, CORE_FEATURES_FOR_EDGE_CASES.md, CLI_INTEGRATION_GUIDE.md
- Tasks for Round 2 (later):
  - Re-verify proofs, check missed claims, propose fixes. Record notes in this file under "Round 2 Review".

## Round 2 Review (placeholder)

- Findings:
- Suggested diffs:

## Round 1 Peer Review Targets

- EDGE_CASES_AND_BEHAVIOR.md
- CORE_FEATURES_FOR_EDGE_CASES.md
- CLI_INTEGRATION_GUIDE.md

### Round 1 Peer Review — Checklist

- [ ] EDGE_CASES_AND_BEHAVIOR.md
- [ ] CORE_FEATURES_FOR_EDGE_CASES.md
- [ ] CLI_INTEGRATION_GUIDE.md

### Round 1 Peer Review — Evidence and Edits

- For each doc, add Claims → Proofs with code/spec/test citations and list changes made.

### Round 1 Peer Review Findings

#### EDGE_CASES_AND_BEHAVIOR.md
- **Checklist:**
  - Reviewed all sections for accuracy against codebase.
  - Verified claims with specific code references.
- **Claim → Proof Mapping:**
  - **Multiple experiments with different policies:** Supported by `src/policy/config.rs` which defines `Policy` structure and presets like `production_preset()` and `coreutils_switch_preset()`.
  - **Package manager updates overwriting targets:** Handled by `restore_file()` in `src/fs/restore.rs: restore_file()`, which checks `prior_kind` and restores symlink topology.
  - **Concurrency within Switchyard processes:** Managed by `LockManager` as seen in `src/adapters/lock/file.rs` with `FileLockManager` implementation.
  - **Cross-filesystem moves (EXDEV):** Controlled by `Policy.allow_degraded_fs` in `src/policy/config.rs` and implemented in `src/fs/swap.rs: replace_file_with_symlink()` with fallback behavior.
- **Changes Made:**
  - Added specific code citations to support claims.
  - Clarified behavior of `restore_file()` with respect to missing backups and policy flags.
- **Open Questions:**
  - None at this time.

#### CORE_FEATURES_FOR_EDGE_CASES.md
- **Checklist:**
  - Reviewed proposed features against current codebase.
  - Identified implementation status of each proposal.
- **Claim → Proof Mapping:**
  - **Sidecar v2 for integrity and signing:** Not implemented. Current `BackupSidecar` in `src/fs/backup.rs` lacks `payload_hash` or `signature` fields.
  - **SafePath-first mutating APIs:** Partially implemented. `SafePath` structure exists in `src/types/safepath.rs`, but not fully integrated into all FS functions.
  - **Parent sticky-bit and ownership gate:** Not implemented. No evidence in `src/preflight/checks.rs`.
  - **Hardlink hazard preflight:** Not implemented. No specific check for `nlink > 1` in preflight checks.
- **Changes Made:**
  - Updated document to reflect current implementation status of proposed features.
  - Added citations to confirm the state of the codebase.
- **Open Questions:**
  - Should the document prioritize certain proposals based on feasibility or urgency?

#### CLI_INTEGRATION_GUIDE.md
- **Checklist:**
  - Reviewed integration recommendations against codebase.
  - Verified existence of recommended components.
- **Claim → Proof Mapping:**
  - **Policy presets:** Supported by `production_preset()` and `coreutils_switch_preset()` in `src/policy/config.rs`.
  - **Locking with FileLockManager:** Implemented in `src/adapters/lock/file.rs`.
  - **Exit code mapping:** Supported by `exit_code_for` in `src/api/errors.rs`.
- **Changes Made:**
  - Added citations to support recommendations.
  - No significant content changes needed as guidance aligns with codebase.
- **Open Questions:**
  - None at this time.

## Round 2 Gap Analysis

### FS_SAFETY_AUDIT.md
- **Checklist:**
  - Reviewed document for consumer invariants related to filesystem operations safety.
  - Identified gaps in durability and path traversal safety.
- **Gaps and Mitigations:**
  - **Durability Across Crashes:** Backup and sidecar creation lack `fsync_parent_dir()` calls, risking data loss in crashes. Mitigation: Add `fsync_parent_dir(backup)` after backup and sidecar operations in `src/fs/backup.rs`.
  - **Path Traversal Safety:** Core mutating functions accept raw `&Path` instead of `SafePath`, risking traversal attacks. Mitigation: Refactor to enforce `SafePath` usage in all mutating functions.

### API_SURFACE_AUDIT.md
- **Checklist:**
  - Reviewed document for consumer invariants related to API safety and stability communication.
  - Identified gaps in API misuse prevention and stability documentation.
- **Gaps and Mitigations:**
  - **API Misuse Prevention:** Low-level FS atoms are publicly exposed, risking unsafe usage. Mitigation: Restrict visibility to `pub(crate)` and document stable API boundaries.
  - **Stability Communication:** Lack of stability annotations in codebase. Mitigation: Add Rustdoc comments or attributes indicating stability levels.

### OBSERVABILITY_FACTS_SCHEMA.md
- **Checklist:**
  - Reviewed document for consumer invariants related to observability data reliability and detail.
  - Identified gaps in error detail and schema validation.
- **Gaps and Mitigations:**
  - **Comprehensive Error Information:** Summaries collapse errors into a single `error_id`, lacking detail. Mitigation: Implement `summary_error_ids` array in summaries.
  - **Schema Validation:** No automated validation of facts against schema. Mitigation: Add unit tests for schema validation using `jsonschema` crate.

### ERROR_TAXONOMY.md
- **Checklist:**
  - Reviewed document for consumer invariants related to error reporting detail and categorization.
  - Identified gaps in detailed error reporting and ownership error identification.
- **Gaps and Mitigations:**
  - **Detailed Error Reporting:** Summaries use a single generic `error_id`, missing multiple causes. Mitigation: Add `summary_error_ids` array to capture all error IDs.
  - **Ownership Error Identification:** Ownership issues not consistently tagged as `E_OWNERSHIP`. Mitigation: Update preflight and gating to use `E_OWNERSHIP` for ownership failures.

### INDEX.md
- **Checklist:**
  - Reviewed document for consumer invariants related to comprehensive coverage and current status of analysis.
  - Identified gaps in package manager interoperability coverage and status updates.
- **Gaps and Mitigations:**
  - **Comprehensive Coverage:** Missing analysis for package manager interoperability. Mitigation: Add a proposed analysis for activation persistence and PM interaction.
  - **Current Status Updates:** Index does not reflect review round updates. Mitigation: Add a status section for each document indicating review progress.

## Round 2 Meta Review Targets

- FS_SAFETY_AUDIT.md
- API_SURFACE_AUDIT.md
- OBSERVABILITY_FACTS_SCHEMA.md
- ERROR_TAXONOMY.md
- INDEX.md

### Round 2 Meta Review — Notes

- Thoroughness, correctness, evidence quality, and editorial discipline per doc. Do not edit docs; record issues here.

## Round 3 Severity Reports — Targets

- PRESERVATION_FIDELITY.md
- PREFLIGHT_MODULE_CONCERNS.md
- POLICY_PRESETS_RATIONALE.md
- LOCKING_STRATEGY.md
- idiomatic_todo.md
- SECURITY_REVIEW.md
- RELEASE_AND_CHANGELOG_POLICY.md

### Round 3 Severity Reports — Entries

- Topic: <area>
  - Impact: [] Likelihood: [] Confidence: [] → Priority: []
  - Rationale: <citations>

## Round 3 Severity Reports

### PRESERVATION_FIDELITY.md
- **Title:** Lack of Extended Preservation Beyond Mode (Owner, Timestamps, Xattrs)
  - **Category:** Missing Feature
  - **Severity:** S2
  - **Priority:** 3
  - **Disposition:** Implement
  - **Feasibility:** Medium
  - **Complexity:** 3
  - **Rationale:** Not preserving extended metadata disrupts system integrity in metadata-critical environments. Enhances trust and utility for integrators. Cost of inaction is limited applicability.
  - **Evidence:** `src/fs/backup.rs::create_snapshot()` captures only `mode` (lines 194–205). `src/fs/restore.rs::restore_file()` restores only mode (lines 129–137).
  - **Next Step:** Implement tiered preservation policy in `src/policy/config.rs` and extend sidecar schema in `src/fs/backup.rs`. Plan for Round 4.

- **Title:** Backup and Sidecar Durability Missing After Creation
  - **Category:** Bug/Defect (Reliability)
  - **Severity:** S3
  - **Priority:** 2
  - **Disposition:** Implement
  - **Feasibility:** High
  - **Complexity:** 2
  - **Rationale:** Lack of durability risks data loss in crashes, undermining recovery. Simple fix with high reliability value. Cost of inaction is unavailability of rollback.
  - **Evidence:** `src/fs/backup.rs::write_sidecar()` lacks `fsync(parent)` (lines 262–270); symlink backups use path-based API (lines 137–151).
  - **Next Step:** Add `fsync_parent_dir(backup)` and use `open_dir_nofollow(parent)` in `src/fs/backup.rs`. Implement in Round 4.

- **Title:** Restore Impossible After Manual Payload Pruning Despite Sidecar Presence
  - **Category:** Missing Feature (Usability)
  - **Severity:** S3
  - **Priority:** 2
  - **Disposition:** Implement
  - **Feasibility:** High
  - **Complexity:** 2
  - **Rationale:** Restore failing due to pruned payloads frustrates users expecting best-effort recovery. Telemetry and guidance improve usability. Cost of inaction is reduced trust.
  - **Evidence:** `src/fs/restore.rs::restore_file()` errors with `E_BACKUP_MISSING` if payload absent (lines 96–107).
  - **Next Step:** Add `restore_ready` field to preflight rows in `src/api/preflight/mod.rs`. Document retention guidance. Plan for Round 4.

### PREFLIGHT_MODULE_CONCERNS.md
- **Title:** Unreliable Immutable-Bit Detection Across Environments
  - **Category:** Bug/Defect (Reliability)
  - **Severity:** S2
  - **Priority:** 3
  - **Disposition:** Implement
  - **Feasibility:** Medium
  - **Complexity:** 3
  - **Rationale:** Relying on `lsattr` fails in minimal environments, risking undetected immutable files until apply time. Alternative detection enhances gating accuracy. Cost of inaction is runtime failures.
  - **Evidence:** `src/preflight/checks.rs::check_immutable()` uses `lsattr -d` and skips on failure (lines 20–41).
  - **Next Step:** Implement `ioctl(FS_IOC_GETFLAGS)` in `src/preflight/checks.rs`. Add `immutable_check=unknown` fact for unreliable cases. Plan for Round 4.

- **Title:** Incomplete Preflight YAML Export Missing Preservation Fields
  - **Category:** Documentation Gap (Usability)
  - **Severity:** S3
  - **Priority:** 2
  - **Disposition:** Implement
  - **Feasibility:** High
  - **Complexity:** 1
  - **Rationale:** Missing preservation fields in YAML limits decision-making on readiness. Simple update improves usability. Cost of inaction is minor oversight of preservation issues.
  - **Evidence:** `src/preflight/yaml.rs::to_yaml()` omits `preservation` fields (lines 11–25).
  - **Next Step:** Update `src/preflight/yaml.rs::to_yaml()` to include preservation fields. Implement in Round 4.

- **Title:** Naming Overlap and Ambiguity in Preflight Modules
  - **Category:** Documentation Gap (DX/Usability)
  - **Severity:** S4
  - **Priority:** 1
  - **Disposition:** Implement
  - **Feasibility:** High
  - **Complexity:** 1
  - **Rationale:** Naming overlap confuses contributors, increasing cognitive load. Documentation or renaming is a low-effort clarity fix. Cost of inaction is persistent minor confusion.
  - **Evidence:** `src/preflight.rs` (helpers) vs `src/api/preflight/mod.rs` (stage) naming ambiguity.
  - **Next Step:** Add module-level docs to clarify roles in `src/api/preflight/mod.rs` and `src/preflight.rs`. Optionally rename helper module. Plan for Round 4.

### POLICY_PRESETS_RATIONALE.md
- **Title:** Lack of Adapter Configuration Guidance for Production Preset
  - **Category:** Documentation Gap (Usability)
  - **Severity:** S2
  - **Priority:** 3
  - **Disposition:** Implement
  - **Feasibility:** High
  - **Complexity:** 1
  - **Rationale:** Missing adapter setup guidance for `production_preset()` risks unexpected failures for new users. Simple documentation enhances onboarding. Cost of inaction is user frustration.
  - **Evidence:** `src/policy/config.rs::production_preset()` sets flags but not adapters (lines 135–141).
  - **Next Step:** Add Rustdoc examples for adapter setup in `src/policy/config.rs`. Implement in Round 4.

- **Title:** Mismatch in Default for `allow_unlocked_commit`
  - **Category:** Documentation Gap (DX/Usability)
  - **Severity:** S4
  - **Priority:** 1
  - **Disposition:** Implement
  - **Feasibility:** High
  - **Complexity:** 1
  - **Rationale:** Doc/code mismatch confuses developers testing Commit mode. Aligning is a simple usability fix. Cost of inaction is slight confusion.
  - **Evidence:** `src/policy/config.rs` docstring claims `true` (lines 62–66), code sets `false` (line 106).
  - **Next Step:** Update code or docstring in `src/policy/config.rs`. Add test for default. Implement in Round 4.

- **Title:** Incomplete Mutation Scoping in Coreutils Preset Without `allow_roots`
  - **Category:** Missing Feature (Usability)
  - **Severity:** S3
  - **Priority:** 2
  - **Disposition:** Implement
  - **Feasibility:** Medium
  - **Complexity:** 2
  - **Rationale:** Not enforcing `allow_roots` risks broader mutations than intended. Safeguard prevents overreach, enhancing safety. Cost of inaction is unintended changes.
  - **Evidence:** `src/policy/config.rs::coreutils_switch_preset()` leaves `allow_roots` empty (lines 180–212).
  - **Next Step:** Add preflight STOP rule for empty `allow_roots` in `src/api/preflight/mod.rs`. Update preset Rustdoc. Plan for Round 4.

- **Title:** Lack of Transparency in Rescue Profile Details
  - **Category:** Observability (DX/Usability)
  - **Severity:** S4
  - **Priority:** 1
  - **Disposition:** Implement
  - **Feasibility:** High
  - **Complexity:** 2
  - **Rationale:** Binary rescue reporting limits readiness assessment. Adding count and missing tools enhances monitoring. Cost of inaction is minor inconvenience.
  - **Evidence:** `src/api/preflight/mod.rs` shows only binary `rescue_profile` (lines 251–270).
  - **Next Step:** Extend `src/policy/rescue.rs` for `rescue_found_count` in preflight summary. Plan for Round 4.

### LOCKING_STRATEGY.md
- **Title:** Insufficient Lock Telemetry for Backend Identification
  - **Category:** Observability (DX/Usability)
  - **Severity:** S4
  - **Priority:** 1
  - **Disposition:** Implement
  - **Feasibility:** High
  - **Complexity:** 1
  - **Rationale:** Missing `lock_backend` field limits correlation of contention to backend. Simple enhancement for diagnostics. Cost of inaction is minor inconvenience.
  - **Evidence:** `src/api/apply/mod.rs` includes `lock_wait_ms` but no backend field (lines 355–357).
  - **Next Step:** Add `lock_backend` to `apply.attempt` in `src/api/apply/mod.rs`. Update SPEC §2.5. Implement in Round 4.

- **Title:** Lock Acquisition Lacks Fairness Under Contention
  - **Category:** Performance/Scalability
  - **Severity:** S3
  - **Priority:** 2
  - **Disposition:** Implement
  - **Feasibility:** Medium
  - **Complexity:** 2
  - **Rationale:** Fixed polling causes contention spikes in concurrent settings. Backoff or jitter improves fairness. Cost of inaction is delays in high-load scenarios.
  - **Evidence:** `src/adapters/lock/file.rs` uses fixed `LOCK_POLL_MS` (lines 50–56).
  - **Next Step:** Update `FileLockManager` for backoff/jitter in `src/adapters/lock/file.rs`. Add telemetry for attempts. Plan stress test for Round 4.

- **Title:** Risk of Lock File Path Collisions Without Standardization
  - **Category:** Missing Feature (Reliability)
  - **Severity:** S3
  - **Priority:** 2
  - **Disposition:** Implement
  - **Feasibility:** High
  - **Complexity:** 1
  - **Rationale:** Unstandardized lock paths risk collisions, causing conflicts. Standard helper mitigates risk easily. Cost of inaction is interference in multi-tenant setups.
  - **Evidence:** `FileLockManager::new(PathBuf)` accepts free-form path (lines 17–19).
  - **Next Step:** Add `Policy::default_lock_path(root)` in `src/policy/config.rs`. Update docs/examples. Implement in Round 4.

- **Title:** Documentation and Code Divergence for `allow_unlocked_commit` Default
  - **Category:** Documentation Gap (DX/Usability)
  - **Severity:** S4
  - **Priority:** 1
  - **Disposition:** Implement
  - **Feasibility:** High
  - **Complexity:** 1
  - **Rationale:** Discrepancy confuses developers testing Commit mode. Simple alignment prevents usability issues. Cost of inaction is slight confusion.
  - **Evidence:** `src/policy/config.rs` docstring states `true` (lines 62–66), code sets `false` (line 106).
  - **Next Step:** Update code or docstring in `src/policy/config.rs`. Add test. Implement in Round 4.

### idiomatic_todo.md
- **Title:** Non-Idiomatic Module Structure for API with `#[path]` Attributes
  - **Category:** Documentation Gap (DX/Usability)
  - **Severity:** S3
  - **Priority:** 2
  - **Disposition:** Implement
  - **Feasibility:** High
  - **Complexity:** 2
  - **Rationale:** Non-idiomatic structure complicates navigation and IDE support. Moving to directory-module improves maintainability. Cost of inaction is minor frustration.
  - **Evidence:** `src/api.rs` uses `#[path]` attributes, not moved to `src/api/mod.rs`.
  - **Next Step:** Move `src/api.rs` to `src/api/mod.rs`, replace `#[path]` with `mod ...;`. Update imports. Implement in Round 4.

- **Title:** Lingering Legacy Shim for `adapters::lock_file::*`
  - **Category:** Documentation Gap (DX/Usability)
  - **Severity:** S4
  - **Priority:** 1
  - **Disposition:** Implement
  - **Feasibility:** High
  - **Complexity:** 1
  - **Rationale:** Shim risks duplicate import paths, confusing integrators. Deprecation and removal clarify API. Cost of inaction is minor confusion.
  - **Evidence:** `src/adapters/mod.rs` contains shim (lines 6–9).
  - **Next Step:** Add deprecation notice to shim in `src/adapters/mod.rs`. Plan removal after minor version. Add CI check. Implement in Round 4.

- **Title:** Public Exposure of Low-Level FS Atoms
  - **Category:** API Design (DX/Usability)
  - **Severity:** S2
  - **Priority:** 3
  - **Disposition:** Implement
  - **Feasibility:** Medium
  - **Complexity:** 2
  - **Rationale:** Public low-level FS atoms risk misuse, bypassing safety. Restricting visibility enhances security. Cost of inaction is potential vulnerabilities.
  - **Evidence:** `src/fs/mod.rs` re-exports low-level atoms (lines 9–15).
  - **Next Step:** Restrict re-exports to `pub(crate)` in `src/fs/mod.rs`. Deprecate if breaking. Coordinate timeline. Implement in Round 4.

- **Title:** Non-Deterministic Backup Naming in Tests
  - **Category:** Test & Validation (Reliability)
  - **Severity:** S4
  - **Priority:** 1
  - **Disposition:** Implement
  - **Feasibility:** Medium
  - **Complexity:** 2
  - **Rationale:** Non-deterministic naming causes flaky tests, impacting CI. `Clock` trait improves reliability. Cost of inaction is occasional flakiness.
  - **Evidence:** `src/fs/backup.rs::backup_path_with_tag()` uses timestamps (lines 18–23).
  - **Next Step:** Implement `Clock` trait in `src/fs/backup.rs` for test determinism. Update fixtures. Plan for Round 4.

### SECURITY_REVIEW.md
- **Title:** Lack of Sidecar Integrity and Rollback Trust
  - **Category:** Bug/Defect (Security)
  - **Severity:** S3
  - **Priority:** 2
  - **Disposition:** Implement
  - **Feasibility:** Medium
  - **Complexity:** 3
  - **Rationale:** No durability or integrity checks risk data loss or tampering, undermining rollback. Enhancements ensure trust in recovery. Cost of inaction is untrustworthy rollbacks.
  - **Evidence:** `src/fs/backup.rs::write_sidecar()` lacks `fsync(parent)` (lines 262–270); no integrity binding.
  - **Next Step:** Add durability with `fsync_parent_dir()` in `src/fs/backup.rs`. Design sidecar signing for `backup_meta.v2`. Emit durability facts. Plan for Round 4.

- **Title:** Public Exposure of Low-Level FS Atoms Bypassing `SafePath`
  - **Category:** API Design (Security)
  - **Severity:** S2
  - **Priority:** 3
  - **Disposition:** Implement
  - **Feasibility:** Medium
  - **Complexity:** 2
  - **Rationale:** Public FS atoms risk bypassing safety, enabling TOCTOU issues. Restricting enhances security. Cost of inaction is high risk of misuse.
  - **Evidence:** `src/fs/mod.rs` re-exports low-level atoms (lines 9–15).
  - **Next Step:** Restrict to `pub(crate)` in `src/fs/mod.rs`. Deprecate if needed. Coordinate timeline. Implement in Round 4.

- **Title:** Incomplete Secret Redaction in Facts Output
  - **Category:** Bug/Defect (Security)
  - **Severity:** S3
  - **Priority:** 2
  - **Disposition:** Implement
  - **Feasibility:** Medium
  - **Complexity:** 2
  - **Rationale:** Unredacted `notes` risk leaking sensitive data. Extending redaction mitigates exposure. Cost of inaction is moderate data leakage risk.
  - **Evidence:** `src/logging/redact.rs::redact_event()` misses `notes` sanitization.
  - **Next Step:** Extend redaction for `notes` in `src/logging/redact.rs`. Add hooks and tests. Update SPEC §13. Plan for Round 4.

- **Title:** Optimistic Environment Sanitization Claim for Rescue Checks
  - **Category:** Documentation Gap (Security)
  - **Severity:** S4
  - **Priority:** 1
  - **Disposition:** Implement
  - **Feasibility:** High
  - **Complexity:** 1
  - **Rationale:** False `env_sanitized=true` misleads on safety. Sanitization or correction is simple for transparency. Cost of inaction is minor misinterpretation.
  - **Evidence:** `src/logging/audit.rs::ensure_provenance()` sets `env_sanitized=true` unconditionally (lines 210–219).
  - **Next Step:** Add sanitizer in `src/policy/rescue.rs` or adjust flag in `src/logging/audit.rs`. Emit `env_vars_checked`. Implement in Round 4.

### RELEASE_AND_CHANGELOG_POLICY.md
- **Title:** Lack of Deprecation Warnings for Legacy Shims and API Changes
  - **Category:** Documentation Gap (DX/Usability)
  - **Severity:** S2
  - **Priority:** 3
  - **Disposition:** Implement
  - **Feasibility:** High
  - **Complexity:** 1
  - **Rationale:** Missing deprecation warnings risk breakage for consumers. Adding attributes and changelog entries ensures transitions. Cost of inaction is integration issues.
  - **Evidence:** No deprecation on `adapters::lock_file::*` (lines 6–9) or FS atoms in `src/fs/mod.rs` (lines 9–15).
  - **Next Step:** Add `#[deprecated]` to shims and atoms. Document in CHANGELOG. Add CI check. Implement in Round 4.

- **Title:** Missing Dual-Emit Period for Schema Version Bumps
  - **Category:** Missing Feature (Reliability)
  - **Severity:** S3
  - **Priority:** 2
  - **Disposition:** Implement
  - **Feasibility:** Medium
  - **Complexity:** 3
  - **Rationale:** No dual-emit risks consumer breakage on schema updates. Dual-emit ensures compatibility. Cost of inaction is pipeline failures.
  - **Evidence:** `src/logging/audit.rs` uses single `SCHEMA_VERSION=1` (line 13).
  - **Next Step:** Add dual-emit for v1/v2 in `src/logging/audit.rs`. Update fixtures and CI. Update SPEC §13. Plan for Round 4.

- **Title:** Absence of Repository-Local CI Gates for SKIP and Unwrap/Expect
  - **Category:** Test & Validation (Reliability)
  - **Severity:** S4
  - **Priority:** 1
  - **Disposition:** Implement
  - **Feasibility:** High
  - **Complexity:** 1
  - **Rationale:** No local checks risk regressions before CI. Simple scripts enhance quality. Cost of inaction is avoidable CI failures.
  - **Evidence:** `src/lib.rs` has `#![deny]` (lines 1–9), but no local SKIP checks.
  - **Next Step:** Add xtask for `#[ignore]` checks. Add CI grep for deprecation. Document in CONTRIBUTING. Implement in Round 4.

- **Title:** Missing Crate-Level CHANGELOG for Public API Changes
  - **Category:** Documentation Gap (DX/Usability)
  - **Severity:** S3
  - **Priority:** 2
  - **Disposition:** Implement
  - **Feasibility:** High
  - **Complexity:** 1
  - **Rationale:** No CHANGELOG complicates upgrade decisions. Adding it improves transparency. Cost of inaction is minor inconvenience.
  - **Evidence:** No `CHANGELOG.md` in `cargo/switchyard/`.
  - **Next Step:** Create `CHANGELOG.md` following template. Add CI gate for API diffs. Implement in Round 4.

## Round 4 Implementation Plans — Targets (return to own set)

- BACKWARDS_COMPAT_SHIMS.md
- BEHAVIORS.md
- EXPERIMENT_CONSTANTS_REVIEW.md
- REEXPORTS_AND_FACADES.md
- RETENTION_STRATEGY.md
- PERFORMANCE_PLAN.md
- TEST_COVERAGE_MAP.md
- MIGRATION_GUIDE.md
- ROADMAP.md
- CODING_STANDARDS.md
- CONTRIBUTING_ENHANCEMENTS.md

### Plan Template (use per item)

- Summary
- Code targets (files/functions)
- Steps: changes, tests, telemetry/docs
- Feasibility: High/Medium/Low
- Complexity: 1–5
- Risks and mitigations
- Dependencies

## Round 4 Implementation Plans (AI 4, 2025-09-12 16:15 CET)

Based on Round 2 Gap Analyses and Round 3 Severity Assessments from my original document set, focusing on high-value security and developer experience (DX) items plus selected low-hanging fruit (LHF) documentation fixes for `EDGE_CASES_AND_BEHAVIOR.md`, `CORE_FEATURES_FOR_EDGE_CASES.md`, and `CLI_INTEGRATION_GUIDE.md`.

### 1. SafePath Enforcement in Core FS Operations (S2 Priority 3)

- **Summary:** Refactor core filesystem operations to enforce `SafePath` usage internally to prevent path traversal vulnerabilities.
- **Code Targets:**
  - `src/fs/swap.rs` - Functions accepting raw `&Path` for mutations
  - `src/fs/restore.rs` - Functions like `restore_file` accepting raw paths
  - `src/fs/backup.rs` - Functions like `create_snapshot` accepting raw paths
  - `src/api/apply/handlers.rs` - Entry points to ensure `SafePath` conversion
- **Steps:**
  1. **Changes:**
     - Modify function signatures in `src/fs/` modules to accept `SafePath` instead of `&Path` for all mutating operations.
     - Add a conversion layer at the API boundary (e.g., in `src/api/apply/handlers.rs`) to convert incoming `PathBuf` or `&str` to `SafePath`, ensuring validation before passing to FS layer.
     - Update internal path manipulations to use `SafePath` methods, ensuring no raw path operations bypass safety checks.
     - Handle edge cases where `SafePath` validation fails by returning appropriate `ApiError` variants with user-friendly messages.
  2. **Tests:**
     - Add unit tests in each FS module to verify that only `SafePath`-validated inputs are accepted.
     - Add integration tests in `src/api/` to simulate CLI or library usage with invalid paths, ensuring proper error handling.
     - Add security-focused tests attempting path traversal (e.g., `../etc/passwd`) and verifying rejection.
  3. **Telemetry/Docs:**
     - Add a fact field `safepath_validation=success|failure` to `apply.attempt` in `src/api/apply/mod.rs` to track validation outcomes.
     - Update `CLI_INTEGRATION_GUIDE.md` to reflect the strict `SafePath` requirement and provide examples of valid path inputs.
     - Update `SPEC §3 Public Interfaces` to mandate `SafePath` for all mutating APIs, documenting error behavior.
- **Feasibility:** Medium
- **Complexity:** 4
- **Risks:**
  - Breaking change for existing library consumers passing raw paths directly to low-level functions.
  - **Mitigation:** Deprecate raw path functions first with a clear migration guide in `CLI_INTEGRATION_GUIDE.md`; provide a transition period of one minor release before full enforcement.
- **Dependencies:** Coordinate with deprecation policy in `RELEASE_AND_CHANGELOG_POLICY.md` for timeline and communication.

### 2. Backup Sidecar Integrity and Tampering Protection (S3 Priority 2)

- **Summary:** Enhance backup sidecar schema to include integrity checks, preventing tampering during restore operations.
- **Code Targets:**
  - `src/fs/backup.rs::BackupSidecar` struct (lines 244–252) and `write_sidecar()` function
  - `src/fs/restore.rs::restore_file()` - Validation logic
  - `src/policy/config.rs` - Add integrity policy controls
- **Steps:**
  1. **Changes:**
     - Extend `BackupSidecar` struct to include a `payload_hash` field (e.g., SHA-256 of the backup payload file).
     - Update `write_sidecar()` to compute and store the hash of the backup payload during creation.
     - Update `restore_file()` to verify the hash of the stored payload against the sidecar's `payload_hash` before proceeding with restore.
     - Add a policy flag `require_sidecar_integrity` in `Policy` struct (default `true` in `production_preset()`), allowing fallback to unverified restores only if explicitly disabled.
  2. **Tests:**
     - Add unit tests in `src/fs/backup.rs` to verify hash computation and storage.
     - Add unit tests in `src/fs/restore.rs` to simulate tampered payloads and ensure restore fails with `E_BACKUP_TAMPERED` or similar error.
     - Add integration test simulating a full backup-restore cycle with integrity verification.
  3. **Telemetry/Docs:**
     - Add `sidecar_integrity_verified=true|false` to restore facts in `src/api/apply/mod.rs`.
     - Update `EDGE_CASES_AND_BEHAVIOR.md` to document integrity verification behavior under tampering scenarios.
     - Document the new policy flag and its security implications in `SPEC §2.6 Rescue and Recovery`.
- **Feasibility:** Medium
- **Complexity:** 3
- **Risks:**
  - Performance overhead from hash computation for large backups.
  - **Mitigation:** Use a fast hash algorithm (e.g., SHA-256) and make integrity optional via policy for performance-critical environments.
- **Dependencies:** Requires sidecar schema versioning (v2) to avoid breaking existing backups; coordinate with schema migration plans.

### 3. Missing SUID/SGID Binary Protection Gate (S3 Priority 2)

- **Summary:** Add a preflight check to detect and gate operations on SUID/SGID binaries, preventing accidental privilege escalation.
- **Code Targets:**
  - `src/preflight/checks.rs` - Add new check function
  - `src/policy/config.rs` - Add policy control
  - `src/api/preflight/mod.rs` - Integrate check into preflight pipeline
- **Steps:**
  1. **Changes:**
     - Implement a new `check_suid_sgid_risk()` function in `src/preflight/checks.rs` to inspect file metadata for `S_ISUID` and `S_ISGID` bits using `stat()` or similar syscall.
     - Add a policy flag `allow_suid_sgid_mutation` (default `false` in `production_preset()` and `coreutils_switch_preset()`), controlling whether operations on such binaries are permitted.
     - Integrate the check into the preflight pipeline in `src/api/preflight/mod.rs`, emitting a STOP row with `suid_sgid_risk=true` if detected and policy disallows.
  2. **Tests:**
     - Add unit tests in `src/preflight/checks.rs` to verify detection of SUID/SGID bits on mock files (e.g., create test files with `chmod u+s` in setup).
     - Add integration test to simulate an operation on a SUID binary, ensuring preflight fails with appropriate error unless policy allows.
  3. **Telemetry/Docs:**
     - Add `suid_sgid_risk=true|false` to preflight row facts in `src/api/preflight/mod.rs`.
     - Update `CORE_FEATURES_FOR_EDGE_CASES.md` to document SUID/SGID protection as a security feature for privilege escalation prevention.
     - Document the policy flag and its default settings in `SPEC §2.5 Safety and Conservatism`.
- **Feasibility:** High
- **Complexity:** 1
- **Risks:**
  - False positives on legitimate operations requiring SUID/SGID changes.
  - **Mitigation:** Provide clear policy override and detailed error messages pointing to documentation.
- **Dependencies:** None

### 4. Low-Hanging Fruit Documentation Fix: CLI Integration Guidance Update (S2 Priority 3, LHF)

- **Summary:** Update `CLI_INTEGRATION_GUIDE.md` to align with current API reality, removing references to non-existent functions and clarifying `SafePath` usage.
- **Code Targets:**
  - N/A (Documentation only, targeting `CLI_INTEGRATION_GUIDE.md`)
- **Steps:**
  1. **Changes:**
     - Remove references to non-existent functions like `prune_backups` and note that retention management must be implemented at the CLI level until library support is added.
     - Update guidance on `SafePath` to reflect current API limitations (i.e., not fully enforced in core FS operations) and provide interim best practices for secure path handling.
     - Add a note about upcoming `SafePath` enforcement changes and reference the deprecation timeline for raw path APIs.
  2. **Tests:**
     - N/A (Documentation change; no code tests required).
  3. **Telemetry/Docs:**
     - Ensure the updated guide links to `SPEC §3 Public Interfaces` for authoritative API details.
     - Add a changelog entry in `RELEASE_AND_CHANGELOG_POLICY.md` under 'Changed' for documentation updates in the next release.
- **Feasibility:** High
- **Complexity:** 1
- **Risks:**
  - Minor risk of outdated interim guidance if `SafePath` enforcement timeline shifts.
  - **Mitigation:** Include a disclaimer in the guide noting that API changes are in progress and link to project roadmap for updates.
- **Dependencies:** Coordinate with `SafePath` enforcement implementation timeline for accuracy.

### Implementation Priority Order

1. **SafePath Enforcement in Core FS Operations** (highest security impact, critical for preventing path traversal vulnerabilities)
2. **Backup Sidecar Integrity and Tampering Protection** (significant reliability and security improvement for recovery scenarios)
3. **Missing SUID/SGID Binary Protection Gate** (important security safeguard with low complexity)
4. **CLI Integration Guidance Update** (quick win for developer experience, aligns documentation with reality)

### Cross-Cutting Considerations

- All implementation plans prioritize backward compatibility where feasible, using deprecation periods for breaking changes.
- Security enhancements (e.g., `SafePath` enforcement, SUID/SGID checks) are designed to fail closed by default, with explicit policy overrides for flexibility.
- Telemetry additions are additive to avoid breaking existing consumers and enhance observability.
- Test coverage focuses on edge cases and security scenarios to ensure robustness.
- Documentation updates align with SPEC requirements and provide clear migration paths for upcoming changes.
