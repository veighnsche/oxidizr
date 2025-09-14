# Locking and concurrency — Module

**Owner:** <owner>
**Last Reviewed:** 2025-09-14 · **Next Review:** 2025-10-14

---

## 1) Contract (non-negotiables)

- Commit requires process lock by default; bounded wait with clear failure mapping.
- Missing manager in Commit yields `E_LOCKING` unless policy explicitly allows.
- Lock telemetry is emitted in `apply.attempt`.

Policy Impact:

- `governance.locking`, `governance.allow_unlocked_commit`.

---

## 2) Wiring (code-referential only)

- Adapter: `cargo/switchyard/src/adapters/lock/file.rs:36-64` (fs2 advisory lock)
- Apply acquire: `cargo/switchyard/src/api/apply/mod.rs:69-76,83-89`

---

## 3) Risk & Impact (scored)

- Impact: 4 — Prevents conflicting mutations.
- Likelihood: 3 — Contention; bounded and observable.
- Risk: 12 → Tier: Silver

---

## 4) Maturity / Robustness

- [x] Adapter unit tests (timeout/success)
- [ ] Contention golden

**Current Tier:** Silver
**Target Tier:** Gold

---

## 5) Failure Modes

- Timeout acquiring lock → `E_LOCKING` (exit 30) and early abort.

---

## 6) Evidence

- `adapters/lock/file.rs:72-99` (tests)

---

## 7) Gaps → Actions

| ID | Action | Evidence target | Expected effect | Owner | Due |
|----|--------|-----------------|-----------------|-------|-----|
| G-1 | Golden contention test | CI | L: 3→2 | <owner> | 2025-10-14 |

---

## 8) Notes for IDE-AI

- Verify lock telemetry presence and early failure path in apply.
