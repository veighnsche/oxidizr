# UX, CLI VERBOSITY & LOCK WAIT POLICY

## 1) Scope (What this project changes)

- Explicit boundaries and non-goals.
  - Standardize verbosity levels; demote chatty success logs; simplify pacman lock wait UX; encourage dry-run-first.
  - Non-goal: change of underlying package operations.
- Source files/modules likely touched (paths + key symbols).
  - `src/system/worker/packages.rs::{update_packages, install_package, remove_package, wait_for_pacman_lock_clear}`
  - `src/logging/init.rs` (human formatter levels)
  - `src/cli/{parser.rs, handler.rs}` (defaults/flags)
- User-visible behavior changes.
  - Cleaner INFO logs; progress while waiting on lock; dry-run-first stance.

## 2) Rationale & Safety Objectives

- Why.
  - Reduce noise and align logs to user decisions and safety gates.
- Safety invariants.
  - Clear visibility into waiting states; fewer surprises.
- Overkill → Lean replacement summary.
  - Replace elaborate backoff/daemon logic with a small bounded wait + jitter.

## 3) Architecture & Design

- High-level approach.
  - Map "Expected/Received" success lines to DEBUG; INFO only for decisions/actions. `wait_for_pacman_lock_clear` emits concise progress lines at intervals. `--dry-run` as default is considered, or at least an expanded preflight-only default.

## 4) Failure Modes & Guarantees

- Lock not cleared within timeout → `Error::PacmanLockTimeout` remains; user gets periodic messages.

## 5) Preflight & Post-Change Verification

- Preflight: none.
- Post: manual verification in CI for log level expectations.

## 6) Observability & Audit

- Keep audit sink unchanged; this project is human log policy only.

## 7) Security & Policy

- No effect.

## 8) Migration Plan

- Two-step: message policy first, default dry-run later after user guidance.

## 9) Testing Strategy

- Unit: log policy tests (feature-gated).
- E2E: simulate lock contention and assert messages.

## 10) Acceptance Criteria (Must be true to ship)

- Pacman lock waits display simple progress; INFO logs are concise.

## 11) Work Breakdown & Review Checklist

- Adjust log sites → lock wait messages → flag defaults.

## 12) References (Repo evidence only)

- `src/system/worker/packages.rs::{update_packages, install_package, remove_package, wait_for_pacman_lock_clear}`
- `src/logging/init.rs`
- `src/cli/{parser.rs, handler.rs}`
- TODO_LIST_V2.md items: "Pacman lock handling ergonomics", "Dry-run-first posture", "Owner and trust policies"
