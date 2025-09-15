# Locking

- Provide a `LockManager` in Commit mode (production) to serialize mutations.
- Timeouts map to `E_LOCKING` with `lock_wait_ms` fact.

Citations:
- `cargo/switchyard/src/adapters/lock/mod.rs`
- `cargo/switchyard/src/adapters/lock/file.rs`
- `cargo/switchyard/src/api/errors.rs`
