# Error Codes

Source of truth:
- `cargo/switchyard/SPEC/error_codes.toml`
- Mapping helpers: `cargo/switchyard/src/api/errors.rs`

Usage:
- Stable identifiers are emitted in facts (e.g., `E_POLICY`, `E_LOCKING`). Summary events include a `summary_error_ids` chain from specific→general.

Explicit mapping (SPEC §6):
- `E_POLICY` → `policy_violation = 10`
- `E_OWNERSHIP` → `ownership_error = 20`
- `E_LOCKING` → `lock_timeout = 30`
- `E_ATOMIC_SWAP` → `atomic_swap_failed = 40`
- `E_EXDEV` → `exdev_fallback_failed = 50`
- `E_BACKUP_MISSING` → `backup_missing = 60`
- `E_RESTORE_FAILED` → `restore_failed = 70`
- `E_SMOKE` → `smoke_test_failed = 80`
- `SUCCESS` → `success = 0`
- `GENERIC_ERROR` → `generic_error = 1`

Notes
- Preflight summary maps to `E_POLICY` (exit code 10) when STOP conditions are present.
- Apply/rollback summaries may include multiple identifiers in `summary_error_ids` for routing/analytics.
