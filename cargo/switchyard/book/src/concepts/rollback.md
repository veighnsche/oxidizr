# Rollback

- Reverse-order rollback begins on first failure.
- Idempotent restore paths; manual rescue documented for break-glass.

Details
- Idempotency: applying the same rollback plan twice yields the same filesystem state.
- Partial restoration: if any rollback step fails, `rollback.summary` includes `summary_error_ids` and facts help identify which paths remain unrecovered.
- Sidecar integrity: when `sidecar_integrity=true` and the sidecar includes `payload_hash`, restore verifies the backup payload hash; mismatches fail restore.

Operator guidance
- Prefer `plan_rollback_of(apply_report)` to let Switchyard derive the precise restore plan.
- Use the [Recovery Playbook](../recovery-playbook.md) when library-guided restore is not possible.

Citations:
- `src/api/apply/mod.rs`
- `src/fs/restore/*`
- `DOCS/BACKUPS_AND_RESTORE.md`
- Inventory: `INVENTORY/15_FS_Restore_and_Rollback.md`, `INVENTORY/10_FS_Backup_and_Sidecar.md`
- SPEC: §2.2 (Rollback), §5 (Audit Facts)
