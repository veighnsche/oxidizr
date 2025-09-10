# TESTS & CI PIPELINE

## 1) Scope (What this project changes)

- Explicit boundaries and non-goals.
  - Add unit & E2E coverage for rollback, smoke tests, idempotency, and hook generation; wire CI orchestrators.
  - Non-goal: change container orchestration architecture.
- Source files/modules likely touched (paths + key symbols).
  - `tests/` YAMLs and any new Rust integration tests.
  - `test-orch/` container runner and host orchestrator.
  - `src/system/hook.rs::hook_body`
  - `src/experiments/util.rs::{create_symlinks, restore_targets}`
- User-visible behavior changes.
  - None.

## 2) Rationale & Safety Objectives

- Why.
  - Prove safety properties and prevent regressions.
- Safety invariants.
  - Rollback correctness; smoke tests enforced.

## 3) Architecture & Design

- High-level approach.
  - Expand container matrix; add Rust integration tests with tmp roots; add unit test for `hook_body`.

## 4) Failure Modes & Guarantees

- Flaky containers: cache and isolation per distro (see `test-orch/host-orchestrator`).

## 5) Preflight & Post-Change Verification

- Preflight: ensure required tools present in containers.
- Post: CI gating based on success.

## 6) Observability & Audit

- Test logs archived per run.

## 7) Security & Policy

- No change.

## 8) Migration Plan

- Incrementally enable tests per experiment.

## 9) Testing Strategy

- Idempotency cycles; introduce faults to force rollback; hook body assertion.

## 10) Acceptance Criteria (Must be true to ship)

- New tests pass consistently across supported distros.

## 11) Work Breakdown & Review Checklist

- Add tests → wire CI → stabilize.

## 12) References (Repo evidence only)

- `test-orch/` (host-orchestrator and container-runner)
- `src/system/hook.rs::hook_body`
- `src/experiments/util.rs::{create_symlinks, restore_targets}`
- TODO_LIST_V2.md items: "Unit and e2e tests for rollback and smoke tests", "Idempotency tests for enable/disable/relink", "Unit test for pacman hook body generation"
