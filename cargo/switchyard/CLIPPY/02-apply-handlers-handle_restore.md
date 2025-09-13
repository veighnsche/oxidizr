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

## Proposed helpers

- `fn pre_restore_snapshot_if_enabled(target: &SafePath, api: &Switchyard<_, _>, dry: bool) -> (used_prev: bool, backup_ms: u64)`
- `fn compute_integrity_verified(target: &SafePath, used_prev: bool, tag: &str) -> Option<bool>`
- `fn try_restore(target: &SafePath, used_prev: bool, dry: bool, force: bool, tag: &str) -> std::io::Result<()>`
- `fn emit_restore_success(...)` and `fn emit_restore_failure(...)`

## Architecture alternative (preferred): ActionExecutor pattern

Similar to `EnsureSymlink`, introduce a per-action executor that encapsulates apply-time orchestration and telemetry for `RestoreFromBackup`.

- Define `ActionExecutor` trait (shared with symlink executor), and implement `RestoreFromBackupExec`.
- Move audit field assembly, timing, and error-id mapping into the executor implementation.
- Keep the actual restore call routed through `crate::fs::restore::{restore_file, restore_file_prev}`; this integrates cleanly with a future `RestorePlanner` in the restore engine (see 06-*.md).
- Benefit: isolates per-action logic, reduces growth of `handlers.rs`, and simplifies `apply::run`.

### Updated Implementation TODOs (preferred)

- [ ] Implement `RestoreFromBackupExec` under `api/apply/executors.rs` using the outlined helpers for snapshot/integrity/fallback.
- [ ] Update `apply::run` dispatch to call `RestoreFromBackupExec` for `Action::RestoreFromBackup`.
- [ ] Add StageLogger fluent helpers (e.g., `.perf(..)`, `.error(..)`, `.exit_code(..)`) and adopt them here to shrink boilerplate.
- [ ] Preserve fallback semantics (prev NotFound → latest) and integrity flag emission.
- [ ] Optional temporary `#[allow(clippy::too_many_lines)]` during transition.

## Implementation TODOs (fallback: helper split only)

- [ ] Extract pre-restore snapshot capture + timing into `pre_restore_snapshot_if_enabled`.
- [ ] Extract integrity verification into `compute_integrity_verified` (use sidecar if present; best-effort).
- [ ] Implement `try_restore` that attempts previous→latest fallback on NotFound only.
- [ ] Move JSON building for success/failure into dedicated emitters; reuse in main.
- [ ] Keep `force` logic identical: `best_effort_restore || !sidecar_integrity`.
- [ ] Optional temporary `#[allow(clippy::too_many_lines)]` while landing helpers.

## Acceptance criteria

- [ ] Function < 100 LOC.
- [ ] Identical fallback path behavior and error mapping.
- [ ] Integrity verification flag emitted identical when available.
- [ ] Tests green and clippy clean for this function.

## Test & verification notes

- Add a unit/integration test that simulates previous snapshot NotFound then succeeds on latest.
- Compare audit JSON for a restore success and failure before/after (includes `before_kind`, `after_kind`, `error_id`, `exit_code`, and integrity flag when present).
