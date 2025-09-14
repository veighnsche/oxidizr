# Preflight YAML exporter — API

**Owner:** <owner>
**Last Reviewed:** 2025-09-14 · **Next Review:** 2025-10-14

---

## 1) Contract (non-negotiables)

- Stable field subset and ordering for operator/CI consumption.

Policy Impact:

- Renders outputs of policy-based checks; no direct knobs.

---

## 2) Wiring (code-referential only)

- `cargo/switchyard/src/preflight/yaml.rs:1-35`

---

## 3) Risk & Impact (scored)

- Impact: 1 — UX artifact.
- Likelihood: 2 — Deterministic with upstream sort.
- Risk: 2 → Tier: Bronze

---

## 4) Maturity / Robustness

- [ ] Golden YAML fixtures

**Current Tier:** Bronze
**Target Tier:** Silver

---

## 5) Failure Modes

- Field order drift → golden diffs; mitigated by explicit order list.

---

## 6) Evidence

- YAML exporter code; upstream sort in preflight.

---

## 7) Gaps → Actions

| ID | Action | Evidence target | Expected effect | Owner | Due |
|----|--------|-----------------|-----------------|-------|-----|
| G-1 | Add YAML goldens | tests + CI | L: 2→1 | <owner> | 2025-10-14 |

---

## 8) Notes for IDE-AI

- Confirm field subset and order preserved.
