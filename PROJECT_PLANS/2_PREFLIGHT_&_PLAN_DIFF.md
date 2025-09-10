# PREFLIGHT & PLAN DIFF

## 1) Scope (What this project changes)

- Explicit boundaries and non-goals.
  - Adds deterministic preflight: compute and present the full plan before mutating the system.
  - Non-goal: change of low-level symlink swap semantics (still via `renameat`).
- Source files/modules likely touched (paths + key symbols).
  - `src/experiments/util.rs::{create_symlinks, log_applets_summary}` (plan generator, no-op mode)
  - `src/experiments/*::{enable, list_targets, discover_applets}`
  - `src/cli/{parser.rs, handler.rs}` (flags: `--preflight`, `--assume-yes`, `--dry-run`)
  - `src/system/fs_checks.rs::{ensure_mount_rw_exec, check_source_trust}` (policy checks)
- User-visible behavior changes (if any).
  - New preflight output table: current → planned state, with reasons.

## 2) Rationale & Safety Objectives

- Why this is needed (short).
  - Ensure only intended files change and surface policy failures before commit.
- Safety invariants this project enforces.
  - No mutation until preflight confirms readiness.
  - Fail-closed on policy violations.

## 3) Architecture & Design

- High-level approach (one paragraph).
  - Preflight phase enumerates applets and targets, resolves current state (`symlink_metadata`, `read_link`), runs policy checks, and renders a diff-like table. In `--dry-run` or `--preflight` mode, no writes occur. The plan is included in the audit event payload for the operation.
- Data model & structures (types/fields; JSON examples if relevant).
  - PreflightItem: `{ target, current_kind, current_dest?, planned_kind, planned_dest?, policy_ok }`.
- Control flow: ASCII diagram of phases.
  - Staging → Validation (policy) → Plan Render → Commit (if approved) → Verify → Rollback.
- Public interfaces.
  - CLI: `--preflight` (default true unless `--assume-yes`), `--dry-run` prints plan and exits.

## 4) Failure Modes & Guarantees

- Enumerate failure cases and detection.
  - Missing source, untrusted source, immutable target, ro/noexec mounts → error and abort.
- Rollback strategy.
  - If commit partially applied (legacy path), use `restore_targets`.
- Idempotency rules.
  - Re-running preflight produces identical plan given same inputs.
- Concurrency/locking.
  - Acquire `src/system/lock.rs` only at commit boundary, not for plan generation.

## 5) Preflight & Post-Change Verification

- Preflight checks.
  - `ensure_mount_rw_exec` for sources and `/usr`.
  - `check_source_trust` for applet sources.
- Post-change verification.
  - Re-run preflight against live tree; expect zero planned changes.

## 6) Observability & Audit

- Structured log fields.
  - `event=preflight`, `items=N`, `policy_failures=[…]`.
- Per-operation artifacts.
  - Embed plan rows into `audit-<op_id>.jsonl`.

## 7) Security & Policy

- Ownership and trust checks.
  - Leverage `check_source_trust` path policy.
- Environment sanitization.
  - `LC_ALL=C` for shell-outs in package checks if any.

## 8) Migration Plan

- Introduce preflight in warn-only mode; then enforce block-on-failure with `--force` escape where allowed.

## 9) Testing Strategy

- Unit: render function for plan rows.
- E2E: compare preflight before/after to assert deterministic outcome.

## 10) Acceptance Criteria (Must be true to ship)

- Preflight emits a complete, correct, policy-checked plan.
- `--dry-run` prints plan and never mutates the system.

## 11) Work Breakdown & Review Checklist

- Implement plan row builder, table renderer, CLI flag plumbing.
- Reviewer checks: mutation guarded by preflight outcome.

## 12) References (Repo evidence only)

- `src/experiments/util.rs::{create_symlinks, log_applets_summary}`
- `src/experiments/coreutils.rs::{enable, list_targets, discover_applets}`
- `src/experiments/findutils.rs::{enable, list_targets, discover_applets}`
- `src/system/fs_checks.rs::{ensure_mount_rw_exec, check_source_trust}`
- `src/cli/{parser.rs, handler.rs}`
- TODO_LIST_V2.md items: "Preflight plan and diff", "Preflight for required system tools", "Dry-run-first posture", "Owner and trust policies"
