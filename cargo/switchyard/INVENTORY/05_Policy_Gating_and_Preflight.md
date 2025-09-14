# Policy gating and preflight — Concept

**Owner:** <owner>
**Last Reviewed:** 2025-09-14 · **Next Review:** 2025-10-14

---

## 1) Contract (non-negotiables)

- Preflight produces per-action rows and a summary; Commit refuses when STOPs exist unless `override_preflight=true`.
- Checks include ownership, mounts, immutability, hardlinks, suid/sgid, source trust, preservation.
- Deterministic ordering and YAML export for operators/CI.

Policy Impact:

- Multiple knobs: `strict_ownership`, `extra_mount_checks`, `allow_*`, `require_rescue`, `allow_roots/forbid_paths`, `require_preservation`.

---

## 2) Wiring (code-referential only)

- Evaluation: `cargo/switchyard/src/policy/gating.rs:20-211,213-239`
- Checks: `cargo/switchyard/src/preflight/checks.rs:12-141`
- YAML: `cargo/switchyard/src/preflight/yaml.rs:1-35`
- Enforcement: `cargo/switchyard/src/api/apply/mod.rs:91-94`

---

## 3) Risk & Impact (scored)

- Impact: 5 — Safety barrier before mutations.
- Likelihood: 3 — Environment variability; mitigated by fail-closed defaults.
- Risk: 15 → Tier: Silver

---

## 4) Maturity / Robustness

- [x] Invariants documented
- [x] Integration tests (module tests)
- [ ] Golden YAML fixtures and schema validation

**Current Tier:** Silver
**Target Tier:** Gold

---

## 5) Failure Modes

- STOPs present under Commit (unless override) → `E_POLICY` with summary co-ids.

---

## 6) Evidence

- Gating helper and apply enforcement per lines above; YAML exporter exists.

---

## 7) Gaps → Actions

| ID | Action | Evidence target | Expected effect | Owner | Due |
|----|--------|-----------------|-----------------|-------|-----|
| G-1 | Golden preflight YAML + JSON Schema validation | CI | L: 3→2 | <owner> | 2025-10-14 |

---

## 8) Notes for IDE-AI

- Confirm STOP vs note semantics; ensure apply checks override flag.
