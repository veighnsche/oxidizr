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

## Proposed helpers

- `fn policy_requires_lock(mode: ApplyMode, policy: &Policy) -> bool`
- `fn emit_lock_failure_attempt_and_result(tctx: &AuditCtx, lock_backend: &str, wait_ms: Option<u64>, attempts: u64)`
- `fn early_apply_report(pid: Uuid, t0: Instant, msg: String) -> ApplyReport`

## Implementation TODOs

- [ ] Extract policy decision into `policy_requires_lock`.
- [ ] Extract telemetry emissions for failure into `emit_lock_failure_attempt_and_result`.
- [ ] Add `early_apply_report` to shape `ApplyReport` consistently.
- [ ] Keep bounded wait calculations and logging identical.

## Acceptance criteria

- [ ] Function < 100 LOC; identical audit events and fields.
- [ ] Behavior under `Required` and `allow_unlocked_commit` preserved.
- [ ] Clippy clean for this function.

## Test & verification notes

- Exercise both paths: with/without lock manager; commit vs dry-run; required vs allowed.
- Compare emitted `apply.attempt` and `apply.result` fields before/after on failure.
