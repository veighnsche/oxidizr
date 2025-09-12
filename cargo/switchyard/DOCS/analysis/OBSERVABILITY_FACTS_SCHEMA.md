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

## Round 1 Peer Review (AI 1, 2025-09-12 15:14 +02:00)

- Claims verified
  - Minimal envelope is enforced for all facts.
    - Proof: `src/logging/audit.rs::redact_and_emit()` inserts `schema_version`, `ts`, `plan_id`, `path`, and `dry_run` before emission (lines 51–58).
  - Emission sites per stage are implemented and wired correctly.
    - Proof: `plan` facts via `src/api/plan.rs::build()` calling `emit_plan_fact` (lines 59–65); `preflight` rows via `src/api/preflight/rows.rs::push_row_emit()` calling `emit_preflight_fact_ext`; `preflight` summary via `src/api/preflight/mod.rs::run()` calling `emit_summary_extra` (line 270); `apply.attempt` and `apply.result` via `src/api/apply/mod.rs` (multiple calls, e.g., lines 151–158, 174–183, 185–192, 409–411); `rollback` steps via `src/api/apply/mod.rs` (lines 244–261) calling `emit_rollback_step`.
  - Determinism and redaction behavior are present.
    - Proof: `src/logging/redact.rs::ts_for_mode()` returns `TS_ZERO` for `ApplyMode::DryRun` (lines 57–61); `src/logging/redact.rs::redact_event()` zeroes `ts` and removes `duration_ms`, `lock_wait_ms`, `severity`, `degraded`, and hash fields; masks `provenance.helper` and `attestation.*` (lines 67–101). Plan/action IDs are UUIDv5 via `src/types/ids.rs::{plan_id, action_id}`.
  - Facts schema v1 alignment.
    - Proof: `SPEC/audit_event.schema.json` defines required fields and enumerations; `src/logging/audit.rs` uses `SCHEMA_VERSION=1` and ensures required envelope.
  - Provenance and attestation handling.
    - Proof: `src/logging/audit.rs::ensure_provenance()` ensures `provenance.env_sanitized=true`; `src/api/apply/mod.rs` success path adds an attestation block with `sig_alg`, `signature`, `bundle_hash`, `public_key_id` (lines 359–384).

- Key citations
  - `src/logging/audit.rs::{redact_and_emit, ensure_provenance, emit_*}`
  - `src/logging/redact.rs::{ts_for_mode, redact_event}`
  - `src/api/plan.rs::build`, `src/api/preflight/{mod.rs,rows.rs}`, `src/api/apply/mod.rs`
  - `SPEC/audit_event.schema.json`

- Summary of edits
  - Added explicit code/spec citations proving envelope enforcement, emission coverage across stages, determinism/redaction rules, and attestation/provenance handling. Recommendations stand; consider adding `summary_error_ids` array as noted.

Reviewed and updated in Round 1 by AI 1 on 2025-09-12 15:14 +02:00

## Round 2 Gap Analysis (AI 4, 2025-09-12 15:38 CET)

- **Invariant: Observability facts provide comprehensive error information for debugging and recovery.**
  - **Assumption (from doc):** The document assumes that observability facts, especially in preflight and apply summaries, provide detailed error information to help consumers understand failures and take corrective actions (`OBSERVABILITY_FACTS_SCHEMA.md:19-24`, recommendation for `summary_error_ids` at line 54).
  - **Reality (evidence):** Current implementation in `src/api/preflight/mod.rs` and `src/api/apply/mod.rs` emits summaries with a single `error_id` field (e.g., `E_POLICY` with `exit_code=10` in preflight summary at `src/api/preflight/mod.rs:270`). There is no array or detailed breakdown of multiple error causes as suggested by the recommendation for `summary_error_ids` (`OBSERVABILITY_FACTS_SCHEMA.md:54`).
  - **Gap:** When multiple error conditions contribute to a failure, the current facts schema collapses them into a single `error_id`, often a generic `E_POLICY`. This limits the ability of CLI consumers to pinpoint specific issues, violating the expectation of actionable observability for debugging and recovery.
  - **Mitigations:** Implement the recommended `summary_error_ids` array in preflight and apply summaries to list all contributing error IDs (`src/api/preflight/mod.rs`, `src/api/apply/mod.rs`). Update `SPEC/audit_event.schema.json` to include this optional field for backward compatibility. Ensure error details are logged in per-action facts for granular debugging.
  - **Impacted users:** CLI integrators and end-users who rely on observability data to diagnose and resolve complex failures, especially during preflight or apply stages.
  - **Follow-ups:** Flag this as a medium-severity usability gap for Round 3. Plan to implement `summary_error_ids` in Round 4 to enhance debugging capabilities.

- **Invariant: Observability facts are validated against a schema to ensure consistency and reliability.**
  - **Assumption (from doc):** The document assumes that all emitted facts are validated against the defined JSON schema (`SPEC/audit_event.schema.json`) to ensure they conform to the expected structure and content (`OBSERVABILITY_FACTS_SCHEMA.md:50`, acceptance criteria at line 68).
  - **Reality (evidence):** While the schema is defined in `SPEC/audit_event.schema.json` and emission helpers in `src/logging/audit.rs` enforce a minimal envelope, there is no evidence of automated validation or unit tests in the codebase that check emitted facts against the schema during build or test phases. The recommendation for a unit test using `jsonschema` crate (`OBSERVABILITY_FACTS_SCHEMA.md:50`) is not yet implemented.
  - **Gap:** Without automated validation, there is a risk that emitted facts may deviate from the schema over time due to code changes, violating the consumer expectation of consistent and reliable observability data for parsing and analysis.
  - **Mitigations:** Implement a unit test suite in `src/logging/tests.rs` or a dedicated test module that validates facts emitted from test harnesses (plan, preflight, apply, rollback) against `SPEC/audit_event.schema.json` using a JSON schema validation library like `jsonschema`. Run this as part of the CI pipeline to catch schema violations early.
  - **Impacted users:** CLI developers and downstream tools that parse observability facts, expecting them to conform to the published schema for automated processing or monitoring.
  - **Follow-ups:** Flag this as a medium-severity reliability gap for Round 3. Plan to add schema validation tests in Round 4 to ensure long-term consistency.

Gap analysis in Round 2 by AI 4 on 2025-09-12 15:38 CET
