# Troubleshooting

- Lock timeout: Provide a `LockManager` or increase `lock_timeout_ms`.
- EXDEV disallowed: Set policy `exdev=DegradedFallback` or avoid cross‑FS paths.
- Smoke failures: Provide a `SmokeTestRunner` or disable auto‑rollback.
- Partial restoration: Inspect sidecar and payload; use manual rescue steps if needed.

Citations:
- `cargo/switchyard/src/api/apply/mod.rs`
- `cargo/switchyard/src/adapters/lock/file.rs`
- `cargo/switchyard/src/fs/backup/*`
- `cargo/switchyard/src/fs/restore/*`
