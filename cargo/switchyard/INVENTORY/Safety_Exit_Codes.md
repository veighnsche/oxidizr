# Exit codes taxonomy and mapping — Concept

**Owner:** <owner>
**Last Reviewed:** 2025-09-14 · **Next Review:** 2025-10-14

---

## 1) Contract (non-negotiables)

- Stable `error_id` → `exit_code` mapping; facts include both on failures.

Policy Impact:

- None; used for routing/analytics.

---

## 2) Wiring (code-referential only)

- `cargo/switchyard/src/api/errors.rs:70-133`
- Mapping usage: apply/preflight failure emit paths.

---

## 3) Risk & Impact (scored)

- Impact: 3 — Operator/scripts rely on codes.
- Likelihood: 2 — Static mapping.
- Risk: 6 → Tier: Silver

---

## 4) Maturity / Robustness

- [ ] Tests covering failure scenarios to assert ids/codes

**Current Tier:** Silver
**Target Tier:** Gold

---

## 5) Failure Modes

- Missing `exit_code` on a path → reduced clarity; add tests on regression.

---

## 6) Evidence

- Mapping functions and usage as cited.

---

## 7) Gaps → Actions

| ID | Action | Evidence target | Expected effect | Owner | Due |
|----|--------|-----------------|-----------------|-------|-----|
| G-1 | Golden for representative failures with codes | CI | L: 2→1 | <owner> | 2025-10-14 |

---

## 8) Notes for IDE-AI

- Ensure summary `infer_summary_error_ids()` includes `E_POLICY` and co-ids.
