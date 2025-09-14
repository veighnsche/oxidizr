# Mount checks (rw+exec) — Module

**Owner:** <owner>
**Last Reviewed:** 2025-09-14 · **Next Review:** 2025-10-14

---

## 1) Contract (non-negotiables)

- Do not mutate targets on mounts lacking `rw` or with `noexec` set.
- Fail closed on ambiguity (unable to determine flags).
- Allow operators to add extra roots to verify via policy.

Policy Impact (Debian-style):

- `apply.extra_mount_checks` (additional roots), fail-closed gating.

---

## 2) Wiring (code-referential only — Arch transparency)

- Locations (impl):
  - `cargo/switchyard/src/fs/mount.rs:84-94` (`ensure_rw_exec`)
  - `cargo/switchyard/src/fs/mount.rs:20-31,32-69,72-77` (flag acquisition)
- Callers (consumers):
  - `cargo/switchyard/src/preflight/checks.rs:12-21` (`ensure_mount_rw_exec` wrapper)
  - `cargo/switchyard/src/policy/gating.rs:31-48,137-154` (checks for source/target and extra roots)
- External Surface:
  - Internal helper; surfaced via preflight/gating and policy.

---

## 3) Risk & Impact (scored)

- Impact (1–5): 3 — Prevents unsafe mutations on RO/noexec mounts.
- Likelihood (1–5): 3 — Parser/env variability; statvfs fallback mitigates.
- Risk = I×L: 9 → **Tier:** Silver

---

## 4) Maturity / Robustness (OpenBSD strictness + Fedora roadmap)

- [x] Invariants documented (→ code lines)
- [x] Unit tests for helper behavior (mock inspector)
- [ ] Goldens for common mount setups and /proc parsing

**Current Tier:** Silver  
**Target Tier & ETA:** Gold by next minor

---

## 5) Failure Modes (observable)

- Ambiguous flags or RO/noexec detected → STOP via `E_POLICY` during Commit.

---

## 6) Evidence (links only — Arch style)

- `fs/mount.rs:96-145` (tests)  
- Gating: `policy/gating.rs:31-48,137-154`

---

## 7) Gaps → Actions (mandatory — OpenBSD strictness)

| ID  | Action | Evidence target | Expected effect (I/L ↓) | Owner  | Due        |
|-----|--------|-----------------|--------------------------|--------|-----------|
| G-1 | Add goldens for mount flag parsing | CI goldens | L: 3→2 | <owner> | 2025-10-14 |

---

## 8) Notes for IDE-AI (verification rules)

- Prefer `statvfs` flags, fall back to `/proc/self/mounts`. Confirm fail-closed logic.
