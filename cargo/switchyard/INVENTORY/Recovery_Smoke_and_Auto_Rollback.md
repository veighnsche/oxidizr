# Smoke tests and auto-rollback — Module

**Owner:** <owner>
**Last Reviewed:** 2025-09-14 · **Next Review:** 2025-10-14

---

## 1) Contract (non-negotiables)

- Post-apply smoke validation in Commit mode; failure maps to `E_SMOKE` and triggers auto-rollback unless disabled.
- Missing runner in Commit when required is a failure akin to smoke failure (classification `E_SMOKE`).
- Runner must be deterministic and safe to re-run.

Policy Impact:

- `governance.smoke = Require { auto_rollback } | Off`.

---

## 2) Wiring (code-referential only)

- Adapter: `cargo/switchyard/src/adapters/smoke.rs:18-23,31-66`
- Apply integration: `cargo/switchyard/src/api/apply/mod.rs:138-169`

---

## 3) Risk & Impact (scored)

- Impact: 3 — Detects broken state early; rollback reduces MTTR.
- Likelihood: 3 — Quality of runner dictates false positives/negatives.
- Risk: 9 → Tier: Silver

---

## 4) Maturity / Robustness

- [x] Default minimal runner (link target check)
- [ ] Golden for smoke failure → rollback path

**Current Tier:** Silver
**Target Tier:** Gold

---

## 5) Failure Modes

- Runner returns error or is missing when required → `E_SMOKE`; auto-rollback unless disabled.

---

## 6) Evidence

- See lines cited above; apply summary records error classification and `rolled_back`.

---

## 7) Gaps → Actions

| ID | Action | Evidence target | Expected effect | Owner | Due |
|----|--------|-----------------|-----------------|-------|-----|
| G-1 | Golden tests for smoke failure classification and rollback | CI | L: 3→2 | <owner> | 2025-10-14 |

---

## 8) Notes for IDE-AI

- Ensure `Require` policy is respected; verify `auto_rollback` behavior and facts.
