# Locking & Concurrency (Planning Only)

Defines the concurrency model, `LockManager` behavior, bounded wait semantics, and WARN behavior in dev/test when no lock manager is present.

References:

- SPEC: `cargo/switchyard/SPEC/SPEC.md §2.5 Locking`, `§14 Thread-safety`
- Requirements: `REQ-L1..L4`

## Guarantees

- Only one `apply()` mutator proceeds at a time. (REQ-L1)
- In production, a `LockManager` is required; omission allowed only in dev/test. (REQ-L4)
- Lock acquisition uses a bounded wait with timeout. On timeout → `E_LOCKING`; record `lock_wait_ms`. (REQ-L3)
- Without a `LockManager`, concurrent `apply()` is UNSUPPORTED; emit a WARN fact. (REQ-L2)
- Core types are `Send + Sync`; `apply()` may be invoked from multiple threads, but only one mutator proceeds under the lock. (SPEC §14)

## Rust-like Pseudocode (non-compilable)

```rust
// Planning-only pseudocode

struct Adapters {
    ownership: Box<dyn OwnershipOracle + Send + Sync>,
    lock: Option<Box<dyn LockManager + Send + Sync>>,   // None in dev/test
    path: Box<dyn PathResolver + Send + Sync>,
    attest: Box<dyn Attestor + Send + Sync>,
    smoke: Box<dyn SmokeTestRunner + Send + Sync>,
}

// Lock acquisition helper
fn maybe_acquire_lock(lock: &Option<Box<dyn LockManager>>, timeout_ms: u64) -> Result<Option<LockGuard>, Error> {
    match lock {
        Some(mgr) => {
            let start = now_ms();
            let guard = mgr.acquire_process_lock(timeout_ms)?;  // bounded wait inside
            let waited = now_ms() - start;
            emit_fact(Fact{ stage: ApplyAttempt, decision: Success, severity: Info, lock_wait_ms: Some(waited), ..Default })
            Ok(Some(guard))
        }
        None => {
            emit_fact(Fact{ stage: ApplyAttempt, decision: Warn, severity: Warn, msg: Some("No LockManager; concurrent apply is UNSUPPORTED"), ..Default})
            Ok(None)
        }
    }
}

// Within LockManager adapter
trait LockManager {
    // Implementation MUST enforce bounded wait; return E_LOCKING on timeout.
    fn acquire_process_lock(&self, timeout_ms: u64) -> Result<LockGuard, Error>;
}
```

## Timeouts & Telemetry

- Default timeout: 5000ms (planning default; final value decided in ADR).
- `lock_wait_ms` MUST be captured in facts for observability. (REQ-L3)

## Failure Behavior

- If `acquire_process_lock()` returns timeout → `E_LOCKING`; `apply()` aborts, records failure facts, and performs no mutations.

## Thread-safety

- All adapters used by `apply()` are required to be `Send + Sync` to allow safe sharing across threads, even though mutations are serialized by the lock.

## Tests & Evidence

- BDD: `locking_rescue.feature` scenarios for production bounded locking and dev/test WARN behavior.
- Unit: simulate lock contention and assert `E_LOCKING` mapping and `lock_wait_ms` recorded.
