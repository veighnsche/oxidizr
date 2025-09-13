# Removals Registry

Use this file to track removals/moves/replacements for files that cannot be annotated inline (e.g., JSON schemas), or when you prefer a central checklist. Each line should follow one of the formats below.

Formats

- remove: <repo-relative-path> — <reason or successor>
- move: <old-path> -> <new-path> — <reason>
- replace: <old-path> -> <new-path> — <reason>

Examples

- remove: cargo/switchyard/SPEC/audit_event.schema.json — field superseded; see SPEC.md
- move: cargo/switchyard/src/api.rs -> cargo/switchyard/src/api/mod.rs — idiomatic module layout
- replace: cargo/switchyard/src/logging/audit.rs -> cargo/switchyard/src/logging/audit.rs — StageLogger facade replaces emit_* helpers

Checklist generation

```bash
rg -n "^remove:|^move:|^replace:" cargo/switchyard/zrefactor/removals_registry.md -S
```

Planned entries

- move: cargo/switchyard/src/api.rs -> cargo/switchyard/src/api/mod.rs — idiomatic module layout; drop #[path] includes
- replace: cargo/switchyard/src/fs/backup.rs -> cargo/switchyard/src/fs/backup/{mod.rs,snapshot.rs,sidecar.rs,index.rs} — split monolith per zrefactor/fs_refactor_backup_restore.INSTRUCTIONS.md
- replace: cargo/switchyard/src/fs/restore.rs -> cargo/switchyard/src/fs/restore/{mod.rs,types.rs,selector.rs,idempotence.rs,integrity.rs,steps.rs,engine.rs} — split monolith per zrefactor/fs_refactor_backup_restore.INSTRUCTIONS.md
- replace: cargo/switchyard/src/preflight.rs -> cargo/switchyard/src/preflight/mod.rs — idiomatic module layout; drop #[path] includes
- remove: cargo/switchyard/src/fs/restore/core.rs.bak — merged into restore/{engine,idempotence,integrity,steps}.rs; transitional file left markers
- remove: cargo/switchyard/src/fs/backup.rs.bak — legacy placeholder, superseded by backup directory module
- remove: cargo/switchyard/src/fs/restore.rs.bak — legacy placeholder, superseded by restore directory module
- replace: cargo/switchyard/src/logging/audit.rs -> cargo/switchyard/src/logging/audit.rs — StageLogger facade replaces emit_* helpers per zrefactor/logging_audit_refactor.INSTRUCTIONS.md
- remove: cargo/switchyard/src/adapters/mod.rs::lock_file — deprecated shim; use switchyard::adapters::lock::file::*
- remove: cargo/switchyard/src/lib.rs top-level `pub use policy::rescue` — deprecated facade alias; import from switchyard::policy::rescue instead
- remove: cargo/switchyard/zrefactor/documantation/code_smell.md — BREAKING: superseded by zrefactor/CODE_SMELL_AND_CLEAN_CODE_AUDIT.md

## Planned test moves (crate integration tests)

- move: cargo/switchyard/tests/locking_timeout.rs -> cargo/switchyard/tests/locking/locking_timeout.rs — domain grouping
- move: cargo/switchyard/tests/lock_wait_fact.rs -> cargo/switchyard/tests/locking/lock_wait_fact.rs — domain grouping
- move: cargo/switchyard/tests/lock_attempts.rs -> cargo/switchyard/tests/locking/lock_attempts.rs — domain grouping
- move: cargo/switchyard/tests/locking_required.rs -> cargo/switchyard/tests/locking/locking_required.rs — domain grouping
- move: cargo/switchyard/tests/locking_stage_parity.rs -> cargo/switchyard/tests/locking/locking_stage_parity.rs — domain grouping
- move: cargo/switchyard/tests/preflight_preservation_required.rs -> cargo/switchyard/tests/preflight/preflight_preservation_required.rs — domain grouping
- move: cargo/switchyard/tests/preflight_suid_sgid.rs -> cargo/switchyard/tests/preflight/preflight_suid_sgid.rs — domain grouping
- move: cargo/switchyard/tests/preflight_yaml.rs -> cargo/switchyard/tests/preflight/preflight_yaml.rs — domain grouping
- move: cargo/switchyard/tests/preflight_yaml_golden.rs -> cargo/switchyard/tests/preflight/preflight_yaml_golden.rs — domain grouping
- move: cargo/switchyard/tests/public_api.rs -> cargo/switchyard/tests/apply/public_api.rs — domain grouping
- move: cargo/switchyard/tests/smoke_required.rs -> cargo/switchyard/tests/apply/smoke_required.rs — domain grouping
- move: cargo/switchyard/tests/smoke_rollback.rs -> cargo/switchyard/tests/apply/smoke_rollback.rs — domain grouping
- move: cargo/switchyard/tests/perf_aggregation.rs -> cargo/switchyard/tests/apply/perf_aggregation.rs — domain grouping
- move: cargo/switchyard/tests/attestation_apply_success.rs -> cargo/switchyard/tests/apply/attestation_apply_success.rs — domain grouping
- move: cargo/switchyard/tests/error_policy.rs -> cargo/switchyard/tests/apply/error_policy.rs — domain grouping
- move: cargo/switchyard/tests/error_atomic_swap.rs -> cargo/switchyard/tests/apply/error_atomic_swap.rs — domain grouping
- move: cargo/switchyard/tests/error_exdev.rs -> cargo/switchyard/tests/apply/error_exdev.rs — domain grouping
- move: cargo/switchyard/tests/error_restore_failed.rs -> cargo/switchyard/tests/apply/error_restore_failed.rs — domain grouping
- move: cargo/switchyard/tests/restore_invertible_roundtrip.rs -> cargo/switchyard/tests/fs/restore_invertible_roundtrip.rs — domain grouping
- move: cargo/switchyard/tests/prune_backups.rs -> cargo/switchyard/tests/fs/prune_backups.rs — domain grouping
- move: cargo/switchyard/tests/audit_schema.rs -> cargo/switchyard/tests/audit/audit_schema.rs — domain grouping
- move: cargo/switchyard/tests/provenance_presence.rs -> cargo/switchyard/tests/audit/provenance_presence.rs — domain grouping
- move: cargo/switchyard/tests/preflight_summary_error_id.rs -> cargo/switchyard/tests/audit/preflight_summary_error_id.rs — domain grouping
- move: cargo/switchyard/tests/summary_error_ids_ownership.rs -> cargo/switchyard/tests/audit/summary_error_ids_ownership.rs — domain grouping

## Planned inline test moves

- move: cargo/switchyard/src/api.rs (test `rollback_reverts_first_action_on_second_failure`) -> cargo/switchyard/tests/apply/rollback_reverts_first_action.rs — move heavy flow out of inline tests
