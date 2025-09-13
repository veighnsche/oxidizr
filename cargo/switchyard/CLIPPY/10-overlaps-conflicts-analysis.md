# CLIPPY Plans Overlap & Conflict Analysis

Date: 2025-09-13
Scope: All remediation plans in `cargo/switchyard/CLIPPY/`

## Documents reviewed

- `01-apply-handlers-handle_ensure_symlink.md`
- `02-apply-handlers-handle_restore.md`
- `03-apply-lock-acquire.md`
- `04-apply-run.md`
- `05-preflight-run.md`
- `06-fs-restore-restore_impl.md`
- `07-policy-gating-evaluate_action.md`
- `08-preflight-rows-push_row_emit.md`
- `09-dependency-hardening-rustix-process-statfs.md`

## Summary of overlaps (intentional synergies)

- __StageLogger fluent helpers__: Proposed in 01, 03, 04, and referenced in 08. All suggest adding helpers on `src/logging/audit.rs::EventBuilder` to reduce boilerplate and stabilize fact shapes.
  - Canonicalize on the following helper signatures:
    - `fn perf(self, hash_ms: u64, backup_ms: u64, swap_ms: u64) -> Self`
    - `fn error_id(self, id: ErrorId) -> Self`
    - `fn exit_code_for(self, id: ErrorId) -> Self`
    - `fn action_id(self, aid: impl Into<String>) -> Self` (optional wrapper over existing `.action()`)
- __ActionExecutor pattern__: 01 and 02 propose a shared `ActionExecutor` trait and executor impls per action; 04 adopts the same dispatch in `apply::run`. Keep a single trait in `src/api/apply/executors/mod.rs`.
- __Lock orchestration__: 03 proposes `LockOrchestrator`; 04 references integrating lock outcome/telemetry into `apply::run` summary. Use `LockOrchestrator` and keep a thin compatibility wrapper for the existing `acquire(...)` if needed.
- __Preflight row emission__: 05 proposes `RowEmitter` + `Kind` enum and 08 proposes `PreflightRowArgs` + `RowEmitter` + `Kind`. These align. Implement once and reuse in both `preflight/mod.rs` and `preflight/rows.rs`.
- __Restore refactor__: 06 introduces `RestorePlanner (plan → execute)`; 02’s `RestoreFromBackupExec` notes future alignment. Implement `RestorePlanner` first, then call from the executor to keep `handlers.rs` slim and telemetry stable.
- __EXDEV and low-level ops hardening__: 09 proposes migrating to `rustix` and using `ErrorKind::CrossesDevices`. 01’s `map_swap_error(...)` helper can adopt the new detection method without changing error-id mapping.

## Potential conflicts and recommended resolutions

- __Method naming mismatch for StageLogger helper__
  - Overlap: 01/03/04 specify `.error_id(...)`; 08 loosely mentions `.error(...)`.
  - Resolution: Standardize on `.error_id(...)` to avoid ambiguity with any existing `.error(...)` builder or plain text fields. Update 08-aligned work to use `.error_id(...)`.

- __Multiple edit hotspots in `src/logging/audit.rs`__
  - Overlap: 01/03/04 add fluent helpers; 09 removes `unsafe` and `libc` usages for envmeta.
  - Resolution: Implement fluent helpers first on top of current code; then apply 09’s rustix/envmeta changes. Keep helper method names stable to minimize rebase churn.

- __Executor trait duplication/placement__
  - Overlap: 01/02 define the trait signature in their docs.
  - Resolution: Create a single `src/api/apply/executors/mod.rs` exporting the `ActionExecutor<E, A>` trait; 01 and 02’s executors live under `src/api/apply/executors/ensure_symlink.rs` and `.../restore.rs` respectively. 04’s `apply::run` dispatch imports only from this module.

- __Locking backend evolution vs orchestration facade__
  - Overlap: 03 adds `LockOrchestrator`; 09 optionally migrates `fs2` → `fd-lock`.
  - Resolution: Keep `crate::adapters::lock::LockGuard` as the stable interface returned by `LockOrchestrator`. Migration to `fd-lock` remains an internal change behind the adapter; public telemetry fields (`lock_backend`, `lock_wait_ms`, `lock_attempts`) stay unchanged.

- __Mount detection refactor and gating/preflight checks__
  - Overlap: 07 uses mount checks (often via `preflight::checks`); 09 replaces `/proc/self/mounts` parsing with `rustix::fs::statfs`.
  - Resolution: Preserve the `ProcStatfsInspector` type/entry points but change the implementation to call `statfs`. This avoids signature churn in gating and preflight while improving robustness.

- __Apply summary assembly vs executor/lock refactors__
  - Overlap: 04 introduces `ApplySummary` builder while 01/02/03 refactor handlers and lock acquisition.
  - Resolution: Land `LockOrchestrator` and ActionExecutors first; then introduce `ApplySummary` so it can consume the stabilized perf and lock metadata without double-touching `apply::run`.

- __ErrorKind::CrossesDevices availability__
  - Note: Ensure MSRV supports `ErrorKind::CrossesDevices`. If not, gate with a fallback: check `raw_os_error()` against `libc::EXDEV` behind a small compatibility shim, and remove once MSRV allows.

## Recommended implementation order (to avoid churn)

1. __Add StageLogger fluent helpers__ in `src/logging/audit.rs` (01/03/04/08). Use the canonical signatures above.
2. __Introduce ActionExecutor__ in `src/api/apply/executors/` and refactor:
   - `EnsureSymlinkExec` (01)
   - `RestoreFromBackupExec` (02)
   - Wire dispatch in `apply::run` (04) with temporary wrappers in `handlers.rs`.
3. __Create LockOrchestrator__ (03), wire into `apply::run` (04), keep a thin `acquire(...)` wrapper.
4. __Introduce ApplySummary builder__ (04) to finalize `apply.result` facts; preserve field names/order.
5. __Preflight consolidation__: add `Kind` enum and `RowEmitter` + `PreflightRowArgs` (05/08); refactor `preflight/mod.rs` and `preflight/rows.rs`.
6. __Restore engine__: add `RestorePlanner` (06) and call from `RestoreFromBackupExec` (02); keep behavior identical.
7. __Dependency hardening__ (09): migrate to `rustix` for envmeta, uid/ppid, EXDEV detection, and `statfs`. Optionally migrate file locks to `fd-lock` behind the adapter.
8. __Gating refactor__: implement checklist pipeline (07) reusing the stable `preflight::checks` and preserving strings/order.

## Cross-cutting invariants to protect

- __Telemetry shapes and field names__ must remain byte-for-byte identical (attempt/result ordering, per-action and summary lines, perf structure, error-id/exit-code fields, `summary_error_ids`, attestation presence conditions).
- __Behavioral semantics__ (idempotence, fallback paths, dry-run timing zeros, best-effort restore, bounded lock wait) must be unchanged.
- __Public adapters and trait surfaces__ (`AuditSink`, `FactsEmitter`, lock adapter traits) should remain stable; internal changes (e.g., `fd-lock`) stay behind adapters.

## Dedupe checklist (what to implement once)

- __EventBuilder fluent helpers__: add once in `src/logging/audit.rs` and reuse everywhere (01/03/04/08).
- __ActionExecutor trait__: define once in `src/api/apply/executors/mod.rs` and reuse (01/02/04).
- __Kind enum + RowEmitter + PreflightRowArgs__: define once and reuse across `preflight/` (05/08).
- __RestorePlanner__: define once in `fs/restore/engine.rs` and call from restore executor (02/06).

## Open items / notes

- Verify MSRV for `ErrorKind::CrossesDevices`; add a small compatibility shim if required.
- Keep `ProcStatfsInspector` public surface stable while changing internals to `statfs`.
- Ensure policy-derived force semantics for restore (02) remain consistent when the planner is introduced (06).

---
Generated from the current contents of `cargo/switchyard/CLIPPY/` and intended to guide implementation sequencing while avoiding duplicate or conflicting work.
