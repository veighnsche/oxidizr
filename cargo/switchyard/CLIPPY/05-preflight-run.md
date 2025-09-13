# CLIPPY Remediation Plan: api/preflight/mod.rs::run

- Lint: clippy::too_many_lines (177/100)

## Proof (code reference)

```rust
pub(crate) fn run<E: FactsEmitter, A: crate::logging::AuditSink>(
    api: &super::Switchyard<E, A>,
    plan: &Plan,
) -> PreflightReport {
    // ... 177 LOC total
}
```

Source: `cargo/switchyard/src/api/preflight/mod.rs`

## Goals

- Reduce branching in `run()` by extracting row emission and summary helpers; preserve stable ordering.

## Proposed helpers

- `fn rescue_profile_check(policy: &Policy) -> (bool /*rescue_ok*/, Option<&'static str> /*profile*/)`
- `fn emit_preflight_row_for_symlink(...)`
- `fn emit_preflight_row_for_restore(...)`
- `fn emit_preflight_summary(...)`

## Architecture alternative (preferred): RowEmitter + Kind enum

- Introduce a `Kind` enum for `current_kind`/`planned_kind` rather than using stringly-typed literals.
- Create a `RowEmitter` helper that takes a typed `PreflightRowArgs` (see 08-*.md) and handles both `rows.push` and `StageLogger` emission.
- `preflight::run` becomes a thin orchestrator: compute policy eval, preservation, provenance, and delegate to `RowEmitter`.

### Implementation plan (preferred, granular)

- [ ] Define `enum Kind { File, Symlink, None, Unknown, RestoreFromBackup }` with `Display`/serde mapping to preserve output shape.
- [ ] Introduce `RowEmitter` that encapsulates `StageLogger` emissions and row assembly.
- [ ] Update call sites to pass `PreflightRowArgs` + `Kind` values; preserve row ordering and fields.

## Acceptance criteria

- [ ] Function < 100 LOC; rows and summary facts unchanged.
- [ ] Clippy clean for this function.

## Test & verification notes

- Run preflight on plans with both action kinds; compare `rows` length/order and facts before/after.
