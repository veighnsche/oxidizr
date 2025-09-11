# API Module Responsibilities & Split Plan (Planning Only)

Goal: Split the monolithic `src/api.rs` into a dedicated `src/api/` module to improve cohesion, readability, and testability.

References

- SPEC: `SPEC/SPEC.md` §§ 2.2, 2.3, 2.4, 2.7, 2.10, 13
- PLAN: `35-determinism.md`, `40-facts-logging.md`, `45-preflight.md`, `50-locking-concurrency.md`, `60-rollback-exdev.md`

Scope & Responsibilities

- API orchestrates the high-level workflows exposed to library consumers:
  - Plan: transform `PlanInput` -> `Plan` with stable ordering and deterministic IDs.
  - Preflight: environment checks, ownership/policy gating, structured preflight diff rows.
  - Apply: bounded-lock acquisition, per-action execution, facts emission, hashing, provenance, rollback on failure, optional smoke tests, and attestation on success.
  - Rollback Planning: derive inverse plan where safe (e.g., for symlink replacements) from an `ApplyReport`.
- API owns emission of structured facts (per stage) per `audit_event.schema.json` via the configured `FactsEmitter`.
- API enforces determinism policies (stable ordering, UUIDv5 strategy, redaction rules for DryRun).
- API integrates adapter traits: `LockManager`, `OwnershipOracle`, `Attestor`, `SmokeTestRunner`.

Non-Goals

- API does not implement filesystem primitives (lives in `fs/`).
- API does not implement concrete adapters (lives in `adapters/`), only consumes their traits.
- API does not own policy definitions (lives in `policy/`).

Proposed File Layout (incremental)

- `src/api/mod.rs` — surface re-exports, struct definitions, and builders.
- `src/api/plan.rs` — input normalization, action construction, stable ordering, UUIDv5.
- `src/api/preflight.rs` — preflight gating, ownership/provenance probes, structured diff rows.
- `src/api/apply.rs` — execution loop, lock acquisition, facts emission, hashing, rollback.
- `src/api/report.rs` — report types and conversions (optional, or keep in `types/report.rs`).

Migration Steps

1) Extract `plan()` into `api/plan.rs` and re-export from `api/mod.rs`.
2) Extract `preflight()` into `api/preflight.rs` (wrap existing `preflight.rs` helpers).
3) Extract `apply()` and `plan_rollback_of()` into `api/apply.rs`.
4) Keep `Switchyard` struct and builders in `api/mod.rs` importing the above modules.
5) Update all internal imports accordingly and add unit tests per module.

Acceptance & Review

- No behavior changes; strictly a refactor. All tests remain green.
- Facts emission remains identical (validated by golden fixtures when available).
- Update this PLAN entry and `00-structure.md` once merged.

---

## REVIEW

This is a solid, incremental split. It keeps the public surface where users expect it (`api/mod.rs`) and cleanly peels off the phase logic. A few tweaks will make it sturdier and easier to audit.

## What’s great

- Clear responsibilities: plan / preflight / apply are separated.
- Adapters consumed at the API layer only (good boundary).
- Determinism explicitly owned by API (ordering + UUIDv5).
- Migration plan is mechanical (low-risk).

## Gaps / risks to address

1. **Telemetry drift risk**
   You’ll be duplicating `facts.emit(...)` field shapes across `plan.rs`, `preflight.rs`, and `apply.rs`. That invites schema skew over time.

   - **Fix:** add `api/telemetry.rs` with helpers like `emit_plan_fact(...)`, `emit_preflight_fact(...)`, `emit_apply_attempt(...)`, `emit_apply_result(...)`, `emit_summary(...)`, and a single `const SCHEMA_VERSION: i64 = 1`. All JSON key names live here.

2. **Rollback concerns hidden inside `apply.rs`**
   Planning and executing rollbacks are conceptually distinct. Bundling with `apply` makes reasoning and testing harder.

   - **Fix:** add `api/rollback.rs` for:

     - `plan_inverse(&ApplyReport) -> Plan`
     - `execute_inverse(...) -> RollbackReport`
     - Document “RestoreFromBackup has no inverse” here once, tested here.

3. **Determinism seed placement**
   You mention UUIDv5 but not where the **namespace/seed** is defined (must be stable across processes, versions, and OSes).

   - **Fix:** define `const PLAN_UUID_NS: Uuid` in `api/plan.rs` (or `api/constants.rs`) and unit-test that given the same normalized `Plan`, the ids are identical across runs.

4. **Hashing & provenance helpers**
   The apply path does hashing and symlink resolution today. If that code stays inline, tests will be brittle.

   - **Fix:** add a tiny `api/fs_meta.rs` (or reuse existing module) with:

     - `resolve_symlink_target(&Path) -> Option<PathBuf>`
     - `sha256_hex_of(&Path) -> Option<String>`
     - `kind_of(&Path) -> Kind` (enum).
       Keep them side-effect free (no emits).

5. **Error taxonomy**
   Plan mentions reports, but not where error types live. If you keep stringly-typed errors, mapping to exit codes later gets messy.

   - **Fix:** centralize error enums in `errors.rs` (or `types/errors.rs`) and have `apply/preflight` return typed errors, with one mapping table → facts fields / exit codes.

6. **Redaction + timestamp rules**
   API “owns redaction rules for DryRun” and timestamp strategy (`TS_ZERO` vs `ts_for_mode`)—but that policy should be enforced centrally.

   - **Fix:** in `telemetry.rs`, accept a `TelemetryMode { dry_run: bool, redact: bool }` and centralize `ts_for_mode`, plus a `redact(Value) -> Value` hook.

7. **Test coverage shape**
   Plan says “add unit tests,” but you need **property tests** for determinism and **golden tests** for facts.

   - **Fix:**

     - Property tests (proptest/quickcheck): same `PlanInput` → same ordered `Plan` and IDs.
     - Golden facts fixtures: emit → compare to JSON files under `tests/golden/` (schema + field ordering).

## Slightly adjusted layout

```text
src/api/
  mod.rs            # façade: Switchyard, builders, re-exports
  plan.rs
  preflight.rs
  apply.rs
  rollback.rs       # NEW (pulls plan_rollback_of + execution)
  telemetry.rs      # NEW (all facts emission + schema_version)
  fs_meta.rs        # NEW (hash/symlink/kind helpers)
  report.rs         # optional (or keep in types/)
```

## Minimal signatures (to keep everyone honest)

```rust
// api/mod.rs
pub struct Switchyard<E: FactsEmitter, A: AuditSink> { /* fields */ }

impl<E: FactsEmitter, A: AuditSink> Switchyard<E, A> {
    pub fn plan(&self, input: PlanInput) -> Plan { plan::build(self, input) }
    pub fn preflight(&self, plan: &Plan) -> PreflightReport { preflight::run(self, plan) }
    pub fn apply(&self, plan: &Plan, mode: ApplyMode) -> ApplyReport { apply::run(self, plan, mode) }
    pub fn plan_rollback_of(&self, r: &ApplyReport) -> Plan { rollback::plan_inverse(r) }
}
```

```rust
// api/telemetry.rs
pub const SCHEMA_VERSION: i64 = 1;
pub struct TelemetryCtx<'a> { pub facts: &'a dyn FactsEmitter, pub plan_id: String, pub ts: i64 }

pub fn emit_plan_fact(ctx: &TelemetryCtx, action_id: &str, path: Option<&str>) { /* ... */ }
pub fn emit_preflight_fact(ctx: &TelemetryCtx, action_id: &str, path: Option<&str>, current: &str, planned: &str) { /* ... */ }
pub fn emit_apply_attempt(ctx: &TelemetryCtx, extra: serde_json::Value) { /* ... */ }
pub fn emit_apply_result(ctx: &TelemetryCtx, decision: &str, extra: serde_json::Value) { /* ... */ }
pub fn emit_summary(ctx: &TelemetryCtx, stage: &str, decision: &str) { /* ... */ }
```

## Migration checklist (practical)

- Extract helpers → `fs_meta.rs`; replace inline calls.
- Introduce `telemetry.rs`; replace all direct `facts.emit` with helpers (field names unchanged).
- Move bodies of `plan`, `preflight`, `apply` to their files; keep `Switchyard` delegators in `mod.rs`.
- Create `rollback.rs`; move `plan_rollback_of` and the reverse-execution block from `apply`.
- Add **property tests** for determinism; add **golden facts** test.
- Run `cargo fmt` + `clippy -D warnings` and ensure no behavior change.

## Verdict

Your plan is 90% of the way there. Add **telemetry centralization** and **rollback separation**, and you’ll have a refactor that’s easy to review, safer against schema drift, and friendlier to future audits—without touching behavior.
