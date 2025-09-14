# Determinism and redaction — Concept

**Owner:** <owner>
**Last Reviewed:** 2025-09-14 · **Next Review:** 2025-10-14

---

## 1) Contract (non-negotiables)

- Stable IDs (UUIDv5) and zero timestamps in DryRun.
- Redact volatile fields to enable golden testing.
- Deterministic preflight ordering.

Policy Impact:

- None directly; depends on `ApplyMode`.

---

## 2) Wiring (code-referential only)

- IDs: `cargo/switchyard/src/types/ids.rs:32-50`
- Redaction: `cargo/switchyard/src/logging/redact.rs:6,19-24,30-65`

---

## 3) Risk & Impact (scored)

- Impact: 3 — Golden stability and reproducibility.
- Likelihood: 2 — Helpers are centralized.
- Risk: 6 → Tier: Silver

---

## 4) Maturity / Robustness

- [x] Unit tests for redaction
- [ ] Property tests for UUID stability

**Current Tier:** Silver
**Target Tier:** Gold

---

## 5) Failure Modes

- Missing redaction in call sites → flaky goldens; addressed by StageLogger centralization.

---

## 6) Evidence

- `logging/redact.rs:73-113` (tests)

---

## 7) Gaps → Actions

| ID | Action | Evidence target | Expected effect | Owner | Due |
|----|--------|-----------------|-----------------|-------|-----|
| G-1 | Add property tests for UUIDv5 stability | tests | L: 2→1 | <owner> | 2025-10-14 |

---

## 8) Notes for IDE-AI

- Confirm `TS_ZERO` usage in DryRun and StageLogger usage across stages.
