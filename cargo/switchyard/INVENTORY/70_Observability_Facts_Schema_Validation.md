# Facts schema validation — Concept

**Owner:** <owner>
**Last Reviewed:** 2025-09-14 · **Next Review:** 2025-10-14

---

## 1) Contract (non-negotiables)

- Validate emitted facts against a JSON Schema to prevent drift and enable stable goldens.

Policy Impact:

- None.

---

## 2) Wiring (code-referential only)

- Schema: `cargo/switchyard/SPEC/audit_event.v2.schema.json` (present)
- Emission sites: StageLogger in `logging/audit.rs` (see references)

---

## 3) Risk & Impact (scored)

- Impact: 2 — Prevents analytics breakage.
- Likelihood: 3 — Missing enforcement currently.
- Risk: 6 → Tier: Bronze

---

## 4) Maturity / Robustness

- [ ] Test helper to validate JSONL against schema
- [ ] CI gate

**Current Tier:** Bronze
**Target Tier:** Silver

---

## 5) Failure Modes

- Schema drift → false negatives in analytics or golden instability.

---

## 6) Evidence

- Presence of schema file; StageLogger centralization.

---

## 7) Gaps → Actions

| ID | Action | Evidence target | Expected effect | Owner | Due |
|----|--------|-----------------|-----------------|-------|-----|
| G-1 | Implement schema validation tests and CI job | tests + CI | L: 3→2 | <owner> | 2025-10-14 |

---

## 8) Notes for IDE-AI

- Tie validation harness to StageLogger output; avoid environment-volatile fields.
