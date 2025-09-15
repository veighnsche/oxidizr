# Possible False Positives in Test Suite (switchyard)

This document summarizes areas in the Switchyard test suite that could allow false positives (tests passing without truly verifying behavior) and proposes mitigations. Findings reference specific files under `cargo/switchyard/`.

## Scope and Method

- Reviewed BDD step definitions in `tests/steps/` and supporting test utilities in `tests/bdd_support/` and `tests/bdd_world/`.
- Searched for potential anti-patterns (e.g., no-op Then steps, silent error drops, commented/ignored tests).
- Cross-checked schema/determinism assertions and harness isolation.

## Confirmed Safeguards (Good)

- **Fail-on-skip CI gate**
  - `tests/bdd_main.rs` uses `fail_on_skipped()`, preventing skipped BDD scenarios from passing unnoticed.
- **Harness isolation: no host mutation**
  - All human paths like `"/usr/bin/ls"` are mapped under a per-test TempDir via `tests/bdd_support/mod.rs::util::under_root()` and used by `tests/bdd_world/mod.rs::World::mk_symlink()`.
- **Schema validation and determinism checks**
  - `tests/steps/observability_steps.rs::then_validate_schema()` validates facts against `SPEC/audit_event.v2.schema.json` with helpful error messages.
  - Facts redaction/normalization helpers in `tests/bdd_support/facts.rs` enable deterministic comparisons.
- **Active skip checks**
  - No active `#[ignore]` annotations in tests; prior `#[ignore = ...]` lines are commented out.

## Risks Identified (Actionable)

1) **No-op Then steps (assert nothing)**

- `tests/steps/feature_gaps_steps.rs`:
  - `then_ci_fails(...)` (around line ~258): empty body.
  - `then_semantics_verified(...)` (around line ~605): empty body.
  - `then_runs_dry_default(...)` (around line ~201): empty body; relies on prior steps but does not assert here.

Risk: Any scenario that relies solely on these Then steps may pass without verifying behavior, violating anti-false-positive policy.

Mitigations:

- Replace with aliases to assertive steps or add concrete assertions:
  - `then_ci_fails` → assert a concrete failure signal (e.g., presence of `preflight.summary` with `decision=failure`, or a specific `error_id`/`exit_code`) or alias to an existing step such as `then_policy_violation`.
  - `then_semantics_verified` → alias to `then_apply_fails_exdev_50` and/or `then_emitted_degraded_true_reason` to concretely verify degraded/EXDEV semantics per SPEC.
  - `then_runs_dry_default` → alias to `then_side_effects_not_performed`, which already asserts no `apply.*` facts in the DryRun-by-default flow.

2) **Result-dropping of critical calls without unwrap (context-dependent)**

Some step definitions intentionally drop `Result` values (e.g., `let _ = api.apply(...);`) to allow scenarios to continue and assert via emitted facts later. Examples:

- `tests/steps/locks_steps.rs::when_two_apply_overlap()`
- `tests/steps/apply_steps.rs::{when_attempt_apply_commit, when_attempt_apply}`
- `tests/steps/safety_preconditions_steps.rs::when_restore_with_integrity()`
- Various helpers in `tests/steps/feature_gaps_steps.rs`

This is acceptable if every such `When` step is always followed by an assertive `Then` step that checks concrete outcomes (e.g., `E_LOCKING`, `E_POLICY`, `E_EXDEV`, `lock_wait_ms`). However, if a scenario omits those Then checks, errors may be silently ignored.

Mitigations:

- Where an error is expected, prefer capturing the `Result` and assert with `expect_err(...)` in a Then step, or record the outcome explicitly.
- Add a style rule: any `When` that drops a `Result` must be paired with a specific assertive `Then` in the same scenario.
- Optional CI lint: grep for `let _ = .*apply\(` and ensure the feature includes a matching assertive Then step name (heuristic allowlist).

3) **Heuristic code scanning assertions (brittle under refactor)**

- `tests/steps/feature_gaps_steps.rs::then_signature_requires_safepath()` scans source via `include_str!("../../src/api/mod.rs")` to check that public APIs don’t accept `&PathBuf` and reference `SafePath`.
- `tests/steps/feature_gaps_steps.rs::then_toctou_sequence_present()` scans for `open_dir_nofollow`, `renameat`, and `fsync_parent_dir` in implementation files.

These provide useful guardrails but are string-level checks that can false-negative/positive under non-semantic refactors.

Mitigations:

- Complement with unit/integration tests that exercise the behavior (e.g., TOCTOU-safe swap verified by filesystem effects and auditing) — many already exist.
- Consider compile-time checks (e.g., ensuring mutate APIs accept `&SafePath` in signatures via doc tests or dedicated unit tests that use type-checks).

## Additional Observations

- `observability_steps.rs` includes robust checks for schema version, hash fields, provenance fields, and timing; these materially reduce false positives.
- `bdd_support/facts.rs` redaction and normalization functions actively reduce flakiness without hiding behavior.
- `bdd_support/mod.rs::util::sp()` and `bdd_world::World` helpers ensure consistent SafePath construction within the temp root.

## Recommendations Summary

- **Implement/alias no-op Then steps** in `tests/steps/feature_gaps_steps.rs` to assert concrete outcomes.
- **Document style guidance**: dropping `Result` in a `When` requires subsequent assertive `Then` checks; otherwise, unwrap or expect errors.
- **Optional CI linting**:
  - Flag empty Then bodies in `tests/steps/*.rs`.
  - Flag `let _ = api.apply(...)` without subsequent assertive Then in the same scenario (heuristic).
- **Supplement heuristic scans** with existing behavioral tests and, where useful, add small compile-time or doctest checks tying public mutate APIs to `SafePath`.

## Evidence Pointers

- Fail-on-skip: `tests/bdd_main.rs`
- No-op Then steps: `tests/steps/feature_gaps_steps.rs::{then_ci_fails, then_semantics_verified, then_runs_dry_default}`
- Result dropping examples: `tests/steps/locks_steps.rs::when_two_apply_overlap`, `tests/steps/apply_steps.rs::{when_attempt_apply_commit, when_attempt_apply}`, `tests/steps/safety_preconditions_steps.rs::when_restore_with_integrity`
- Schema checks: `tests/steps/observability_steps.rs::then_validate_schema`
- Temp root mapping: `tests/bdd_support/mod.rs::util::under_root`, `tests/bdd_world/mod.rs::World::mk_symlink`
