# API DX/DW Overhaul — Actionable Steps (breaking)

Redesign the top-level API for developer experience (DX) and developer workflow (DW). Keep it consistent, typed, and stage-agnostic. Prefer clarity over compatibility.

## 1) Make `src/api` an idiomatic module and central entrypoint

- Move: `src/api.rs` → `src/api/mod.rs`.
- In `src/api/mod.rs` declare submodules only via `mod ...;` (no `#[path]`):
  - `mod apply;`
  - `pub mod errors;`
  - `mod plan;`
  - `mod preflight;`
  - `mod rollback;`
- Update all imports accordingly.
- Acceptance: `cargo check && cargo test` pass; `grep -R "#\[path\]" cargo/switchyard/src` returns 0.

/// remove this file after migration: `src/api.rs`

## 2) Introduce an `ApiBuilder` for ergonomic construction

- File: `src/api/mod.rs`
  - Add `pub struct ApiBuilder<E: FactsEmitter, A: AuditSink>` with setters mirroring the existing `Switchyard` fluent methods:
    - `with_lock_manager(Box<dyn LockManager>)`
    - `with_ownership_oracle(Box<dyn OwnershipOracle>)`
    - `with_attestor(Box<dyn Attestor>)`
    - `with_smoke_runner(Box<dyn SmokeTestRunner>)`
    - `with_lock_timeout_ms(u64)`
  - Add `impl ApiBuilder { pub fn build(self, facts: E, audit: A, policy: Policy) -> Switchyard<E, A> }`.
  - Coexist with current API: keep `Switchyard::new(facts, audit, policy)` and the existing fluent `.with_*` methods intact. Implement `Switchyard::new` via builder defaults to reduce duplication.
- Acceptance: Existing callers using `.with_*` keep working; new builder is available and covered by docs.

## 3) StageLogger integration (centralized audit)

- Build one `StageLogger` per stage orchestrator:
  - `plan::run()` (or `build()`), `preflight::run()`, `apply::run()`.
- Replace all direct emissions with the logging facade (see `zrefactor/logging_audit_refactor.INSTRUCTIONS.md`).
- Acceptance: `rg -n "FactsEmitter::emit\(" cargo/switchyard/src/api -S` returns 0; only logging facade is used.

## 4) Policy-owned gating usage everywhere

- Preflight: for each `Action`, call `policy::gating::evaluate_action(&self.policy, self.owner.as_deref(), action)`.
- Apply: call the same evaluator before mutating; if `policy.apply.override_preflight == false` and evaluator returns `stops`, abort.
- Remove any gating logic from API files.
- Acceptance: `rg -n "evaluate_action\(" cargo/switchyard/src/api -S` shows calls only; no duplicated gating helpers in API.

## 5) Tighten API input/output types

- Inputs:
  - `plan(input: PlanInput) -> Plan` remains, but validate `SafePath` invariants at plan time.
- Outputs:
  - Ensure consistent signatures:
    - `preflight(&self, plan: &Plan) -> Result<PreflightReport, errors::ApiError>`
    - `apply(&self, plan: &Plan, mode: ApplyMode) -> Result<ApplyReport, errors::ApiError>`
  - `ApiError` should map to the documented taxonomy (align with `ERROR_TAXONOMY.md`).
- Acceptance: Rustdoc for `ApiError` lists variants and mapping to exit codes (if applicable).

## 6) Locking and smoke test workflow (DW)

- Locking:
  - If `self.policy.governance.locking == Required`, acquire lock up-front with bounded wait (`lock_timeout_ms`).
  - On timeout, return `ApiError::Locking(E_LOCKING)` and emit `lock_wait_ms` fact.
- Smoke tests:
  - If `self.policy.governance.smoke == Require{..}`, run `SmokeTestRunner` after a successful commit; include results in `ApplyReport` and emitted summary.
- Acceptance: Integration test covers lock timeout path and smoke success/failure, with facts emitted.

## 7) Determinism and IDs (DW)

- Use deterministic IDs per SPEC (e.g., UUIDv5) for `plan_id`/`action_id` in the plan builder.
- Ensure dry-run timestamps are zeroed in emitted facts.
- Acceptance: golden tests confirm deterministic IDs and timestamp behavior.

## 8) Error taxonomy and surfaces

- Consolidate `src/api/errors.rs` into `src/api/errors/mod.rs` (directory module) with clear, typed variants.
- Map lower-level errors (FS, locking, integrity) to `ApiError` consistently.
- Acceptance: `ApiError` variants have `From` impls or explicit mappings; tests cover representative conversions.

/// remove this file after migration: `src/api/errors.rs` (rename to directory module)

## 9) Rollback ergonomics

- Keep `plan_rollback_of(&ApplyReport) -> Plan`, but ensure it reads all necessary context (e.g., backup tags) from the report.
- Emit rollback planning facts via logging facade.
- Acceptance: happy-path and failure-path tests for rollback planning and subsequent apply.

## 10) CI guardrails

- Add grep checks:
  - No `#[path]` under `src/api/`.
  - No `FactsEmitter::emit(` usage under `src/api/`.
  - No ad-hoc gating logic in `src/api/**` (must use `policy::gating::evaluate_action`).
- Acceptance: CI fails if these are violated.

## 11) PR plan (breaking)

- PR1: Module reshaping — move `src/api.rs` → `src/api/mod.rs`; fix imports; no behavior changes.
- PR2: Introduce `ApiBuilder` and migrate constructors; add `StageLogger` wiring in orchestrators.
- PR3: Replace emissions with logging facade and integrate policy gating calls in preflight/apply.
- PR4: Tighten `ApiError` surface; migrate to `errors/mod.rs`; add mappings and docs.
- PR5: Locking + smoke workflow; deterministic IDs; tests and golden fixtures.
- PR6: CI guardrails; remove legacy usages.

## 12) Cleanups

- /// remove this file: `src/api/telemetry.rs` (if present; superseded by logging facade)
- /// remove shims or legacy re-exports in API after migration.

---

## Meta

- Scope: API structure, constructors, error surface, and stage orchestration wiring
- Status: Breaking allowed (pre-1.0)
- Index: See `zrefactor/README.md`

## Related

- Logging facade and event building: `zrefactor/logging_audit_refactor.INSTRUCTIONS.md`
- Policy groups and evaluator: `zrefactor/policy_refactor.INSTRUCTIONS.md`
- Preflight orchestrator using evaluator: `zrefactor/preflight_gating_refactor.INSTRUCTIONS.md`
- FS backup/restore split (call sites): `zrefactor/fs_refactor_backup_restore.INSTRUCTIONS.md`
- Cohesion targets and guardrail greps: `zrefactor/responsibility_cohesion_report.md`
- Removal plan for shims: `zrefactor/backwards_compat_removals.md`, `zrefactor/removals_registry.md`
