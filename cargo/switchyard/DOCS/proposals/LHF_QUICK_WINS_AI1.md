# Low-Hanging Fruit (Quick Wins) — AI 1
Generated: 2025-09-12 16:26 +02:00
Author: AI 1

## Quick Win 1: Public API compile test to enforce surface
- Type: Test
- Change:
  - Add `tests/public_api.rs` that imports only intended public helpers (`fs::{replace_file_with_symlink, restore_file, restore_file_prev, create_snapshot, has_backup_artifacts}` and types needed) and compiles. Do not import `atomic_symlink_swap`, `open_dir_nofollow`, or `fsync_parent_dir`.
- Scope (files):
  - `cargo/switchyard/tests/public_api.rs`
- Why now:
  - Enforces API surface stability and prevents accidental re-export of internals (ties to API_SURFACE_AUDIT, S2).
- Time estimate: a few hours
- Risk: Low
- Acceptance criteria:
  - Test compiles and passes in CI; importing internal atoms fails to compile (if trybuild used) or is not visible.
- Evidence:
  - `src/fs/mod.rs:L9-L15` currently re-exports atoms.

## Quick Win 2: Facts schema CI validation with fixtures
- Type: CI
- Change:
  - Add a small Rust test `tests/facts_schema.rs` or Python script in CI that validates representative facts (Apply failure, Preflight failure, Rollback failure) against `SPEC/audit_event.schema.json` using `jsonschema` crate or Python tool.
- Scope (files):
  - `cargo/switchyard/tests/facts_schema.rs` and sample JSON fixtures under `cargo/switchyard/tests/fixtures/`
- Why now:
  - Prevents drift (OBSERVABILITY_FACTS_SCHEMA, S2).
- Time estimate: a few hours
- Risk: Low
- Acceptance criteria:
  - CI fails if schema or events diverge; fixtures include `error_id` and `summary_error_ids` once implemented.
- Evidence:
  - Missing tests noted; schema file at `SPEC/audit_event.schema.json`.

## Quick Win 3: Deprecation attributes on legacy shims
- Type: Docs/Refactor
- Change:
  - Add `#[deprecated]` with notes to `adapters::lock_file::*` and crate-root `policy::rescue` re-export (if present). Link MIGRATION_GUIDE.
- Scope (files):
  - `cargo/switchyard/src/adapters/mod.rs`, `cargo/switchyard/src/lib.rs`
- Why now:
  - Immediate feedback to consumers; aligns with Round 3 S2 (BACKWARDS_COMPAT_SHIMS, API_SURFACE_AUDIT).
- Time estimate: < 1 day
- Risk: Low
- Acceptance criteria:
  - Building with legacy imports produces deprecation warnings; docs list replacements.
- Evidence:
  - `src/lib.rs:L20-L21` re-export; `src/adapters/` exists.

## Quick Win 4: Redaction docs explicitly note timing removal in DryRun
- Type: Docs
- Change:
  - Update BEHAVIORS and SPEC §13 commentary to state that DryRun redaction removes timing fields (`duration_ms`, `lock_wait_ms`) and volatile flags (`severity`, `degraded`).
- Scope (files):
  - `cargo/switchyard/DOCS/analysis/BEHAVIORS.md`, `cargo/switchyard/SPEC/SPEC.md` (commentary).
- Why now:
  - Sets expectations; reduces confusion (BEHAVIORS, S2).
- Time estimate: a few hours
- Risk: Low
- Acceptance criteria:
  - Docs updated; reviewers confirm consistency with `logging/redact.rs` behavior.
- Evidence:
  - `src/logging/redact.rs:L64-L80` removes timing and flags.

## Quick Win 5: Add links and status table to INDEX.md
- Type: Docs
- Change:
  - Update `DOCS/analysis/INDEX.md` to include missing analyses (package manager integration, activation persistence) and per-doc R1–R4 status table.
- Scope (files):
  - `cargo/switchyard/DOCS/analysis/INDEX.md`, new stubs as needed under `DOCS/analysis/`
- Why now:
  - Increases planning clarity (INDEX Round 2 gap).
- Time estimate: a few hours
- Risk: Low
- Acceptance criteria:
  - INDEX shows entries and status; links resolve.
- Evidence:
  - Round 2 INDEX.md gaps.

## Quick Win 6: Add provenance helper masking test
- Type: Test
- Change:
  - Extend `logging/redact.rs` tests to also assert that provenance fields other than `helper` remain unchanged and that masking applies conditionally only when present.
- Scope (files):
  - `cargo/switchyard/src/logging/redact.rs`
- Why now:
  - Clarifies redaction scope; prevents accidental over-redaction.
- Time estimate: a few hours
- Risk: Low
- Acceptance criteria:
  - New tests pass; behavior documented in comments.
- Evidence:
  - `logging/redact.rs:L80-L87` current masking logic.

## Quick Win 7: Document FSYNC_WARN_MS behavior and thresholds
- Type: Docs
- Change:
  - Add documentation in PERFORMANCE_PLAN and SPEC commentary on `FSYNC_WARN_MS` use and where `severity=warn` is emitted.
- Scope (files):
  - `cargo/switchyard/DOCS/analysis/PERFORMANCE_PLAN.md`, `cargo/switchyard/SPEC/SPEC.md` notes.
- Why now:
  - Clarifies telemetry semantics.
- Time estimate: a few hours
- Risk: Low
- Acceptance criteria:
  - Docs describe warn threshold and example; consistent with `apply/audit_fields.rs::maybe_warn_fsync`.
- Evidence:
  - `src/api/apply/audit_fields.rs:L23-L30`.

Proposals authored by AI 1 on 2025-09-12 16:26 +02:00
