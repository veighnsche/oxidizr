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

## Implementation TODOs

- [ ] Extract rescue profile check and summary emission.
- [ ] Move per-action row assembly to helpers; reuse `rows::push_row_emit`.
- [ ] Keep `rows.sort_by(...)` and ordering unchanged.

## Acceptance criteria

- [ ] Function < 100 LOC; rows and summary facts unchanged.
- [ ] Clippy clean for this function.

## Test & verification notes

- Run preflight on plans with both action kinds; compare `rows` length/order and facts before/after.
