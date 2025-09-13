# CLIPPY Remediation Plan: api/apply/mod.rs::run

- Lint: clippy::too_many_lines (182/100)

## Proof (code reference)

```rust
pub(crate) fn run<E: FactsEmitter, A: AuditSink>(
    api: &super::Switchyard<E, A>,
    plan: &Plan,
    mode: ApplyMode,
) -> ApplyReport {
    // ... 182 LOC total
}
```

Source: `cargo/switchyard/src/api/apply/mod.rs`

## Goals

- Split orchestration into phases with helpers; keep behavior identical.

## Architecture alternative (preferred): ActionExecutor + ApplySummary builder

Beyond local helpers, adopt:

- `ActionExecutor` dispatch for per-action execution (see 01/02 plans). This turns the actions loop into a thin dispatcher and shrinks `run()` considerably.
- A typed `ApplySummary` builder to assemble the final `apply.result` summary with:
  - lock metadata (backend, wait_ms, attempts)
  - perf aggregate
  - error_id/exit_code mapping (including smoke → E_SMOKE; default → E_POLICY) and `summary_error_ids`
  - optional attestation field
  - fluent StageLogger helpers (e.g., `.perf(..)`, `.error(..)`, `.exit_code(..)`)

### Implementation plan (preferred, granular)

- [ ] Add ApplySummary builder module
  - [ ] Create `src/api/apply/summary.rs` with:

    ```rust
    pub(crate) struct ApplySummary {
        fields: serde_json::Value,
    }
    impl ApplySummary {
        pub fn new(lock_backend: String, lock_wait_ms: Option<u64>) -> Self { /* ... */ }
        pub fn perf(mut self, total: super::perf::PerfAgg) -> Self { /* insert hash_ms, backup_ms, swap_ms */ }
        pub fn errors(mut self, errors: &Vec<String>) -> Self { /* infer summary_error_ids via errors::infer_summary_error_ids */ }
        pub fn smoke_or_policy_mapping(mut self, errors: &Vec<String>) -> Self { /* E_SMOKE vs default E_POLICY */ }
        pub fn attestation(mut self, api: &super::Switchyard<impl FactsEmitter, impl AuditSink>, pid: uuid::Uuid, executed_len: usize, rolled_back: bool) -> Self { /* build attestation if available */ }
        pub fn emit(self, slog: &StageLogger<'_>, decision: &str) { /* slog.apply_result().merge(&fields).emit_*() */ }
    }
    ```

  - [ ] Ensure identical field names to current summary, including nested `perf` and attestation payload.
- [ ] Add StageLogger fluent helpers in `src/logging/audit.rs`
  - [ ] Implement `EventBuilder::perf(...)`, `EventBuilder::error_id(ErrorId)`, and `EventBuilder::exit_code_for(ErrorId)`; use in apply and handlers/executors.
- [ ] Integrate ActionExecutor dispatch
  - [ ] Use the executors introduced in 01/02 docs to replace bulky match bodies:

    ```rust
    for (idx, act) in plan.actions.iter().enumerate() {
        let (exec, err, perf) = match act {
            Action::EnsureSymlink { .. } => executors::EnsureSymlinkExec.execute(api, &tctx, &pid, act, idx, dry),
            Action::RestoreFromBackup { .. } => executors::RestoreFromBackupExec.execute(api, &tctx, &pid, act, idx, dry),
        };
        // aggregate perf, collect exec/err
    }
    ```

  - [ ] Keep rollback and smoke sections but extract emission to small local helpers where needed.
- [ ] Replace ad-hoc final summary construction with ApplySummary builder
  - [ ] Instantiate with `lock_backend` and `lock_wait_ms` from the lock outcome.
  - [ ] Chain `.perf(perf_total)`, `.errors(&errors)`, `.smoke_or_policy_mapping(&errors)`.
  - [ ] On success and non-dry, conditionally `.attestation(...)`.
  - [ ] Emit with `decision` derived from presence of errors.
- [ ] Keep policy gate enforcement and lock orchestration as-is (or updated to LockOrchestrator), using fluent helpers for emissions.
- [ ] Remove dead code paths in `apply::run` created by the refactor; keep function < 100 LOC.
- [ ] Invariants
  - [ ] Summary fields (lock_backend, lock_wait_ms, perf, error_id, exit_code, summary_error_ids) identical to current behavior.
  - [ ] Attestation field only present on success + commit; payload format unchanged.
  - [ ] Stage order (attempts before results; per-action and final summary) unchanged.
- [ ] Tests
  - [ ] End-to-end: success, policy stop, action failure + rollback, smoke failure with/without auto rollback — compare summary JSON before/after.
  - [ ] Ensure summary_error_ids inferred chain remains stable for representative error sets.

## Acceptance criteria

- [ ] Function < 100 LOC.
- [ ] No change to summary decision or fields (including `summary_error_ids`).
- [ ] All tests pass; clippy clean for this function.

## Test & verification notes

- Run end-to-end apply flows for: success, policy stop, action failure with rollback, smoke failure with/without auto rollback.
- Compare final `apply.result` JSON before/after.
