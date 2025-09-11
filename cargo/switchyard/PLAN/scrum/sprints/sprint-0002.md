# Switchyard Sprint 02 Plan (Draft)

Duration: 2 weeks
Team: 5 AI engineers (parallel lanes)
Drift Guard: Doc-sync (SPEC_UPDATE/ADR/PLAN) remains hard gate; no scope creep past drift threshold

Sprint Theme: Preflight Diff + Error Mapping + Smoke + Acceptance + Traceability

## Objectives

- Close remaining SPEC v1.1 gaps in preflight and error taxonomy; add minimal smoke runner; expand golden suite; wire traceability.
- Exercise acceptance-level behaviors deterministically (non-containerized subset), keep CI Gold gate stable.

## Stories & Tasks

1) Preflight Diff Rows (SPEC §4)
   - Implement per-action preflight diff rows per `SPEC/preflight.yaml` with stable ordering
   - Include keys: `action_id`, `path`, `current_kind`, `planned_kind`, `policy_ok`, `provenance{uid,gid,pkg}`, `notes`, `preservation{}` and `preservation_supported`
   - Ensure DryRun vs Commit byte identity post-redaction
   - Tests: unit tests for row building; extend golden canon to include preflight canon array for complex scenario
   - Docs: SPEC_UPDATE (if schema nuance changes); PLAN impl note

2) Error Model → Exit Codes (SPEC §6)
   - Map failure sites to stable `ErrorId` + `exit_code` (e.g., `E_POLICY`, `E_OWNERSHIP`, `E_EXDEV`, `E_BACKUP_MISSING`, `E_RESTORE_FAILED`, `E_SMOKE`)
   - Ensure facts consistently emit `error_id`/`exit_code` on failures; update `api/errors.rs`
   - Tests: targeted unit tests per failure site; golden canon fragments assert error fields present
   - Docs: SPEC_UPDATE if codes or mappings change; ADR if taxonomy decision shifts

3) Minimal Smoke Runner (SPEC §11)
   - Implement `SmokeTestRunner` default adapter for minimal suite (ls, cp, mv, rm, ln, stat, readlink, sha256sum, sort, date)
   - Wire into `apply()` Commit path; auto-rollback on failure unless disabled by policy
   - Tests: unit tests for invocation paths; golden asserts for rollback on smoke failure (deterministic subset)
   - Docs: PLAN impl note; SPEC clarifications if command args refined

4) Acceptance (Deterministic subset)
   - Add one non-containerized acceptance scenario runnable in CI (no privileged ops): e.g., three-action plan with intentional mid-plan failure to assert rollback facts
   - Extend golden scenarios (`--golden all`) to include it; ensure zero-SKIP enforcement in CI job
   - Prepare `test-orch/` integration notes for EXDEV matrix (execution remains outside this sprint’s CI)

5) Traceability (SPEC/tools)
   - Introduce a lightweight traceability generator (`SPEC/tools/traceability.py`) to map REQ-* to tests and fixtures
   - CI: generate a machine-readable traceability report artifact; non-blocking initially
   - Docs: update `SPEC/traceability.md`

6) Policy & Preservation
   - Wire preservation gating: STOP when required-by-policy but unsupported
   - Extend provenance completeness (origin/helper/env_sanitized) in facts (still redacted for canon as policy dictates)
   - Tests: unit tests and golden fragments assert presence (with masking where appropriate)

## Deliverables

- Preflight diff rows emitted and covered by tests
- Error→exit code mapping implemented and verified by tests
- Minimal smoke runner wired, with auto-rollback behavior tested
- New acceptance-style golden scenario added; CI runs `--golden all` (already) with zero-SKIP policy enforced
- Initial traceability report generated in CI (artifact)
- PLAN/SPEC/ADR updates for any normative changes

## Acceptance Criteria

- `cargo test -p switchyard` is green with new tests
- Golden suite expanded; CI gate stays blocking and green
- Preflight rows match YAML schema; DryRun vs Commit identity holds after redaction
- Error facts include `error_id` and `exit_code` consistently on failures
- Smoke tests run in Commit mode; failures trigger auto-rollback (unless disabled) and are reflected in facts
- A traceability artifact is generated in CI and lists coverage for updated REQs

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
