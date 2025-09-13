# CLIPPY Remediation Plan: api/apply/handlers.rs::handle_restore

- Lint: clippy::too_many_lines (192/100)
- Status: Denied at crate level

## Proof (code reference)

```rust
/// Handle a `RestoreFromBackup` action: perform restore and emit per-action facts.
/// Returns (`executed_action_if_success`, `error_message_if_failure`).
pub(crate) fn handle_restore<E: FactsEmitter, A: AuditSink>(
    api: &super::super::Switchyard<E, A>,
    tctx: &AuditCtx<'_>,
    pid: &Uuid,
    act: &Action,
    idx: usize,
    dry: bool,
    _slog: &StageLogger<'_>,
) -> (Option<Action>, Option<String>, PerfAgg) {
    // ... 192 LOC total
}
```

Source: `cargo/switchyard/src/api/apply/handlers.rs`

## Goals

- Reduce to < 100 LOC while preserving fallback semantics and emitted fields.

## Architecture alternative (preferred): ActionExecutor pattern

Similar to `EnsureSymlink`, introduce a per-action executor that encapsulates apply-time orchestration and telemetry for `RestoreFromBackup`.

- Define `ActionExecutor` trait (shared with symlink executor), and implement `RestoreFromBackupExec`.
- Move audit field assembly, timing, and error-id mapping into the executor implementation.
- Keep the actual restore call routed through `crate::fs::restore::{restore_file, restore_file_prev}`; this integrates cleanly with a future `RestorePlanner` in the restore engine (see 06-*.md).
- Benefit: isolates per-action logic, reduces growth of `handlers.rs`, and simplifies `apply::run`.

### Implementation TODOs (preferred, granular)

- [ ] Create executor skeleton
  - [ ] Add `src/api/apply/executors/restore.rs`.
  - [ ] Define `struct RestoreFromBackupExec;` implementing the shared `ActionExecutor` trait:

    ```rust
    impl<E: FactsEmitter, A: AuditSink> ActionExecutor<E, A> for RestoreFromBackupExec {
        fn execute(
            &self,
            api: &super::super::Switchyard<E, A>,
            tctx: &crate::logging::audit::AuditCtx<'_>,
            pid: &uuid::Uuid,
            act: &crate::types::Action,
            idx: usize,
            dry: bool,
        ) -> (Option<crate::types::Action>, Option<String>, super::perf::PerfAgg) { /* ... */ }
    }
    ```

- [ ] Private helpers inside executor (encapsulate incumbent logic)
  - [ ] `fn pre_restore_snapshot_if_enabled(target: &SafePath, api: &Switchyard<_, _>, dry: bool, tag: &str) -> (used_prev: bool, backup_ms: u64)`
  - [ ] `fn compute_integrity_verified(target: &SafePath, used_prev: bool, tag: &str) -> Option<bool>`
    - [ ] Use `fs::backup::{find_latest_backup_and_sidecar, find_previous_backup_and_sidecar}` and `read_sidecar`.
    - [ ] Compare `sha256_hex_of(backup)` with sidecar `payload_hash` when present.
  - [ ] `fn try_restore_force(target: &SafePath, used_prev: bool, dry: bool, force: bool, tag: &str) -> std::io::Result<()>`
    - [ ] Attempt `restore_file_prev` when `used_prev`, else `restore_file`.
    - [ ] If `used_prev` and NotFound, fallback to `restore_file`.
- [ ] Emit attempt/result via StageLogger
  - [ ] Attempt: include `action_id`, `path`, `safepath_validation=success`, `backup_durable`.
  - [ ] Result success: include `before_kind`, `after_kind` (respect dry), `backup_durable`, optional `sidecar_integrity_verified`.
  - [ ] Result failure: include `error_id` and `exit_code` using `ErrorId` mapping; carry `sidecar_integrity_verified` when computed.
  - [ ] Adopt fluent helpers added to `EventBuilder` (perf, error_id, exit_code) to reduce boilerplate.
- [ ] Force semantics and policy wiring
  - [ ] Preserve: `force = api.policy.apply.best_effort_restore || !api.policy.durability.sidecar_integrity`.
  - [ ] Dry-run behavior: no I/O actions performed; success path emits appropriate kinds, timings zeroed.
- [ ] Wire dispatch in `src/api/apply/mod.rs::run`
  - [ ] Replace direct call to `handlers::handle_restore` with executor dispatch using `RestoreFromBackupExec`.
  - [ ] Keep perf aggregation identical (hash_ms, backup_ms, swap_ms).
- [ ] Remove/Deprecate legacy handler
  - [ ] Keep a thin wrapper `handle_restore` temporarily delegating to the executor to de-risk rollout; remove after stable.
- [ ] Telemetry invariants
  - [ ] `apply.attempt`/`apply.result` fields and values identical; especially `before_kind`/`after_kind`, integrity flag, and error mapping.
  - [ ] Fallback semantics (prev NotFound → latest) unchanged.
  - [ ] Hash timing (`hash_ms`) maintained from pre-check computations.

## Acceptance criteria

- [ ] Function < 100 LOC.
- [ ] Identical fallback path behavior and error mapping.
- [ ] Integrity verification flag emitted identical when available.
- [ ] Tests green and clippy clean for this function.

## Test & verification notes

- Add a unit/integration test that simulates previous snapshot NotFound then succeeds on latest.
- Compare audit JSON for a restore success and failure before/after (includes `before_kind`, `after_kind`, `error_id`, `exit_code`, and integrity flag when present).
