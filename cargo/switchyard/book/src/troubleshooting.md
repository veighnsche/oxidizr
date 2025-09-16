# Troubleshooting

- __Lock timeout__
  - Symptom: `E_LOCKING`, `exit_code=30`, `apply.attempt.lock_wait_ms` large.
  - Fix: Provide a `LockManager`, reduce contention, or increase `.with_lock_timeout_ms(...)`.

- __EXDEV disallowed__
  - Symptom: `exdev_fallback_failed`, `E_EXDEV`, `degraded=false`, `degraded_reason="exdev_fallback"`.
  - Fix: Either avoid cross‑FS target/staging, or set `apply.exdev=DegradedFallback` and accept degraded behavior.

- __Smoke failures__
  - Symptom: `E_SMOKE` in `summary_error_ids`; auto‑rollback executed.
  - Fix: Provide a `SmokeTestRunner` in production or disable auto‑rollback explicitly (not recommended). Investigate failing tools and PATH.

- __Preflight STOPs__
  - Symptom: `E_POLICY` during preflight summary; rows list STOP reasons (e.g., mount `ro`/`noexec`, ownership mismatch, SUID/SGID or hardlink hazards).
  - Fix: Adjust policy (only if risk‑accepted), or remedy environment (e.g., mount options, ownership, rescue profile).

- __Partial restoration after rollback__
  - Symptom: `rollback.summary` includes `summary_error_ids` such as `E_RESTORE_FAILED`, `E_BACKUP_MISSING`.
  - Fix: Follow the [Recovery Playbook](recovery-playbook.md). Verify backup payload/sidecar integrity; restore topology manually if necessary.

References
- `src/api/apply/mod.rs`
- `src/adapters/lock/file.rs`
- `src/fs/backup/*`
- `src/fs/restore/*`
- SPEC §6 (Error Taxonomy & Exit Codes), §11 (Smoke)

Citations:
- `src/api/apply/mod.rs`
- `src/adapters/lock/file.rs`
- `src/fs/backup/*`
- `src/fs/restore/*`
