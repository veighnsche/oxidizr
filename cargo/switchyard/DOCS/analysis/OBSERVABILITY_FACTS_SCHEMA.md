# Observability & Facts Schema Review

**Status:** Draft  
**Generated:** 2025-09-12  
**Scope:** Inventory facts emitted across stages (plan, preflight rows/summary, apply.attempt, apply.result, rollback), field requirements, redaction rules, determinism constraints, and versioning/compatibility policy.  
**Inputs reviewed:** SPEC §5 (Audit Facts schema v1), SPEC/audit_event.schema.json, SPEC §2.4 (Observability), PLAN/40-facts-logging.md, CODE: `src/logging/audit.rs`, `src/logging/facts.rs`, `src/logging/redact.rs`, `src/api/{plan,preflight,apply}/**`
**Affected modules:** `src/logging/**`, `src/api/**`

## Summary

- Minimal Facts v1 is consistently emitted via `logging/audit.rs` helpers with a stable envelope: `schema_version=1`, `ts`, `plan_id`, `path`, `stage`, `decision`, `dry_run`.
- Determinism is enforced: `dry_run` paths use `ts=TS_ZERO` and `redact_event(...)` masks volatile fields (`duration_ms`, `lock_wait_ms`, hashes, severity, degraded, attestation secrets).
- Apply emits per-action attempt/result and a final summary; preflight emits per-action rows with extended fields and a summary; rollback emits per-step entries.
- Forward/backward compatibility is achieved via `schema_version` and additive fields; masking policy protects secrets and volatile values.

## Inventory / Findings

- Emission sites
  - `plan`: `logging::audit::emit_plan_fact` — Fields: `action_id`, `path`, `stage="plan"`, `decision="success"`. Source: `src/api/plan.rs`.
  - `preflight` rows: `logging::audit::emit_preflight_fact_ext` — Fields: `action_id`, `path`, `current_kind`, `planned_kind`, optional `policy_ok`, `provenance`, `notes`, `preservation`, `preservation_supported`. Source: `src/api/preflight/rows.rs` (via `preflight/mod.rs`).
  - `preflight` summary: `logging::audit::emit_summary_extra("preflight", ...)` — Adds `rescue_profile` and on STOP sets `error_id=E_POLICY`, `exit_code=10`. Source: `src/api/preflight/mod.rs`.
  - `apply.attempt`: `logging::audit::emit_apply_attempt` — Top-level lock acquisition and per-action attempts. Fields may include `lock_wait_ms`, `error`, `error_id`, `exit_code`. Source: `src/api/apply/mod.rs`, `src/api/apply/handlers.rs`.
  - `apply.result`: `logging::audit::emit_apply_result` — Per-action results and final summary with optional `attestation`, `degraded`, `duration_ms`, `before_hash/after_hash/hash_alg`, `error_id`, `exit_code`. Source: `src/api/apply/{handlers,mod}.rs`.
  - `rollback`: `logging::audit::emit_rollback_step` — Path per reverted action and decision. Source: `src/api/apply/mod.rs`.

- Envelope and redaction
  - Envelope is enforced in `logging/audit.rs::redact_and_emit`: ensures `schema_version`, `ts`, `plan_id`, `path`, `dry_run` presence on every fact.
  - Redaction policy in `logging/redact.rs::redact_event`:
    - Zero `ts` to `TS_ZERO`.
    - Remove `duration_ms`, `lock_wait_ms`, `severity`, `degraded`, `before_hash`, `after_hash`, `hash_alg`.
    - Mask `provenance.helper` and `attestation.{signature,bundle_hash,public_key_id}` with `"***"`.
  - Timestamp selection via `ts_for_mode(mode)`: `DryRun` → `TS_ZERO`, `Commit` → RFC3339 now.

- Facts schema v1 (SPEC/audit_event.schema.json)
  - Required: `ts`, `plan_id`, `stage`, `decision`, `path`. `schema_version` is fixed to 1.
  - Enumerations: `stage` ∈ {`plan`, `preflight`, `apply.attempt`, `apply.result`, `rollback`}; `decision` ∈ {`success`, `failure`, `warn`}.
  - Optional: `action_id`, `degraded`, `current_kind`, `planned_kind`, `hash_alg`, `before_hash`, `after_hash`, `attestation{...}`, `provenance{...}`, `preservation{...}`, `preservation_supported`, `exit_code`, `duration_ms`, `lock_wait_ms`, `error_id`, `error_detail`.

- Determinism constraints
  - Plan and preflight rows/summary use `TS_ZERO` and redaction by default.
  - Apply dry-run uses `TS_ZERO` and redaction; Commit uses real time but redaction is off by default (unless requested via mode).
  - `plan_id` and `action_id` are UUIDv5 derived from deterministic serialization (`src/types/ids.rs`).

- Provenance and attestation
  - `ensure_provenance` inserts `provenance.env_sanitized=true` and preserves adapter-provided `origin/helper/uid/gid/pkg` if present.
  - On successful commit, `apply.result` summary may include `attestation` block with `sig_alg`, `signature` (masked in redaction), `bundle_hash`, `public_key_id`.

## Recommendations

1. Add a unit test that validates every emitted fact (from plan/preflight/apply/rollback test harnesses) validates against `SPEC/audit_event.schema.json` using `jsonschema` crate.
2. Extend redaction to mask `notes` values that contain environment paths or command-line args per policy; add hooks for custom redaction.
3. Stabilize `FactsEmitter` contract with a Rustdoc stability note (Provisional) and define additive evolution policy. Consider introducing a versioned wrapper type around `serde_json::Value` for compile-time hints.
4. Consider adding `summary_error_ids: [string]` to apply/preflight summaries to avoid collapsing multiple causes into `E_POLICY` only (see ERROR_TAXONOMY.md).
5. Ensure `before_hash/after_hash` presence is covered by tests for mutated paths in Commit mode (REQ-O5), while redacted view remains deterministic.

## Risks & Trade-offs

- Increased schema strictness may require fixture updates; mitigate via dual-emit period when bumping `schema_version`.
- Over-redaction may hide useful diagnostics; mitigate by preserving raw sinks out-of-band while keeping public/minimal facts redacted.

## Spec/Docs deltas

- SPEC §5: Add `summary_error_ids` optional array to summaries; clarify redaction of `attestation.*` and `provenance.helper` as mandatory.
- PLAN/40-facts-logging.md: Document new validation test and redaction hooks.

## Acceptance Criteria

- Facts emitted in tests validate against JSON Schema v1.
- Redaction unit test covers helper and attestation masking (already present) and extended notes masking.
- Apply/preflight summaries optionally include `summary_error_ids` with backward-compatible behavior.

## References

- SPEC: §2.4 Observability; §5 Audit Facts; `SPEC/audit_event.schema.json`
- PLAN: 40-facts-logging.md; 35-determinism.md
- CODE: `src/logging/audit.rs`, `src/logging/redact.rs`, `src/api/{plan,preflight,apply}/**`
