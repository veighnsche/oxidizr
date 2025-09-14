# Rescue profile verification — Concept

**Owner:** <owner>
**Last Reviewed:** 2025-09-14 · **Next Review:** 2025-10-14

---

## 1) Contract (non-negotiables)

- Verify BusyBox or GNU subset presence on PATH; enforceable via policy.

Policy Impact:

- `rescue.require`, `rescue.exec_check`, `rescue.min_count`.

---

## 2) Wiring (code-referential only)

- `cargo/switchyard/src/policy/rescue.rs:45-96` (`verify_rescue_min`)
- Gating: `cargo/switchyard/src/policy/gating.rs:222-230`

---

## 3) Risk & Impact (scored)

- Impact: 3 — Ensures recovery tooling exists.
- Likelihood: 3 — Env-dependent; mitigated by policy.
- Risk: 9 → Tier: Silver

---

## 4) Maturity / Robustness

- [x] Unit tests (serial env overrides)
- [ ] Goldens for missing profiles

**Current Tier:** Silver
**Target Tier:** Gold

---

## 5) Failure Modes

- Unavailable rescue when required → STOP (`E_POLICY`).

---

## 6) Evidence

- Unit tests in `policy/rescue.rs:124-147`.

---

## 7) Gaps → Actions

| ID | Action | Evidence target | Expected effect | Owner | Due |
|----|--------|-----------------|-----------------|-------|-----|
| G-1 | Golden for missing rescue and guidance | CI + docs | L: 3→2 | <owner> | 2025-10-14 |

---

## 8) Notes for IDE-AI

- Confirm policy gating integration; ensure overrides are test-scoped.
