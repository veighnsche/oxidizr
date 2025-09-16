# Locking

- Provide a `LockManager` in Commit mode (production) to serialize mutations. Without it, concurrent `apply()` is unsupported and emits a WARN fact (dev/test only).
- Lock acquisition uses a bounded wait with timeout → maps to `E_LOCKING` (exit 30). Facts include `lock_wait_ms` and may include `lock_attempts`.
- In production deployments, a `LockManager` is required by policy. Omission is permitted only in development/testing contexts.

Telemetry (facts)
- `apply.attempt.lock_backend`
- `apply.attempt.lock_wait_ms`
- `apply.attempt.lock_attempts` (approximate)

Operator guidance
- Configure `with_lock_manager(...)` and tune `.with_lock_timeout_ms(...)` on `ApiBuilder`.
- If contention occurs frequently, inspect `lock_wait_ms` distributions and consider backoff.
- For safety, keep plan sizes reasonable (see Operational Bounds) and avoid long-held locks.

Citations:
- `src/adapters/lock/mod.rs`
- `src/adapters/lock/file.rs`
- `src/api/errors.rs`
- SPEC §2.5 Locking; `SPEC/SPEC.md`
