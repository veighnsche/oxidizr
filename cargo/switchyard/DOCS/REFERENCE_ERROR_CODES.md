# Reference: Error Codes

Source files:
- `cargo/switchyard/SPEC/error_codes.toml`
- `cargo/switchyard/src/api/errors.rs`

Common codes:
- E_POLICY — Policy violation during preflight or apply
- E_LOCKING — Lock acquisition required/failed
- E_EXDEV — Cross-filesystem operation disallowed
- E_BACKUP_MISSING — No backup artifacts present for restore
- E_RESTORE_FAILED — Restore failure
- E_SMOKE — Smoke runner failure in Commit mode
