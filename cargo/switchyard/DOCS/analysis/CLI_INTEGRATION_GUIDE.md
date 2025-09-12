# CLI Integration Best Practices

**Status:** Draft  
**Generated:** 2025-09-12  
**Scope:** Guidance for downstream CLIs to compose policies, handle exit-codes and facts, choose tags/retention, and interoperate with package managers.  
**Inputs reviewed:** SPEC §3 Public Interfaces; SPEC §6 Error Codes; PLAN/12-api-module.md; CODE: `src/api.rs`, `src/policy/config.rs`, `src/logging/*`  
**Affected modules:** `api`, `policy`, `logging`

## Summary

- Construct `Switchyard` with sinks and a hardened `Policy` preset; scope operations via `allow_roots` and forbid lists.
- Use `SafePath::from_rooted` for all inputs; never accept raw absolute paths for mutations.
- Map `ErrorId` to process exit codes consistently; surface apply/preflight facts to users.

## Integration Skeleton

- Policy: start from `Policy::production_preset()` or `::coreutils_switch_preset()`; set `allow_roots` to the intended subtree and `backup_tag` appropriately.
- Locking: configure a `FileLockManager` (or custom) and set `with_lock_timeout_ms`.
- Facts: implement a `FactsEmitter` (e.g., file JSONL). Consider also emitting human summaries.
- Attestation: optional `Attestor` adapter to sign success bundles.

## Exit Codes

- Map `ErrorId` to `SPEC/error_codes.toml` via `api::errors::exit_code_for`. On success exit 0; on `E_LOCKING` exit 30, etc.

## Retention

- Expose a `prune` subcommand that calls the proposed `prune_backups(...)` hook (see RETENTION_STRATEGY.md).

## Package Manager Interop

- Lock ordering: acquire PM lock, then Switchyard lock; release in reverse order.
- Dry-run default: show preflight rows and planned actions; require explicit approval for commit (conservatism).

## References

- CODE: `src/api.rs`, `src/policy/config.rs`, `src/logging/facts.rs`

## Round 1 Peer Review (AI 4, 2025-09-12 15:16 CET)

- **Claims Verified:**
  - Policy construction using `production_preset()` and `coreutils_switch_preset()` is supported. Cited `src/policy/config.rs` for policy presets.
  - Locking configuration with `FileLockManager` is implemented. Cited `src/adapters/lock/file.rs` for `FileLockManager` implementation.
  - Exit code mapping using `exit_code_for` is implemented for error handling. Cited `src/api/errors.rs` for exit code mapping.
- **Key Citations:**
  - `src/policy/config.rs`: Policy presets for production and coreutils switching.
  - `src/adapters/lock/file.rs`: Implementation of `FileLockManager` for process locking.
  - `src/api/errors.rs`: Exit code mapping for error handling.
- **Summary of Edits:**
  - Added specific code citations to support recommendations on policy construction, locking, and exit code handling.
  - No major content changes were necessary as the guidance aligns well with the current codebase.

Reviewed and updated in Round 1 by AI 4 on 2025-09-12 15:16 CET
