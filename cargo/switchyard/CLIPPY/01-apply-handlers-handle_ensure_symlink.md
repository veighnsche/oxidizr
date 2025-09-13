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

## Proposed helpers (new, `pub(super)` in `api/apply/`)

- `fn build_apply_attempt_fields(aid: &Uuid, target: &SafePath, api: &Switchyard<_, _>) -> serde_json::Value`
- `fn compute_symlink_hashes(source: &SafePath, target: &SafePath) -> (String /*before*/, String /*after*/, u64 /*hash_ms*/)`
- `fn map_swap_error(e: &std::io::Error) -> ErrorId`
- `fn build_apply_result_fields(..., degraded_used: bool, fsync_ms: u64, before_kind: String, after_kind: String) -> serde_json::Value`

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

### Updated Implementation TODOs (preferred)

- [ ] Create `api/apply/executors.rs` and define `ActionExecutor` trait.
- [ ] Implement `EnsureSymlinkExec` using existing logic but moving field assembly and perf into small private fns.
- [ ] Update `apply::run` to instantiate/dispatch to `EnsureSymlinkExec` for `Action::EnsureSymlink`.
- [ ] Add fluent helpers to `logging/audit.rs::EventBuilder` (e.g., `.perf(..)`, `.error(..)`, `.exit_code(..)`) and replace ad-hoc field merges where possible.
- [ ] Keep `swap::replace_file_with_symlink` semantics unchanged; preserve error-id mapping and degraded flags.
- [ ] Optionally add a temporary `#[allow(clippy::too_many_lines)]` while landing the trait.

## Implementation TODOs (fallback: helper split only)

- [ ] Create `api/apply/ops.rs` or extend `audit_fields.rs` with the helpers above.
- [ ] Replace inline JSON construction in `handle_ensure_symlink` with `build_apply_attempt_fields` and `build_apply_result_fields`.
- [ ] Replace inline hashing section with `compute_symlink_hashes`.
- [ ] Replace inline EXDEV/generic swap mapping with `map_swap_error`.
- [ ] Keep call to `swap::replace_file_with_symlink` and error handling semantics identical.
- [ ] Optionally add a targeted `#[allow(clippy::too_many_lines, reason = "orchestrator; split into helpers")]` as a temporary gate if needed.

## Acceptance criteria

- [ ] Function < 100 LOC.
- [ ] No changes in audit/logging fields (manual diff or snapshot test).
- [ ] All tests pass (`cargo test -p switchyard`).
- [ ] `cargo clippy -p switchyard` shows no `too_many_lines` for this item.

## Test & verification notes

- Run a representative plan containing `EnsureSymlink` and compare JSONL logs before/after (hashes, degraded flags, fsync warnings, error_id/exit_code when failing).
- Ensure error mapping still produces `E_EXDEV` when appropriate and `E_ATOMIC_SWAP` otherwise.
