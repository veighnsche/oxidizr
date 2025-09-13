# CLIPPY Remediation Plan: api/apply/lock.rs::acquire

- Lint: clippy::too_many_lines (107/100)

## Proof (code reference)

```rust
pub(crate) fn acquire<E: FactsEmitter, A: AuditSink>(
    api: &super::super::Switchyard<E, A>,
    t0: Instant,
    pid: Uuid,
    mode: ApplyMode,
    tctx: &crate::logging::audit::AuditCtx<'_>,
) -> LockInfo {
    // ... 107 LOC total
}
```

Source: `cargo/switchyard/src/api/apply/lock.rs`

## Goals

- Reduce below 100 LOC by extracting policy decision & failure emission helpers.

## Architecture alternative (preferred): LockOrchestrator facade

Encapsulate lock acquisition lifecycle (policy decision, bounded wait metrics, audit emissions, and early report shaping) in a dedicated orchestrator type. This reduces duplication and stabilizes telemetry shape.

### Implementation plan (preferred, granular)

- [ ] Define types and module layout
  - [ ] Keep implementation in `src/api/apply/lock.rs` or move to `src/api/apply/lock/mod.rs` if file grows.
  - [ ] Introduce types:

    ```rust
    pub(crate) struct LockOutcome {
        pub backend: String,
        pub wait_ms: Option<u64>,
        pub approx_attempts: u64,
        pub guard: Option<Box<dyn crate::adapters::lock::LockGuard>>,
    }
    pub(crate) struct LockOrchestrator;
    ```

- [ ] Implement API surface
  - [ ] `impl LockOrchestrator {
          pub fn acquire<E: FactsEmitter, A: AuditSink>(api: &super::super::Switchyard<E,A>, mode: ApplyMode) -> LockOutcome {
              // Port policy decision and acquisition from acquire()
              // Compute wait_ms and approx_attempts, set backend via util::lock_backend_label
          }
          pub fn emit_failure(slog: &StageLogger<'_>, backend: &str, wait_ms: Option<u64>, attempts: u64) {
              // Emit apply.attempt and apply.result failures for locking with E_LOCKING and exit_code 30
          }
          pub fn early_report(pid: Uuid, t0: Instant, error_msg: &str) -> ApplyReport {
              // Shape ApplyReport exactly as current failure path
          }
        }`
  - [ ] Reuse `util::lock_backend_label(api.lock.as_deref())` to name backend consistently.
  - [ ] Preserve bounded wait computation: `approx_attempts = wait_ms.map_or_else(|| u64::from(api.lock.is_some()), |ms| 1 + (ms / LOCK_POLL_MS))`.
- [ ] StageLogger fluent helpers adoption (`src/logging/audit.rs`)
  - [ ] Add `EventBuilder::perf(hash, backup, swap)`, `EventBuilder::error_id(ErrorId)`, `EventBuilder::exit_code_for(ErrorId)`; update emissions here and in `apply::run`.
- [ ] Migrate `apply::run` to orchestrator
  - [ ] Replace `lock::acquire(...) -> LockInfo` with `LockOrchestrator::acquire(...) -> LockOutcome` and a light `LockInfo` wrapper if needed for backward compatibility.
  - [ ] When lock acquisition fails, call `emit_failure(...)` (attempt + result + summary parity) and return `early_report(...)`.
  - [ ] On success, attach `backend`, `wait_ms`, `approx_attempts` to the apply attempt and final summary.
- [ ] Remove/Refactor legacy `acquire`
  - [ ] Keep a thin `acquire(...)` wrapper that calls `LockOrchestrator::acquire` and builds the existing `LockInfo` struct (to minimize churn), then schedule removal.
- [ ] Invariants and telemetry parity
  - [ ] Maintain identical emissions on failure: per-action `apply.result` failures include `error_id = E_LOCKING` and `exit_code = 30`, plus a summary `apply.result` failure with perf zeros.
  - [ ] Ensure audit log line `apply: lock acquisition failed (E_LOCKING)` still appears with `Level::Error`.
  - [ ] Preserve success path metadata: `lock_backend`, `lock_wait_ms`, and `lock_attempts` fields.
- [ ] Tests
  - [ ] Unit-test `approx_attempts` calculation across `None`, short waits, long waits.
  - [ ] Integration tests against `apply::run` in both modes:
    - [ ] No lock manager + Commit + policy requires lock → failure (E_LOCKING), early report, no executed actions.
    - [ ] Lock manager present + bounded wait success → attempt includes wait metadata, final summary present.
  - [ ] Run `cargo clippy -p switchyard` to ensure the long function is reduced and no `too_many_lines` remains.

## Acceptance criteria

- [ ] Function < 100 LOC; identical audit events and fields.
- [ ] Behavior under `Required` and `allow_unlocked_commit` preserved.
- [ ] Clippy clean for this function.

## Test & verification notes

- Exercise both paths: with/without lock manager; commit vs dry-run; required vs allowed.
- Compare emitted `apply.attempt` and `apply.result` fields before/after on failure.
