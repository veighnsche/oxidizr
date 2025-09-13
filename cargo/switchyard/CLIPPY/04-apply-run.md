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

## Proposed helpers

- `fn acquire_lock_and_emit_attempt(...) -> (LockInfo, Option<ApplyReport>)`
- `fn enforce_policy_gate_or_early_return(...) -> Option<ApplyReport>`
- `fn execute_actions_loop(...) -> (Vec<Action>, Vec<String>, PerfAgg)`
- `fn post_apply_smoke_and_maybe_rollback(...) -> (bool /*rolled_back*/, Vec<String> /*rb_errors*/)`
- `fn emit_summary_result(...)`

## Implementation TODOs

- [ ] Extract locking stage into helper (reusing `lock::acquire`).
- [ ] Extract policy gating into helper (reusing `policy_gate::enforce`).
- [ ] Extract actions loop and perf aggregation.
- [ ] Extract smoke/rollback handling.
- [ ] Extract summary emission including attestation optional field.

## Acceptance criteria

- [ ] Function < 100 LOC.
- [ ] No change to summary decision or fields (including `summary_error_ids`).
- [ ] All tests pass; clippy clean for this function.

## Test & verification notes

- Run end-to-end apply flows for: success, policy stop, action failure with rollback, smoke failure with/without auto rollback.
- Compare final `apply.result` JSON before/after.
