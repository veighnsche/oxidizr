# 12 — Root Cause Analysis of Complexity Hotspots

Date: 2025-09-13
Scope: Explain fundamental causes behind long/complex functions and duplicated logic across `switchyard`, and justify the holistic refactor plan beyond simple helper-splitting.

## Method

- Reviewed CLIPPY remediation docs 01–09 and the consolidated plans (10, TODO‑ULTIMATE).
- Read upstream/downstream modules to validate patterns and find systemic causes:
  - `src/api/apply/{mod.rs,handlers.rs,lock.rs,policy_gate.rs,rollback.rs}`
  - `src/api/preflight/{mod.rs,rows.rs}` and `src/preflight/{checks,yaml}.rs`
  - `src/api/{mod.rs,plan.rs}` (plan/prune orchestrators)
  - `src/fs/{meta.rs,mount.rs,restore/engine.rs,restore/steps.rs,swap.rs,backup/snapshot.rs,atomic.rs}`
  - `src/logging/audit.rs` (StageLogger)
  - `src/adapters/lock/file.rs` (fs2 locking)

## Symptoms (what we see)

- `clippy::too_many_lines` in: `apply::run`, `handlers::handle_ensure_symlink`, `handlers::handle_restore`, `lock::acquire`, `preflight::run`, `gating::evaluate_action`, `restore::engine::restore_impl`.
- `clippy::too_many_arguments` in: `preflight/rows.rs::push_row_emit` (14 params).
- Repeated manual `json!` construction for fields like `error_id`, `exit_code`, and perf aggregates.
- Mixed responsibilities (orchestration + I/O + telemetry) in single functions.

## Root causes (why this happens)

1) Centralized telemetry primitives are underutilized

- Evidence: repeated ad‑hoc merges in `apply/*`, `preflight/*`, `plan.rs`, `api/mod.rs::prune_backups`.
- Missing fluent helpers (e.g., `.perf(...)`, `.error_id(...)`, `.exit_code_for(...)`, `.action_id(...)`) forces verbose JSON assembly, inflating line counts and risking drift.

2) Orchestration and per‑action logic are entangled

- Evidence: `handlers.rs` blends hashing/timing, swap/restore calls, error mapping, and StageLogger emissions per action.
- Consequence: long functions with branching per action; hard to test in isolation.

3) Lock acquisition intermixes policy, wait metrics, and emissions

- Evidence: `lock.rs::acquire` computes attempts, emits attempt+result (plus an extra parity summary), returns early reports.
- Consequence: repeated field assembly, long function; difficult to reuse in other stages.

4) Preflight row emission API is shape‑hostile

- Evidence: `preflight/rows.rs::push_row_emit` has 14 args, stringly `current_kind`/`planned_kind`.
- Consequence: call sites are verbose and brittle; kind literals invite typos and drift.

5) Restore engine conflates selection, integrity, idempotence, and execution

- Evidence: `fs/restore/engine.rs::restore_impl` handles all phases inline.
- Consequence: high branching/line count; limited unit testability of plan logic.

6) Dependency surface leaks (`libc` + `/proc`) and errno bridging sprinkled

- Evidence:
  - `logging/audit.rs` (envmeta) uses unsafe `libc::*`.
  - `fs/meta.rs` parses `/proc/self/status` for euid; `fs/mount.rs` parses `/proc/self/mounts`.
  - EXDEV mapping in `apply/handlers.rs` uses `raw_os_error()==libc::EXDEV`.
  - Multiple `from_raw_os_error(e.raw_os_error())` in fs modules.
- Consequence: portability risks, unsafe blocks, harder error mapping, more verbose glue code.

7) Error mapping is repeated across modules

- Evidence: `apply::{handlers,lock,policy_gate,mod,rollback}`, `api/mod.rs::prune_backups`.
- Consequence: boilerplate proliferation and potential inconsistencies.

8) Weak typing of repeated constants

- Evidence: string kinds (`"symlink"`, `"restore_from_backup"`, etc.) passed around.
- Consequence: duplication and room for subtle divergence.

## Deeper architectural opportunity (better than just helpers)

- __StageLogger enhancements__: add fluent helpers so facts read as intent, not JSON wiring.
- __ActionExecutor__: per‑action executors encapsulate logic + telemetry; `apply::run` becomes a thin dispatcher.
- __LockOrchestrator__: single facade to compute wait metrics, emit parity attempt/result/summary, and return early report.
- __ApplySummary builder__: centralized final summary fields (perf, error mapping, `summary_error_ids`, optional attestation).
- __Preflight RowEmitter + Kind enum + PreflightRowArgs__: typed, compact API for rows and facts, preventing drift.
- __RestorePlanner (plan→execute)__: decouple selection/integrity/idempotence planning from execution; test planning in isolation.
- __rustix migration__: replace libc/`/proc` for euid/ppid/statfs and EXDEV; keep public surfaces/stable fields.
- __Error mapping shim__: tiny `api/errors/map.rs` to normalize io→ErrorId mapping used by executors and summary.

## Impact if we do nothing

- Persistent clippy denials, increasing code size and complexity.
- Diverging telemetry fields across modules, weakening audit guarantees.
- Harder testing of planning logic (restore/preflight), more fragile refactors later.

## Risks & mitigations

- __Telemetry shape changes__: Mitigate by adding helpers first, then refactors; keep wrappers temporarily and assert byte‑for‑byte equality in tests.
- __Churn across hotspots__: Sequence changes (helpers → executors/lock → summary → preflight → restore → rustix) to limit rebasing.
- __MSRV & EXDEV__: Guard `ErrorKind::CrossesDevices` with a fallback to `raw_os_error()==libc::EXDEV` behind a compatibility shim.

## Acceptance criteria

- Long functions under thresholds; argument count reduced for preflight rows.
- Emitted facts (per‑action and summary) are byte‑for‑byte identical for golden scenarios.
- No unsafe in envmeta; no `/proc` parsing for euid; mount flags via `statfs`.
- Tests pass: unit (planner, lock attempts, error mapping), integration (apply flows, preflight rows), locking parity tests still green.

## Sequencing (minimal churn)

1. StageLogger fluent helpers.
2. ActionExecutor + dispatch in `apply::run` (thin `handlers::*` adapters retained temporarily).
3. LockOrchestrator wired into `apply::run`.
4. ApplySummary builder for final summary.
5. Preflight Kind/RowEmitter/PreflightRowArgs.
6. RestorePlanner with executor integration.
7. rustix migration (envmeta/euid/statfs/EXDEV shim); optional fd-lock behind adapter.
8. Optional error mapping shim adoption.

## Files most affected (evidence)

- `src/api/apply/{mod.rs,handlers.rs,lock.rs,policy_gate.rs,rollback.rs}`
- `src/api/preflight/{mod.rs,rows.rs}`
- `src/api/{plan.rs,mod.rs::prune_backups}`
- `src/fs/{meta.rs,mount.rs,restore/engine.rs,restore/steps.rs,backup/snapshot.rs,atomic.rs,swap.rs}`
- `src/logging/audit.rs`
- `src/adapters/lock/file.rs`

---
This RCA motivates the cohesive abstractions in `TODO-ULTIMATE.md`, showing they address fundamental causes (duplication and entanglement) rather than superficially splitting functions.
