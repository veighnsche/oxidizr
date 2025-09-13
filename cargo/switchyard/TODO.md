# Switchyard CLIPPY Remediation — Step‑by‑Step Implementation Guide (Verified)

Date: 2025-09-13
Audience: New Rust developers joining Switchyard
Scope: Replace the prior “Ultimate TODO” with a clear, PR‑sized, sequential plan. Every item below was verified against the current code in `cargo/switchyard/src/` and cites concrete files/symbols.

## Execution Mode: One Big PR (breaking allowed)

- We will deliver this as a single, comprehensive PR, structured into clearly labeled internal commits (Commit 1..10 below).
- Backward compatibility is NOT required. Where cleaner designs demand it, we will:
  - Remove transitional wrappers and shims once their replacements are wired.
  - Rename/move internal modules and types for clarity.
  - Evolve internal APIs and JSON field shapes if there is clear value. Tests, examples, and docs will be updated in lockstep.
- If a change is purely mechanical and parity is trivial to keep, we will prefer keeping shapes to minimize diff noise.

## 0) Objectives and Non‑Negotiable Invariants

- Telemetry shape parity (must hold)
  - All emitted JSON fields, names, event ordering, and summary `error_id` / `exit_code` / `summary_error_ids` must remain identical to today. Redaction behavior must not change.
- Behavior parity (must hold)
  - Idempotence, dry‑run timestamp zeroing, bounded lock wait, policy gating behavior, smoke post‑checks and auto‑rollback rules remain the same.
- Public surfaces stable (must hold)
  - Keep adapter traits (e.g., `FactsEmitter`, `AuditSink`, lock/smoke/ownership adapters) and API entrypoints stable. New helpers are internal only.

References for current behavior:

- Logging facade: `src/logging/audit.rs::StageLogger`, `src/logging/redact.rs`, `src/logging/facts.rs`
- Apply orchestrator: `src/api/apply/{mod.rs,handlers.rs,lock.rs,rollback.rs}`
- Preflight: `src/api/preflight/{mod.rs,rows.rs}` and checks: `src/preflight/checks.rs`
- Restore engine: `src/fs/restore/{engine.rs,steps.rs,idempotence.rs,integrity.rs,selector.rs}`
- Policy gating: `src/policy/gating.rs` (uses `preflight::checks`)

---

## 1) Reality check — what exists today (verified)

- Logging facade is already centralized
  - ✅ `StageLogger` and `EventBuilder` exist and are used across stages (`plan`, `preflight`, `apply.*`, `rollback.*`, `prune.result`). See `src/logging/audit.rs` and call sites in `src/api/*`.
  - ❌ Fluent helpers for `perf(...)`, `error_id(...)`, `exit_code_for(...)`, `action_id(...)` do not exist yet; call sites merge ad‑hoc `json!` objects.

- Apply handlers contain orchestration+IO+telemetry
  - ✅ `handle_ensure_symlink()` and `handle_restore()` live in `src/api/apply/handlers.rs` and compute hashes, map errors, and emit attempt/result.
  - ❌ There is no executor trait; logic is inline and long.

- Lock acquisition telemetry is verbose at call site
  - ✅ `src/api/apply/lock.rs::acquire()` handles lock manager variance and emits per‑attempt/result/summary
  - ❌ No `LockOrchestrator` facade; approx attempts + summary emissions are inlined.

- Preflight emits per‑row facts and a summary
  - ✅ `src/api/preflight/rows.rs::push_row_emit()` serializes `types::preflight::PreflightRow` and emits a fact.
  - ❌ No typed `Kind` or `PreflightRowArgs`/`RowEmitter` to shrink argument sprawl.

- Restore engine intermixes selection/integrity/execute
  - ✅ `src/fs/restore/engine.rs::restore_impl()` selects latest/previous, reads sidecar, checks idempotence/integrity, executes through `steps.rs`.
  - ❌ Not yet split into `RestorePlanner { plan -> action }` + tiny executor.

- Mount and envmeta plumbing
  - ⚠️ `src/fs/mount.rs` parses `/proc/self/mounts`. `src/fs/meta.rs::effective_uid_is_root()` parses `/proc/self/status`. `src/logging/audit.rs` uses `libc` in the `envmeta` feature.
  - ✅ `Cargo.toml` already has `rustix = { version = "0.38", features = ["fs"] }`.
  - ❌ `process` feature not enabled; rustix replacements not yet wired.

Conclusion: the “A–I” plan is compatible with the current code; most items are additive refactors preserving surfaces.

---

## 2) Single‑PR internal commit plan and order of operations

We will implement all steps within one PR as separate commits to preserve reviewability. Run “Verification” after each commit locally and before pushing the next.

1) Logging fluent helpers (A)
2) Apply executors (B)
3) Lock orchestrator (C)
4) Apply summary builder (D)
5) Preflight typed row emitter (E)
6) Restore planner (F)
7) Dependency hardening with rustix (G)
8) Policy gating checklist wrappers (H)
9) Optional error mapping facade (I)
10) Parity tests + clippy budget (J)

---

## 3) Step‑by‑step instructions (granular)

### PR1 — Logging fluent helpers (A) — [DONE]

Goal: Remove ad‑hoc `json!` merges for common fields by adding fluent helpers to `EventBuilder`.

Files to change:

- `src/logging/audit.rs` (only)

Edits:

- In `impl EventBuilder<'_>` add methods:
  - `fn perf(self, hash_ms: u64, backup_ms: u64, swap_ms: u64) -> Self`
    - Insert object `{"perf": {"hash_ms":..., "backup_ms":..., "swap_ms":...}}`
  - `fn error_id(self, id: crate::api::errors::ErrorId) -> Self`
    - Insert `error_id: crate::api::errors::id_str(id)`
  - `fn exit_code_for(self, id: crate::api::errors::ErrorId) -> Self`
    - Insert `exit_code: crate::api::errors::exit_code_for(id)`
  - `fn action_id(self, aid: impl Into<String>) -> Self`
    - Thin wrapper over existing `.action(...)`

Adoption (limited, no shape change):

- Replace inline merges only where those exact fields are being set today:
  - `src/api/apply/lock.rs` — attempt/result/summary emissions
  - `src/api/apply/mod.rs` — final apply summary (use `.perf(...)`)
  - `src/api/plan.rs` — use `.action_id(...)` and `.path(...)`
  - `src/api/preflight/rows.rs` — keep as‑is or optionally `.action_id(...)`

Verification:

- Build and run unit tests
- Visual compare the emitted JSON by running a representative dry‑run (see “Smoke test recipe” at bottom) and ensuring no field deltas
- `cargo clippy -p switchyard`

Acceptance:

- No field additions/renames; purely a call‑site cleanup
- DONE: Implemented fluent helpers (`perf`, `error_id`, `exit_code_for`, `action_id`) in `src/logging/audit.rs` and adopted in `apply::lock`, `apply::run` summary, `plan`, and `preflight` rows. `cargo test -p switchyard` passes with telemetry parity.

### PR2 — Apply executors (B) — [DONE]

Goal: Move per‑action logic out of handlers into small executors to reduce function length and clarify responsibilities.

Files to add:

- `src/api/apply/executors/mod.rs`
- `src/api/apply/executors/ensure_symlink.rs`
- `src/api/apply/executors/restore.rs`

Files to change:

- `src/api/apply/handlers.rs` (becomes thin adapters)
- `src/api/apply/mod.rs` (dispatch through executors)

Edits:

- Define trait in `executors/mod.rs`:

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

- `EnsureSymlinkExec` port: lift hashing (`fs::meta::{resolve_symlink_target, sha256_hex_of}`), EXDEV mapping (preserve libc::EXDEV mapping used today), perf accumulation, and attempt/result events. Use fluent helpers from PR1.
- `RestoreFromBackupExec` port: preserve snapshot‑before‑restore behavior when `capture_restore_snapshot=true`, precompute best‑effort `integrity_verified`, attempt fallback from `restore_file_prev()` → `restore_file()` on NotFound.
- `handlers.rs`: keep function signatures but forward to the appropriate executor implementation.
- `apply::run`: dispatch using `match` to the executors; aggregation logic remains unchanged.

Verification:

- Run apply on a plan that includes both actions and verify per‑action facts unchanged (dry‑run and commit in a temp tree)
- `cargo clippy -p switchyard`

Acceptance:

- No changes to external API or telemetry fields; only internal structure
- DONE: Added `src/api/apply/executors/{mod,ensure_symlink,restore}.rs`, moved per-action logic, and delegated from `handlers.rs`. All apply/preflight tests pass with telemetry parity.

### PR3 — Lock orchestrator (C) — [DONE]

Goal: Centralize lock acquisition bookkeeping and emissions into a facade.

Files to change:

- `src/api/apply/lock.rs`

Edits:

- Add:
  - `struct LockOutcome { backend: String, wait_ms: Option<u64>, approx_attempts: u64, guard: Option<Box<dyn LockGuard>> }`
  - `struct LockOrchestrator;` with:
    - `fn acquire<E,A>(api: &Switchyard<E,A>, mode: ApplyMode) -> LockOutcome`
    - `fn emit_failure(slog: &StageLogger<'_>, backend: &str, wait_ms: Option<u64>, attempts: u64)` — emits attempt and result failures with `E_LOCKING`
    - `fn early_report(pid: Uuid, t0: Instant, error_msg: &str) -> ApplyReport`
- Keep `pub(crate) fn acquire(...) -> LockInfo` as today, implemented via `LockOrchestrator`, to avoid touching call sites.

Verification:

- Simulate: (1) no lock manager + Commit with `LockingPolicy::Required` and (2) failing FileLockManager timeout (see `adapters/lock/file.rs` tests for guidance). Ensure emitted events match prior behavior.

Acceptance:

- Same events and fields; `approx_attempts` math unchanged
- DONE: Introduced `LockOrchestrator` and `LockOutcome` in `apply/lock.rs`, refactored `acquire()` to use it, and preserved event parity including the historical minimal `apply.result` failure emission.

### PR4 — Apply summary builder (D) — [DONE]

Goal: Replace manual summary JSON in `apply::run` with a builder.

Files to add:

- `src/api/apply/summary.rs`

Files to change:

- `src/api/apply/mod.rs`

Edits:

- Implement `ApplySummary` with:
  - `new(lock_backend: String, lock_wait_ms: Option<u64>)`
  - `perf(self, total: PerfAgg) -> Self`
  - `errors(self, errors: &Vec<String>) -> Self` (adds `summary_error_ids` using `errors::infer_summary_error_ids`)
  - `smoke_or_policy_mapping(self, errors: &Vec<String>) -> Self` (summary‑level `E_SMOKE` override else default `E_POLICY` on failure)
  - `attestation(self, api, pid, executed_len, rolled_back)` (only when success & Commit)
  - `emit(self, slog: &StageLogger<'_>, decision: &str)`
- Swap the inline summary assembly in `apply::run` with the builder calls; decision logic unchanged.

Verification:

- Run plans that produce: success, policy‑stop, smoke‑failure with/without auto‑rollback. Compare final `apply.result` summary JSON to baseline.

Acceptance:

- Byte‑for‑byte identical summary fields
- DONE: Implemented `ApplySummary` in `src/api/apply/summary.rs` and replaced inline summary construction in `apply::run`. Tests including `rollback_summary.rs` and acceptance suite pass with identical fields.

### PR5 — Preflight typed row emitter (E) — [DONE]

Goal: Reduce argument sprawl and centralize serialization.

Files to add:

- `src/api/preflight/row_emitter.rs` (or extend `rows.rs`)

Files to change:

- `src/api/preflight/{mod.rs,rows.rs}`
- (No change) `src/types/preflight.rs` remains the serialized data type

Edits:

- Define `PreflightRowArgs` builder struct holding: path, current_kind, planned_kind, policy_ok, provenance, notes, preservation, preservation_supported, restore_ready.
- Add `RowEmitter::emit_row(...)` that:
  - Converts `PreflightRowArgs` → `types::preflight::PreflightRow` and pushes into `rows`
  - Emits a `preflight` fact via `StageLogger` with the corresponding fields
- Update `preflight::run` call sites to construct `PreflightRowArgs` and delegate to `RowEmitter`.

Verification:

- Compare per‑action preflight facts and the `rows` content (order, keys) against baseline on a mixed plan (link + restore)

Acceptance:

- Fields and ordering preserved
- DONE: Added `row_emitter.rs` with `PreflightRowArgs` and `RowEmitter::emit_row`, updated `preflight::run` to use it, and shimmed `rows.rs`. Ordering and schema confirmed by existing tests.

### PR6 — Restore planner (F)

Goal: Split restore selection/integrity/execute into plan → execute to shrink function size and clarify branches.

Files to change:

- `src/fs/restore/engine.rs`
- (No change) `src/fs/restore/steps.rs` keeps I/O primitives

Edits:

- Introduce:
  - `enum RestoreAction { Noop, FileRename { backup: PathBuf, mode: Option<u32> }, SymlinkTo { dest: PathBuf, cleanup_backup: bool }, EnsureAbsent, LegacyRename { backup: PathBuf } }`
  - `struct RestorePlanner;`
    - `fn plan(target: &Path, sel: SnapshotSel, opts: &RestoreOptions) -> io::Result<(Option<PathBuf>, Option<Sidecar>, RestoreAction)>`
    - `fn execute(action: RestoreAction) -> io::Result<()>` mapping to `steps::{restore_file_bytes, restore_symlink_to, legacy_rename, ensure_absent}`
- Refactor `restore_impl` to: `let (backup_opt, sidecar_opt, action) = plan(...)?; if opts.dry_run { return Ok(()) } execute(action)` preserving best‑effort rules.

Verification:

- Existing unit tests in `steps.rs` continue to pass; add targeted tests for planner cases (file/symlink/none; integrity mismatch; previous vs latest selection)

Acceptance:

- Public functions `restore_file`/`restore_file_prev` retain signatures/behavior

### PR7 — Dependency hardening with rustix (G)

Goal: Replace ad‑hoc `/proc` parsing and `libc` calls with `rustix` where safe, without changing public behavior.

Files to change:

- `Cargo.toml` — add `process` feature to rustix: `rustix = { version = "0.38", features = ["fs","process"] }`
- `src/fs/meta.rs` — replace `effective_uid_is_root()` implementation with `rustix::process::geteuid().as_raw() == 0`
- `src/logging/audit.rs` (guarded by `#[cfg(feature = "envmeta")]`)
  - Replace `libc::getppid/geteuid/getegid` with `rustix::process::{getppid, geteuid, getegid}`
- `src/fs/mount.rs`
  - Keep the `ProcStatfsInspector` API but implement using `rustix::fs::statfs()` mapping to `types::mount::MountFlags` (fallback to `/proc/self/mounts` parsing if needed)
- Optional: create small `fs/errno.rs` for `rustix::io::Errno -> std::io::Error` bridging (helper already in `fs/atomic.rs` as `errno_to_io` — reuse or centralize)

Verification:

- Unit tests for `mount.rs` still pass; add tests for the `statfs` mapping if implemented
- `--features envmeta`: ensure `StageLogger` events still contain process/actor fields

Acceptance:

- No public API changes; behavior and fields preserved

### PR8 — Policy gating checklist wrappers (H)

Goal: Make `policy/gating.rs::evaluate_action` read as a small pipeline of typed checks while preserving strings/order.

Files to add:

- `src/policy/gating/checks.rs`

Files to change:

- `src/policy/gating.rs`

Edits:

- In `checks.rs` add:
  - `struct CheckOutput { stop: Option<String>, note: Option<String> }`
  - `trait GateCheck { fn run(&self) -> CheckOutput }`
  - Lightweight wrappers that delegate to existing `preflight::checks`: mount rw/exec, immutable, hardlink hazard, suid/sgid risk, source trust, scope allow/forbid, and strict ownership.
- In `evaluate_action`, assemble `Vec<Box<dyn GateCheck>>` per action and fold into `Evaluation { policy_ok, stops, notes }` using existing wording.

Verification:

- Preflight/Apply behavior identical on plans that hit STOP and WARN paths; strings and order preserved

Acceptance:

- No change in returned `Evaluation` shape or messages

### PR9 — Optional error mapping facade (I)

Goal: Centralize small error‑to‑`ErrorId` helpers used inside executors.

Files to add:

- `src/api/errors/map.rs`

Edits:

- Implement `map_swap_error(&io::Error) -> ErrorId` and `map_restore_error_kind(std::io::ErrorKind) -> ErrorId` with matches identical to today (`E_EXDEV` vs `E_ATOMIC_SWAP`, `E_BACKUP_MISSING` vs `E_RESTORE_FAILED`).
- Use inside executors only; no external changes.

Verification:

- Unit tests for mapping functions; executor behavior/telemetry unchanged

Acceptance:

- Pure internal tidy‑up

### PR10 — Parity tests and clippy budget (J)

Goal: Lock in behavior/telemetry parity and keep functions under clippy thresholds via refactors above, not superficial splits.

Tests to add:

- Unit tests
  - `map_swap_error` EXDEV vs other
  - `LockOrchestrator` approx attempts math
  - `RestorePlanner` plan matrix
  - `fs/mount.rs` flag mapping (if switched to `statfs`)
- Integration / snapshot tests
  - Preflight rows: count, order by `(path, action_id)`, and keys
  - Apply success / policy stop / smoke failure (+ auto‑rollback) — compare `apply.result` summary JSON via `redact_event(...)`
  - Plan stage: per‑action `plan` facts unchanged
  - Prune stage: `prune.result` success/failure fields unchanged (`api::prune_backups`)

Developer recipe:

```bash
# From cargo/switchyard/
cargo test -p switchyard
cargo clippy -p switchyard -- -D warnings
```

Acceptance:

- Tests pass and emit identical JSON after redaction; targeted functions no longer trigger `too_many_lines` / `too_many_arguments` for clippy

---

## 4) Traceability matrix — where each step applies

- PR1 (A): `src/logging/audit.rs` plus small call‑site cleanups in `src/api/{apply,plan}`
- PR2 (B): `src/api/apply/{executors/*,handlers.rs,mod.rs}`
- PR3 (C): `src/api/apply/lock.rs`
- PR4 (D): `src/api/apply/summary.rs`, `src/api/apply/mod.rs`
- PR5 (E): `src/api/preflight/{mod.rs,rows.rs}` (+ new `row_emitter.rs`)
- PR6 (F): `src/fs/restore/engine.rs` (+ uses of `steps.rs`)
- PR7 (G): `Cargo.toml`, `src/fs/{meta.rs,mount.rs}`, `src/logging/audit.rs` (`envmeta`)
- PR8 (H): `src/policy/{gating.rs,gating/checks.rs}`
- PR9 (I): `src/api/errors/map.rs` (internal)
- PR10 (J): `tests` in existing files and/or `tests/` module as needed

---

## 5) Risk and rollback

- Single‑PR with purposeful breaking changes: temporary wrappers and shims added early in the series will be removed in later commits of the same PR.
- Where parity is easy, we keep it to reduce diff size; where it blocks clarity, we intentionally break and update tests/docs in the same PR.
- Rustix migration remains isolated behind `envmeta` where applicable; libc and `/proc` usage are removed by the end of the PR.

---

## 6) Quick “smoke test” you can run locally

```bash
# Create a temp root and a simple plan (one link, one restore)
python - <<'PY'
import json, os, tempfile, pathlib
root = pathlib.Path(tempfile.mkdtemp())
(src, tgt) = (root/"bin-new", root/"usr/bin/app")
os.makedirs(tgt.parent, exist_ok=True)
(src).write_text("new")
plan = {
  "link": [{"source": str(src), "target": str(tgt)}],
  "restore": [{"target": str(tgt)}]
}
print(json.dumps(plan))
print("ROOT=", root)
PY
```

Then in a small harness, construct `SafePath`s under that `root`, build a `Plan` and call `preflight()` and `apply(Commit)` using `JsonlSink` for both facts and audit. Compare emitted JSON before/after refactors with `redact_event(...)` to confirm parity.

---

## 7) What success looks like

- Clippy no longer flags targeted functions for “too long/too many args” because logic moved into composable helpers/traits.
- Behavior and telemetry remain identical; only internal structure improved.
- New contributors can navigate by small, single‑purpose modules with focused unit tests.
