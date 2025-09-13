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

### Implementation plan (preferred, granular)
- [ ] Define `Kind` enum and its mapping (serde + Display) to match existing string values ("file", "symlink", "none", "unknown", "restore_from_backup").
- [ ] Introduce `PreflightRowArgs` as a typed struct with builder-style setters where needed; derive `Default` when possible.
- [ ] Implement `RowEmitter` with method `emit_row(args: &PreflightRowArgs, kind_current: Kind, kind_planned: Kind)` that:
  - [ ] Builds and pushes `PreflightRow` into `rows`.
  - [ ] Emits `StageLogger` event with fields `current_kind`, `planned_kind`, `path`, and optional fields (`policy_ok`, `provenance`, `notes`, `preservation`, `preservation_supported`, `backup_tag`).
  - [ ] Uses StageLogger fluent helpers where available.
- [ ] Refactor `push_row_emit` to a thin wrapper or deprecate it in favor of `RowEmitter` calls from `preflight/mod.rs`.
- [ ] Update `preflight/mod.rs` to construct `PreflightRowArgs` and call `RowEmitter` for both symlink and restore actions.

## Acceptance criteria

- [ ] Argument count reduced; original too_many_arguments warning resolved.
- [ ] Row contents and fact shapes unchanged; stable order preserved.

## Test & verification notes

- [ ] Run preflight on plans containing both action kinds; diff resulting rows and facts before/after.
