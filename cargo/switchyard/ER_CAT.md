# Switchyard Error Catalog (ER_CAT)

Source of truth for exit codes:

- Mapping implementation: `cargo/switchyard/src/api/errors.rs`
- SPEC mapping: `cargo/switchyard/SPEC/error_codes.toml`

The table below confirms each catalog entry’s exit code and shows how it maps to the stable `ErrorId`. Where an entry is a preflight/apply gate instead of a stable `ErrorId`, it maps to `E_POLICY` (exit code 10) on failure, per Silver‑tier rules. Warning‑only entries have no exit code.

| Catalog ID | Maps to ErrorId | Phase | Decision Type | Exit Code | Notes |
|---|---|---|---|---:|---|
| E_LOCKING | E_LOCKING | apply (lock acquisition) | hard_fail | 30 | Timeout acquiring lock or commit without configured `LockManager` when required. Emits `lock_wait_ms`. |
| E_POLICY | E_POLICY | apply (preflight gating or action execution) | hard_fail | 10 | Fail‑closed policy stop (e.g., preflight gates) or backup sidecar write failure. |
| E_EXDEV | E_EXDEV | apply (action execution) | hard_fail | 50 | Cross‑filesystem swap not allowed (policy `allow_degraded_fs=false`). |
| E_ATOMIC_SWAP | E_ATOMIC_SWAP | apply (action execution) | hard_fail | 40 | Atomic replace failed for unexpected IO error (not EXDEV, not sidecar). |
| E_BACKUP_MISSING | E_BACKUP_MISSING | apply (action execution) | hard_fail | 60 | Restore requested but artifacts missing and best‑effort not forced. |
| E_RESTORE_FAILED | E_RESTORE_FAILED | apply (action execution) | hard_fail | 70 | Restore failed with IO error other than NotFound. |
| E_SMOKE | E_SMOKE | apply (post‑action smoke) | rollback_triggered | 80 | Post‑apply smoke runner failed or was required but missing. |
| RESCUE_PROFILE_UNAVAILABLE | E_POLICY | preflight (global rescue check) | hard_fail | 10 | `require_rescue=true` and rescue toolset verification failed. |
| MOUNT_NOT_RW_EXEC | E_POLICY | preflight (mount check) | hard_fail | 10 | `/usr` or target mount not `rw+exec`. |
| IMMUTABLE_TARGET | E_POLICY | preflight (immutable check) | hard_fail | 10 | Target has immutable bit (or equivalent) set. |
| UNTRUSTED_SOURCE | E_POLICY | preflight (ownership check) | hard_fail / warning_only | 10 | Fails closed unless `force_untrusted_source=true`, in which case it downgrades to a warning (no exit code). |
| STRICT_OWNERSHIP_FAILED | E_POLICY | preflight (ownership oracle) | hard_fail | 10 | `strict_ownership=true` and `OwnershipOracle` check failed. Note: `E_OWNERSHIP (20)` reserved for future granularity. |
| OWNERSHIP_ORACLE_MISSING | E_POLICY | preflight (ownership oracle) | hard_fail | 10 | `strict_ownership=true` but no `OwnershipOracle` configured. |
| TARGET_OUTSIDE_ALLOWED_ROOTS | E_POLICY | preflight (path scope) | hard_fail | 10 | Target not under any `allow_roots` prefix. |
| TARGET_IN_FORBIDDEN_PATH | E_POLICY | preflight (path scope) | hard_fail | 10 | Target falls under one of the `forbid_paths` prefixes. |
| PRESERVATION_UNSUPPORTED | E_POLICY | preflight (capability probe) | hard_fail | 10 | `require_preservation=true` but preservation unsupported for target. |
| NO_BACKUP_ARTIFACTS | E_POLICY | preflight (rescue/restore check) | hard_fail | 10 | Restore requested but no backup artifacts present and not in best‑effort mode. |
| UNTRUSTED_SOURCE_ALLOWED | — | preflight (ownership check) | warning_only | — | `force_untrusted_source=true`; informational warning only (no exit code). |
| NO_LOCK_MANAGER_WARN | — | apply (lock acquisition) | warning_only | — | Dry‑run or commit allowed without lock; warning only (no exit code). |
| NO_LOCK_MANAGER_COMMIT | E_LOCKING | apply (lock acquisition) | hard_fail | 30 | Commit attempted without required lock manager; maps to `E_LOCKING`. |

Additional stable `ErrorId`s (not explicitly listed as catalog rows above):

- E_OWNERSHIP → 20 (reserved; not currently emitted directly, ownership‑related stops map to `E_POLICY` at Silver tier)
- E_GENERIC → 1 (fallback for unexpected, uncategorized errors)

Status: Silver‑tier coverage

- Covered IDs in code today: `E_LOCKING`, `E_POLICY`, `E_ATOMIC_SWAP`, `E_EXDEV`, `E_BACKUP_MISSING`, `E_RESTORE_FAILED`, `E_SMOKE`.
- Deferred for later tier: direct `E_OWNERSHIP` emission; broader granular IDs as the failure surface stabilizes.

Traceability references

- Implementation references: `src/api/apply.rs`, `src/api/apply/handlers.rs`, `src/api/preflight.rs`, `src/policy/gating.rs`.
- Mapping: `src/api/errors.rs::exit_code_for()`, `SPEC/error_codes.toml`.
