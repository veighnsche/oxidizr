# ADR-0014: Defer Exit Code Mapping to End-of-Project Code Audit

Date: 2025-09-11
Status: Accepted
Decision Makers: Switchyard Team

## Context

The Switchyard SPEC v1.1 defines a stable error taxonomy and calls for mapping failures to
exit codes and stable identifiers (e.g., `E_LOCKING`, `E_RESTORE_FAILED`). While we already
emit partial `error_id` fields in facts and have scaffolding for `ErrorId -> exit_code`,
completing the mapping now risks churn and rework, because failure surfaces may evolve as
we implement remaining features (preservation probes, EXDEV matrix, rescue profile,
attestation, etc.).

## Decision

- We will defer final exit code mapping and enforcement until the end of the Switchyard project.
- The final step will be a comprehensive code audit to enumerate all exit reasons across modules
  (plan, preflight, apply, rollback, adapters) and produce a complete, stable mapping from
  `ErrorId` to exit codes and facts fields.
- Until then, we may emit provisional `error_id` values in facts for observability, but we
  will not lock down exit codes or rely on them for CI gates.

## Rationale

- Avoids repeated renumbering and churn while features and failure paths stabilize.
- Ensures a single authoritative pass captures all exit reasons, consistent with the SPEC.
- Keeps sprint velocity focused on feature completeness (determinism, preflight diffs,
  preservation probes, acceptance), deferring finalization to a planned audit.

## Consequences

- Short-term: Some facts may carry `error_id` without a guaranteed final `exit_code` value.
- Final milestone: A dedicated “exit codes” task will be executed as part of the
  end-of-project audit, producing:
  - A complete `ErrorId` registry and mapping table (`exit_code_for()`),
  - Updated facts emission including `exit_code`,
  - SPEC update with the final, versioned error table,
  - Golden fixtures updates to reflect finalized fields.

## References

- SPEC v1.1 Requirements (Errors & Exit Codes)
- `src/api/errors.rs` (`ErrorId`, `exit_code_for`) scaffolding
- Sprint 01 scope and TODOs in `cargo/switchyard/TODO.md`
