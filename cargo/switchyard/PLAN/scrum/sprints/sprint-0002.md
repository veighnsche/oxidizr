# Switchyard Sprint 02 Plan (Planned)

Duration: 2 weeks
Team: 5 AI engineers (parallel lanes)
Drift Guard: Doc-sync (SPEC_UPDATE/ADR/PLAN) remains hard gate; no scope creep past drift threshold

Sprint Theme: Preflight Diff + Error Mapping + Smoke + Acceptance + Traceability

## Objectives

- Close remaining SPEC v1.1 gaps in preflight and error taxonomy; add minimal smoke runner; expand golden suite; wire traceability.
- Exercise acceptance-level behaviors deterministically (non-containerized subset), keep CI Gold gate stable.

## Capacity & Cadence

- 2-week sprint; 5 engineers at high utilization.
- Allocate 1 owner per story; pair where risk is higher (Errors/Exit Codes, Preflight Gating).
- Drift threshold enforced: pause scope if SPEC/PLAN/code sync risks appear.

## Stories & Tasks

1) Preflight Diff Rows (SPEC §4)
   - Implement per-action preflight diff rows per `SPEC/preflight.yaml` with stable ordering
   - Include keys: `action_id`, `path`, `current_kind`, `planned_kind`, `policy_ok`, `provenance{uid,gid,pkg}`, `notes`, `preservation{}` and `preservation_supported`
   - Ensure DryRun vs Commit byte identity post-redaction
   - Tests: unit tests for row building; extend golden canon to include preflight canon array for complex scenario
   - Docs: SPEC_UPDATE (if schema nuance changes); PLAN impl note

   Acceptance Criteria
   - Preflight emits per-action rows with all fields above, stable ordering.
   - `policy_ok` reflects gating; when false, story 6 enforces fail-closed behavior in `apply()`.
   - Golden includes at least one scenario with `policy_ok=false` and corresponding summary.
   Owner: Dev-C

2) Error Model → Exit Codes (SPEC §6)
   - Tier Target: Silver, per `DOCS/EXIT_CODES_TIERS.md` and ADR-0014 (deferral).
   - Map a curated subset of failure sites to stable `ErrorId` + `exit_code` (e.g., `E_LOCKING`, `E_POLICY`, `E_BACKUP_MISSING`, `E_RESTORE_FAILED`, `E_SMOKE`). Keep others provisional.
   - Ensure facts consistently emit `error_id`/`exit_code` on failures; update `api/errors.rs`
   - Tests: targeted unit tests per failure site; golden canon fragments assert error fields present
   - Docs: SPEC_UPDATE if codes or mappings change; ADR if taxonomy decision shifts

   Acceptance Criteria
   - `exit_code_for()` covers the Silver subset; code comments mark coverage and deferrals.
   - Facts for covered sites include both `error_id` and `exit_code` with redaction-compatible canon.
   - Non-blocking coverage report artifact produced in CI.
   Owner: Dev-B

3) Minimal Smoke Runner (SPEC §11)
   - Implement `SmokeTestRunner` default adapter for minimal suite (ls, cp, mv, rm, ln, stat, readlink, sha256sum, sort, date)
   - Wire into `apply()` Commit path; auto-rollback on failure unless disabled by policy
   - Tests: unit tests for invocation paths; golden asserts for rollback on smoke failure (deterministic subset)
   - Docs: PLAN impl note; SPEC clarifications if command args refined

   Acceptance Criteria
   - Default smoke runner available behind adapter; `apply()` invokes it in Commit mode.
   - On failure, auto-rollback occurs (unless disabled) and facts contain `E_SMOKE` with exit code.
   Owner: Dev-E

4) Acceptance (Deterministic subset)
   - Add one non-containerized acceptance scenario runnable in CI (no privileged ops): e.g., three-action plan with intentional mid-plan failure to assert rollback facts
   - Extend golden scenarios (`--golden all`) to include it; ensure zero-SKIP enforcement in CI job
   - Prepare `test-orch/` integration notes for EXDEV matrix (execution remains outside this sprint’s CI)

   Acceptance Criteria
   - CI runs the deterministic scenario; zero-SKIP enforced; goldens stable across runs.
   - Rollback facts captured (including partial restoration notes where applicable).
   Owner: Dev-A

5) Traceability (SPEC/tools)
   - Introduce a lightweight traceability generator (`SPEC/tools/traceability.py`) to map REQ-* to tests and fixtures
   - CI: generate a machine-readable traceability report artifact; non-blocking initially
   - Docs: update `SPEC/traceability.md`

   Acceptance Criteria
   - Traceability script emits REQ→tests/fixtures mapping; artifact archived in CI.
   - sprint-0002 stories reference REQs; report reflects coverage increases.
   Owner: Dev-D

6) Policy & Preservation
   - Wire preservation gating: STOP when required-by-policy but unsupported (fail-closed)
   - Extend provenance completeness (origin/helper/env_sanitized) in facts (still redacted for canon as policy dictates)
   - Tests: unit tests and golden fragments assert presence (with masking where appropriate)

   Acceptance Criteria
   - `apply()` refuses to proceed when preflight `policy_ok=false` unless an explicit override is set in `Policy`.
   - Provenance fields are present (and masked in redacted canon) where applicable.
   Owner: Dev-C

## Deliverables

- Preflight diff rows emitted and covered by tests
- Error→exit code mapping implemented and verified by tests
- Minimal smoke runner wired, with auto-rollback behavior tested
- New acceptance-style golden scenario added; CI runs `--golden all` (already) with zero-SKIP policy enforced
- Initial traceability report generated in CI (artifact)
- PLAN/SPEC/ADR updates for any normative changes

## Definition of Done (DoD)

- `cargo test -p switchyard` green; new tests for each story.
- Redacted DryRun vs Commit canon identical for covered scenarios.
- CI green: golden diff gate remains blocking and stable; traceability artifact produced.
- Doc-sync complete: SPEC_UPDATEs filed, ADRs added when taxonomy/policy decisions change, PLAN notes updated.

## Acceptance Criteria

- `cargo test -p switchyard` is green with new tests
- Golden suite expanded; CI gate stays blocking and green
- Preflight rows match YAML schema; DryRun vs Commit identity holds after redaction
- Error facts include `error_id` and `exit_code` consistently on failures
- Smoke tests run in Commit mode; failures trigger auto-rollback (unless disabled) and are reflected in facts
- A traceability artifact is generated in CI and lists coverage for updated REQs

## Risks / Mitigations

- Environment variance causes flakiness → keep acceptance deterministic; redact/zero volatile fields; avoid timing asserts.
- Exit code churn → adhere to Silver tier; defer broad mapping per ADR-0014; document deferrals in code comments.
- Over-scoping EXDEV matrix → document in `test-orch/` notes; keep out of CI; run locally when needed.

## Tier Target & Scope (Exit Codes)

- Target: Silver tier per `cargo/switchyard/DOCS/EXIT_CODES_TIERS.md`.
- Scope: cover `E_LOCKING`, `E_POLICY`, `E_BACKUP_MISSING`, `E_RESTORE_FAILED`, `E_SMOKE`. Others remain provisional (Bronze) this sprint.

## Week-by-Week Milestones

- Week 1: Preflight diff rows + fail-closed gating draft; Error Silver mapping in `api/errors.rs`; start smoke runner adapter; add deterministic acceptance scenario; initial traceability script.
- Week 2: Complete tests/goldens; wire smoke auto-rollback facts; finalize Silver coverage; produce CI traceability artifact; doc-sync (SPEC_UPDATE/ADR/PLAN) and stabilize goldens.

## Traceability

- REQ coverage focus: REQ-O3, REQ-O6, REQ-C2, REQ-L3, REQ-H2/H3 (partial), REQ-TOCTOU1 (unchanged), REQ-VERS1 (unchanged).
- Code touchpoints: `src/api/{preflight,apply,audit,errors}.rs`, `src/logging/redact.rs`, `src/adapters/smoke.rs`.

## References

- `cargo/switchyard/DOCS/EXIT_CODES_TIERS.md`
- `PLAN/30-errors-and-exit-codes.md`, `PLAN/45-preflight.md`, `PLAN/40-facts-logging.md`
- `PLAN/adr/ADR-0014-exit-codes-deferral.md`

## Risks / Mitigations

- Flakiness due to environment variance → keep acceptance deterministic; redact/zero volatile fields; avoid timing-dependent asserts
- Scope creep on acceptance matrix → keep EXDEV matrix as documented next-step; not in-sprint CI
- Traceability false negatives → start non-blocking; iterate

## Doc-Sync Plan

- Open SPEC_UPDATE for any normative changes to preflight rows or error codes
- Add ADR for material taxonomy decisions
- Update PLAN impl notes for preflight, errors, smoke runner, and CI gates

## References

- `SPEC/SPEC.md` §§2,4,5,6,9,11,12
- `SPEC/audit_event.schema.json`, `SPEC/preflight.yaml`, `SPEC/traceability.md`
- `src/api/{plan,preflight,apply,audit,errors}.rs`, `src/logging/redact.rs`, `src/adapters/*`
- `tests/sprint_acceptance-0001.rs` golden scenarios; `test_ci_runner.py`

---

## Sprint 02 — Execution Update (2025-09-11)

### Completed

- Preflight Diff Rows (SPEC §4)
  - Implemented per-action preflight rows with full fields and stable ordering.
  - Fields: `action_id`, `path`, `current_kind`, `planned_kind`, `policy_ok`, `provenance{uid,gid,pkg}`, `notes`, `preservation`, `preservation_supported`.
  - Code: `src/api/preflight.rs`, `src/types/report.rs` (added `PreflightReport.rows`).
  - Tests: `tests/sprint_acceptance-0001.rs::preflight_rows_schema_and_ordering`.

- Error Model → Exit Codes (Silver)
  - Mapped covered failure sites to `ErrorId`→`exit_code` and emitted in facts (`apply.result`).
  - Covered: `E_POLICY`, `E_ATOMIC_SWAP`, `E_EXDEV`, `E_BACKUP_MISSING`, `E_RESTORE_FAILED`, `E_SMOKE` (+ fallback `E_GENERIC`).
  - Code: `src/api/apply.rs` (per-action + summary); taxonomy lives in `src/api/errors.rs`.
  - Tests: assertions added in acceptance tests to verify `error_id/exit_code` emission.

- Minimal Smoke Runner
  - Added default adapter `DefaultSmokeRunner` (`src/adapters/smoke.rs`).
  - `apply()` invokes smoke in Commit mode when adapter is provided; auto-rollback on failure unless disabled by policy.
  - Test: `tests/smoke_rollback.rs` verifies rollback and `E_SMOKE` with exit code.

- Policy & Preservation Gating
  - Fail-closed gating in `apply()` unless `Policy.override_preflight=true`.
  - Added `Policy.require_preservation`; preflight marks STOP when unsupported.
  - Code: `src/policy/config.rs`, `src/api/preflight.rs`, `src/api/apply.rs`.
  - Test: `apply_fail_closed_on_policy_violation` in `tests/sprint_acceptance-0001.rs`.

- Deterministic Acceptance Scenarios
  - Extended acceptance tests; ensured DryRun vs Commit identity post-redaction.
  - Goldens remain stable; optional canon writers preserved.

### Remaining (Sprint 02)

- Traceability (non-blocking CI artifact)
  - Tool exists at `SPEC/tools/traceability.py`; CI wiring to generate/upload artifact is pending.
  - Deliverable: attach machine-readable REQ→tests/fixtures mapping to CI artifacts.

- Doc-Sync
  - SPEC_UPDATE: note `override_preflight` and `require_preservation` policy flags and preflight row emission status.
  - PLAN/ADR updates: brief notes on Silver coverage scope and policy gating decisions.

### Drift Assessment

- SPEC vs Code
  - Preflight rows now match `SPEC/preflight.yaml` (fields present, ordering deterministic). No drift.
  - Error Silver subset matches `DOCS/EXIT_CODES_TIERS.md`; additional IDs remain Bronze/deferred as planned. No drift.
  - Policy surface expanded (`override_preflight`, `require_preservation`) — requires SPEC_UPDATE (minor). Drift: Pending doc-sync only.

- PLAN vs Code
  - Stories 1–4 and 6 are implemented and tested.
  - Story 5 (Traceability CI artifact) is partially done (tool present, CI wiring pending). Drift: Execution-only; plan remains accurate.

### DoD Status

- Code/tests: `cargo test -p switchyard` is green with new tests.
- Redaction parity holds for DryRun vs Commit scenarios.
- CI: traceability artifact not yet wired (non-blocking). Doc-sync updates pending.

---
