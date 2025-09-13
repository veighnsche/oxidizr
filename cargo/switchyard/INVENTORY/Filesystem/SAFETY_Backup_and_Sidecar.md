# Backup and sidecar

- Category: Safety
- Maturity: Silver

## Summary

Creates adjacent timestamped backups and sidecars when mutating targets, preserving mode and recording provenance and optional payload hash.

Pros & Cons

| Pros | Proof (code/tests) |
| --- | --- |
| Safety net before mutations | `cargo/switchyard/src/fs/backup.rs::create_snapshot()` called prior to swap/restore |
| Sidecar provenance + integrity | sidecar v1/v2 fields; payload hash when enabled |
| Durability via parent fsync (policy-aware) | `create_snapshot()` fsyncs parent; `require_backup_durability` policy |

| Cons | Notes |
| --- | --- |
| Extra I/O overhead | Hashing + fsync add latency on slow disks |
| Integrity enforcement optional | `require_sidecar_integrity=false` permits best-effort restores |

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

## Feature Analytics

- Complexity: Medium. Snapshot + sidecar schema + durability sync.
- Risk & Blast Radius: Medium; failures fall back to proceed or STOP depending on policy; restore depends on sidecar integrity.
- Performance Budget: Hashing + fsync overhead; acceptable for safety-critical paths.
- Observability: Sidecar fields recorded; apply facts include durability and hashes.
- Test Coverage: Unit tests for snapshot; gaps: golden sidecar schema and hash parity tests.
- Determinism & Redaction: Sidecar timestamps are not redacted; facts redacted in DryRun.
- Policy Knobs: `backup_tag`, `require_backup_durability`, `require_sidecar_integrity`.
- Exit Codes & Error Mapping: Interacts with restore codes (`E_BACKUP_MISSING`/`E_RESTORE_FAILED`).
- Concurrency/Locking: Independent.
- Cross-FS/Degraded: N/A for snapshot; swap path handles EXDEV.
- Platform Notes: POSIX filesystems; behavior depends on fsync semantics.
- DX Ergonomics: Clear helpers; tag supports multi-use.

Policy Controls Matrix

| Flag | Default | Effect |
| --- | --- | --- |
| `backup_tag` | `DEFAULT_BACKUP_TAG` | Names backup/sidecar artifacts |
| `require_backup_durability` | `true` | Attempt fsync; record `backup_durable` |
| `require_sidecar_integrity` | `true` | Enforce payload hash on restore |

Exit Reasons / Error → Exit Code Map

| Error ID | Exit Code | Where mapped |
| --- | --- | --- |
| `E_BACKUP_MISSING` | `60` | `cargo/switchyard/src/api/errors.rs::{exit_code_for, exit_code_for_id_str}` |
| `E_RESTORE_FAILED` | `70` | Same mapping; when sidecar verification or writeback fails |

Observability Map

| Fact | Fields (subset) | Schema |
| --- | --- | --- |
| `apply.result` | `before_hash`, `after_hash`, `backup_durable` | Minimal Facts v1 |
| Sidecar | schema v1/v2; `payload_hash`, provenance | Sidecar schema (internal) |

Test Coverage Map

| Path | Test name | Proves |
| --- | --- | --- |
| `src/fs/backup.rs` | `snapshot_*` | sidecar creation; durability behavior |
| `src/fs/restore.rs` | restore tests | integrity enforcement path |

## Maturity & Upgrade Path

| Tier | Capabilities | Required Guarantees | Tests/Proofs | Ops/Tooling | Relationship to Previous Tier |
| --- | --- | --- | --- | --- | --- |
| Bronze | Basic snapshot + sidecar | Backup exists; minimal schema | Unit tests | None | Additive |
| Silver (current) | Durability sync; hash capture; facts emission | Durable backups; hash parity on restore | Unit + integration | Inventory docs | Additive |
| Gold | Golden sidecar fixtures; schema validation; perf bounds | Validated sidecar schema + integrity | Goldens + CI | CI validation | Additive |
| Platinum | Platform matrix; resilience under faults | Strong guarantees under fs/IO faults | Fault injection tests | Continuous compliance | Additive |

## Maintenance Checklist

- [x] Code citations are accurate (paths and symbol names)
- [x] Policy knobs documented reflect current `policy::Policy`
- [x] Error mapping and `exit_code` coverage verified (via restore)
- [x] Emitted facts fields listed and schema version up to date
- [ ] Determinism parity (DryRun vs Commit) verified in tests
- [ ] Goldens added/updated and CI gates green
- [ ] Sidecar schema validated in CI
- [ ] Security considerations reviewed; payload hashing coverage adequate
- [ ] Licensing impact considered (deps changed? update licensing inventory)

## Gaps and Risks

- Sidecar integrity is best-effort unless `require_sidecar_integrity` is enforced in restore.

## Next Steps to Raise Maturity

- Golden fixtures asserting sidecar schema and payload_hash parity.
- CI-backed retention tests.

## Related

- `cargo/switchyard/src/api/apply/handlers.rs` (hashing and provenance fields).
- SPEC v1.1 (sidecar and preservation).
