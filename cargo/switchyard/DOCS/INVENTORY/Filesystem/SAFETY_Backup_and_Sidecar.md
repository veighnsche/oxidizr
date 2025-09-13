# Backup and sidecar

- Category: Safety
- Maturity: Silver

## Summary

Creates adjacent timestamped backups and sidecars when mutating targets, preserving mode and recording provenance and optional payload hash.

## Behaviors

- Derives backup and sidecar paths using `backup_path_with_tag()` naming convention.
- Captures current node state (file/symlink/none) and preserves mode/ownership.
- Writes sidecar v1/v2 with provenance and optional `payload_hash` if hashing enabled.
- Fsyncs parent directory best-effort to improve durability of backup artifacts.
- Provides helpers to locate and read sidecars for restore and integrity checks.

## Implementation

- Backup API: `cargo/switchyard/src/fs/backup.rs`
  - `backup_path_with_tag()` naming: `.<name>.<tag>.<millis>.bak`
  - `create_snapshot()` handles file, symlink, and none topologies; writes sidecar v1/v2; fsyncs parent.
  - `read_sidecar()`, `sidecar_path_for_backup()` helpers.

## Wiring Assessment

- Used by `fs/swap.rs::replace_file_with_symlink()` before swap and by restore engine.
- Policy flags `backup_tag`, `require_backup_durability` influence behavior and facts.
- Conclusion: wired correctly; used in apply and restore paths.

## Evidence and Proof

- Tests: `snapshot_*` tests in `fs/backup.rs`.
- Facts: apply.extra includes backup_durable flag and before/after hashes.

## Gaps and Risks

- Sidecar integrity is best-effort unless `require_sidecar_integrity` is enforced in restore.

## Next Steps to Raise Maturity

- Golden fixtures asserting sidecar schema and payload_hash parity.
- CI-backed retention tests.

## Related

- `cargo/switchyard/src/api/apply/handlers.rs` (hashing and provenance fields).
- SPEC v1.1 (sidecar and preservation).
