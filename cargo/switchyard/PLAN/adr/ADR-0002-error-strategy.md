# ADR Template

- Title: Error strategy and taxonomy mapping
- Status: Accepted
- Date: 2025-09-10

## Context

SPEC defines a stable error taxonomy and exit codes in `/SPEC/error_codes.toml`. The library should expose a coherent error type while emitting stable identifiers and exit codes in facts for CI/policy stability.

## Decision

- Introduce a library error enum with categories that map 1:1 to SPEC identifiers:
  - `E_POLICY` → `policy_violation`
  - `E_OWNERSHIP` → `ownership_error`
  - `E_LOCKING` → `lock_timeout`
  - `E_ATOMIC_SWAP` → `atomic_swap_failed`
  - `E_EXDEV` → `exdev_fallback_failed`
  - `E_BACKUP_MISSING` → `backup_missing`
  - `E_RESTORE_FAILED` → `restore_failed`
  - `E_SMOKE` → `smoke_test_failed`
- Facts include `exit_code` and stable identifiers; human messages remain non-normative.

## Consequences

+ Stable CI and policy handling across versions.
+ Clear separation between internal error details and external identifiers.
- Requires diligence to keep mapping synchronized with SPEC.

## Links

- `cargo/switchyard/SPEC/error_codes.toml`
- `cargo/switchyard/PLAN/10-architecture-outline.md`
