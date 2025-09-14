# Backup and sidecar — Module

**Owner:** <owner>
**Last Reviewed:** 2025-09-14 · **Next Review:** 2025-10-14

> Scope: Risk posture, policy, wiring, and proofs for snapshots and sidecars.

---

## 1) Contract (non-negotiables)

- Capture backup before mutating targets; record prior topology (file/symlink/none).
- Sidecar records provenance and integrity (payload hash when available).
- Best-effort durability: sync backup file and parent directory.

Policy Impact:

- `durability.backup_durability`, `durability.sidecar_integrity`, `backup.tag`.

---

## 2) Wiring (code-referential only)

- Locations (impl):
  - `cargo/switchyard/src/fs/backup/snapshot.rs:36-151` (`create_snapshot`)
  - `cargo/switchyard/src/fs/backup/snapshot.rs:11-26` (`backup_path_with_tag`)
  - `cargo/switchyard/src/fs/backup/sidecar.rs:22-33` (`write_sidecar`)
- Callers:
  - `cargo/switchyard/src/fs/swap.rs:70-75,98-107,126-156` (snapshot prior to swap)
  - `cargo/switchyard/src/api/apply/executors/restore.rs:57-62` (pre-restore snapshot when enabled)
- Callees:
  - `fs::atomic::open_dir_nofollow`, `fs::atomic::fsync_parent_dir`, `fs::meta::sha256_hex_of`
- External Surface:
  - Internal API via `create_snapshot(target, tag)` and helpers.

---

## 3) Risk & Impact (scored)

- Impact: 4 — Safety net for mutations; correctness critical for rollback.
- Likelihood: 3 — Common FS operations; integrity optional unless enforced.
- Risk: 12 → Tier: Silver (current)

---

## 4) Maturity / Robustness

- [x] Invariants documented
- [x] Unit tests (snapshot variants)
- [ ] Negative tests (hash mismatch handling)
- [x] E2E (swap→restore)
- [ ] Platform/FS matrix
- [x] Telemetry (backup_durable flag via apply extras)
- [x] Determinism (facts redacted in dry-run)

**Current Tier:** Silver
**Target Tier:** Gold by next minor

---

## 5) Failure Modes

- Missing payload when `sidecar_integrity=true` → `E_BACKUP_MISSING` on restore.
- Sidecar write fails → mapped to policy error by caller (`sidecar write failed`).

---

## 6) Evidence

- Tests: `cargo/switchyard/src/fs/backup/snapshot.rs:174-210`
- Apply extras carry `backup_durable`.

---

## 7) Gaps → Actions

| ID | Action | Evidence target | Expected effect | Owner | Due |
|----|--------|-----------------|-----------------|-------|-----|
| G-1 | Golden sidecar schema v1/v2; hash parity | CI goldens | L: 3→2 | <owner> | 2025-10-14 |

---

## 8) Notes for IDE-AI

- Verify fsync of backup and parent dir; confirm payload_hash presence when v2.
- Confirm snapshot for file/symlink/none cases.
