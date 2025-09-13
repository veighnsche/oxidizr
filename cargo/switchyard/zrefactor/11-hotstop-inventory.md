# 11 — Hotspot Inventory (for holistic refactor reuse)

Date: 2025-09-13
Scope: Identify recurring refactor hotspots across the codebase to apply shared fixes from `TODO-ULTIMATE.md` broadly (not just per-CLIPPY item).

## Method

- Grepped for StageLogger usage and ad-hoc JSON merges containing `error_id`, `exit_code`, perf fields.
- Grepped for libc and `/proc` usage, EXDEV/errno patterns, and `fs2` locking.
- Reviewed upstream/downstream modules cited by CLIPPY plans.

## Hotspot categories and files

- __Audit emission boilerplate (StageLogger + ad‑hoc merges)__
  - `src/api/apply/mod.rs::run` — final `apply.result` summary fields, manual perf aggregation and error mapping.
  - `src/api/apply/handlers.rs::{handle_ensure_symlink, handle_restore}` — attempt/result emissions, inline `error_id`/`exit_code`.
  - `src/api/apply/lock.rs::acquire` — attempt, result, and parity summary line with repeated fields.
  - `src/api/apply/policy_gate.rs::enforce` — per-action `apply.result` failures + summary failure.
  - `src/api/apply/rollback.rs::{do_rollback, emit_summary}` — rollback per-action and summary with manual error fields.
  - `src/api/preflight/mod.rs::run` — per-row emission via rows helper + preflight summary.
  - `src/api/preflight/rows.rs::push_row_emit` — StageLogger usage with many parameters.
  - `src/api/plan.rs::build` — per-action plan facts (could leverage `.action_id()` helper).
  - `src/api/mod.rs::prune_backups` — success/failure emissions carrying `error_id`/`exit_code`.

- __Inline `error_id` / `exit_code` mapping (duplicated across modules)__
  - `src/api/apply/handlers.rs` — multiple inserts of `error_id` + `exit_code` for swap/restore.
  - `src/api/apply/lock.rs` — E_LOCKING mapping in attempt/result/summary lines.
  - `src/api/apply/mod.rs` — summary mapping to E_SMOKE or default E_POLICY + `summary_error_ids`.
  - `src/api/apply/policy_gate.rs` — E_POLICY mapping per-action and summary.
  - `src/api/apply/rollback.rs::emit_summary` — E_RESTORE_FAILED + chain.
  - `src/api/mod.rs::prune_backups` — generic error mapping on failure.

- __libc and `/proc` usage (portability/unsafe)__
  - `src/logging/audit.rs` (feature `envmeta`) — `libc::getppid/geteuid/getegid` in unsafe blocks; `/proc/version` best-effort.
  - `src/fs/meta.rs` — `effective_uid_is_root()` parses `/proc/self/status`.
  - `src/fs/mount.rs` — `ProcStatfsInspector::parse_proc_mounts` reads `/proc/self/mounts`.

- __EXDEV and errno bridging patterns__
  - `src/api/apply/handlers.rs` — maps EXDEV via `e.raw_os_error() == libc::EXDEV`.
  - `src/fs/atomic.rs`, `src/fs/backup/snapshot.rs`, `src/fs/restore/steps.rs`, `src/fs/swap.rs` — `from_raw_os_error(e.raw_os_error())` wrappers sprinkled around.
  - Action: centralize EXDEV detection (`ErrorKind::CrossesDevices` if MSRV permits) + helper for errno→io translation where needed.

- __Locking backend (fs2) and truncation pattern__
  - `src/adapters/lock/file.rs` — uses `fs2::FileExt` and `.truncate(true)` on open; candidate for `fd-lock` under same trait.

- __Preflight row shape and stringly `kind` values__
  - `src/api/preflight/rows.rs::push_row_emit` — 14 parameters and string kinds; candidate for `PreflightRowArgs` + `Kind` enum + `RowEmitter`.
  - `src/api/preflight/mod.rs::run` — supplies string literals for `current_kind`/`planned_kind`.
  - `src/types/preflight.rs` — typed `PreflightRow` already exists (serialize-only data shape).

- __Rollback and prune emissions mirror apply patterns__
  - `src/api/apply/rollback.rs` — summary error fields and per-action emits align with StageLogger helpers and summary builder ideas.
  - `src/api/mod.rs::prune_backups` — similar builder helpers can reduce duplication.

## Evidence (grep themes)

- StageLogger usage: `StageLogger::new` found in apply/preflight/plan/prune modules.
- `"error_id"`, `"exit_code"` string merges across apply, preflight summary, prune, rollback.
- `libc::` and `/proc/` occurrences in `logging/audit.rs`, `fs/meta.rs`, `fs/mount.rs`.
- `raw_os_error()` and `from_raw_os_error(...)` patterns in fs and apply layers.
- `fs2::` usage in `adapters/lock/file.rs`.

## Mapping to the holistic refactor (TODO‑ULTIMATE)

- StageLogger fluent helpers → apply/preflight/plan/rollback/prune.
- ActionExecutor pattern → apply handlers; `apply::run` dispatch.
- LockOrchestrator → apply lock and failure emissions.
- ApplySummary builder → `apply::run` final summary and parity with smoke/policy chains.
- Preflight `Kind`/`RowEmitter`/`PreflightRowArgs` → preflight rows + summary emitter consolidation.
- RestorePlanner → restore engine + restore executor.
- rustix migration → envmeta, euid check, mount flags; retain public surfaces.
- Error mapping shim → io/errno/EXDEV normalization without changing emitted fields.

## Priority suggestion for coverage expansion

- First adopt StageLogger helpers in: `apply/mod.rs`, `apply/lock.rs`, `apply/policy_gate.rs`, `apply/rollback.rs`, `preflight/rows.rs`, `preflight/mod.rs`, `api/mod.rs::prune_backups`, `api/plan.rs`.
- Then refactor apply via executors + LockOrchestrator + ApplySummary.
- Proceed with preflight consolidation, restore planner, rustix, and gating checklist.
