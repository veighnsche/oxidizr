# SMOKE TESTS & AUTO ROLLBACK

## 1) Scope (What this project changes)

- Explicit boundaries and non-goals.
  - Introduce minimal smoke tests for key applets; on failure, revert changes.
  - Uses pointer flip rollback when profiles are present, else per-target restore.
- Source files/modules likely touched (paths + key symbols).
  - `src/experiments/util.rs::create_symlinks` (hook post-commit verifier)
  - `src/experiments/sudors.rs::verify_post_enable` (pattern reference)
  - `src/symlink/ops.rs::{restore_file}` (fallback path)
- User-visible behavior changes.
  - Post-enable tests run automatically and revert on failure.

## 2) Rationale & Safety Objectives

- Why.
  - Fail fast and recover if replacement applets are not healthy.
- Safety invariants.
  - Post-change verification is mandatory; rollback is guaranteed on failure.

## 3) Architecture & Design

- High-level approach.
  - After commit, run small `--version` checks for a representative set (e.g., `ls`, `cp`, `find`, `xargs`). If any exits non-zero, rollback.
- Data model.
  - None.
- Control flow.
  - Commit → Run tests → If pass: persist → If fail: rollback, surface error.
- Public interfaces.
  - CLI: `--no-smoke-tests` to skip (discouraged); `--smoke-timeout`.

## 4) Failure Modes & Guarantees

- A failing or missing applet triggers rollback.
- Profiles flip rollback is preferred; otherwise `restore_targets`.

## 5) Preflight & Post-Change Verification

- Preflight: verify applets exist.
- Post: smoke tests act as verification; optionally, extended suite behind a flag.

## 6) Observability & Audit

- Log `smoke_test_started`, per-applet results, and `rollback_triggered` if needed.

## 7) Security & Policy

- Environment sanitization for test processes (`LC_ALL=C`).

## 8) Migration Plan

- Start with small default suite; allow opt-outs for CI experimentation.

## 9) Testing Strategy

- Unit: test runner harness.
- E2E: intentionally break one applet to ensure rollback triggers.

## 10) Acceptance Criteria (Must be true to ship)

- Any smoke test failure reverts state and leaves system healthy.

## 11) Work Breakdown & Review Checklist

- Implement minimal runner → wire into enable flow → add E2E.

## 12) References (Repo evidence only)

- `src/experiments/sudors.rs::verify_post_enable` (reference for revert-on-failure)
- `src/experiments/util.rs::{create_symlinks, restore_targets}`
- `src/symlink/ops.rs::restore_file`
- TODO_LIST_V2.md items: "Automatic smoke tests and rollback triggers", "Profiles layout + single active pointer flip"
