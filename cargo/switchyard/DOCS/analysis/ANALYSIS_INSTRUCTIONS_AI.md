# Analysis Work Plan for 4 AI Analysts (4 Rounds)

Generated: 2025-09-12 14:34:15+02:00
Coordinator: Cascade
Scope: All docs in `cargo/switchyard/DOCS/analysis/` (excluding this file). AIs must verify claims, provide proofs, and patch gaps in their assigned analysis docs. Four rounds are planned; each round has its own instruction file. Begin with Round 1 and await coordinator signal between rounds.

## Objectives

- Verify every claim in each analysis document by citing code, tests, and SPEC/PLAN references.
- Patch missing or incorrect content directly in the assigned analysis files.
- Produce a per-AI report (`AI_ANALYSIS_#.md`) containing evidence and change summaries.
- Execute four rotations: peer review, meta-review, severity reporting, and implementation planning.

## Working Rules

- Only modify:
  1) Your assigned analysis files (listed below for your AI number), and
  2) Your own `AI_ANALYSIS_#.md` file.
- Do NOT edit peers’ `AI_ANALYSIS_#.md` files or analysis docs outside your assignment.
- Use precise citations to code and specs. Preferred citation styles:
  - Code: `path/to/file.rs: function_name()` or `path/to/file.rs: Lx–Ly`
  - SPEC/PLAN: `SPEC/<file>#section` or `PLAN/<file>#section`
- Provide proofs for all normative statements (e.g., “fsync is called after rename” → cite `src/fs/atomic.rs::atomic_symlink_swap()` and show `fsync_parent_dir()` invocation).
- If a claim cannot be proven, either remove it or fix it with an accurate replacement and add references.

## Accepted Evidence

- Code references (functions, modules, constants) matching current repo state.
- Test references and outputs, including env-gated behavior (e.g., `SWITCHYARD_FORCE_EXDEV`, `SWITCHYARD_FORCE_RESCUE_OK`).
- SPEC and PLAN references (Reproducible v1.1) aligning with the claim.

## Rounds and instruction files

- Round 1 — Peer Review: `ROUND_1_PEER_REVIEW.md`
- Round 2 — Meta Review: `ROUND_2_META_REVIEW.md`
- Round 3 — Severity Reports: `ROUND_3_SEVERITY_REPORTS.md`
- Round 4 — Implementation Plans: `ROUND_4_IMPL_PLANS.md`

Follow these sequentially. Do not start a new round until the coordinator signals.

## Work-Effort Scoring (for fair delegation)

Effort score ~= complexity-weighted length (approx points by size × complexity).

- API_SURFACE_AUDIT.md — 10
- BACKWARDS_COMPAT_SHIMS.md — 6
- BEHAVIORS.md — 9
- CLI_INTEGRATION_GUIDE.md — 2
- CODING_STANDARDS.md — 1
- CONTRIBUTING_ENHANCEMENTS.md — 1
- CORE_FEATURES_FOR_EDGE_CASES.md — 15
- EDGE_CASES_AND_BEHAVIOR.md — 20
- ERROR_TAXONOMY.md — 7
- EXPERIMENT_CONSTANTS_REVIEW.md — 4
- FS_SAFETY_AUDIT.md — 10
- INDEX.md — 2
- LOCKING_STRATEGY.md — 6
- MIGRATION_GUIDE.md — 1
- OBSERVABILITY_FACTS_SCHEMA.md — 8
- PERFORMANCE_PLAN.md — 3
- POLICY_PRESETS_RATIONALE.md — 6
- PREFLIGHT_MODULE_CONCERNS.md — 8
- PRESERVATION_FIDELITY.md — 8
- REEXPORTS_AND_FACADES.md — 4
- RELEASE_AND_CHANGELOG_POLICY.md — 1
- RETENTION_STRATEGY.md — 3
- ROADMAP.md — 1
- SECURITY_REVIEW.md — 2
- TEST_COVERAGE_MAP.md — 4
- idiomatic_todo.md — 6

Total points: 148; target per AI ≈ 37.

## Assignments (Round 1 ownership)

- AI 1 — 37 pts: EDGE_CASES_AND_BEHAVIOR.md (20), CORE_FEATURES_FOR_EDGE_CASES.md (15), CLI_INTEGRATION_GUIDE.md (2)
- AI 2 — 37 pts: FS_SAFETY_AUDIT.md (10), API_SURFACE_AUDIT.md (10), OBSERVABILITY_FACTS_SCHEMA.md (8), ERROR_TAXONOMY.md (7), INDEX.md (2)
- AI 3 — 37 pts: PRESERVATION_FIDELITY.md (8), PREFLIGHT_MODULE_CONCERNS.md (8), POLICY_PRESETS_RATIONALE.md (6), LOCKING_STRATEGY.md (6), idiomatic_todo.md (6), SECURITY_REVIEW.md (2), RELEASE_AND_CHANGELOG_POLICY.md (1)
- AI 4 — 37 pts: BACKWARDS_COMPAT_SHIMS.md (6), BEHAVIORS.md (9), EXPERIMENT_CONSTANTS_REVIEW.md (4), REEXPORTS_AND_FACADES.md (4), RETENTION_STRATEGY.md (3), PERFORMANCE_PLAN.md (3), TEST_COVERAGE_MAP.md (4), MIGRATION_GUIDE.md (1), ROADMAP.md (1), CODING_STANDARDS.md (1), CONTRIBUTING_ENHANCEMENTS.md (1)

## Round gates

- Proceed to Round 1 now, following `ROUND_1_PEER_REVIEW.md`.
- Await coordinator signal before starting Round 2, 3, and 4.

## Suggested Commands (optional)

- Search: `rg --smart-case "fn\s+atomic_symlink_swap|fsync_parent_dir" cargo/switchyard/src`
- Tests: `cargo test -p switchyard` (add env vars as needed)
- Schema validation: use `jsonschema` tooling for facts fixtures

## Quality Bar

- Clear, reproducible citations.
- Minimal, precise edits.
- Alignment with SPEC Reproducible v1.1 and current code.
