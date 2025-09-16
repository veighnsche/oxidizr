# Apply

- Atomic symlink swap with backup/restore.
- Facts: per-action attempt/result and final summary.
- Degraded EXDEV support when policy allows, records `degraded=true`.

Backup & sidecar
- Before mutating a target, a snapshot (payload + sidecar) is captured when applicable.
- Sidecar may include a `payload_hash` (v2). When `sidecar_integrity=true`, restore verifies the hash.
- Best-effort durability: fsync the backup file(s) and the parent directory after creation/rename.

Degraded EXDEV
- Cross-filesystem operations cannot use a single atomic rename for symlinks.
- When `apply.exdev=DegradedFallback`, the engine uses safe copy+sync+rename where applicable or unlink+`symlinkat` for links and emits `degraded=true` with `degraded_reason`.
- When disallowed, apply fails with `exdev_fallback_failed` and no visible change occurs.

Operator notes
- Observe `apply.attempt` for locking telemetry and `apply.result` for before/after hashes and provenance.
- Keep policy conservative in production; prefer fail‑closed on unsupported capabilities.

Citations:
- `cargo/switchyard/src/api/apply/mod.rs`
- `cargo/switchyard/src/fs/{atomic.rs,swap.rs}`
- Inventory: `INVENTORY/10_FS_Backup_and_Sidecar.md`, `INVENTORY/80_FS_Backup_Retention_Prune.md`
- SPEC: §2.1 (Atomicity), §2.10 (Filesystems & Degraded Mode), §5 (Audit Facts)
