# Attestation — API

**Owner:** <owner>
**Last Reviewed:** 2025-09-14 · **Next Review:** 2025-10-14

---

## 1) Contract (non-negotiables)

- On successful Commit, optionally emit an attestation bundle with signature.
- Attestation is non-fatal to apply: failure does not abort the stage.
- Fields: `sig_alg`, `signature`, `bundle_hash`, `public_key_id`.

Policy Impact:

- None; enable by providing an `Attestor` via builder.

---

## 2) Wiring (code-referential only)

- Trait: `cargo/switchyard/src/adapters/attest.rs:24-35` (`Attestor`)
- Build fields: `cargo/switchyard/src/adapters/attest.rs:37-53` (`build_attestation_fields`)
- Apply summary integration: `cargo/switchyard/src/api/apply/mod.rs:178-183` (attach attestation on success)

---

## 3) Risk & Impact (scored)

- Impact: 2 — Evidence/audit enrichment post-apply.
- Likelihood: 2 — Deterministic bundle and signing call.
- Risk: 4 → Tier: Bronze

---

## 4) Maturity / Robustness

- [x] Interface and summary wiring
- [ ] Sample attestor and golden facts

**Current Tier:** Bronze
**Target Tier:** Silver

---

## 5) Failure Modes

- Signing failure → attestation omitted; apply still succeeds.

---

## 6) Evidence

- Lines cited above; tests can assert presence of fields under success path.

---

## 7) Gaps → Actions

| ID | Action | Evidence target | Expected effect | Owner | Due |
|----|--------|-----------------|-----------------|-------|-----|
| G-1 | Provide sample ed25519 attestor with test keys; add golden | tests + CI | L: 2→1 | <owner> | 2025-10-14 |

---

## 8) Notes for IDE-AI

- Ensure attestation only in Commit on success; mask volatile pieces in redaction when necessary.
