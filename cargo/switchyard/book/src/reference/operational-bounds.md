# Operational Bounds

- fsync(parent) must occur ≤50ms after rename (bounded durability window).
- Default plan size limit: 1000 actions (configurable via policy/config).
- Supported/tested filesystems for rename/degraded semantics: ext4, xfs, btrfs, tmpfs.

Notes
- EXDEV degraded behavior is controlled by policy; see Cross‑filesystem (EXDEV) concept and SPEC §10.
- Smoke runs are part of Commit; see SPEC §11 (minimal suite and auto‑rollback).

Citations:
- `cargo/switchyard/SPEC/SPEC.md`
