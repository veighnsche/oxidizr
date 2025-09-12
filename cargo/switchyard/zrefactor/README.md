# Switchyard Refactor Workspace (pre-1.0; breaking allowed)

This folder contains focused refactor plans, audits, and rulebooks. The goal is to land a clean, cohesive codebase with pristine file/folder organization. Breaking changes are allowed pre-1.0; mark removals per the rulebook and sweep in a final PR.

## How to use this folder

1) Read the Rulebook and add markers to files you plan to remove/move/replace.
2) Execute domain-specific INSTRUCTIONS in small PRs (see PR plans inside each doc).
3) Keep `removals_registry.md` up to date for files that can’t be annotated inline.
4) Cross-check the Cohesion Report for structure and guardrail greps.

## Status snapshot (last synced: 2025-09-12 23:16:50 +02:00)

- API module shape: NOT LANDED — `src/api.rs` still uses `#[path]` includes; planned move to `src/api/mod.rs` remains.
- Logging facade (StageLogger/EventBuilder): NOT LANDED — `src/logging/audit.rs` still uses `emit_*` helpers; facade not present.
- Policy-owned gating: PARTIAL — `src/policy/gating.rs` has `gating_errors(...)`, but `src/api/preflight/mod.rs` duplicates checks (including a duplicate SUID/SGID block) and still hard-codes "/usr" for one branch. No typed `evaluate_action(...)` in use across stages yet.
- FS split (backup/restore into cohesive submodules): NOT LANDED — monoliths `src/fs/backup.rs` and `src/fs/restore.rs` remain.
- FS atoms re-export tightening: LANDED (INTERNALIZED) — `src/fs/mod.rs` re-exports atoms as `pub(crate)`, so no public atoms leak; final removal optional.
- Deprecated shims: PRESENT — `src/adapters/mod.rs` still exposes `adapters::lock_file::*` (deprecated); `src/lib.rs` still has the top-level `pub use policy::rescue` alias (deprecated).
- Determinism and dry-run envelope: PRESENT — Minimal Facts v1 in `src/logging/audit.rs` includes `schema_version=1`, `plan_id`, and `dry_run` fields.

Refactor-only scope reminder: files ending with `.PROPOSAL.md` or marked "additive" describe new features — do not implement them while landing refactors.

## Table of Contents

- Rulebooks and Registries
  - [refactor_rulebook.INSTRUCTIONS.md](./refactor_rulebook.INSTRUCTIONS.md)
  - [removals_registry.md](./removals_registry.md)
  - [CODE_SMELL_AND_CLEAN_CODE_AUDIT.md](./CODE_SMELL_AND_CLEAN_CODE_AUDIT.md)

- Architecture and Organization
  - [responsibility_cohesion_report.md](./responsibility_cohesion_report.md)

- Domain Refactors (Breaking)
  - API
    - [api_refactor.INSTRUCTIONS.md](./api_refactor.INSTRUCTIONS.md)
    - [preflight_gating_refactor.INSTRUCTIONS.md](./preflight_gating_refactor.INSTRUCTIONS.md)
  - Filesystem
    - [fs_refactor_backup_restore.INSTRUCTIONS.md](./fs_refactor_backup_restore.INSTRUCTIONS.md)
  - Policy
    - [policy_refactor.INSTRUCTIONS.md](./policy_refactor.INSTRUCTIONS.md)
  - Logging / Audit
    - [logging_audit_refactor.INSTRUCTIONS.md](./logging_audit_refactor.INSTRUCTIONS.md)

- Consumer DX (Additive)
  - [library_consumer_dx.INSTRUCTIONS.md](./library_consumer_dx.INSTRUCTIONS.md)

- Features & UX
  - [FEATURES_CATALOG.md](./FEATURES_CATALOG.md)
  - [features_ux_refactor.PROPOSAL.md](./features_ux_refactor.PROPOSAL.md)

- Tests
  - [tests_refactor.INSTRUCTIONS.md](./tests_refactor.INSTRUCTIONS.md)

## Cross-links and Scope

- API refactor references
  - Logging facade: [logging_audit_refactor.INSTRUCTIONS.md](./logging_audit_refactor.INSTRUCTIONS.md)
  - Policy evaluator: [policy_refactor.INSTRUCTIONS.md](./policy_refactor.INSTRUCTIONS.md), [preflight_gating_refactor.INSTRUCTIONS.md](./preflight_gating_refactor.INSTRUCTIONS.md)
  - Cohesion targets: [responsibility_cohesion_report.md](./responsibility_cohesion_report.md)

- FS refactor references
  - Cohesion targets: [responsibility_cohesion_report.md](./responsibility_cohesion_report.md)
  - Removals: [removals_registry.md](./removals_registry.md)

- Policy/Gating references
  - API and Preflight: [api_refactor.INSTRUCTIONS.md](./api_refactor.INSTRUCTIONS.md), [preflight_gating_refactor.INSTRUCTIONS.md](./preflight_gating_refactor.INSTRUCTIONS.md)

- Logging facade references
  - API/Preflight call-site migrations: [api_refactor.INSTRUCTIONS.md](./api_refactor.INSTRUCTIONS.md), [preflight_gating_refactor.INSTRUCTIONS.md](./preflight_gating_refactor.INSTRUCTIONS.md)

- Consumer DX references
  - Builders, prelude, examples: [library_consumer_dx.INSTRUCTIONS.md](./library_consumer_dx.INSTRUCTIONS.md)

## Notes on overlaps

- The consolidated checklist is [CODE_SMELL_AND_CLEAN_CODE_AUDIT.md](./CODE_SMELL_AND_CLEAN_CODE_AUDIT.md). The older duplicate at `documantation/code_smell.md` is superseded and marked for removal.
- Each INSTRUCTIONS doc now includes a Meta section with Scope, Breaking/Status, and Related links to reduce drift.

## Suggested global PR order

1) Logging facade + API module reshaping (low risk)
2) Policy evaluator centralization (medium)
3) Preflight/apply wiring to evaluator (medium)
4) FS backup/restore split (medium)
5) Consumer DX (additive)
6) Final sweep of removals and CI grep guardrails
