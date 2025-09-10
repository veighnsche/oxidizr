# ADR Template

- Title: Locking and concurrency model
- Status: Proposed
- Date: 2025-09-11

## Context

SPEC requires that only one `apply()` mutator proceeds at a time in production under a `LockManager` with bounded wait and timeout → `E_LOCKING`, with `lock_wait_ms` recorded in facts. Dev/test allows no lock manager but must WARN that concurrent apply is unsupported.

## Decision

- Define a `LockManager` adapter trait with `acquire_process_lock()` returning a guard.
- In production, require a `LockManager`; absence is an error at configuration time.
- Lock acquisition uses bounded wait with configurable timeout; on timeout, emit `E_LOCKING` and record `lock_wait_ms`.
- In dev/test, if no `LockManager` is provided, emit a WARN fact and proceed best-effort with a process-local mutex to reduce accidental overlap (still unsupported).

## Consequences

+ Clear separation of concerns; production-safe under adapter.
+ Telemetry for contention issues.
- Adds adapter dependency for production deployments.

## Links

- `cargo/switchyard/SPEC/SPEC.md` §§ 2.5, 14
- `cargo/switchyard/PLAN/10-architecture-outline.md`
