# Restore and rollback — Module

**Owner:** <owner>
**Last Reviewed:** 2025-09-14 · **Next Review:** 2025-10-14

---

## 1) Contract (non-negotiables)

- Idempotent restore: short-circuit when current state matches sidecar prior.
- Enforce integrity when configured; best-effort allowed only when policy says so.
- Reverse-order rollback attempted on first apply failure.

Policy Impact:

- `apply.best_effort_restore`, `durability.sidecar_integrity`, `apply.capture_restore_snapshot`.

---

## 2) Wiring (code-referential only)

- Locations (impl):
  - `cargo/switchyard/src/fs/restore/engine.rs:15-27` (`restore_file`)
  - `cargo/switchyard/src/fs/restore/engine.rs:35-47` (`restore_file_prev`)
  - `cargo/switchyard/src/fs/restore/engine.rs:58-71` (`restore_impl`, plan/execute)
- Callers:
  - `cargo/switchyard/src/api/apply/executors/restore.rs:90-104` (prev/latest selection and telemetry)
  - `cargo/switchyard/src/api/apply/mod.rs:127-136,138-169` (rollback orchestration, smoke failure paths)
- Callees:
  - `fs/backup::{find_latest_backup_and_sidecar, read_sidecar}`; `fs/restore/steps::*`

---

## 3) Risk & Impact (scored)

- Impact: 4 — Broken rollback leaves system inconsistent.
- Likelihood: 3 — Common I/O errors; mitigated by policies and idempotence check.
- Risk: 12 → Tier: Silver (current)

---

## 4) Maturity / Robustness

- [x] Invariants documented
- [x] Unit/integration tests
- [ ] Goldens for `E_BACKUP_MISSING` / `E_RESTORE_FAILED`
- [x] Telemetry (rollback.step/summary)

**Current Tier:** Silver
**Target Tier:** Gold by next minor

---

## 5) Failure Modes

- `NotFound` (missing backup) → `E_BACKUP_MISSING`.
- Integrity mismatch with `sidecar_integrity=true` → restore error (mapped to `E_RESTORE_FAILED`).

---

## 6) Evidence

- Executor: `cargo/switchyard/src/api/apply/executors/restore.rs:54-178` (mapping + facts)
- Engine: `cargo/switchyard/src/fs/restore/engine.rs:93-240` (planner and steps)

---

## 7) Gaps → Actions

| ID | Action | Evidence target | Expected effect | Owner | Due |
|----|--------|-----------------|-----------------|-------|-----|
| G-1 | Add goldens for missing payload and integrity fail | CI | L: 3→2 | <owner> | 2025-10-14 |

---

## 8) Notes for IDE-AI

- Verify idempotence branch and integrity check; confirm prev/latest selection.
