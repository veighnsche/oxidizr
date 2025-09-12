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
