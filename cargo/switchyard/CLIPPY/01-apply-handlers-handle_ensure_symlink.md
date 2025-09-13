# CLIPPY Remediation Plan: api/apply/handlers.rs::handle_ensure_symlink

- Lint: clippy::too_many_lines (116/100)
- Status: Denied at crate level (`src/lib.rs`: `#![deny(clippy::too_many_lines)]`)

## Proof (code reference)

Function signature and context as of current HEAD:

```rust
/// Handle an `EnsureSymlink` action: perform the operation and emit per-action facts.
/// Returns (`executed_action_if_success`, `error_message_if_failure`).
pub(crate) fn handle_ensure_symlink<E: FactsEmitter, A: AuditSink>(
    api: &super::super::Switchyard<E, A>,
    tctx: &AuditCtx<'_>,
    pid: &Uuid,
    act: &Action,
    idx: usize,
    dry: bool,
    _slog: &StageLogger<'_>,
) -> (Option<Action>, Option<String>, PerfAgg) {
    // ... 116 LOC total
}
```

Source: `cargo/switchyard/src/api/apply/handlers.rs`

## Goals

- Reduce function to < 100 LOC without changing behavior or audit field shapes.

## Architecture alternative (preferred): ActionExecutor pattern

Rather than only splitting into helpers, define a per-action executor that owns the orchestration and telemetry for this action type. This encodes the concept and minimizes `handlers.rs` growth.

- Define trait:

  ```rust
  trait ActionExecutor {
      fn execute(
          &self,
          api: &Switchyard<impl FactsEmitter, impl AuditSink>,
          tctx: &AuditCtx<'_>,
          pid: &Uuid,
          act: &Action,
          idx: usize,
          dry: bool,
      ) -> (Option<Action>, Option<String>, PerfAgg);
  }
  ```

- Provide `EnsureSymlinkExec` implementing `ActionExecutor`.
- In `apply::run`, dispatch by action kind to the appropriate executor (this will also shrink `apply::run`).
- Keep `StageLogger` usage inside the executor and reuse fluent helpers (see cross-cutting notes below).

### Updated Implementation TODOs (preferred, granular)

- [ ] Create module layout for executors
  - [ ] Add `src/api/apply/executors/mod.rs` with the trait and re-exports.
  - [ ] Define trait:

    ```rust
    pub(crate) trait ActionExecutor<E: FactsEmitter, A: AuditSink> {
        fn execute(
            &self,
            api: &super::super::Switchyard<E, A>,
            tctx: &crate::logging::audit::AuditCtx<'_>,
            pid: &uuid::Uuid,
            act: &crate::types::Action,
            idx: usize,
            dry: bool,
        ) -> (Option<crate::types::Action>, Option<String>, super::perf::PerfAgg);
    }
    ```

- [ ] Implement `EnsureSymlinkExec` in `src/api/apply/executors/ensure_symlink.rs`
  - [ ] Factor tiny private helpers inside this file:
    - [ ] `compute_hashes(source: &SafePath, target: &SafePath) -> (String, String, u64)`
    - [ ] `map_swap_error(e: &std::io::Error) -> ErrorId` (preserve EXDEV vs swap mapping)
    - [ ] `after_kind(dry, target) -> String` uses `kind_of()` when non-dry
  - [ ] Emit attempt/result via `StageLogger` with fluent helpers (see below).
  - [ ] Call `crate::fs::swap::replace_file_with_symlink` exactly as today; propagate `(degraded_used, fsync_ms)` and timing into `PerfAgg`.
- [ ] Wire dispatch in `src/api/apply/mod.rs::run`
  - [ ] Replace direct call to `handlers::handle_ensure_symlink` with executor dispatch:

    ```rust
    match act {
        Action::EnsureSymlink { .. } => {
            let exec = executors::EnsureSymlinkExec;
            exec.execute(api, &tctx, &pid, act, idx, dry)
        }
        // ...
    }
    ```

  - [ ] Ensure perf aggregation and error collection remain unchanged.
- [ ] Add StageLogger fluent helpers in `src/logging/audit.rs`
  - [ ] Methods on `EventBuilder`:
    - [ ] `fn perf(self, hash_ms: u64, backup_ms: u64, swap_ms: u64) -> Self`
    - [ ] `fn error_id(self, id: ErrorId) -> Self`
    - [ ] `fn exit_code_for(self, id: ErrorId) -> Self`
    - [ ] `fn action_id(self, aid: impl Into<String>) -> Self` (wrapper over existing `.action()` if desired)
  - [ ] Replace ad-hoc `.merge(json!({...}))` sites in the new executor with these helpers to reduce lines.
- [ ] Delete or deprecate `handle_ensure_symlink`
  - [ ] Keep a thin wrapper temporarily that calls the executor to minimize churn; remove after all call sites use the executor.
- [ ] Telemetry invariants (document and assert during review)
  - [ ] `apply.attempt` and `apply.result` must retain all fields and values (including `degraded`/`degraded_reason`, `before_kind`/`after_kind`, `backup_durable`).
  - [ ] Hash fields and `hash_alg` unchanged; `maybe_warn_fsync` semantics preserved.
  - [ ] Error-id mapping and `exit_code` values identical.
- [ ] Tests
  - [ ] Unit-test `map_swap_error` with EXDEV and non-EXDEV paths.
  - [ ] Integration test a plan with `EnsureSymlink` to confirm no-op on identical symlink and success path telemetry.
  - [ ] Run `cargo clippy -p switchyard` to confirm the function no longer triggers `too_many_lines`.

## Acceptance criteria

- [ ] Function < 100 LOC.
- [ ] No changes in audit/logging fields (manual diff or snapshot test).
- [ ] All tests pass (`cargo test -p switchyard`).
- [ ] `cargo clippy -p switchyard` shows no `too_many_lines` for this item.

## Test & verification notes

- Run a representative plan containing `EnsureSymlink` and compare JSONL logs before/after (hashes, degraded flags, fsync warnings, error_id/exit_code when failing).
- Ensure error mapping still produces `E_EXDEV` when appropriate and `E_ATOMIC_SWAP` otherwise.
