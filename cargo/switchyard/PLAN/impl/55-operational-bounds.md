# Operational Bounds (Planning Only)

Captures timing and size bounds required by SPEC §9 and related deterministic behavior.

References:

- SPEC: `cargo/switchyard/SPEC/SPEC.md §9 Operational Bounds`
- Requirements: `REQ-BND1`

## Bounds

- `fsync(parent)` MUST occur ≤50ms after `rename`. (REQ-BND1)
- Default maximum plan size = 1000 actions (configurable), to keep apply times bounded and logs manageable. (SPEC §9)

## Enforcement Plan (Planning)

- `fs/atomic.rs` records the timestamp for the `rename` and `fsync(parent)` and asserts `duration_ms <= 50`.
- Facts include `duration_ms` for the rename+fsync step to allow CI monitoring.
- In debug/test builds, add an internal assertion/log when bound is exceeded.

## Pseudocode (non-compilable)

```rust
// Planning-only pseudocode
fn atomic_rename(parent, staged, target) -> Result<(), Error> {
    let t0 = now_ms();
    renameat(staged, target)?;
    fsync_parent(parent)?;
    let dt = now_ms() - t0;
    emit_fact(Fact{ stage: ApplyResult, decision: Success, duration_ms: Some(dt), ..Default });
    if dt > 50 { /* consider warning/logging; SPEC requires ≤50ms */ }
    Ok(())
}
```

## Tests & Evidence

- Unit: mock timers to force `duration_ms` over/under 50 and assert WARN/notes emitted.
- BDD: `operational_bounds.feature` scenario exercises the bound.
- CI: Golden fixtures include `duration_ms` field; monitor distributions over time.
