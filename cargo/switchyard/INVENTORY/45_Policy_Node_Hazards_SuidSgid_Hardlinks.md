# Node hazards: SUID/SGID and hardlinks — Concept

**Owner:** <owner>
**Last Reviewed:** 2025-09-14 · **Next Review:** 2025-10-14

---

## 1) Contract (non-negotiables)

- Detect SUID/SGID bits and multi-link (hardlink) hazards before mutation.
- Fail-closed by default; operators may explicitly downgrade via policy.
- Resolve symlinks for SUID/SGID inspection to evaluate the real target.

Policy Impact:

- `risks.allow_suid_sgid_mutation`, `risks.allow_hardlink_breakage`.

---

## 2) Wiring (code-referential only)

- Checks: `cargo/switchyard/src/preflight/checks.rs:50-71` (`check_suid_sgid_risk`), `:30-39` (`check_hardlink_hazard`)
- Enforcement: `cargo/switchyard/src/policy/gating.rs:57-81` (suid/sgid), `:57-69` (hardlinks)

---

## 3) Risk & Impact (scored)

- Impact: 3 — Elevated privilege risks and mutation ambiguity.
- Likelihood: 3 — Environment-dependent; symlinks and metadata vary.
- Risk: 9 → Tier: Silver

---

## 4) Maturity / Robustness

- [x] Checks implemented with best-effort semantics
- [ ] Explicit tests covering STOP vs allow policy paths

**Current Tier:** Silver
**Target Tier:** Gold

---

## 5) Failure Modes

- Hazard detected with policy set to STOP → `E_POLICY` on Commit.

---

## 6) Evidence

- Lines cited above; preflight rows record notes/STOPs.

---

## 7) Gaps → Actions

| ID | Action | Evidence target | Expected effect | Owner | Due |
|----|--------|-----------------|-----------------|-------|-----|
| G-1 | Tests for SUID/SGID and hardlink STOP vs allow | tests + CI | L: 3→2 | <owner> | 2025-10-14 |

---

## 8) Notes for IDE-AI

- Ensure symlink resolution in SUID/SGID; hardlink hazard only for regular files (nlink > 1).
