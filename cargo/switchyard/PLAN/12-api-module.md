# API Module Responsibilities & Split Plan

**Goal**
Split the monolithic `src/api.rs` into a dedicated `src/api/` module to improve cohesion, readability, and testability.

**References**

* SPEC: `SPEC/SPEC.md` §§ 2.2, 2.3, 2.4, 2.7, 2.10, 13
* PLAN: `35-determinism.md`, `40-facts-logging.md`, `45-preflight.md`, `50-locking-concurrency.md`, `60-rollback-exdev.md`

---

## Scope & Responsibilities

The **API layer** orchestrates high-level workflows exposed to library consumers:

* **Plan**: transform `PlanInput` → `Plan` with stable ordering and deterministic IDs.
* **Preflight**: environment checks, ownership/policy gating, structured preflight diff rows.
* **Apply**: bounded-lock acquisition, per-action execution, facts emission, hashing, provenance, rollback on failure, optional smoke tests, and attestation.
* **Rollback planning**: derive inverse plans where safe (symlinks etc.) from an `ApplyReport`.
* **Telemetry**: centralize facts emission (per schema v1) and apply redaction/timestamp policies.
* **Error taxonomy**: unify typed errors and mapping to exit codes + facts fields.

**Non-Goals**

* No filesystem primitives (lives in `fs/`).
* No concrete adapters (live in `adapters/`), only traits are consumed.
* No policy definitions (live in `policy/`).

---

## Consolidation with Current Code (Reality Check)

* Determinism/redaction already in `logging/redact.rs` (`TS_ZERO`, `ts_for_mode`, `redact_event`). API will reuse, not re-implement.
* UUIDv5 utilities live in `types/ids.rs` → reused for `plan_id`/`action_id`.
* Inline helpers (`resolve_symlink_target`, `sha256_hex_of`, `kind_of`) → move to `api/fs_meta.rs`.
* Locking traits already in `adapters/lock.rs` with impl in `adapters/lock_file.rs` → API just consumes, adds bounded-wait + `lock_wait_ms`.
* Unsafe forbidden crate-wide (`#![forbid(unsafe_code)]`). No libc calls. UID/GID provenance will be routed via adapters later.

---

## Proposed File Layout

```text
src/api/
  mod.rs         # façade: Switchyard, delegators, re-exports
  plan.rs        # plan() extraction (ordering, UUIDv5)
  preflight.rs   # preflight() extraction (ownership/policy/probes)
  apply.rs       # apply() extraction (locking, facts, rollback trigger, attestation)
  rollback.rs    # plan_rollback_of() + inverse execution docs/tests
  telemetry.rs   # centralized facts emission (SCHEMA_VERSION=1, redaction, ts_for_mode)
  fs_meta.rs     # tiny helpers: resolve_symlink_target, sha256_hex_of, kind_of
  errors.rs      # typed errors + mapping table → facts/exit codes
  # report.rs    # optional; or keep in types/report.rs
```

---

## Telemetry Consolidation

* **`api/telemetry.rs`** owns all facts emission.
* Single `SCHEMA_VERSION = 1`.
* Uses `logging::redact::ts_for_mode()` and `redact_event()`.
* Enforces `TelemetryMode { dry_run: bool, redact: bool }`.
* Helpers: `emit_plan_fact`, `emit_preflight_fact`, `emit_apply_attempt`, `emit_apply_result`, `emit_summary`.
* Field names identical to current facts; no new fields.

---

## Rollback Separation

* Move `plan_rollback_of()` → `api/rollback.rs`.
* Document non-invertible actions (e.g. `RestoreFromBackup`).
* Keep rollback execution code in `apply.rs`, but cross-reference from `rollback.rs`.
* Add unit tests for inverse plan derivation in `rollback.rs`.

---

## Error Taxonomy

* Introduce `api/errors.rs` (or `types/errors.rs`):

  ```rust
  pub enum ApiError {
      PolicyViolation,
      LockingTimeout,
      FilesystemError,
      ExdevDegraded,
      SmokeFailed,
      // ...
  }
  ```

* Map these errors → exit codes + stable identifiers (facts schema).
* Apply/preflight/rollback return typed errors; API translates to facts + exit codes consistently.

---

## Migration Steps

1. Create stubs: `plan.rs`, `preflight.rs`, `apply.rs`, `rollback.rs`, `telemetry.rs`, `fs_meta.rs`, `errors.rs`.
2. Extract helpers into `fs_meta.rs`; update `api.rs` to call them.
3. Add `telemetry.rs`; replace inline `facts.emit(...)` with helpers (identical fields).
4. Split `plan()`, then `preflight()`, then `apply()` into separate files; add `api/mod.rs`. Switch `lib.rs` to `mod api` when green twice in CI.
5. Move `plan_rollback_of()` into `rollback.rs`.
6. Introduce typed `ApiError`; refactor apply/preflight to return them.
7. Update golden fixtures and facts tests to ensure schema unchanged.

---

## Acceptance & Review

* **No behavior change**: strictly refactor. All tests green.
* **Facts emission**: identical JSON fields + schema.
* **Unsafe**: crate still passes `#![forbid(unsafe_code)]`.
* **Determinism**: property tests for stable ordering + UUIDv5 determinism.
* **Schema stability**: golden fixtures prevent drift.
* **Error taxonomy**: one mapping table, no stringly errors.

---

## Minimal Delegator Signatures

```rust
// api/mod.rs
pub struct Switchyard<A: Adapters, F: FactsEmitter> {
    adapters: A,
    facts: F,
}

impl<A: Adapters, F: FactsEmitter> Switchyard<A, F> {
    pub fn plan(&self, input: PlanInput) -> Plan { plan::build(self, input) }
    pub fn preflight(&self, plan: &Plan) -> Result<PreflightReport, ApiError> { preflight::run(self, plan) }
    pub fn apply(&self, plan: &Plan, mode: ApplyMode) -> Result<ApplyReport, ApiError> { apply::run(self, plan, mode) }
    pub fn plan_rollback_of(&self, r: &ApplyReport) -> Plan { rollback::inverse(r) }
}
```

```rust
// api/telemetry.rs
pub const SCHEMA_VERSION: i64 = 1;

pub struct TelemetryCtx<'a> {
    pub facts: &'a dyn FactsEmitter,
    pub plan_id: String,
    pub ts: i64,
    pub mode: TelemetryMode,
}

pub fn emit_plan_fact(ctx: &TelemetryCtx, action_id: &str, path: Option<&str>) { /* ... */ }
pub fn emit_preflight_fact(ctx: &TelemetryCtx, action_id: &str, path: Option<&str>, current: &str, planned: &str) { /* ... */ }
pub fn emit_apply_attempt(ctx: &TelemetryCtx, extra: serde_json::Value) { /* ... */ }
pub fn emit_apply_result(ctx: &TelemetryCtx, decision: &str, extra: serde_json::Value) { /* ... */ }
pub fn emit_summary(ctx: &TelemetryCtx, stage: &str, decision: &str) { /* ... */ }
```

---

## Test Strategy

* **Unit tests**: per module (`plan`, `preflight`, `apply`, `rollback`, `fs_meta`).
* **Property tests**: determinism (same input → same plan/IDs).
* **Golden tests**: facts JSON, compared byte-for-byte against fixtures.
* **Integration tests**: full apply cycle with mock adapters.

## UPDATE #2025-09-11T14:32:33+02:00

Author: Cascade (AI)

Summary of work implemented in code to align with this plan and SPEC terminology:

* Renamed planned "telemetry" surface to `audit` to match SPEC (§2.4, §5) and the project's existing `SPEC/audit_event.schema.json`.
* Created `src/api/audit.rs` centralizing Minimal Facts v1 emission:
  * Helpers: `emit_plan_fact`, `emit_preflight_fact`, `emit_apply_attempt`, `emit_apply_result`, `emit_summary`, and `emit_rollback_step`.
  * Uses `schema_version=1`, `logging::redact::TS_ZERO` and `ts_for_mode()` for timestamp policy.
* Split monolithic `src/api.rs` into modules and delegated calls:
  * `src/api/plan.rs` (build plan, per-action plan facts)
  * `src/api/preflight.rs` (checks + per-action and summary preflight facts)
  * `src/api/apply.rs` (locking, per-action attempt/result, rollback, attestation placeholder)
  * `src/api/rollback.rs` (inverse planning for rollback)
  * `src/api/fs_meta.rs` (helpers: `sha256_hex_of`, `resolve_symlink_target`, `kind_of`)
  * `src/api/errors.rs` (introduces `ApiError` scaffolding)
* Adjusted facade signatures to typed errors (Plan step 6):
  * `preflight(&self, plan: &Plan) -> Result<PreflightReport, ApiError>`
  * `apply(&self, plan: &Plan, mode: ApplyMode) -> Result<ApplyReport, ApiError>`
  * Tests updated to unwrap; behavior unchanged.
* Routed rollback facts through `emit_rollback_step` to keep all emissions centralized.
* Preserved Minimal Facts v1 fields exactly; hashing/provenance remain placeholders where already present.
* Added `PLAN/discrepancies/0001-terminology-and-api-surface.md` documenting:
  * Telemetry→Audit naming decision
  * `Result`-returning API vs SPEC §3.1 prior signatures
  * Pending gaps: exit-code mapping, structured preflight rows, expanded redaction, and emission helper usage for every path.

State:

* `cargo test -p switchyard` passes locally.
* No behavioral changes intended; refactor only.

Next steps proposed:

* Map `ApiError` → `SPEC/error_codes.toml` and include `exit_code` + stable identifiers (`E_POLICY`, `E_LOCKING`, …) in facts.
* Implement structured preflight diff rows per `SPEC/preflight.yaml` with byte-identical dry-run.
* Enforce central `redact_event()` in `api/audit.rs` for all emissions (mask secrets and volatile fields).
* Remove unused `src/api/telemetry.rs` file.

— Cascade
