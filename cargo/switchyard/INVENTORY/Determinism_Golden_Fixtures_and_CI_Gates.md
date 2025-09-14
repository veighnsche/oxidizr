# Golden fixtures and CI gates — Concept

**Owner:** <owner>
**Last Reviewed:** 2025-09-14 · **Next Review:** 2025-10-14

---

## 1) Contract (non-negotiables)

- Curate deterministic golden artifacts for key scenarios and gate CI against them; upload diffs on failure.

Policy Impact:

- None directly; depends on determinism and stage emissions.

---

## 2) Wiring (code-referential only)

- Determinism helpers: `cargo/switchyard/src/logging/redact.rs`, `cargo/switchyard/src/types/ids.rs`
- YAML exporter for preflight scenarios: `cargo/switchyard/src/preflight/yaml.rs`

---

## 3) Risk & Impact (scored)

- Impact: 2 — Prevents drift; improves regression detection.
- Likelihood: 3 — Requires discipline and CI scripting.
- Risk: 6 → Tier: Bronze

---

## 4) Maturity / Robustness

- [ ] CI job comparing curated set
- [ ] Diff artifact uploader

**Current Tier:** Bronze
**Target Tier:** Silver

---

## 5) Failure Modes

- Non-deterministic fields leak into goldens → flaky gates; mitigate via redaction and strict emission paths.

---

## 6) Evidence

- Determinism helpers and exporter referenced above.

---

## 7) Gaps → Actions

| ID | Action | Evidence target | Expected effect | Owner | Due |
|----|--------|-----------------|-----------------|-------|-----|
| G-1 | Add CI golden job and upload diffs | CI | L: 3→2 | <owner> | 2025-10-14 |

---

## 8) Notes for IDE-AI

- Ensure DryRun redaction and UUIDv5 usage; keep curated set small but representative.
