# SafePath — API

**Owner:** <owner>
**Last Reviewed:** 2025-09-14 · **Next Review:** 2025-10-14

---

## 1) Contract (non-negotiables)

- Reject `..` traversal and unsupported components; require absolute root.
- Preserve stable relative portion for determinism and logging.
- All mutating APIs accept `SafePath` (not raw `Path`).

Policy Impact:

- `scope.allow_roots`, `scope.forbid_paths` limit blast radius.

---

## 2) Wiring (code-referential only)

- Locations (impl): `cargo/switchyard/src/types/safepath.rs:16-83,85-104`
- Callers: plan/apply/restore types and handlers accept `SafePath`; `fs/swap.rs:18-27`.
- External Surface: `SafePath::from_rooted(root, candidate)`; `SafePath::as_path()`, `rel()`

---

## 3) Risk & Impact (scored)

- Impact: 5 — Prevents root escape; fundamental invariant.
- Likelihood: 2 — Deterministic checks; tested.
- Risk: 10 → Tier: Silver

---

## 4) Maturity / Robustness

- [x] Invariants documented
- [x] Unit tests for dotdot/curdir/absolute cases
- [ ] Property tests for normalization/idempotence

**Current Tier:** Silver
**Target Tier:** Gold by next minor

---

## 5) Failure Modes

- Invalid root/candidate or escape attempt → `ErrorKind::Policy` → `E_POLICY`.

---

## 6) Evidence

- `cargo/switchyard/src/types/safepath.rs:112-146` (tests)

---

## 7) Gaps → Actions

| ID | Action | Evidence target | Expected effect | Owner | Due |
|----|--------|-----------------|-----------------|-------|-----|
| G-1 | Property tests for normalization | tests | L: 2→1 | <owner> | 2025-10-14 |

---

## 8) Notes for IDE-AI

- Confirm `ParentDir` rejection and `CurDir` normalization; ensure all mutating paths use `SafePath`.
