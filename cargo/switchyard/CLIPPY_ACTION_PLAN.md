# Switchyard Clippy Action Plan

Generated from `cargo clippy -p switchyard` on 2025-09-13 20:48:53 +02:00

## How to regenerate

```bash
cargo clippy -p switchyard
```

Note: `clippy::too_many_lines` is denied at the crate level in `cargo/switchyard/src/lib.rs`, so these are hard errors until fixed or allowed at the item level.

## Summary of current lints

Errors (deny):

- `too_many_lines` (116/100) in `api/apply/handlers.rs::handle_ensure_symlink`
- `too_many_lines` (192/100) in `api/apply/handlers.rs::handle_restore`
- `too_many_lines` (107/100) in `api/apply/lock.rs::acquire`
- `too_many_lines` (182/100) in `api/apply/mod.rs::run`
- `too_many_lines` (177/100) in `api/preflight/mod.rs::run`
- `too_many_lines` (107/100) in `fs/restore/engine.rs::restore_impl`
- `too_many_lines` (175/100) in `policy/gating.rs::evaluate_action`

Warnings:

- `too_many_arguments` (14/7) in `api/preflight/rows.rs::push_row_emit`

## Proposed plan per item

1) `api/apply/handlers.rs::handle_ensure_symlink` (116/100)

- Refactor helpers to introduce in `api/apply/` (new module `ops.rs` or extend `audit_fields.rs`):
  - `fn build_apply_attempt_fields(aid: &Uuid, target: &SafePath, api: &Switchyard<_, _>) -> serde_json::Value`
  - `fn compute_symlink_hashes(source: &SafePath, target: &SafePath) -> (String, String, u64)`
  - `fn map_swap_error(e: &std::io::Error) -> ErrorId` (maps EXDEV vs generic swap errors)
  - `fn build_apply_result_fields(...) -> serde_json::Value` (base fields + provenance, hashes, fsync warning)
- Code changes (steps):
  1. Add the above helpers; keep them `pub(super)` within `apply` for locality.
  2. Replace inline JSON assembly and hash timing with calls to helpers.
  3. Keep `replace_file_with_symlink()` invocation unchanged; only extract error mapping and telemetry.
- Acceptance criteria:
  - Function length < 100 lines.
  - No change to emitted audit fields (manually compare before/after or snapshot test).
  - All unit/integration tests pass.
- Risks and mitigations:
  - Risk: telemetry shape drift. Mitigate with snapshot of emitted JSON fields in an ad-hoc test or by diffing logs locally.
- Interim unblocker: add `#[allow(clippy::too_many_lines, reason = "orchestrator; split into helpers")]` if we need CI green before the refactor lands.

2) `api/apply/handlers.rs::handle_restore` (192/100)

- Refactor helpers (same module):
  - `fn pre_restore_snapshot_if_enabled(target: &SafePath, api: &Switchyard<_, _>, dry: bool) -> (used_prev: bool, backup_ms: u64)`
  - `fn compute_integrity_verified(target: &SafePath, used_prev: bool, tag: &str) -> Option<bool>`
  - `fn try_restore(target: &SafePath, used_prev: bool, dry: bool, force: bool, tag: &str) -> std::io::Result<()>` (encapsulates previous→latest fallback)
  - `fn emit_restore_success(...)` and `fn emit_restore_failure(...)` (centralize JSON building and emission)
- Code changes (steps):
  1. Move snapshot timing and integrity check into helpers.
  2. Implement `try_restore()` with the same fallback semantics currently inline.
  3. Reuse success/failure emitters to minimize JSON assembly in the main function.
- Acceptance criteria:
  - Function length < 100 lines.
  - Fallback semantics preserved (prev NotFound → latest).
  - Integrity bit emission identical when available.
  - Tests pass; add a unit test for fallback path if feasible.
- Risks:
  - Subtle change in error_id/exit_code mapping—keep mapping logic inside emitters to avoid drift.
- Interim unblocker: temporary `#[allow(clippy::too_many_lines)]` with reason.

3) `api/apply/lock.rs::acquire` (107/100)

- Refactor helpers (new private fns in `lock.rs`):
  - `fn policy_requires_lock(mode: ApplyMode, policy: &Policy) -> bool`
  - `fn emit_lock_failure_attempt_and_result(tctx: &AuditCtx, lock_backend: &str, wait_ms: Option<u64>, attempts: u64)`
  - `fn early_apply_report(pid: Uuid, t0: Instant, msg: String) -> ApplyReport`
- Steps:
  1. Extract policy decision and emission paths.
  2. Construct `LockInfo` using a small builder-like helper if needed.
- Acceptance:
  - Function length < 100 lines.
  - Identical audit emission on failure paths (attempt + result + summary failure).
  - Behavior unchanged under all policies.
- Interim: temporary allow if needed.

4) `api/apply/mod.rs::run` (182/100)

- Refactor helpers (same module or `handlers.rs`/`rollback.rs`):
  - `fn acquire_lock_and_emit_attempt(...) -> (LockInfo, Option<ApplyReport>)`
  - `fn enforce_policy_gate_or_early_return(...) -> Option<ApplyReport>`
  - `fn execute_actions_loop(...) -> (Vec<Action>, Vec<String>, PerfAgg)`
  - `fn post_apply_smoke_and_maybe_rollback(...) -> (bool /*rolled_back*/, Vec<String> /*rb_errors*/)`
  - `fn emit_summary_result(...)`
- Steps:
  1. Extract the above phases preserving the current order and semantics.
  2. Keep attestation emission in `emit_summary_result()` with current safe unwrap logic.
- Acceptance:
  - Function length < 100 lines.
  - Summary decision and fields match before.
  - All tests pass.
- Interim: temporary allow.

5) `api/preflight/mod.rs::run` (177/100)

- Refactor helpers (same module):
  - `fn rescue_profile_check(policy: &Policy) -> (bool /*rescue_ok*/, Option<&'static str> /*profile label*/)`
  - `fn emit_preflight_row_for_symlink(...)` and `fn emit_preflight_row_for_restore(...)`
  - `fn emit_preflight_summary(...)`
- Steps:
  1. Move per-action row construction to helpers to reduce branching in `run()`.
  2. Centralize summary emission.
- Acceptance:
  - Function length < 100 lines.
  - Stable row ordering preserved.
  - Facts’ shapes unchanged.
- Interim: temporary allow.

6) `fs/restore/engine.rs::restore_impl` (107/100)

- Refactor helpers (same module):
  - `fn select_backup_pair(target: &Path, sel: SnapshotSel, tag: &str) -> Option<(Option<PathBuf>, PathBuf)>`
  - `fn early_exit_if_idempotent(target: &Path, sc: &Sidecar) -> bool`
  - `fn restore_file_bytes_kind(target: &Path, backup: &Path, mode_oct: Option<u32>) -> std::io::Result<()>`
  - `fn restore_symlink_kind(target: &Path, dest: &Path, backup_opt: Option<&Path>) -> std::io::Result<()>`
  - `fn legacy_rename_or_best_effort(target: &Path, backup: Option<&Path>, force: bool) -> std::io::Result<()>`
- Steps:
  1. Split on `prior_kind` cases into dedicated helpers.
  2. Keep best-effort behavior identical; unit-test file vs symlink cases if possible.
- Acceptance:
  - Function length < 100 lines.
  - Behavior identical for dry-run, best-effort, and integrity checks.
- Interim: temporary allow.

7) `policy/gating.rs::evaluate_action` (175/100)

- Refactor helpers (same module):
  - `fn eval_ensure_symlink(policy: &Policy, owner: Option<&dyn DebugOwnershipOracle>, source: &SafePath, target: &SafePath) -> Evaluation`
  - `fn eval_restore_from_backup(policy: &Policy, owner: Option<&dyn DebugOwnershipOracle>, target: &SafePath) -> Evaluation`
  - Common checks as helpers returning `(Option<String> /*stop*/, Option<String> /*note*/)` or vectors:
    - `mount_rw_exec_check(p: &Path)`
    - `immutable_check(p: &Path)`
    - `hardlink_risk_check(policy: &Policy, p: &Path)`
    - `suid_sgid_risk_check(policy: &Policy, p: &Path)`
    - `source_trust_check(policy: &Policy, src: &Path)`
    - `scope_allow_forbid_check(policy: &Policy, target: &Path)`
- Steps:
  1. Route match arms to `eval_*` fns.
  2. DRY up repeated checks via common helpers.
- Acceptance:
  - Function length < 100 lines.
  - Same set of stops/notes for sample inputs (add quick tests around helpers).
- Interim: temporary allow.

8) `api/preflight/rows.rs::push_row_emit` — `too_many_arguments` (14/7)

- Interface refactor:
  - Introduce `PreflightRowArgs` builder struct:

    ```rust
    struct PreflightRowArgs {
      path: String,
      current_kind: String,
      planned_kind: String,
      policy_ok: Option<bool>,
      provenance: Option<Value>,
      notes: Option<Vec<String>>,
      preservation: Option<Value>,
      preservation_supported: Option<bool>,
      restore_ready: Option<bool>,
    }
    ```

  - Change signature to `push_row_emit(..., args: &PreflightRowArgs)` and update call sites in `preflight/mod.rs`.
- Acceptance:
  - Lint resolved; function args ≤ 7.
  - No change in emitted row/fields.
- Interim: add `#[allow(clippy::too_many_arguments)]` with reason until the interface change lands.

Rollout & sequencing

- PR-0: Optional — add targeted `#[allow(clippy::too_many_lines)]` with reasons to unblock CI.
- PR-1: handlers.rs (ensure_symlink) helpers + shrink function.
- PR-2: handlers.rs (restore) helpers + shrink function; add small unit test for fallback.
- PR-3: lock.rs (acquire) helpers.
- PR-4: apply/mod.rs (run) orchestration helpers.
- PR-5: preflight/mod.rs run split + rows builder types (may be PR-5a/5b).
- PR-6: fs/restore/engine.rs split per kind.
- PR-7: policy/gating.rs split + common checks.

Verification

- Add a lightweight “audit-shape” snapshot check locally: run representative plans and diff JSON fields before/after.
- Ensure `cargo clippy -p switchyard` returns no `too_many_lines` errors after each PR, and no new warnings are introduced.

## Prioritization

- P0: Unblock CI by temporarily allowing `too_many_lines` at specific functions listed above with explicit `reason` comments.
- P1: Refactor `api/apply/mod.rs::run` and `api/apply/handlers.rs::handle_restore` (largest wins, central orchestration).
- P2: Refactor `policy/gating.rs::evaluate_action` and `fs/restore/engine.rs::restore_impl`.
- P3: Preflight refactors and `push_row_emit` argument consolidation.

## Trace (clippy output)

```
error: this function has too many lines (116/100)
   --> cargo/switchyard/src/api/apply/handlers.rs:20:1

error: this function has too many lines (192/100)
   --> cargo/switchyard/src/api/apply/handlers.rs:159:1

error: this function has too many lines (107/100)
   --> cargo/switchyard/src/api/apply/lock.rs:29:1

error: this function has too many lines (182/100)
   --> cargo/switchyard/src/api/apply/mod.rs:35:1

warning: this function has too many arguments (14/7)
  --> cargo/switchyard/src/api/preflight/rows.rs:10:1

error: this function has too many lines (177/100)
   --> cargo/switchyard/src/api/preflight/mod.rs:22:1

error: this function has too many lines (107/100)
   --> cargo/switchyard/src/fs/restore/engine.rs:54:1

error: this function has too many lines (175/100)
   --> cargo/switchyard/src/policy/gating.rs:16:1
```
