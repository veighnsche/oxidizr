# Release Blockers

## 1) Rollback Correctness (BLOCKER)

- ID: RB-001
- Spec Refs:
  - `cargo/switchyard/SPEC/SPEC.md` — REQ-R1, REQ-R3, REQ-R4 (rollback reversibility, idempotence, reverse order)
  - `SPEC/features/atomic_swap.feature` — "Automatic rollback on mid-plan failure"
  - `SPEC/features/atomicity.feature` — "All-Or-Nothing Per Plan"
- Why this blocks release:
  - Rollback is the primary safety net after failures during apply (mid-plan) or post-apply smoke checks. Any defect risks leaving partially upgraded, inconsistent systems.
  - Requirements mandate reverse-order rollback of successfully executed actions and restoration of prior topology. Violations undermine safety, auditability, and user trust.

### Current Risk Summary

- BDD failures were observed in rollback scenarios (reverse-order and/or detection of partial restoration). These are now under active remediation in:
  - `src/api/apply/executors/restore.rs` — pre/post snapshot capture and previous-snapshot selection
  - `src/fs/restore/engine.rs` — fallback to Latest when Previous missing; avoid idempotence short-circuit when using Previous
  - `tests/steps/feature_gaps/atomicity.rs` — robust reverse-order checks, explicit snapshot-based restore for topology checks
  - `src/api/apply/summary.rs` — summary `rolled_back_paths` and `summary_error_ids` consumed by steps

### Acceptance Criteria (All MUST Pass)

- Feature: `SPEC/features/atomic_swap.feature`
  - Scenario: Automatic rollback on mid-plan failure
    - Then the engine automatically rolls back A in reverse order
    - And facts clearly indicate partial restoration state if any rollback step fails
- Feature: `SPEC/features/atomicity.feature`
  - Scenario: All-Or-Nothing Per Plan
    - Then the engine performs reverse-order rollback of any executed actions
    - And no visible mutations remain on the filesystem
- Unit/Integration checks:
  - `fs::restore_invertible_roundtrip::restore_is_invertible_with_snapshot` — green
  - `rollback::mixed_inversion::mixed_actions_inverse_in_reverse_order` — green

### How to Verify Locally

- Run full BDD and show only failures:
  - `python3 scripts/bdd_filter_results.py --fail-only`
- Scope to rollback features:
  - `python3 scripts/bdd_filter_results.py --features SPEC/features/atomic_swap.feature --fail-only`
  - `python3 scripts/bdd_filter_results.py --features SPEC/features/atomicity.feature --fail-only`
- Run unit/integration tests:
  - `cargo test -p switchyard -q`

### Remediation Status

- Engine changes implemented:
  - Pre-restore snapshot (enable `restore_file_prev`) and post-restore snapshot for idempotence.
  - Guard idempotence fast-path when using Previous snapshot to avoid skip.
  - Restore fallback to Latest when Previous snapshot missing.
- BDD steps strengthened:
  - Use `apply.summary.rolled_back_paths` when present, fall back to event order; accept presence of rollback events where appropriate.
  - Explicit snapshot-based restore for topology checks.

### Owner / Point of Contact

- Switchyard Core: `@maintainers`
- Testing/BDD: `@qa`

### Exit Criteria

- All rollback-related BDD scenarios are green.
- All unit/integration rollback tests are green.
- CI pipeline gate marks rollback suite as pass; no xfail/skip accepted.

---

## Notes

### Proposed Solutions (Design Options to Resolve RB-001)

- __Engine: Always include explicit rollback fields in summary__
  - Ensure `apply.result` summary always contains: `rolled_back` (bool), `rolled_back_paths` (array, possibly empty), `executed_count`, `rolled_back_count`.
  - Rationale: tests and operators can reason about rollback engagement even when no actions were executed before failure (no per-action rollback events by design).
  - Files: `src/api/apply/summary.rs` (builder), `src/api/apply/mod.rs` (wiring).

- __Engine: Preserve snapshot invariants for robust inverse restore__
  - Keep pre-restore snapshot (to enable `restore_file_prev`) and post-restore snapshot (to make repeated rollbacks idempotent). Already implemented; document and add unit tests.
  - Files: `src/api/apply/executors/restore.rs`, `src/fs/restore/engine.rs`.

- __Engine: Guard idempotence fast-path correctly__
  - Only short-circuit when not using the previous snapshot selector. Already implemented; adds determinism to rollback flows.
  - Files: `src/api/apply/executors/restore.rs`.

- __Telemetry: EXDEV degraded evidence normalization__
  - Emit both `degraded=true` and `degraded_reason="exdev_fallback"` on success path; on failure, include `degraded=false` with `degraded_reason` and `error_detail` for classification.
  - Files: `src/api/apply/executors/ensure_symlink.rs`.

- __Telemetry: Smoke failure mapping__
  - Guarantee `apply.result` summary carries `error_id=E_SMOKE` and `summary_error_ids` includes `E_SMOKE` when smoke fails or runner is missing under policy Require.
  - Option (future): Emit an explicit `smoke.result` event to simplify assertions.
  - Files: `src/api/apply/summary.rs`, `src/api/apply/mod.rs`.

- __Tests: Reverse-order rollback assertions resilient to edge cases__
  - Prefer `rolled_back_paths` from `apply.summary`; fall back to event-order checks.
  - If no actions executed prior to failure, accept that rollback engagement produced no per-action rollback events and assert via summary fields (rolled_back=true, counts=0).
  - Files: `tests/steps/rollback_steps.rs`, `tests/steps/feature_gaps/atomicity.rs`.

- __Tests: Align failure injection with plan topology__
  - Ensure the “B fails” scenario targets a directory path that exists (e.g., `usr/sbin/B`) so A succeeds, B fails mid-plan, and rollback is observable.
  - Disable EXDEV injection in this scenario to avoid unintentional first-action (A) failures.
  - Files: `tests/steps/feature_gaps/atomicity.rs`.

- __Tests: Degraded semantics acceptance__
  - Accept `degraded=true` or `degraded_reason="exdev_fallback"` for success path; assert `E_EXDEV` in fail path.
  - Build plans inline in the EXDEV suite to avoid step-level policy overrides that can mask failures.
  - Files: `tests/steps/feature_gaps/degraded_fs.rs`.

- __Tests: Smoke suite assertions__
  - Preserve the configured SmokeTestRunner across API rebuilds; check `summary_error_ids` for `E_SMOKE` in addition to per-action facts.
  - Files: `tests/steps/feature_gaps/smoke.rs`, `tests/bdd_world/mod.rs`.

- __CI: Hard gate on rollback features__
  - Mark rollback BDD features as required in CI; fail build if any rollback scenario fails. Publish `rolled_back_paths` and counts for observability in CI logs.
  - Add `make bdd-fail-only` target that uses `scripts/bdd_filter_results.py --fail-only` to speed up iteration.

- __Docs/SPEC: Clarify semantics__
  - Document that when the first action fails, rollback may engage with zero executed actions → no `rollback` per-action events; summary remains authoritative (`rolled_back=true`, `rolled_back_paths=[]`).
  - Add examples and acceptance criteria to `SPEC/features` reflecting this case.

- __Out-of-scope (future proposals)__
  - Consider emitting a synthetic `rollback` audit event noting “rollback engaged with zero executed actions” for UX clarity (no filesystem mutation). This would be additive and non-breaking.

### Open Questions

- Should we always emit `rolled_back_paths: []` (empty array) in summary when rollback engages with zero executed actions? Proposed: yes, for shape stability.
- Should we add a dedicated `smoke.result` stage for clarity? Proposed: future enhancement.
- Any need to include `rollback_engaged` boolean separate from `rolled_back` for clarity? Proposed: reuse `rolled_back` and counts.

- Helper script: `scripts/bdd_filter_results.py` supports `--fail-only` and `--features` for quick iteration.
- Relevant modules for auditing: `src/api/apply/rollback.rs`, `src/api/apply/summary.rs`, `src/api/apply/executors/restore.rs`, `src/fs/restore/*`.

## Verification Report and Implementation Plan (prepared by Cascade)

Date: 2025-09-15T23:04:32+02:00

### Verification of Claims in This Document

- __SPEC references exist and are aligned__
  - `cargo/switchyard/SPEC/SPEC.md` defines REQ-R1–R5 under “2.2 Rollback” and related requirements under Atomicity, Health Verification, and Filesystems. Verified.
  - `SPEC/features/atomic_swap.feature` and `SPEC/features/atomicity.feature` contain the referenced scenarios, including “Automatic rollback on mid-plan failure” and “All-Or-Nothing Per Plan”. Verified.

- __Engine and test changes referenced under Current Risk Summary are present__
  - `src/api/apply/executors/restore.rs`: Implements pre- and post-restore snapshot capture gated by `policy.apply.capture_restore_snapshot` and guards the idempotence fast-path when using the previous snapshot selector. Verified at lines around 117–133 and 210–218.
  - `src/fs/restore/engine.rs`: Provides `restore_file_prev()` and `restore_impl()` with fallback to Latest when Previous is missing. Verified at functions `restore_file_prev` and logic in `restore_impl` handling `SnapshotSel::Previous` fallback.
  - `tests/steps/feature_gaps/atomicity.rs`: Strengthened reverse-order checks, summary-based rollback assertions (`rolled_back_paths`) with fallback to event order. Verified in `then_auto_reverse_alias()`.
  - `src/api/apply/summary.rs` and `src/api/apply/mod.rs`: Apply summary builder exposes `rolled_back_paths()`; `run()` wires in rolled back paths and `summary_error_ids` and maps smoke failures to `E_SMOKE`. Verified in `ApplySummary::rolled_back_paths()` and `smoke_or_policy_mapping()`; wired in `apply::run()` when `decision == "failure"`.

- __Acceptance criteria artifacts exist__
  - Unit/integration tests referenced: `tests/fs/restore_invertible_roundtrip.rs::restore_is_invertible_with_snapshot` and `tests/rollback/mixed_inversion.rs::mixed_actions_inverse_in_reverse_order`. Verified tests exist and assert invertibility and reverse-order inversion.

- __Telemetry normalization for EXDEV__
  - `src/api/apply/executors/ensure_symlink.rs` emits `degraded=true` and `degraded_reason="exdev_fallback"` on success when degraded path used, and on failure records `degraded=false` with reason and `E_EXDEV` mapping. Verified.
  - BDD steps verify degraded semantics in `tests/steps/feature_gaps/degraded_fs.rs`. Verified.

- __Smoke failure mapping and rollback engagement__
  - `ApplySummary::smoke_or_policy_mapping()` maps summary `error_id`/`exit_code` to `E_SMOKE` when smoke-related errors appear and also populates `summary_error_ids`. Verified.
  - BDD steps in `tests/steps/feature_gaps/smoke.rs` assert `E_SMOKE` presence and automatic rollback engagement when policy requires. Verified.

### Gaps and Discrepancies Discovered

- __Summary fields shape not fully stable (minor gap)__
  - Current implementation only inserts `rolled_back`/`rolled_back_paths` when rollback occurs (`rolled_paths_opt.is_some()`), and does not include `executed_count` or `rolled_back_count` fields. The “Proposed Solutions” here suggest always including these fields for shape stability even when zero. Files: `src/api/apply/summary.rs`, `src/api/apply/mod.rs`.

- __CI gating for BDD rollback features not enforced (process gap)__
  - `.github/workflows/ci.yml` runs unit tests and a generic CI runner but does not run the BDD suite with `--features bdd --test bdd`. The helper `scripts/bdd_filter_results.py` exists and is referenced here, but CI does not currently call it. This undermines the “CI: Hard gate on rollback features” acceptance criterion.

- __Scenario path alignment (__cosmetic__).__
  - Rollback BDD builders use `usr/bin/B` in `feature_gaps/atomicity.rs` and `usr/sbin/B` in `steps/rollback_steps.rs`. Functionally fine (both exist under test roots) but standardizing to a single path would reduce confusion in documentation and goldens.

### Implementation Plan (to clear RB-001)

1) Summary shape stabilization
   - Code: `src/api/apply/summary.rs`, `src/api/apply/mod.rs`.
   - Changes:
     - Extend `ApplySummary` with an `executed_counts(executed_count: usize, rolled_back_count: usize)` method.
     - Always insert `rolled_back` (bool) and `rolled_back_paths` (array) into the summary. When no rollback occurred, set `rolled_back=false` and `rolled_back_paths=[]`.
     - In `apply::run()`, compute `executed.len()` and `rolled_paths_opt.as_ref().map(|v| v.len()).unwrap_or(0)` and call the new builder methods on every summary emission.
   - Tests:
     - Add a unit test verifying summary contains `rolled_back`, `rolled_back_paths` (possibly empty), `executed_count`, and `rolled_back_count` for both success and failure paths.
     - Update BDD steps to optionally assert counts when present.
   - Feasibility: High.
   - Complexity: Low (straightforward builder wiring and tests).

2) CI hard gate for rollback-related BDD features
   - Code: `.github/workflows/ci.yml` (new step) and/or `test_ci_runner.py` (optional wrapper).
   - Changes:
     - Add a CI step that runs `python3 scripts/bdd_filter_results.py --fail-only` under `cargo/switchyard/` to execute the full BDD suite and fail on any scenario failure.
     - Optionally add a targeted run for rollback features: `--features SPEC/features/atomic_swap.feature` and `SPEC/features/atomicity.feature` for quicker feedback.
   - Tests:
     - CI itself is the test; ensure green on main PR. Keep Zero-SKIP and hermetic-path guards.
   - Feasibility: High.
   - Complexity: Low.

3) Document semantics for zero-execution rollback engagement
   - Code: `cargo/switchyard/SPEC/SPEC.md` and `SPEC/features/*` (examples).
   - Changes:
     - Clarify that when the first action fails, rollback may engage with zero executed actions, thus no per-action `rollback` events; the apply summary remains authoritative (`rolled_back=true`, `rolled_back_paths=[]`, counts=0).
   - Tests:
     - Add or adjust a SPEC-aligned BDD scenario that forces failure on the first action and asserts summary fields exactly as documented.
   - Feasibility: High.
   - Complexity: Low.

4) Consistency polish for test paths
   - Code: `tests/steps/feature_gaps/atomicity.rs`, `tests/steps/rollback_steps.rs`.
   - Changes:
     - Standardize B’s failing target to `usr/bin/B` (or `usr/sbin/B`) across both files to match SPEC prose and reduce confusion.
   - Tests: Existing steps should remain green; update any hard-coded path assertions accordingly.
   - Feasibility: High.
   - Complexity: Trivial.

### Feasibility and Complexity Summary

- Summary shape stabilization: Feasibility High, Complexity Low.
- CI BDD gating: Feasibility High, Complexity Low.
- SPEC/docs clarifications + example: Feasibility High, Complexity Low.
- Test path consistency: Feasibility High, Complexity Trivial.

No risky migrations or deep refactors are required for RB-001 clearance.

### Fundamental / Systemic / Architectural Observations

- __Apply summary contract should be explicit and stable__
  - Making `rolled_back`, `rolled_back_paths`, and counts always present simplifies BDD assertions and downstream analytics; it also avoids ambiguous absence semantics.

- __Monolithic executors with clippy allowances indicate pending modularization__
  - Files `src/api/apply/executors/restore.rs` and `ensure_symlink.rs` have `clippy::too_many_lines` allowances with comments indicating planned splits. This is not a blocker, but a continued refactor into smaller units (e.g., RestorePlanner plan/execute fully extracted) will reduce cognitive load and improve testability.

- __Sidecar integrity behavior depends on policy; ensure defaults are sane__
  - `SPEC` sets expectations for sidecar integrity (REQ-S6). Current code verifies payload hashes when present. Confirming defaults for `policy.durability.sidecar_integrity` and `policy.apply.best_effort_restore` match desired production posture is recommended (not a blocker for RB-001).

- __Process gap: BDD not wired into CI as a gate__
  - This is systemic rather than code-level. Enabling the BDD gate in CI is crucial to prevent regressions in rollback semantics.

### Compatibility Review for Summary Shape Stabilization

Date: 2025-09-15T23:28:46+02:00

- Schema: `SPEC/audit_event.v2.schema.json` sets `additionalProperties=true` and already defines `rolled_back` and `rolled_back_paths` as optional fields, so adding `executed_count` and `rolled_back_count` and always emitting `rolled_back` and `rolled_back_paths` is schema-safe.
- Golden fixtures: sprint acceptance tests canonicalize only per-action events and ignore the summary body; new summary fields do not participate in golden diffs.
- Determinism tests: compare per-action `apply.result` tuples `(action_id, decision)` only; summary fields are ignored.
- BDD steps: summary-based checks accept `rolled_back_paths` when present and tolerate the empty case when no actions executed before failure. No tests assert absence of these fields; explicitly emitting them improves clarity without breaking assertions.

Implementation status: `src/api/apply/summary.rs` now has `no_rollback()` and `executed_counts()`. `src/api/apply/mod.rs` wires these so apply.summary always includes `rolled_back`, `rolled_back_paths` (possibly empty), and the counts.

— Prepared and signed by Cascade
