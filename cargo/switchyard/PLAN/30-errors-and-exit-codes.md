# Errors and Exit Codes (Planning Only)

Maps planning error taxonomy to stable identifiers and exit codes per SPEC.

References:

- SPEC: `cargo/switchyard/SPEC/SPEC.md §6 Error Taxonomy & Exit Codes`
- SPEC TOML: `cargo/switchyard/SPEC/error_codes.toml`
- Requirements: `REQ-L3`, `REQ-R5`, `REQ-C2`

## Error Taxonomy (planning)

```rust
// Planning-only pseudocode; not actual code

enum ErrorKind {
    E_POLICY,           // policy_violation (e.g., SafePath invalid, preflight fail-closed)
    E_OWNERSHIP,        // ownership_error
    E_LOCKING,          // lock_timeout (bounded wait exceeded)
    E_ATOMIC_SWAP,      // atomic_swap_failed
    E_EXDEV,            // exdev_fallback_failed
    E_BACKUP_MISSING,   // backup_missing
    E_RESTORE_FAILED,   // restore_failed
    E_SMOKE,            // smoke_test_failed
}

struct Error { kind: ErrorKind, msg: String }
```

## Exit Code Mapping (stable)

```text
E_POLICY           -> error_codes.policy_violation (10)
E_OWNERSHIP        -> error_codes.ownership_error  (20)
E_LOCKING          -> error_codes.lock_timeout     (30)
E_ATOMIC_SWAP      -> error_codes.atomic_swap_failed (40)
E_EXDEV            -> error_codes.exdev_fallback_failed (50)
E_BACKUP_MISSING   -> error_codes.backup_missing   (60)
E_RESTORE_FAILED   -> error_codes.restore_failed   (70)
E_SMOKE            -> error_codes.smoke_test_failed (80)
```

## Pseudocode: Converting Errors to Facts/Exit Codes

```rust
fn to_exit_code(err: &ErrorKind) -> i32 {
    match err {
        E_POLICY => 10,
        E_OWNERSHIP => 20,
        E_LOCKING => 30,
        E_ATOMIC_SWAP => 40,
        E_EXDEV => 50,
        E_BACKUP_MISSING => 60,
        E_RESTORE_FAILED => 70,
        E_SMOKE => 80,
    }
}

fn record_failure_fact(action_id: Option<Uuid>, path: Option<&SafePath>, kind: ErrorKind, msg: &str) {
    emit_fact({
        stage: "apply.result",
        decision: "failure",
        severity: "error",
        action_id,
        path,
        exit_code: to_exit_code(&kind),
        msg,
    })
}
```

## Policy and Failure Behavior

- Preflight failures that are policy-related MUST fail closed (REQ-C2) and yield `E_POLICY`.
- Lock acquisition timeouts MUST yield `E_LOCKING` and capture `lock_wait_ms` (REQ-L3).
- On rollback failure, facts MUST capture partial restoration state and guidance (REQ-R5). Prefer specific kinds (`E_RESTORE_FAILED`).

## Sprint 02 Tier Target (Silver)

- Covered this sprint (Silver): `E_LOCKING`, `E_POLICY`, `E_BACKUP_MISSING`, `E_RESTORE_FAILED`, `E_SMOKE`.
- Deferred (remain Bronze): `E_ATOMIC_SWAP`, `E_EXDEV`, `E_OWNERSHIP` (beyond policy stops), and any additional granular IDs discovered during development.
- Policy: Follow `DOCS/EXIT_CODES_TIERS.md` and ADR-0014 — do not over-claim coverage; mapping comments must clearly mark Covered vs Deferred.

## Determinism Considerations

- Error handling MUST NOT introduce nondeterministic fields into facts. For dry-run parity, redact timestamps and keep field ordering stable (see `impl/40-facts-logging.md`).
