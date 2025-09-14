# Ownership and provenance — Concept

**Owner:** <owner>
**Last Reviewed:** 2025-09-14 · **Next Review:** 2025-10-14

---

## 1) Contract (non-negotiables)

- Surface uid/gid (and optional pkg) provenance for targets.
- When `strict_ownership=true`, STOP on missing/mismatched ownership information.
- Do not fabricate provenance; emit only when oracle available.

Policy Impact:

- `risks.ownership_strict` (STOP vs advisory).

---

## 2) Wiring (code-referential only)

- Oracle: `cargo/switchyard/src/adapters/ownership/fs.rs:11-34` (`FsOwnershipOracle::owner_of`)
- Apply extras attach provenance (best-effort):
  - EnsureSymlink: `cargo/switchyard/src/api/apply/executors/ensure_symlink.rs:108-118,155-166`
  - Restore: `cargo/switchyard/src/api/apply/executors/restore.rs:143-153,191-201`
- Gating: `cargo/switchyard/src/policy/gating.rs:97-107` (strict ownership STOP when oracle missing or check fails)

---

## 3) Risk & Impact (scored)

- Impact: 3 — Auditability and risk gating.
- Likelihood: 3 — Env/FS variability.
- Risk: 9 → Tier: Silver

---

## 4) Maturity / Robustness

- [x] Oracle implementation (uid/gid)
- [ ] Package provenance (pkg) source
- [ ] Dedicated enforcement tests for strict_ownership

**Current Tier:** Silver
**Target Tier:** Gold

---

## 5) Failure Modes

- Strict ownership enabled without oracle → STOP (`E_POLICY`).

---

## 6) Evidence

- Lines cited above; apply extras show `provenance{uid,gid,pkg}` when available.

---

## 7) Gaps → Actions

| ID | Action | Evidence target | Expected effect | Owner | Due |
|----|--------|-----------------|-----------------|-------|-----|
| G-1 | Provide package DB oracle example | examples + tests | L: 3→2 | <owner> | 2025-10-14 |

---

## 8) Notes for IDE-AI

- Confirm STOP when `ownership_strict` and oracle missing; verify extras include provenance when oracle present.
