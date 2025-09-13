# CLIPPY Remediation Plan: api/preflight/rows.rs::push_row_emit

- Lint: clippy::too_many_arguments (14/7) — currently Warning

## Proof (code reference)

```rust
pub(crate) fn push_row_emit<E: FactsEmitter, A: AuditSink>(
    api: &super::super::Switchyard<E, A>,
    plan: &Plan,
    act: &Action,
    rows: &mut Vec<Value>,
    ctx: &AuditCtx<'_>,
    path: String,
    current_kind: &str,
    planned_kind: &str,
    policy_ok: Option<bool>,
    provenance: Option<Value>,
    notes: Option<Vec<String>>,
    preservation: Option<Value>,
    preservation_supported: Option<bool>,
    restore_ready: Option<bool>,
) {
    // 14 parameters total
}
```

Source: `cargo/switchyard/src/api/preflight/rows.rs`

## Goals

- Reduce API surface arguments by introducing a builder struct while preserving behavior.

## Proposed interface change

- Introduce:

  ```rust
  struct PreflightRowArgs {
    path: String,
    current_kind: String,
    planned_kind: String,
    policy_ok: Option<bool>,
    provenance: Option<Value>,
    notes: Option<Vec<String>>,
    preservation: Option<Value>,
    preservation_supported: Option<bool>,
    restore_ready: Option<bool>,
  }
  ```

- Change signature to: `push_row_emit<E, A>(api: &Switchyard<E, A>, plan: &Plan, act: &Action, rows: &mut Vec<Value>, ctx: &AuditCtx<'_>, args: &PreflightRowArgs)`

## Architecture alternative (preferred): PreflightRowArgs + RowEmitter + Kind

- Introduce a `Kind` enum shared with preflight run (see 05-*.md) and ensure serde/Display preserves output shapes.
- Add a `RowEmitter` helper that:
  - Builds the typed `PreflightRow` and pushes into `rows`.
  - Emits the `StageLogger` event with fluent helpers (consider adding `.perf(..)`, `.error(..)`, `.exit_code(..)` if applicable here or reuse `.field`).
  - Accepts `PreflightRowArgs` + `Kind` inputs to avoid argument sprawl.
- This consolidates both the row JSON construction and event emission logic and prevents drift.

### Updated Implementation TODOs (preferred)

- [ ] Define `Kind` enum and its mapping.
- [ ] Implement `RowEmitter` with `emit_row(args: &PreflightRowArgs)` that pushes row and logs the event.
- [ ] Refactor `push_row_emit` to a thin wrapper or deprecate it in favor of `RowEmitter` usage at call sites.
- [ ] Update `preflight/mod.rs` to construct `PreflightRowArgs` and call `RowEmitter`.

## Implementation TODOs (fallback: interface change only)

- [ ] Define `PreflightRowArgs` near `push_row_emit` (same file/mod) and derive `Default` where sensible.
- [ ] Update `preflight/mod.rs` call sites to construct and pass `PreflightRowArgs`.
- [ ] Keep emitted facts and row contents identical.
- [ ] Optionally add `#[allow(clippy::too_many_arguments)]` temporarily while landing changes.

## Acceptance criteria

- [ ] Argument count ≤ 7; warning resolved.
- [ ] No changes in emitted row fields or order.
- [ ] Tests remain green.

## Test & verification notes

- Run preflight and compare rows/facts before/after.
