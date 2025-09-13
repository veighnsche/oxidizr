# Switchyard CLIPPY Remediation — Ultimate TODO (Holistic, Minimal-Churn)

Date: 2025-09-13
Scope: Consolidate 01–09 plans plus upstream/downstream code review into a single streamlined refactor plan with the fewest moving parts while preserving behavior and telemetry shapes.

## Objective

Bring long functions under Clippy thresholds without mere helper-splitting by introducing a few coherent, shared abstractions that reduce duplication across apply, preflight, policy gating, restore, and logging — while keeping public surfaces and emitted JSON facts byte-for-byte identical per SPEC v1.1.

## Findings from upstream/downstream review

- __Inline JSON assembly causes length and drift__
  - Files: `src/api/apply/{mod.rs,handlers.rs,lock.rs}`, `src/api/preflight/{mod.rs,rows.rs}`
  - Root cause: repeated manual `json!` + field merges for perf, error_id, exit_code, attestation.
  - Remedy: StageLogger `EventBuilder` fluent helpers for perf/error/action simplify emissions and stabilize shape.

- __Handlers blend orchestration + FS ops + telemetry__
  - Files: `src/api/apply/handlers.rs`
  - Root cause: per-action branching and telemetry inside monolithic functions.
  - Remedy: per-action `ActionExecutor` with a thin orchestrator dispatch in `apply::run`.

- __Lock acquisition mixes policy, metrics, facts__
  - Files: `src/api/apply/lock.rs`
  - Root cause: lifecycle + failure tri-emission + early report formatting combined.
  - Remedy: `LockOrchestrator` facade with stable output fields; keep thin wrapper for `acquire(...)` to limit churn.

- __Preflight row construction has argument sprawl__
  - Files: `src/api/preflight/{mod.rs,rows.rs}`
  - Root cause: `push_row_emit` 14-arg surface; stringly `current_kind`/`planned_kind`.
  - Remedy: typed `PreflightRowArgs` + `Kind` enum + `RowEmitter` that both pushes rows and emits facts.

- __Policy gating duplicates checks and strings__
  - Files: `src/policy/gating.rs`, `src/preflight/checks/`
  - Root cause: repeated calls and message text mapping per action.
  - Remedy: small “Checklist” pipeline of typed checks feeding `Evaluation` with preserved wording and order.

- __Restore engine intermixes selection, integrity, execution__
  - Files: `src/fs/restore/engine.rs`
  - Root cause: selection + integrity + execution in one function.
  - Remedy: `RestorePlanner` (plan→execute) returning an enum action for a tiny executor to run.

- __Proc parsing and libc usage leak into reliability surface__
  - Files: `src/fs/mount.rs`, `src/fs/meta.rs`, `src/logging/audit.rs` (feature `envmeta`)
  - Root cause: `/proc` string parsing and `unsafe` libc calls.
  - Remedy: migrate to `rustix::{process,fs::statfs}`; keep public surfaces stable to avoid churn.

## Cross-cutting invariants (must hold)

- __Telemetry shape parity__: exact fields, names, decision ordering, and `summary_error_ids` behavior.
- __Behavior parity__: idempotence, dry-run zeros, best-effort restore, bounded lock wait, error-id mapping.
- __Public surfaces stable__: adapters (`AuditSink`, `FactsEmitter`, lock traits), policy types, and emitted JSON schema.

## Streamlined plan (with minimal churn)

### A) Logging/Audit primitives

- [ ] Add fluent helpers to `src/logging/audit.rs::EventBuilder`:
  - [ ] `fn perf(self, hash_ms: u64, backup_ms: u64, swap_ms: u64) -> Self`
  - [ ] `fn error_id(self, id: crate::api::errors::ErrorId) -> Self`
  - [ ] `fn exit_code_for(self, id: crate::api::errors::ErrorId) -> Self`
  - [ ] `fn action_id(self, aid: impl Into<String>) -> Self` (wrapper around `.action()`)
- [ ] Replace ad-hoc merges in apply/preflight/lock with fluent calls (no field changes).
- [ ] Defer rustix/envmeta migration to stream G to avoid rebase churn.
  
  Initial adoption targets (no field changes):
  - `src/api/apply/{mod.rs,lock.rs,policy_gate.rs,rollback.rs}`
  - `src/api/preflight/{rows.rs,mod.rs}`
  - `src/api/plan.rs`
  - `src/api/mod.rs::prune_backups`

  References: `CLIPPY/01-apply-handlers-handle_ensure_symlink.md`, `CLIPPY/03-apply-lock-acquire.md`, `CLIPPY/04-apply-run.md`, `CLIPPY/08-preflight-rows-push_row_emit.md`.

  Granular steps:
  - Add helper methods in `src/logging/audit.rs` within `impl EventBuilder<'_>`:
    - `perf(...)` inserts `perf.hash_ms/backup_ms/swap_ms`.
    - `error_id(...)` inserts normalized `error_id` string via `errors::id_str`.
    - `exit_code_for(...)` inserts `exit_code` via `errors::exit_code_for`.
    - `action_id(...)` delegates to `.action(...)` to keep one call-site.
  - Update call sites to replace inline `.merge(json!({"error_id":...,"exit_code":...}))` with fluent helpers:
    - `src/api/apply/lock.rs::acquire()` attempt/result/summary failure emissions.
    - `src/api/apply/policy_gate.rs::enforce()` per-action failures + summary.
    - `src/api/apply/rollback.rs::{do_rollback,emit_summary}` per-action and summary.
    - `src/api/preflight/rows.rs::push_row_emit()` where applicable (only add fields present today).
    - `src/api/plan.rs::build()` use `.action_id()`.
    - `src/api/mod.rs::prune_backups()` success/failure mapping.
  - Keep emitted fields byte-for-byte identical; do not add or rename fields.

### B) Apply: ActionExecutor pattern

- [ ] Create `src/api/apply/executors/mod.rs` exporting:
  - [ ] `pub(crate) trait ActionExecutor<E: FactsEmitter, A: AuditSink> { fn execute(&self, api: &super::super::Switchyard<E,A>, tctx: &AuditCtx<'_>, pid: &Uuid, act: &Action, idx: usize, dry: bool) -> (Option<Action>, Option<String>, PerfAgg); }`
  - [ ] Lightweight `ApplyCtx` struct if parameter fanout persists.
- [ ] Implement `EnsureSymlinkExec` in `executors/ensure_symlink.rs`:
  - [ ] Factor tiny private helpers: `map_swap_error`, `after_kind`, `compute_hashes`.
  - [ ] Use fluent helpers for perf/error/action_id.
- [ ] Implement `RestoreFromBackupExec` in `executors/restore.rs`:
  - [ ] Precompute integrity verification; preserve fallback from `restore_file_prev` → `restore_file`.
- [ ] In `apply::run`, dispatch by action kind; keep `handlers::handle_*` as thin adapters initially.

  References: `CLIPPY/01-apply-handlers-handle_ensure_symlink.md`, `CLIPPY/02-apply-handlers-handle_restore.md`, `CLIPPY/04-apply-run.md`.

  Granular steps:
  - Create directory `src/api/apply/executors/` with `mod.rs` exporting trait and executors.
  - Implement trait in `mod.rs`:
    - `pub(crate) trait ActionExecutor<E: FactsEmitter, A: AuditSink> { fn execute(...) -> (Option<Action>, Option<String>, PerfAgg); }`.
  - Port EnsureSymlink flow into `executors/ensure_symlink.rs`:
    - Extract `compute_hashes(source: &SafePath, target: &SafePath) -> (before_hash, after_hash, hash_ms)`.
    - Extract `map_swap_error(e: &io::Error) -> ErrorId` (EXDEV mapping preserved).
    - Build attempt/result via `StageLogger` using fluent helpers; set `degraded` flags exactly as today.
    - Return `(exec, err, PerfAgg)` identical to handlers.
  - Port RestoreFromBackup flow into `executors/restore.rs`:
    - Capture optional previous snapshot timing (`backup_ms`) when policy requires.
    - Precompute `integrity_verified` (best effort) using current sidecar logic.
    - Execute `restore_file_prev` then fallback to `restore_file` on NotFound; emit attempt/result using helpers.
  - Thin adapters in `src/api/apply/handlers.rs` call into the corresponding executor (temporary shim to limit churn).
  - Wire `apply::run` match arms to call executors; aggregate `PerfAgg` exactly as before.

### C) Locking orchestration

- [ ] Add `LockOrchestrator` (can live in `src/api/apply/lock.rs` to minimize moves):
  - [ ] `acquire(...) -> LockOutcome { backend, wait_ms, approx_attempts, guard }`
  - [ ] `emit_failure(...)` to produce attempt/result parity with E_LOCKING and summary line.
  - [ ] `early_report(...) -> ApplyReport` for failure path.
- [ ] Keep a thin `acquire(...)` wrapper returning `LockInfo` to avoid touching external call sites.

  References: `CLIPPY/03-apply-lock-acquire.md`, `CLIPPY/04-apply-run.md`.

  Granular steps:
  - In `src/api/apply/lock.rs`:
    - Add `struct LockOutcome { backend: String, wait_ms: Option<u64>, approx_attempts: u64, guard: Option<Box<dyn LockGuard>> }`.
    - Add `struct LockOrchestrator;` with:
      - `fn acquire<E,A>(api,&Switchyard<E,A>, mode: ApplyMode) -> LockOutcome` (compute `approx_attempts` like today).
      - `fn emit_failure(slog: &StageLogger<'_>, backend: &str, wait_ms: Option<u64>, attempts: u64)` that emits attempt and result (and summary parity line) using fluent helpers.
      - `fn early_report(pid: Uuid, t0: Instant, error_msg: &str) -> ApplyReport` with same shape.
    - Keep existing `acquire(...)` as wrapper building `LockInfo` from `LockOutcome`, preserving return type.
  - In `apply::run`, keep usage unchanged short-term; later migrate to `LockOutcome` directly.

### D) Apply summary

- [ ] Add `src/api/apply/summary.rs` with `ApplySummary` builder:
  - [ ] `new(lock_backend, lock_wait_ms)` → chain `.perf(...)`, `.errors(&Vec<String>)`, `.smoke_or_policy_mapping(...)`, optional `.attestation(...)`.
  - [ ] Emit via `StageLogger` with `.apply_result()`.
- [ ] Replace manual summary assembly in `apply::run` with builder (fields identical).

  References: `CLIPPY/04-apply-run.md`.

  Granular steps:
  - Create `src/api/apply/summary.rs` with `ApplySummary { fields: serde_json::Value }`.
  - Implement:
    - `new(lock_backend: String, lock_wait_ms: Option<u64>) -> Self`.
    - `perf(self, total: PerfAgg) -> Self` inserts hash_ms/backup_ms/swap_ms.
    - `errors(self, errors: &Vec<String>) -> Self` adds `summary_error_ids` via `errors::infer_summary_error_ids`.
    - `smoke_or_policy_mapping(self, errors: &Vec<String>) -> Self` sets E_SMOKE or default E_POLICY (only on failure).
    - `attestation(self, api, pid, executed_len, rolled_back)` builds bundle when applicable.
    - `emit(self, slog: &StageLogger<'_>, decision: &str)` outputs the event.
  - Replace manual JSON assembly in `src/api/apply/mod.rs::run` with builder chaining; keep decision logic unchanged.

### E) Preflight rows and kind typing

- [ ] Define `enum Kind { File, Symlink, Dir, None, Unknown, RestoreFromBackup }` with serde/Display mapping identical to current literals.
- [ ] Introduce `PreflightRowArgs` (path, current_kind, planned_kind, policy_ok, provenance, notes, preservation, preservation_supported, restore_ready).
- [ ] Create `RowEmitter` that pushes a typed `PreflightRow` into `rows` and emits the corresponding fact via StageLogger.
- [ ] Refactor `preflight::run` and `preflight::rows::push_row_emit` to use `RowEmitter` and typed `Kind`.
- [ ] Preserve serialized row shape by reusing `src/types/preflight.rs::PreflightRow` (serialize-only data type).

  References: `CLIPPY/05-preflight-run.md`, `CLIPPY/08-preflight-rows-push_row_emit.md`.

  Granular steps:

- Add `enum Kind` with serde/Display mapping preserving strings: "file", "symlink", "dir", "none", "unknown", "restore_from_backup".
- Add `struct PreflightRowArgs { path, current_kind, planned_kind, policy_ok, provenance, notes, preservation, preservation_supported, restore_ready }` with builder setters.
- Add `struct RowEmitter;` with `fn emit_row<E,A>(api: &Switchyard<E,A>, rows: &mut Vec<Value>, slog: &StageLogger<'_>, aid: &str, args: &PreflightRowArgs, kind_current: Kind, kind_planned: Kind)` that pushes `PreflightRow` and emits fact.
- Update `preflight/rows.rs::push_row_emit` to delegate to `RowEmitter` (or deprecate after migrating callers).
- Update `preflight/mod.rs::run` to construct `PreflightRowArgs` and call `RowEmitter` for both action kinds.

### F) Restore engine (plan → execute)

- [ ] Add `RestorePlanner` in `src/fs/restore/engine.rs` returning `(backup_opt, sidecar_opt, RestoreAction)` where `RestoreAction` ∈ {`Noop`, `FileRename{..}`, `SymlinkTo{..}`, `EnsureAbsent`, `LegacyRename{..}`}.
- [ ] Map `RestoreAction` to existing `steps::*` in a tiny executor; keep `restore_file(_prev)` behavior identical.
- [ ] Call planner from `RestoreFromBackupExec` (B) to keep handlers slim.

  References: `CLIPPY/06-fs-restore-restore_impl.md`, `CLIPPY/02-apply-handlers-handle_restore.md`.

  Granular steps:
  - Define `enum RestoreAction { Noop, FileRename { backup: PathBuf, mode: Option<u32> }, SymlinkTo { dest: PathBuf, cleanup_backup: bool }, EnsureAbsent, LegacyRename { backup: PathBuf } }`.
  - Add `struct RestorePlanner;` with `fn plan(target: &Path, sel: SnapshotSel, opts: &RestoreOptions) -> io::Result<(Option<PathBuf>, Option<Sidecar>, RestoreAction)>` selecting latest/previous, reading sidecar, idempotence, integrity checks.
  - Add `fn execute(action: RestoreAction)` mapping to `steps::{restore_file_bytes, restore_symlink_to, legacy_rename, ensure_absent}`.
  - Refactor `restore_impl` to `plan` + `execute` pattern while preserving dry-run and best-effort behavior.

### G) Dependency hardening (rustix)

- [ ] Cargo: `rustix = { version = "0.38", features = ["fs","process"] }`.
- [ ] `fs/meta.rs`: replace `effective_uid_is_root()` `/proc/self/status` parse with `rustix::process::geteuid().as_raw() == 0`.
- [ ] `logging/audit.rs` (`envmeta`): replace libc `getppid/geteuid/getegid` with rustix; remove unsafe blocks.
- [ ] `api/apply/handlers.rs`: EXDEV mapping via `ErrorKind::CrossesDevices` (add MSRV guard; fallback to `raw_os_error()==libc::EXDEV` in a tiny shim if needed).
- [ ] `fs/mount.rs`: keep `ProcStatfsInspector` API; implement via `rustix::fs::statfs()` flag mapping; retire `/proc/self/mounts` parsing.
- [ ] Optional: `adapters/lock/file.rs` migrate `fs2` → `fd-lock` under the same trait; keep `.truncate(true)` removal.
- [ ] `fs/{atomic.rs,backup/snapshot.rs,restore/steps.rs,swap.rs}`: centralize errno→io bridging in a tiny compat helper (wrapping `from_raw_os_error(e.raw_os_error())`) to keep callers concise and consistent.

  References: `CLIPPY/09-dependency-hardening-rustix-process-statfs.md`.

  Granular steps:

- Add dependency in `Cargo.toml`: `rustix = { version = "0.38", features = ["fs","process"] }`.
- Replace libc envmeta calls in `logging/audit.rs` (feature `envmeta`) with `rustix::process::{getppid, geteuid, getegid}`.
- Replace `/proc/self/status` parsing in `fs/meta.rs` with `geteuid().as_raw()==0`.
- Replace `/proc/self/mounts` parsing in `fs/mount.rs` with `statfs()` and map flags to `MountFlags`.
- Create `fs/errno.rs` (or similar) providing `fn io_from_errno(e: rustix::io::Errno) -> io::Error` and helpers; update call sites.
- For EXDEV detection inside executors, prefer `ErrorKind::CrossesDevices` with MSRV fallback to raw errno comparison behind a tiny shim.

### H) Policy gating as a checklist

- [ ] Add `policy/gating/checks.rs` with:
  - [ ] `struct CheckOutput { stop: Option<String>, note: Option<String> }`
  - [ ] `trait GateCheck { fn run(&self) -> CheckOutput }`
  - [ ] Concrete check wrappers: `MountRwExecCheck`, `ImmutableCheck`, `HardlinkRiskCheck`, `SuidSgidRiskCheck`, `SourceTrustCheck`, `ScopeCheck`, and strict ownership inline.
- [ ] In `gating::evaluate_action`, assemble per-action pipelines using these checks; fold outputs preserving wording and order.

  References: `CLIPPY/07-policy-gating-evaluate_action.md`.

  Granular steps:
  - Create `src/policy/gating/checks.rs` with `CheckOutput` and `GateCheck`.
  - Implement wrappers over existing `preflight::checks`:
    - `MountRwExecCheck { path }`, `ImmutableCheck { path }`, `HardlinkRiskCheck { policy, path }`, `SuidSgidRiskCheck { policy, path }`, `SourceTrustCheck { policy, source }`, `ScopeCheck { policy, target }`.
  - In `evaluate_action`, build vectors of `Box<dyn GateCheck>` per action and run; convert outputs to `Evaluation { policy_ok, stops, notes }`.
  - Preserve exact wording, order, and stop/note mapping.

### I) Small error mapping facade (optional, low churn)

- [ ] Add `api/errors/map.rs` with helpers: `map_restore_error_kind(kind: std::io::ErrorKind) -> ErrorId`, `map_swap_error(e: &io::Error) -> ErrorId`.
- [ ] Use in executors; keep `ErrorId` mapping identical.

  References: `CLIPPY/01-apply-handlers-handle_ensure_symlink.md`, `CLIPPY/02-apply-handlers-handle_restore.md`.

  Granular steps:
  - Implement mapping helpers with exhaustive matches for current behavior (E_EXDEV vs E_ATOMIC_SWAP, E_BACKUP_MISSING vs E_RESTORE_FAILED, etc.).
  - Update executors to call the helpers; keep any special-cases (e.g., sidecar write failed → E_POLICY) intact.

### J) Tests & verification

- [ ] Unit tests:
  - [ ] `LockOrchestrator::approx_attempts` calculations.
  - [ ] `map_swap_error` EXDEV vs non-EXDEV.
  - [ ] `RestorePlanner` plan cases (file/symlink/none; integrity mismatch; best-effort).
  - [ ] `fs/mount.rs` flag mapping and `ensure_rw_exec()`.
- [ ] Integration tests:
  - [ ] `apply::run` success, policy stop, action failure + rollback, smoke failure with/without auto rollback — compare summary JSON before/after.
  - [ ] Preflight rows ordering and field parity across both action kinds.
  - [ ] Plan stage: per-action `plan` facts unchanged for mixed link/restore inputs.
  - [ ] Prune stage: `prune.result` success and failure shapes unchanged.
- [ ] Grep diff: ensure no loss of fields; CI runs `cargo clippy -p switchyard` clean for targeted functions.

  References: all CLIPPY docs 01–09; this test plan enforces telemetry and behavior parity.

  Granular steps:
  - Add snapshot tests (or golden JSON compare) for per-action apply facts (symlink/restore) and final summary under success/failure/smoke.
  - Add preflight rows parity tests (length/order/kinds/optional fields present when expected).
  - Add prune success/failure parity tests.
  - Add lock parity tests to ensure attempt/result/summary emissions on failure remain identical (existing tests cover much of this).
  - Run `cargo clippy -p switchyard` and confirm targeted functions no longer violate `too_many_lines`/`too_many_arguments`.

## CLIPPY doc and code map (quick reference)

- 01 → `apply/handlers.rs::handle_ensure_symlink` → Executors (B), StageLogger helpers (A), Error mapping (I)
- 02 → `apply/handlers.rs::handle_restore` → Executors (B), RestorePlanner (F), StageLogger helpers (A), Error mapping (I)
- 03 → `apply/lock.rs::acquire` → LockOrchestrator (C), StageLogger helpers (A)
- 04 → `apply/mod.rs::run` → ApplySummary (D), Executors dispatch (B), StageLogger helpers (A)
- 05 → `api/preflight/mod.rs::run` → RowEmitter/Kind/Args (E), StageLogger helpers (A)
- 06 → `fs/restore/engine.rs::restore_impl` → RestorePlanner (F)
- 07 → `policy/gating.rs::evaluate_action` → Checklist pipeline (H)
- 08 → `api/preflight/rows.rs::push_row_emit` → RowEmitter/Kind/Args (E), StageLogger helpers (A)
- 09 → `logging/audit.rs` envmeta, `fs/meta.rs`, `fs/mount.rs` → rustix migration (G)

## Extended codebase coverage (from docs 11 & 12)

- __Apply stage__
  - `src/api/apply/{mod.rs,handlers.rs,lock.rs,policy_gate.rs,rollback.rs}` — adopt StageLogger helpers; introduce executors, LockOrchestrator, ApplySummary; keep thin adapters initially.

- __Preflight stage__
  - `src/api/preflight/{mod.rs,rows.rs}` — adopt RowEmitter + Kind + Args; keep `types/preflight.rs` as the serialized row type.

- __Plan and prune__
  - `src/api/plan.rs` — use `.action_id()` helper in plan facts.
  - `src/api/mod.rs::prune_backups` — use fluent helpers for success/failure; keep field names identical.

- __Filesystem + logging infrastructure__
  - `src/logging/audit.rs` — add fluent helpers; later migrate envmeta off libc.
  - `src/fs/{meta.rs,mount.rs}` — migrate euid and mount flags to rustix.
  - `src/fs/{atomic.rs,backup/snapshot.rs,restore/steps.rs,swap.rs}` — consolidate errno bridging; leave public behavior unchanged.
  - `src/adapters/lock/file.rs` — optional `fd-lock` migration behind existing trait.

## Implementation order (minimize churn)

1) A: StageLogger fluent helpers
2) B: ActionExecutor + EnsureSymlinkExec + RestoreFromBackupExec; dispatch in `apply::run` (temporary wrappers retained)
3) C: LockOrchestrator integrated into `apply::run` (keep `acquire(...)` wrapper)
4) D: ApplySummary builder to replace manual summary assembly
5) E: Preflight RowEmitter + Kind enum; refactor `preflight` emitters
6) F: RestorePlanner; call from restore executor
7) G: rustix migration (envmeta, euid, statfs, EXDEV shim); optional fd-lock under adapter
8) H: Gating checklist pipeline; preserve strings/order
9) I: Optional error mapping facade tidy-up
10) J: Tests pass; clippy clean for targeted items; telemetry snapshot diffs are identical

## Churn control tactics

- Keep function names and modules; add new ones alongside and route via thin wrappers.
- Re-export new modules where needed; avoid moving public types.
- Maintain field names/values exactly; add helpers but not new fields.
- Gate rustix EXDEV via MSRV check + fallback; drop libc fully only after migration.

## Open items / notes

- Confirm MSRV for `ErrorKind::CrossesDevices`; if older MSRV, temporarily keep libc-based EXDEV under a small shim.
- Verify `ProcStatfsInspector` callers (preflight checks) require no signature changes.
- Keep attestation emission payload and presence conditions unchanged.

---
This TODO consolidates CLIPPY 01–09 with upstream/downstream code realities for a coherent, low-churn refactor that addresses fundamentals rather than superficial helper splits.
