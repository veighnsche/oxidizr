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

## Round 2 Gap Analysis (AI 1, 2025-09-12 15:22 +02:00)

### Per-doc checklist

- [x] PRESERVATION_FIDELITY.md — Gap Analysis appended with invariants, evidence, gaps, mitigations, users, follow-ups
- [x] PREFLIGHT_MODULE_CONCERNS.md — Gap Analysis appended
- [x] POLICY_PRESETS_RATIONALE.md — Gap Analysis appended
- [x] LOCKING_STRATEGY.md — Gap Analysis appended
- [x] idiomatic_todo.md — Gap Analysis appended
- [x] SECURITY_REVIEW.md — Gap Analysis appended
- [x] RELEASE_AND_CHANGELOG_POLICY.md — Gap Analysis appended

### Consolidated findings and proposed mitigations

- PRESERVATION_FIDELITY.md
  - Gaps
    - Extended preservation (owner, mtime, xattrs) not implemented despite capability probe signals.
    - Backup/sidecar durability missing parent `fsync`; path-based `symlink` and `File::create` usage.
    - Restore impossible when payload pruned but sidecar remains.
  - Mitigations
    - Introduce `preservation_tier` policy; extend sidecar v2 with `uid/gid`, `mtime`, `xattrs`; apply via `fchownat`, `utimensat`, xattr writes.
    - Use `open_dir_nofollow` + `*at` and `fsync_parent_dir(backup)`; add crash-sim durability tests.
    - Emit `restore_ready` in preflight; document retention rules.

- PREFLIGHT_MODULE_CONCERNS.md
  - Gaps
    - `lsattr`-based immutable check fails open when tool missing; YAML omits preservation fields.
    - Naming overlap (`preflight` helpers vs stage) still confusing.
  - Mitigations
    - Prefer `FS_IOC_GETFLAGS` ioctl when available; add fact `immutable_check=unknown` and STOP unless overridden.
    - Add `preservation` and `preservation_supported` to YAML or clearly document minimal scope; update SPEC §4 and fixtures.
    - Add module-level docs to clarify ownership of helpers vs stage.

- POLICY_PRESETS_RATIONALE.md
  - Gaps
    - Doc/code divergence: `allow_unlocked_commit` default documented as true; code default is false.
    - `coreutils_switch_preset()` relies on caller to set `allow_roots`, risking broad scope.
    - Rescue profile summary lacks counts/names for readiness.
  - Mitigations
    - Reconcile default (flip to true for dev or update docs to false); add test.
    - Add STOP when preset active and `allow_roots` empty; document sample.
    - Emit `rescue_found_count` and `rescue_missing` in preflight summary.

- LOCKING_STRATEGY.md
  - Gaps
    - No `lock_backend` fact; fixed polling without backoff; no standard lock path helper; default for unlocked Commit unclear to devs.
  - Mitigations
    - Emit `lock_backend`; add backoff/jitter; provide `Policy::default_lock_path(root)`; align docs/code for `allow_unlocked_commit`.

- idiomatic_todo.md
  - Gaps
    - `src/api.rs` not yet moved to `src/api/mod.rs`.
    - Legacy shim `adapters::lock_file::*` still public; low-level FS atoms re-exported publicly.
    - Non-deterministic backup names hinder tests.
  - Mitigations
    - Complete API module move; deprecate and remove shim after window; restrict low-level FS atoms to `pub(crate)`; introduce `Clock` trait for deterministic backups.

- SECURITY_REVIEW.md
  - Gaps
    - Sidecar durability/integrity; low-level atom exports; incomplete redaction for `notes`; optimistic `env_sanitized` flag.
  - Mitigations
    - Make backups durable and sign sidecars (v2); restrict low-level exports; extend redaction; implement real env sanitizer and truthfully emit flags.

- RELEASE_AND_CHANGELOG_POLICY.md
  - Gaps
    - No `#[deprecated]` on shims; no dual-emit scaffolding for schema bumps; repo-local CI checks for SKIP/deprecations missing; no crate-local CHANGELOG.
  - Mitigations
    - Annotate deprecations and gate new use; add feature-gated dual-emit with fixtures; add CI checks; create and enforce `CHANGELOG.md`.

### Next actions (for Round 3 prioritization)

- Durability of backups/sidecars (crash safety) — high impact, medium effort.
- Preservation tiers (mtime + xattrs) — medium impact/effort.
- Policy/docs alignment for `allow_unlocked_commit` default — quick fix.
- Add preservation fields to YAML exporter — low effort, clarity win.
- Deprecate and restrict low-level FS atoms — medium effort, safety win.

Recorded in Round 2 by AI 1 on 2025-09-12 15:22 +02:00

## Round 3 Severity Reports (AI 1, 2025-09-12 15:44 +02:00)

### Per-doc checklist

- [x] BACKWARDS_COMPAT_SHIMS.md
- [x] BEHAVIORS.md
- [x] EXPERIMENT_CONSTANTS_REVIEW.md
- [x] REEXPORTS_AND_FACADES.md
- [x] RETENTION_STRATEGY.md
- [x] PERFORMANCE_PLAN.md
- [x] TEST_COVERAGE_MAP.md
- [x] MIGRATION_GUIDE.md
- [x] ROADMAP.md
- [x] CODING_STANDARDS.md
- [x] CONTRIBUTING_ENHANCEMENTS.md

### Triage Board

- Doc: BACKWARDS_COMPAT_SHIMS.md — Title: Shim deprecation signaling and migration path are missing
  - Category: Documentation Gap
  - Impact: 3  Likelihood: 3  Confidence: 4  → Priority: 3  Severity: S2
  - Disposition: Implement  LHF: Yes
  - Feasibility: High  Complexity: 2
  - Why update vs why not: Add `#[deprecated]` to legacy re-exports to avoid silent breakage later.
  - Evidence: `src/adapters/mod.rs:6-9`, `src/lib.rs:21`.
  - Next step: Add deprecation attributes and CHANGELOG/MIGRATION pointers.

- Doc: BACKWARDS_COMPAT_SHIMS.md — Title: Remove `adapters::lock_file` shim after migration window
  - Category: DX/Usability
  - Impact: 2  Likelihood: 3  Confidence: 4  → Priority: 2  Severity: S3
  - Disposition: Backlog  LHF: No
  - Feasibility: High  Complexity: 3
  - Why update vs why not: Keeps API clean; defer removal until usage is zero and release timing aligns.
  - Evidence: Shim at `src/adapters/mod.rs:6-9`.
  - Next step: Migrate internal usages, add CI grep, remove in next breaking release.

- Doc: BEHAVIORS.md — Title: Dry-run redaction removes timing data needed for analysis
  - Category: Documentation Gap
  - Impact: 3  Likelihood: 3  Confidence: 4  → Priority: 3  Severity: S2
  - Disposition: Spec-only  LHF: Yes
  - Feasibility: High  Complexity: 1
  - Why update vs why not: Clarify DryRun limitations to set expectations.
  - Evidence: `src/logging/redact.rs::redact_event()`.
  - Next step: Update SPEC/docs; consider policy to keep timings in perf-only dry runs.

- Doc: BEHAVIORS.md — Title: Preflight override surprises consumers with apply-stage policy failures
  - Category: DX/Usability
  - Impact: 3  Likelihood: 3  Confidence: 4  → Priority: 3  Severity: S2
  - Disposition: Implement  LHF: Yes
  - Feasibility: High  Complexity: 2
  - Why update vs why not: Add explicit fact/warn when preflight is overridden to avoid surprises.
  - Evidence: `src/policy/gating.rs`, `src/api/apply/mod.rs`.
  - Next step: Emit `preflight_overridden=true` in `apply.attempt`; doc update.

- Doc: BEHAVIORS.md — Title: Test-only environment knobs risk accidental production use
  - Category: Documentation Gap
  - Impact: 2  Likelihood: 3  Confidence: 4  → Priority: 2  Severity: S3
  - Disposition: Spec-only  LHF: Yes
  - Feasibility: High  Complexity: 1
  - Why update vs why not: Reduce misuse by documenting test-only knobs; optionally warn in Commit.
  - Evidence: `src/fs/atomic.rs`, `src/policy/rescue.rs`.
  - Next step: Document knobs; add WARN on detection in Commit.

- Doc: EXPERIMENT_CONSTANTS_REVIEW.md — Title: Make preserve list a policy knob instead of hard constant
  - Category: Policy/Default Mismatch
  - Impact: 3  Likelihood: 3  Confidence: 4  → Priority: 3  Severity: S2
  - Disposition: Implement  LHF: Yes
  - Feasibility: High  Complexity: 2
  - Why update vs why not: Environment variability demands configurability.
  - Evidence: `oxidizr-arch/src/experiments/constants.rs`.
  - Next step: Add `preserve_bins` to `Policy` and thread through.

- Doc: EXPERIMENT_CONSTANTS_REVIEW.md — Title: Guardrails to prevent env-specific lists in core
  - Category: Documentation Gap
  - Impact: 2  Likelihood: 3  Confidence: 4  → Priority: 2  Severity: S3
  - Disposition: Spec-only  LHF: Yes
  - Feasibility: High  Complexity: 1
  - Why update vs why not: ADR + standards reduce future drift.
  - Evidence: Guidance in doc; no ADR exists.
  - Next step: Write ADR; update coding standards.

- Doc: REEXPORTS_AND_FACADES.md — Title: No API surface stability tests for facades
  - Category: Missing Feature
  - Impact: 3  Likelihood: 3  Confidence: 4  → Priority: 3  Severity: S2
  - Disposition: Implement  LHF: Yes
  - Feasibility: High  Complexity: 2
  - Why update vs why not: Prevents accidental import breakage.
  - Evidence: `src/fs/mod.rs`, `src/types/mod.rs`, `src/logging/mod.rs`.
  - Next step: Add `tests/public_api.rs` importing all public paths.

- Doc: REEXPORTS_AND_FACADES.md — Title: Lifecycle policy distinguishing facades vs shims missing
  - Category: Documentation Gap
  - Impact: 2  Likelihood: 3  Confidence: 4  → Priority: 2  Severity: S3
  - Disposition: Spec-only  LHF: Yes
  - Feasibility: High  Complexity: 1
  - Why update vs why not: Clarifies stability guarantees.
  - Evidence: Shim at `src/adapters/mod.rs:6-9`, root re-export at `src/lib.rs:21`.
  - Next step: Document lifecycle in SPEC §3; tag shims deprecated.

- Doc: REEXPORTS_AND_FACADES.md — Title: Over-broad glob re-exports risk namespace pollution
  - Category: DX/Usability
  - Impact: 2  Likelihood: 2  Confidence: 3  → Priority: 2  Severity: S3
  - Disposition: Backlog  LHF: No
  - Feasibility: Medium  Complexity: 3
  - Why update vs why not: Consider narrowing in future with deprecations.
  - Evidence: `src/types/mod.rs` glob exports.
  - Next step: Analyze consumer usage; plan changes in major/minor.

- Doc: RETENTION_STRATEGY.md — Title: No retention enforcement → risk of disk fill
  - Category: Missing Feature
  - Impact: 4  Likelihood: 3  Confidence: 4  → Priority: 3  Severity: S2
  - Disposition: Implement  LHF: No
  - Feasibility: High  Complexity: 3
  - Why update vs why not: Prevent unbounded growth; improve predictability.
  - Evidence: `src/fs/backup.rs` has no enforcement.
  - Next step: Implement policy knobs + `prune_backups`.

- Doc: RETENTION_STRATEGY.md — Title: Pruning may orphan payload/sidecar pairs
  - Category: Bug/Defect
  - Impact: 3  Likelihood: 2  Confidence: 3  → Priority: 2  Severity: S3
  - Disposition: Implement  LHF: No
  - Feasibility: High  Complexity: 3
  - Why update vs why not: Maintain integrity; atomic pair deletion needed.
  - Evidence: Discovery helpers; no pairwise delete today.
  - Next step: Pair validation and atomic delete with `unlinkat` + parent fsync.

- Doc: RETENTION_STRATEGY.md — Title: Safety invariants for pruning not enforced
  - Category: Missing Feature
  - Impact: 3  Likelihood: 2  Confidence: 4  → Priority: 2  Severity: S3
  - Disposition: Implement  LHF: Yes
  - Feasibility: High  Complexity: 2
  - Why update vs why not: Enforce TOCTOU-safe pruning.
  - Evidence: No current implementation.
  - Next step: Use `open_dir_nofollow` + `unlinkat` + pattern validation.

- Doc: PERFORMANCE_PLAN.md — Title: No performance telemetry fields
  - Category: Missing Feature
  - Impact: 3  Likelihood: 3  Confidence: 4  → Priority: 3  Severity: S2
  - Disposition: Implement  LHF: Yes
  - Feasibility: High  Complexity: 2
  - Why update vs why not: Improves monitoring and regression detection.
  - Evidence: Only per-action `fsync_ms` via helpers; no summary perf fields.
  - Next step: Add `perf` object to summary; SPEC thresholds.

- Doc: PERFORMANCE_PLAN.md — Title: Large-file hashing may cause delays
  - Category: Performance/Scalability
  - Impact: 3  Likelihood: 3  Confidence: 4  → Priority: 3  Severity: S2
  - Disposition: Implement  LHF: No
  - Feasibility: Medium  Complexity: 3
  - Why update vs why not: Add size-aware policy.
  - Evidence: `src/fs/meta.rs::sha256_hex_of` streams whole file.
  - Next step: Policy threshold; benchmarks.

- Doc: PERFORMANCE_PLAN.md — Title: Directory scans degrade with artifact count
  - Category: Performance/Scalability
  - Impact: 3  Likelihood: 2  Confidence: 4  → Priority: 2  Severity: S3
  - Disposition: Backlog  LHF: No
  - Feasibility: Medium  Complexity: 3
  - Why update vs why not: Consider index after retention.
  - Evidence: `src/fs/backup.rs` scans.
  - Next step: Optional index or rely on retention.

- Doc: TEST_COVERAGE_MAP.md — Title: Facts schema validation tests are missing
  - Category: Missing Feature
  - Impact: 3  Likelihood: 3  Confidence: 4  → Priority: 3  Severity: S2
  - Disposition: Implement  LHF: Yes
  - Feasibility: High  Complexity: 2
  - Why update vs why not: Prevent schema drift.
  - Evidence: No schema validation tests.
  - Next step: Add `jsonschema` tests across all fact types.

- Doc: TEST_COVERAGE_MAP.md — Title: EXDEV degraded-path behavior lacks tests
  - Category: Missing Feature
  - Impact: 3  Likelihood: 2  Confidence: 4  → Priority: 2  Severity: S3
  - Disposition: Implement  LHF: Yes
  - Feasibility: High  Complexity: 2
  - Why update vs why not: Validate telemetry/policy under cross-FS.
  - Evidence: No current tests.
  - Next step: Add allowed/disallowed EXDEV tests with assertions.

- Doc: TEST_COVERAGE_MAP.md — Title: Sidecar corruption handling lacks tests
  - Category: Missing Feature
  - Impact: 3  Likelihood: 2  Confidence: 3  → Priority: 2  Severity: S3
  - Disposition: Implement  LHF: No
  - Feasibility: Medium  Complexity: 3
  - Why update vs why not: Hardens restore.
  - Evidence: Only positive-path snapshot tests.
  - Next step: Add malformed JSON and mismatch tests.

- Doc: MIGRATION_GUIDE.md — Title: Deprecation communication lacks codified channel
  - Category: Documentation Gap
  - Impact: 3  Likelihood: 3  Confidence: 4  → Priority: 3  Severity: S2
  - Disposition: Implement  LHF: Yes
  - Feasibility: High  Complexity: 2
  - Why update vs why not: Reduce surprises with clear comms.
  - Evidence: Shims without `#[deprecated]`; guide lacks concrete comms steps.
  - Next step: Add deprecation attributes and CHANGELOG section; examples in guide.

- Doc: MIGRATION_GUIDE.md — Title: No equivalence tests between low-level atoms and helpers
  - Category: Missing Feature
  - Impact: 3  Likelihood: 2  Confidence: 3  → Priority: 2  Severity: S3
  - Disposition: Implement  LHF: No
  - Feasibility: High  Complexity: 3
  - Why update vs why not: Ensure migration preserves behavior.
  - Evidence: High-level helpers exist; no dedicated tests.
  - Next step: Add side-by-side tests and document differences.

- Doc: ROADMAP.md — Title: Roadmap lacks consumer feedback loop
  - Category: Documentation Gap
  - Impact: 3  Likelihood: 3  Confidence: 3  → Priority: 3  Severity: S2
  - Disposition: Spec-only  LHF: Yes
  - Feasibility: High  Complexity: 2
  - Why update vs why not: Aligns priorities with real needs.
  - Evidence: No linked consumer input.
  - Next step: Add feedback column and links; adjust priorities accordingly.

- Doc: ROADMAP.md — Title: No delivery timeline or version targeting
  - Category: Documentation Gap
  - Impact: 2  Likelihood: 3  Confidence: 4  → Priority: 2  Severity: S3
  - Disposition: Spec-only  LHF: Yes
  - Feasibility: High  Complexity: 1
  - Why update vs why not: Improves planning for adopters.
  - Evidence: No versions/dates.
  - Next step: Add target versions/quarters.

- Doc: ROADMAP.md — Title: Acceptance criteria miss E2E consumer validation
  - Category: Missing Feature
  - Impact: 3  Likelihood: 3  Confidence: 4  → Priority: 3  Severity: S2
  - Disposition: Implement  LHF: Yes
  - Feasibility: High  Complexity: 2
  - Why update vs why not: Ensures user-visible success.
  - Evidence: Unit/integration only.
  - Next step: Add E2E workflow criteria.

- Doc: CODING_STANDARDS.md — Title: No automated enforcement beyond clippy
  - Category: Missing Feature
  - Impact: 2  Likelihood: 3  Confidence: 4  → Priority: 2  Severity: S3
  - Disposition: Implement  LHF: Yes
  - Feasibility: High  Complexity: 2
  - Why update vs why not: Reduce review burden with hooks/CI.
  - Evidence: Lints in `src/lib.rs:1-3` only.
  - Next step: Add hooks and CI job.

- Doc: CODING_STANDARDS.md — Title: Error-handling pattern consistency not enforced
  - Category: Documentation Gap
  - Impact: 2  Likelihood: 2  Confidence: 3  → Priority: 2  Severity: S3
  - Disposition: Spec-only  LHF: Yes
  - Feasibility: High  Complexity: 1
  - Why update vs why not: Provide templates to prevent drift.
  - Evidence: `src/types/errors.rs` patterns; no template.
  - Next step: Add pattern section and checklist.

- Doc: CODING_STANDARDS.md — Title: Module organization principles not captured in ADRs
  - Category: Documentation Gap
  - Impact: 2  Likelihood: 2  Confidence: 4  → Priority: 1  Severity: S4
  - Disposition: Spec-only  LHF: Yes
  - Feasibility: High  Complexity: 1
  - Why update vs why not: Capture rationale in ADRs.
  - Evidence: Conventions exist; no ADR.
  - Next step: Write ADR and link here.

- Doc: CONTRIBUTING_ENHANCEMENTS.md — Title: Toolchain pinning and contributor environment guidance
  - Category: Documentation Gap
  - Impact: 2  Likelihood: 2  Confidence: 4  → Priority: 1  Severity: S4
  - Disposition: Spec-only  LHF: Yes
  - Feasibility: High  Complexity: 1
  - Why update vs why not: Reduce drift with explicit toolchain notes.
  - Evidence: `rust-toolchain.toml` exists.
  - Next step: Link file; note required components.

- Doc: CONTRIBUTING_ENHANCEMENTS.md — Title: Prevent tests from using system paths
  - Category: Missing Feature
  - Impact: 2  Likelihood: 3  Confidence: 4  → Priority: 2  Severity: S3
  - Disposition: Implement  LHF: Yes
  - Feasibility: High  Complexity: 2
  - Why update vs why not: Avoid recurring permission flakes.
  - Evidence: Guidance exists; lack of enforcement.
  - Next step: Add CI grep and a test template using `tempfile`.

- Doc: CONTRIBUTING_ENHANCEMENTS.md — Title: Feature flags documentation incomplete
  - Category: Documentation Gap
  - Impact: 2  Likelihood: 3  Confidence: 3  → Priority: 2  Severity: S3
  - Disposition: Spec-only  LHF: Yes
  - Feasibility: High  Complexity: 1
  - Why update vs why not: Improve DX by documenting dev flags.
  - Evidence: Doc references flags without comprehensive list.
  - Next step: Add a feature flags section with examples.

Recorded in Round 3 by AI 1 on 2025-09-12 15:44 +02:00

## Round 4 Implementation Plans (AI 1, 2025-09-12 16:09 +02:00)

Plans target: `FS_SAFETY_AUDIT.md`, `API_SURFACE_AUDIT.md`, `OBSERVABILITY_FACTS_SCHEMA.md`, `ERROR_TAXONOMY.md`, `INDEX.md`.
Prioritization: S2 items first, include LHF quick wins + 1–2 medium items.

---

### Plan 1 — Backup/Sidecar durability: fsync files and parent directories

- Summary
  Ensure backups and sidecars are durably persisted by syncing files and parent dirs after creation.

- Problem/gap addressed
  FS_SAFETY_AUDIT — Round 2 Gap: missing parent-directory fsync after backup/sidecar creation; file `sync_all()` not called.

- Scope: code targets
  - `cargo/switchyard/src/fs/backup.rs::create_snapshot()`
  - `cargo/switchyard/src/fs/backup.rs::write_sidecar()`
  - Uses `cargo/switchyard/src/fs/atomic.rs::fsync_parent_dir()`

- Changes (exact)
  1) In `create_snapshot()` regular-file branch:
     - After `std::io::copy`, call `dfile.sync_all()?`.
     - After sidecar write returns `Ok(())`, call `fsync_parent_dir(&backup_pb)?`.
  2) In symlink branch:
     - After `unix::fs::symlink(curr, &backup)`, call `fsync_parent_dir(&backup)?`.
     - After sidecar write, call `fsync_parent_dir(&backup)?` again (no-op if already synced, but safe).
  3) In tombstone branch:
     - Hold the created `File` handle; call `f.sync_all()?` before drop.
     - After sidecar write, call `fsync_parent_dir(&backup)?`.
  4) In `write_sidecar(backup, sc)`:
     - Keep `let mut f = std::fs::File::create(&sc_path)?;` then `serde_json::to_writer_pretty(&mut f, sc)?;` then `f.sync_all()?;` and finally `fsync_parent_dir(&sc_path)?`.

- Tests
  - Unit: Extend `snapshot_*` tests in `fs/backup.rs` to ensure functions still succeed across file, symlink, none.
  - Integration (optional): Add a smoke test that exercises `create_snapshot()` under concurrency (spawn two creators) to ensure no races and successful returns.
  - Note: fsync effects aren’t directly observable; rely on code path execution and no regressions.

- Telemetry/docs
  - Update FS_SAFETY_AUDIT after code lands to reflect durability guarantees (post-Round 4 code PR).

- Feasibility / Complexity / Effort
  High / 2 / S

- Risks and mitigations
  - Small perf hit from fsync; acceptable for backup path. If needed, add a policy toggle in a future change.

- Dependencies
  None.

- Rollout plan
  Single PR; no API changes. Land with changelog note under “Reliability”.

- Acceptance criteria
  - All backup creation paths call `sync_all()` on files and `fsync_parent_dir()` on parent dirs.
  - Existing tests pass; no API changes; CI green.

---

### Plan 2 — Enforce `SafePath` on mutating public APIs

- Summary
  Enforce path-safety by requiring `SafePath` or early validation in all mutating public APIs.

- Problem/gap addressed
  FS_SAFETY_AUDIT — Round 2 Gap: not all mutating APIs enforce `SafePath`.

- Scope: code targets
  - `cargo/switchyard/src/fs/swap.rs::replace_file_with_symlink()`
  - `cargo/switchyard/src/fs/restore.rs::{restore_file, restore_file_prev}`
  - `cargo/switchyard/src/api/**` public entry points that accept filesystem targets
  - `cargo/switchyard/src/fs/paths.rs::is_safe_path()` and `types` SafePath type

- Changes (exact)
  1) Public APIs: add overloads accepting `&SafePath` and prefer them in docs/examples.
  2) Retain `&Path` versions temporarily but validate early with `is_safe_path(&p)`; on failure, return a typed error (see Plan 5 taxonomy).
  3) Annotate `&Path` variants `#[deprecated(note = "Use SafePath variants to enforce traversal safety")]` with a 1-release deprecation window.

- Tests
  - Unit: Add tests that `../..` traversal and absolute unsafe paths are rejected with correct error IDs.
  - Integration: Replace usage in existing higher-level tests with `SafePath` constructors.

- Telemetry/docs
  - Error facts should include `E_TRAVERSAL` (see Plan 5).
  - Update API docs and MIGRATION_GUIDE after code lands.

- Feasibility / Complexity / Effort
  Medium / 3 / M

- Risks and mitigations
  - Potential breaking surface if immediate removal; mitigate with deprecated shims and clear migration guide.

- Dependencies
  Plan 5 (taxonomy) for stable error ID.

- Rollout plan
  Phase 1: add SafePath overloads + deprecate Path versions. Phase 2 (next minor): remove deprecated variants.

- Acceptance criteria
  - All public mutating APIs either take `SafePath` or validate `&Path` and return `E_TRAVERSAL` on unsafe input.
  - New tests cover negative traversal cases.

---

### Plan 3 — Restrict low-level FS atoms from the public API

- Summary
  Make internals non-public: stop re-exporting low-level atoms and hide them from docs.

- Problem/gap addressed
  API_SURFACE_AUDIT — Round 2 Gap: `fs/mod.rs` re-exports internal atoms intended for internal use.

- Scope: code targets
  - `cargo/switchyard/src/fs/mod.rs` — `pub use atomic::{atomic_symlink_swap, fsync_parent_dir, open_dir_nofollow};`
  - `cargo/switchyard/src/lib.rs` — crate-root re-exports

- Changes (exact)
  1) In `fs/mod.rs`, change to `pub(crate) use ...` or remove the re-exports; keep `atomic` module internal to crate consumers.
  2) Add `#[doc(hidden)]` to the low-level functions if they remain reachable.
  3) Add deprecation to any symbols that were public and will be removed next minor.
  4) Add `tests/public_api.rs` that imports only intended stable helpers to enforce surface stability.

- Tests
  - `tests/public_api.rs` compiling against intended imports; ensure no accidental reliance on internal atoms.

- Telemetry/docs
  - Update API_SURFACE_AUDIT and MIGRATION_GUIDE post-merge; CHANGELOG entry with deprecation timeline.

- Feasibility / Complexity / Effort
  High / 2 / S

- Risks and mitigations
  - Downstream breakage if removed too quickly; mitigate with deprecation period and migration notes.

- Dependencies
  None.

- Rollout plan
  Deprecate now, remove next minor release. Provide import examples for high-level helpers.

- Acceptance criteria
  - Low-level atoms no longer publicly re-exported; doc-hidden where applicable.
  - `tests/public_api.rs` passes; downstream example builds use only high-level helpers.

---

### Plan 4 — Observability: add `summary_error_ids` to facts and emitters

- Summary
  Introduce `summary_error_ids: string[]` in summary facts and populate from error chains.

- Problem/gap addressed
  OBSERVABILITY_FACTS_SCHEMA — Round 2 Gap: summary only exposes a single `error_id`, limiting diagnostics.

- Scope: code targets
  - `cargo/switchyard/SPEC/audit_event.schema.json`
  - `cargo/switchyard/src/api/apply/**` facts emitters (e.g., `apply/audit_fields.rs`, `apply/handlers.rs`)
  - `cargo/switchyard/src/logging/redact.rs` (ensure redaction preserves `summary_error_ids`)
  - Preflight and rollback facts builders (modules under `src/api/`)

- Changes (exact)
  1) Schema: add optional `summary_error_ids` to `apply.result.summary`, `preflight.summary`, and rollback summaries.
  2) Implement helper `fn collect_error_ids(err: &impl std::error::Error) -> Vec<&'static str>` in `types/errors.rs` or `api/errors.rs`.
  3) Populate `summary_error_ids` from `collect_error_ids()` where facts summaries are built.
  4) Update redactor to keep the field intact in both normal and dry-run modes.

- Tests
  - Add `tests/facts_schema.rs` to validate events against the updated JSON Schema (preflight, apply, rollback sample fixtures).
  - Unit: construct a synthetic multi-cause error and verify emitted `summary_error_ids` includes all expected IDs.

- Telemetry/docs
  - Update OBSERVABILITY_FACTS_SCHEMA after code lands; document field semantics and stability.

- Feasibility / Complexity / Effort
  High / 3 / M

- Risks and mitigations
  - Consumer parsers may ignore the new field; additive and optional minimizes risk.

- Dependencies
  Plan 5 for consistent error IDs across layers.

- Rollout plan
  Single PR updating schema + emitters + tests; version bump for SPEC minor.

- Acceptance criteria
  - JSON Schema includes `summary_error_ids` and validates.
  - Apply/Preflight/Rollback summaries populate the field in error cases.
  - Redaction preserves the field.

---

### Plan 5 — Error taxonomy: multi-cause surfacing and consistent ownership tags

- Summary
  Provide canonical error IDs with chain collection and ensure ownership-related errors are consistently tagged.

- Problem/gap addressed
  ERROR_TAXONOMY — Round 2 Gap: only a single summary ID; inconsistent `E_OWNERSHIP` tagging.

- Scope: code targets
  - `cargo/switchyard/src/types/errors.rs`
  - `cargo/switchyard/src/api/errors.rs`
  - Call sites constructing/propagating errors in `src/fs/**` and `src/api/**`

- Changes (exact)
  1) Define `ErrorId` enum and implement `fn error_id(&self) -> &'static str` on relevant error types.
  2) Add `fn error_ids_chain(&self) -> Vec<&'static str>` walking `source()` chain.
  3) Normalize ownership errors to include `E_OWNERSHIP` alongside specific causes.
  4) Expose helper used by Plan 4 emitters.

- Tests
  - Unit: construct chained errors and assert `error_ids_chain()` content.
  - Integration: trigger an ownership failure path and assert presence of `E_OWNERSHIP` in facts (via Plan 4).

- Telemetry/docs
  - Update ERROR_TAXONOMY post-merge with examples and mapping table.

- Feasibility / Complexity / Effort
  Medium / 3 / M

- Risks and mitigations
  - Slight increase in error plumbing; mitigate with helper functions and clear patterns.

- Dependencies
  None (but Plan 4 will consume this).

- Rollout plan
  Single PR adjusting error types and adding tests.

- Acceptance criteria
  - Error types expose stable IDs; chain collection works.
  - Ownership failures surface `E_OWNERSHIP` consistently and appear in facts via Plan 4.

---

### Plan 6 — INDEX: add missing analyses and dynamic status tracking

- Summary
  Add coverage for package manager interoperability and activation persistence; introduce per-doc round status tracking.

- Problem/gap addressed
  INDEX — Round 2 Gap: missing dedicated analyses and lack of dynamic round status tracking.

- Scope: code/docs targets
  - `cargo/switchyard/DOCS/analysis/INDEX.md`
  - New docs: `PKG_MGR_INTEGRATION.md`, `ACTIVATION_PERSISTENCE.md` (under `DOCS/analysis/`)

- Changes (exact)
  1) Add table rows in INDEX for the two new analyses with owners and objectives.
  2) Create stub docs with outline (problem statements, scope, evidence checklist).
  3) Add a status table per doc indicating R1–R4 progress and links.

- Tests
  - CI docs link checker to ensure references resolve (add or extend existing doc-check workflow).

- Telemetry/docs
  - N/A beyond documentation updates.

- Feasibility / Complexity / Effort
  High / 1 / S

- Risks and mitigations
  - Minimal; keep scope-focused to avoid thrash.

- Dependencies
  None.

- Rollout plan
  Single docs PR; assign owners and due dates.

- Acceptance criteria
  - INDEX shows two new analyses with owners; status table present.
  - Stubs exist with outlines ready for R1.

Recorded in Round 4 by AI 1 on 2025-09-12 16:09 +02:00
