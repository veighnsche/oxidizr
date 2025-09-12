# AI 1 — Round 1 Analysis Report

Generated: 2025-09-12 14:24:34+02:00
Analyst: AI 1
Coordinator: Cascade

Scope: Verify claims, provide proofs, and patch gaps in the assigned documents only. Record evidence and edits here. Do not start Round 2 until instructed.

## Assigned Documents (37 pts)

- EDGE_CASES_AND_BEHAVIOR.md — 20
- CORE_FEATURES_FOR_EDGE_CASES.md — 15
- CLI_INTEGRATION_GUIDE.md — 2

## Round 1 Checklist

- [ ] Evidence mapping completed for all assigned docs
- [ ] Patches applied to assigned docs where needed
- [ ] All claims verified or corrected with citations
- [ ] Open questions recorded

## Evidence — EDGE_CASES_AND_BEHAVIOR.md

- Claims → Proofs
  - [ ] Claim: <state the claim>
    - Proof: cite code `cargo/switchyard/src/<file>.rs: fn_name()` and/or tests/specs
  - [ ] Claim: <state the claim>
    - Proof: <citations>

## Changes Made — EDGE_CASES_AND_BEHAVIOR.md

- [ ] Edit summary 1: <what changed and why>
- [ ] Edit summary 2: <what changed and why>

## Evidence — CORE_FEATURES_FOR_EDGE_CASES.md

- Claims → Proofs
  - [ ] Claim: <state the claim>
    - Proof: <citations>

## Changes Made — CORE_FEATURES_FOR_EDGE_CASES.md

- [ ] Edit summary: <what changed and why>

## Evidence — CLI_INTEGRATION_GUIDE.md

- Claims → Proofs
  - [ ] Claim: Exit-code mapping guidance
    - Proof: `cargo/switchyard/src/api/errors.rs` (ErrorId → exit code), `SPEC/error_codes.toml`
  - [ ] Claim: Policy presets usage examples
    - Proof: `cargo/switchyard/src/policy/config.rs::Policy::{production_preset, coreutils_switch_preset}`

## Changes Made — CLI_INTEGRATION_GUIDE.md

- [ ] Edit summary: <what changed and why>

## Open Questions

- [ ] <question>

## Round 2 Plan (Do NOT start yet)

- You will peer review AI 2’s outputs and assigned docs in Round 2:
  - FS_SAFETY_AUDIT.md, API_SURFACE_AUDIT.md, OBSERVABILITY_FACTS_SCHEMA.md, ERROR_TAXONOMY.md, INDEX.md
- Tasks for Round 2 (later):
  - Re-verify proofs, check missed claims, propose fixes. Record notes in this file under "Round 2 Review".

## Round 1 Peer Review Targets

- FS_SAFETY_AUDIT.md
- API_SURFACE_AUDIT.md
- OBSERVABILITY_FACTS_SCHEMA.md
- ERROR_TAXONOMY.md
- INDEX.md

### Round 1 Peer Review — Checklist

- [x] FS_SAFETY_AUDIT.md
- [x] API_SURFACE_AUDIT.md
- [x] OBSERVABILITY_FACTS_SCHEMA.md
- [x] ERROR_TAXONOMY.md
- [x] INDEX.md

### Round 1 Peer Review — Evidence and Edits

- For each doc, add Claims → Proofs with code/spec/test citations and list changes made.

#### FS_SAFETY_AUDIT.md — Claims → Proofs

- Atomic swap sequence and fsync parent after rename
  - Proof: `cargo/switchyard/src/fs/atomic.rs::atomic_symlink_swap()` uses `open_dir_nofollow`, `symlinkat`, `renameat`, and `fsync_parent_dir` (lines 56–96).
- Capability unlink via dirfd
  - Proof: `cargo/switchyard/src/fs/swap.rs::replace_file_with_symlink()` calls `unlinkat(&dirfd, fname)` after `open_dir_nofollow(parent)` (lines 70–81, 125–133).
- Restore uses `renameat`, `fchmod`, and fsync
  - Proof: `cargo/switchyard/src/fs/restore.rs::restore_file()` `renameat` (126–127), `fchmod` (134–137), `fsync_parent_dir` (139–140), plus other branches (171–174, 223–225, 259–261).
- EXDEV degraded path behavior
  - Proof: `cargo/switchyard/src/fs/atomic.rs` EXDEV branch (86–93) calls `fsync_parent_dir`; `SWITCHYARD_FORCE_EXDEV` knob (74–76).
- Backup/sidecar durability gap
  - Proof: `cargo/switchyard/src/fs/backup.rs::create_snapshot()` uses path-based `unix::fs::symlink` for symlink backups (137–139); `write_sidecar()` path-based create (262–270); no explicit parent fsync.

Changes Made: Appended “Round 1 Peer Review” section with citations and confirmed gaps; left recommendations intact.

#### API_SURFACE_AUDIT.md — Claims → Proofs

- Public facade and re-exports
  - Proof: `cargo/switchyard/src/lib.rs` exposes `pub mod` and `pub use api::*` (lines 11–21).
- Low-level FS atoms publicly re-exported
  - Proof: `cargo/switchyard/src/fs/mod.rs` re-exports `atomic_symlink_swap`, `fsync_parent_dir`, `open_dir_nofollow` (lines 9–15).
- Adapters traits and default impls
  - Proof: `cargo/switchyard/src/adapters/mod.rs` re-exports `FileLockManager`, `FsOwnershipOracle`, traits in `adapters/lock/mod.rs`, `adapters/smoke.rs`, `adapters/path.rs`.
- Logging sinks and redaction public; audit helpers crate-internal
  - Proof: `cargo/switchyard/src/logging/mod.rs` re-exports sinks/redaction; `logging/audit.rs` functions used internally from API modules.
- Preflight naming duplication
  - Proof: `cargo/switchyard/src/fs/mount.rs::ensure_rw_exec` vs `cargo/switchyard/src/preflight/checks.rs::ensure_mount_rw_exec`.

Changes Made: Appended round summary with citations; noted recommendation to mark low-level FS atoms Internal/deprecate re-exports.

#### OBSERVABILITY_FACTS_SCHEMA.md — Claims → Proofs

- Envelope enforcement
  - Proof: `cargo/switchyard/src/logging/audit.rs::redact_and_emit()` inserts `schema_version`, `ts`, `plan_id`, `path`, and `dry_run` (51–58).
- Emission coverage
  - Proof: plan via `api/plan.rs::build()`; preflight rows via `api/preflight/rows.rs::push_row_emit()`; preflight summary via `api/preflight/mod.rs::run()` (270); apply attempt/result via `api/apply/mod.rs` (151–158, 174–183, 185–193, 409–411); rollback via `emit_rollback_step` (244–261).
- Determinism and redaction
  - Proof: `logging/redact.rs::{ts_for_mode (57–61), redact_event (67–101)}`; IDs via `types/ids.rs`.
- Schema alignment and attestation/provenance
  - Proof: `SPEC/audit_event.schema.json`; `api/apply/mod.rs` attestation block (359–384); `audit::ensure_provenance()`.

Changes Made: Appended round summary with citations and reiterated recommendations (schema validation test, `summary_error_ids`).

#### ERROR_TAXONOMY.md — Claims → Proofs

- ErrorId→exit-code mapping
  - Proof: `cargo/switchyard/src/api/errors.rs::exit_code_for()` (61–73) equals `SPEC/error_codes.toml` (1–11).
- Emission sites and summary behavior
  - Proof: Locking (apply/mod.rs 66–87, 101–131, 114–121); policy gating (160–202, 167–183, 185–193); EXDEV/atomic swap (handlers.rs 61–70, 81–85, 91–95); restore (handlers.rs 191–209, 206–208); preflight summary (preflight/mod.rs 255–270); summary default E_POLICY unless E_SMOKE (apply/mod.rs 390–406).

Changes Made: Appended round summary with citations, clarified summary mapping default.

#### INDEX.md — Claims → Proofs

- Presence and scope alignment of listed analyses
  - Proof: Files exist in `cargo/switchyard/DOCS/analysis/`; scopes match code/spec areas (`src/fs/**`, `src/lib.rs`, `src/logging/**`, `SPEC/*.json|.toml`).

Changes Made: Appended round summary confirming index alignment with repository content.

## Round 2 Meta Review Targets

- PRESERVATION_FIDELITY.md
- PREFLIGHT_MODULE_CONCERNS.md
- POLICY_PRESETS_RATIONALE.md
- LOCKING_STRATEGY.md
- idiomatic_todo.md
- SECURITY_REVIEW.md
- RELEASE_AND_CHANGELOG_POLICY.md

### Round 2 Meta Review — Notes

- Thoroughness, correctness, evidence quality, and editorial discipline per doc. Do not edit docs; record issues here.

## Round 3 Severity Reports — Targets

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

### Round 3 Severity Reports — Entries

- Topic: <area>
  - Impact: [] Likelihood: [] Confidence: [] → Priority: []
  - Rationale: <citations>

## Round 4 Implementation Plans — Targets (return to own set)

- EDGE_CASES_AND_BEHAVIOR.md
- CORE_FEATURES_FOR_EDGE_CASES.md
- CLI_INTEGRATION_GUIDE.md

### Plan Template (use per item)

- Summary
- Code targets (files/functions)
- Steps: changes, tests, telemetry/docs
- Feasibility: High/Medium/Low
- Complexity: 1–5
- Risks and mitigations
- Dependencies

## Round 2 Review (placeholder)

- Findings:
- Suggested diffs:
