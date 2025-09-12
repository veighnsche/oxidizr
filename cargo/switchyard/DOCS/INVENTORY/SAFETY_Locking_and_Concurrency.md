# Locking and concurrency

- Category: Safety
- Maturity: Silver (with adapter), Bronze (without)

## Summary

Apply enforces a process lock by default in Commit. Missing `LockManager` yields `E_LOCKING` unless policy allows unlocked commits.

## Implementation

- Adapter: `cargo/switchyard/src/adapters/lock/file.rs::FileLockManager` (fs2-based advisory file lock, bounded wait with `LOCK_POLL_MS`).
- API: `cargo/switchyard/src/api/apply/mod.rs` acquires lock, emits attempt/result facts with `lock_backend`, `lock_wait_ms` and error mapping to `E_LOCKING` (exit 30).
- Policy knobs: `require_lock_manager`, `allow_unlocked_commit` in `policy::Policy`.

## Wiring Assessment

- `Switchyard::with_lock_manager()` injects manager. Apply respects policy and mode (DryRun vs Commit).
- Facts include backend label and attempts; errors abort early.
- Conclusion: wired correctly when adapter provided; dev ergonomics allow no-lock with warning in DryRun.

## Evidence and Proof

- Unit tests: `FileLockManager` timeout/success test.
- Apply-stage tests check error mapping and facts in aggregate.

## Gaps and Risks

- Only file-backed lock provided; no per-target granularity.

## Next Steps to Raise Maturity

- Add contention tests and a golden for timeout path.
- Consider per-target locks if required by consumers.

## Related

- SPEC v1.1 locking requirement and bounded wait.
- `cargo/switchyard/src/constants.rs::{LOCK_POLL_MS, DEFAULT_LOCK_TIMEOUT_MS}`.
