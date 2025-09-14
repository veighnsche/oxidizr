# Atomic symlink swap — Module

**Owner:** <owner>
**Last Reviewed:** 2025-09-14 · **Next Review:** 2025-10-14

> Scope: Deep Wiki already documents behavior. This file records risk posture, policy context, wiring, and proof.

---

## 1) Contract (non-negotiables)

- Uses dirfd-based TOCTOU-safe sequence: open parent O_DIRECTORY|O_NOFOLLOW → symlinkat(tmp) → renameat(tmp→final) → fsync(dirfd).
- Cross-FS rename (EXDEV) does not degrade unless policy explicitly allows degraded fallback.
- Parent directory is fsynced best-effort to persist metadata changes.

Policy Impact (Debian-style):

- SPEC §FS Atomicity, §Determinism; Policy: `apply.exdev`, `durability.backup_durability`.

---

## 2) Wiring (code-referential only — Arch transparency)

- Locations (impl):
  - `cargo/switchyard/src/fs/atomic.rs:69-153` (`atomic_symlink_swap`)
  - `cargo/switchyard/src/fs/atomic.rs:31-42` (`open_dir_nofollow`)
  - `cargo/switchyard/src/fs/atomic.rs:49-55` (`fsync_parent_dir`)
- Callers (consumers):
  - `cargo/switchyard/src/fs/swap.rs:18-158` (`replace_file_with_symlink` orchestrates snapshot + swap)
  - `cargo/switchyard/src/api/apply/executors/ensure_symlink.rs:70-83` (executor invoking swap and emitting facts)
- Callees (deps):
  - `rustix::fs::{openat, symlinkat, renameat, unlinkat}`
- External Surface:
  - Internal crate API via `fs::swap::replace_file_with_symlink(source, target, dry_run, allow_degraded, backup_tag)`

---

## 3) Risk & Impact (scored)

- Impact (1–5): 5 — Mutates critical binaries/targets; incorrect behavior can break systems.
- Likelihood (1–5): 4 — FS variance (EXDEV), timing, platform differences; mitigated by policy and tests.
- Risk = I×L: 20 → Tier: Silver (current)

---

## 4) Maturity / Robustness (OpenBSD strictness + Fedora roadmap)

- [x] Invariants documented (→ code lines)
- [x] Unit tests (→ `fs/swap.rs::tests`)
- [x] Boundary / negative tests (basic)
- [ ] Property / fuzz / failure-injection (EXDEV, unlink races)
- [x] E2E tests (round-trip with restore)
- [ ] Platform/FS matrix (tmpfs/ext4/btrfs)
- [x] Telemetry (perf.swap_ms, degraded)
- [x] Rollback / Recovery proof (restore inverse)
- [x] Determinism check (dry-run redaction)
- [ ] Independent review recorded

**Current Tier:** Silver
**Target Tier & ETA (Fedora-style):** Gold by next minor release

---

## 5) Failure Modes (observable)

- EXDEV without degraded allowed → `E_EXDEV` (`cargo/switchyard/src/api/errors.rs:104-117`).
- Syscall failures (unlinkat/symlinkat/renameat/fsync) → `E_ATOMIC_SWAP` classification; executor maps error.
- Temp name conflicts theoretically avoided via process-unique counter (`NEXT_TMP_COUNTER`).

---

## 6) Evidence (links only — Arch style)

- Tests: `cargo/switchyard/src/fs/swap.rs:173-241` (atomic swap + restore round-trip)
- Telemetry: `apply.result` fields (`degraded`, `perf.swap_ms`) via executor `ensure_symlink.rs:142-175`.
- Policy: `cargo/switchyard/src/policy/config.rs:80-132` (presets) and `types` for `ExdevPolicy`.

---

## 7) Gaps → Actions (mandatory — OpenBSD strictness)

| ID  | Action | Evidence target | Expected effect (I/L ↓) | Owner | Due        |
|-----|--------|-----------------|--------------------------|-------|------------|
| G-1 | Golden for EXDEV degraded vs fail | add golden facts + CI | L: 4→3 | <owner> | 2025-10-14 |

---

## 8) Notes for IDE-AI (verification rules)

- Resolve all references to file:line at HEAD.
- Derive callers/callees via static analysis.
- Confirm invariants with guard conditions (dirfd, fsync) and post-effects.
- Verify tests assert link target and restore inverse.
- Locate telemetry emit sites; ensure redaction in dry-run.
- Detect policy knobs that affect degraded swaps.
- If unverifiable → mark as Gap.
