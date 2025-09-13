# Restore and rollback

- Category: Safety
- Maturity: Silver

## Summary

Restores targets from latest or previous backups using sidecar-guided logic. Apply performs reverse-order rollback on first failure.

## Implementation

- Restore engine: `cargo/switchyard/src/fs/restore.rs::{restore_file, restore_file_prev}`
  - Idempotence short-circuits when current state matches sidecar prior_kind/dest.
  - Verifies payload hash when present; maps failures to `NotFound` (E_BACKUP_MISSING) or restore failure.
- Apply rollback loop: `cargo/switchyard/src/api/apply/mod.rs` emits `rollback` facts per step and a summary extra when failures occur.

## Wiring Assessment

- `apply` calls `restore_file()` on executed actions when errors occur; policy `force_restore_best_effort` influences behavior.
- Facts emitted: `rollback` step events.
- Conclusion: wired correctly; inverse operation supported for symlink swaps and file writes.

## Evidence and Proof

- Tests in `fs/restore.rs` cover symlink/file/none topologies, idempotence, integrity mismatch behavior.
- Apply test `rollback_reverts_first_action_on_second_failure` validates end-to-end.

## Gaps and Risks

- `RestoreFromBackup` inverse is not fully invertible when prior state unknown.

## Next Steps to Raise Maturity

- Golden fixtures for E_BACKUP_MISSING and E_RESTORE_FAILED paths.

## Related

- `cargo/switchyard/src/fs/backup.rs` sidecar schema.
- `cargo/switchyard/src/api/errors.rs` error id/exit code mapping.
