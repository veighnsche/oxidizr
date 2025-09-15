# Error Codes

Source of truth:
- `cargo/switchyard/SPEC/error_codes.toml`
- Mapping helpers: `cargo/switchyard/src/api/errors.rs`

Usage:
- Preflight STOPs → `E_POLICY` (exit code from mapping)
- Locking timeout → `E_LOCKING`
- Cross-filesystem disallowed → `E_EXDEV`
- Backup missing → `E_BACKUP_MISSING`
- Restore failure → `E_RESTORE_FAILED`
- Smoke failure → `E_SMOKE`
