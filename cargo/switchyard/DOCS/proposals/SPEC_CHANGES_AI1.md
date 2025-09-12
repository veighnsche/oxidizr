# SPEC Change Proposals — AI 1

Generated: 2025-09-12 16:26 +02:00
Author: AI 1
Inputs: DOCS/analysis/FS_SAFETY_AUDIT.md, DOCS/analysis/API_SURFACE_AUDIT.md, DOCS/analysis/OBSERVABILITY_FACTS_SCHEMA.md, DOCS/analysis/ERROR_TAXONOMY.md, DOCS/analysis/INDEX.md; SPEC.md §§2.6, 3, 6, 13; SPEC/audit_event.schema.json; Code: src/fs/*.rs, src/api/**/*.rs, src/logging/*.rs

## Proposal 1: Add summary_error_ids to facts schema

- Motivation (why): Improve observability and diagnostics by surfacing all relevant error IDs, not just a single summary. Round 3 (OBSERVABILITY_FACTS_SCHEMA) prioritized S2: “No performance/observability summary fields” and S2: “Facts schema lacks array of error IDs.”
- Current spec: SPEC §13 (Audit Event Schema) defines summary objects with an optional `error_id` (string) but no array of error identifiers.
- Proposed change (normative):
  - Add: In `apply.result.summary`, `preflight.summary`, and rollback summaries, define an optional field `summary_error_ids: array[string]` containing canonical error IDs from the error chain, ordered from most specific to most general.
  - Keep existing `error_id` (string) as the primary/highest-severity identifier.
  - Affected sections: SPEC §13; file `cargo/switchyard/SPEC/audit_event.schema.json`.
- Compatibility & migration:
  - Backward compatibility: Yes (additive). `error_id` remains; `summary_error_ids` is optional.
  - Migration plan: Dual emit `error_id` and `summary_error_ids` immediately; document field semantics. Consumers can gradually adopt.
- Security & privacy:
  - Impact: Neutral-positive. No additional sensitive data; IDs are stable constants.
- Acceptance criteria:
  - Schema updated and validated in CI (jsonschema) with fixtures for Apply, Preflight, Rollback.
  - Emitters populate `summary_error_ids` on failures; redaction preserves this field in DryRun and Commit.
  - Tests cover multi-cause errors where 2+ IDs appear.
- Evidence:
  - Code: `cargo/switchyard/src/api/apply/audit_fields.rs` (emitter helpers), `cargo/switchyard/src/api/apply/handlers.rs` (error mapping), `cargo/switchyard/src/logging/redact.rs` (redaction), `cargo/switchyard/SPEC/audit_event.schema.json` (schema file).
  - Analysis: `DOCS/analysis/OBSERVABILITY_FACTS_SCHEMA.md` (Round 2/3 gaps and S2 priority).

## Proposal 2: SafePath mandatory for mutating public APIs

- Motivation (why): Enforce TOCTOU safety and traversal defenses consistently across all mutating entry points. Round 3 (FS_SAFETY_AUDIT) S2: “Enforce SafePath on mutating APIs or validate early.” SPEC v1.1 already states SafePath requirement—this proposal clarifies enforcement and a deprecation timetable for raw Path variants.
- Current spec: SPEC §3 and §2.6 call for TOCTOU-safe operations and SafePath usage but do not pin deprecation windows for legacy `&Path` variants.
- Proposed change (normative):
  - Modify: “All mutating public APIs MUST accept `SafePath`. Legacy `&Path` variants MUST either be removed or explicitly `#[deprecated]` with an immediate validation `is_safe_path()` and a maximum deprecation window of one minor release.”
  - Affected sections: SPEC §3 API safety; SPEC §2.6 syscall sequencing note (reference SafePath at boundary).
- Compatibility & migration:
  - Backward compatibility: No (behavioral expectations change for raw `&Path` callers), mitigated by deprecation window and early-validation behavior.
  - Migration plan: Provide `SafePath` constructors and samples; mark `&Path` overloads as deprecated now; remove next minor.
- Security & privacy:
  - Impact: Improved safety; reduces path traversal and symlink tricks risk.
- Acceptance criteria:
  - All mutating public APIs either take `SafePath` or validate `&Path` inputs and return `E_TRAVERSAL`.
  - MIGRATION_GUIDE updated with examples; compile deprecation warnings on `&Path` use.
- Evidence:
  - Code: `cargo/switchyard/src/fs/mod.rs` (re-exports), `cargo/switchyard/src/fs/paths.rs::is_safe_path()`, `cargo/switchyard/src/fs/swap.rs`, `cargo/switchyard/src/fs/restore.rs`.
  - Analysis: `DOCS/analysis/FS_SAFETY_AUDIT.md` (gaps and mitigations), `DOCS/analysis/API_SURFACE_AUDIT.md`.

## Proposal 3: Error taxonomy — distinct E_OWNERSHIP surfacing and chain semantics

- Motivation (why): Consumers need consistent high-level classification (e.g., ownership issues) alongside specific causes. Round 3 (ERROR_TAXONOMY) S2: “Surface E_OWNERSHIP consistently and include full chain.”
- Current spec: SPEC §6 Error Taxonomy references canonical IDs but does not require `E_OWNERSHIP` co-emission on ownership-related failures or define chain emission obligations.
- Proposed change (normative):
  - Add: “When ownership-related checks fail (permissions/uid/gid), emit `E_OWNERSHIP` in addition to specific error IDs. Summary objects SHOULD include all IDs in `summary_error_ids` with `error_id` set to the most specific cause.”
  - Affected sections: SPEC §6 (taxonomy), §13 (schema semantics for `summary_error_ids`).
- Compatibility & migration:
  - Backward compatibility: Yes (additive classification); `error_id` unchanged.
  - Migration plan: Emit `E_OWNERSHIP` across affected paths; document mapping table.
- Security & privacy:
  - Impact: Positive (clearer classification for policy and alerting).
- Acceptance criteria:
  - Error types provide stable IDs and a chain iterator; ownership paths add `E_OWNERSHIP`.
  - Facts summary shows `summary_error_ids` containing both specific and `E_OWNERSHIP`.
- Evidence:
  - Code: `cargo/switchyard/src/types/errors.rs`, `cargo/switchyard/src/api/errors.rs` (error mapping and IDs), `cargo/switchyard/src/api/apply/audit_fields.rs`.
  - Analysis: `DOCS/analysis/ERROR_TAXONOMY.md`.

Proposals authored by AI 1 on 2025-09-12 16:26 +02:00
