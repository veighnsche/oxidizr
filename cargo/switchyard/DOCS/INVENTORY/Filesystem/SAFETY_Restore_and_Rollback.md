# Restore and rollback

- Category: Safety
- Maturity: Silver

## Summary

Restores targets from latest or previous backups using sidecar-guided logic. Apply performs reverse-order rollback on first failure.

Pros & Cons

| Pros | Proof (code/tests) |
| --- | --- |
| Sidecar-guided, idempotent restore | `cargo/switchyard/src/fs/restore.rs::{restore_file, restore_file_prev}` short-circuit when prior matches |
| Error classification and rollback | `api/apply/mod.rs` emits rollback step/summary; `api/errors.rs` maps IDs |
| Policy-driven safety for integrity | `require_sidecar_integrity`, `force_restore_best_effort` govern strictness |

| Cons | Notes |
| --- | --- |
| Incomplete inverse when prior unknown | Prior state may be ambiguous for some actions |
| Best-effort modes can reduce guarantees | When enabled, restores may proceed without payload or strict verification |

## Behaviors

- Reads sidecar metadata to determine prior state and destination.
- Short-circuits restore when current state already matches prior (idempotence).
- Verifies `payload_hash` when present; maps failures to appropriate error IDs.
- On apply failure, attempts reverse-order rollback for executed actions.
- Emits `rollback.step` events per action and a `rollback.summary` with error classification.

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

## Feature Analytics

- Complexity: Medium. Restore logic + rollback orchestration + error mapping.
- Risk & Blast Radius: Medium-High; incorrect rollback can leave partial state; mitigated via backups and sidecars.
- Performance Budget: Bounded by I/O; hash verification and writeback cost.
- Observability: Emits `rollback.step` and `rollback.summary` facts; errors mapped to stable IDs.
- Test Coverage: Unit + integration as noted; gaps: golden rollback summaries and hash-mismatch goldens.
- Determinism & Redaction: Facts redacted in DryRun; operation only in Commit.
- Policy Knobs: `force_restore_best_effort`, `require_sidecar_integrity`, `capture_restore_snapshot`, `require_backup_durability`.
- Exit Codes & Error Mapping: `E_BACKUP_MISSING` (60), `E_RESTORE_FAILED` (70), plus `E_POLICY` (10) when gated.
- Concurrency/Locking: Follows apply-stage locking; rollback executed under same protection.
- Cross-FS/Degraded: N/A for restore; swap path handles EXDEV separately.
- Platform Notes: Depends on filesystem semantics for permissions and symlink behavior.
- DX Ergonomics: Clear APIs and emitted facts for troubleshooting.

Policy Controls Matrix

| Flag | Default | Effect |
| --- | --- | --- |
| `force_restore_best_effort` | `false` | Allow restore success without payload; otherwise `E_BACKUP_MISSING` |
| `require_sidecar_integrity` | `true` | Enforce payload hash; STOP/FAIL on mismatch |
| `capture_restore_snapshot` | `true` | Take snapshot before restore to make operation invertible |
| `require_backup_durability` | `true` | Ensure snapshot fsyncs are attempted/recorded |

Exit Reasons / Error → Exit Code Map

| Error ID | Exit Code | Where mapped |
| --- | --- | --- |
| `E_BACKUP_MISSING` | `60` | `cargo/switchyard/src/api/errors.rs::{exit_code_for, exit_code_for_id_str}` |
| `E_RESTORE_FAILED` | `70` | Same mapping |
| `E_POLICY` | `10` | When gated from policy checks |

Observability Map

| Fact | Fields (subset) | Schema |
| --- | --- | --- |
| `rollback.step` | `action_id`, `path`, `result` | Minimal Facts v1 |
| `rollback.summary` | `error_ids`, `rolled_back_count` | Minimal Facts v1 |

Test Coverage Map

| Path | Test name | Proves |
| --- | --- | --- |
| `src/fs/restore.rs` | topology/idempotence tests | sidecar-guided restore behavior |
| `src/api/apply/mod.rs` | rollback test(s) | reverse-order rollback on failure |

## Maturity & Upgrade Path

| Tier | Capabilities | Required Guarantees | Tests/Proofs | Ops/Tooling | Relationship to Previous Tier |
| --- | --- | --- | --- | --- | --- |
| Bronze | Basic restore + rollback | Restores latest backup; attempts rollback | Unit tests | None | Additive |
| Silver (current) | Integrity verification; snapshot capture; facts | Idempotence + error mapping | Unit + integration | Inventory docs | Additive |
| Gold | Golden rollback summaries; hash-mismatch goldens | Deterministic reporting; integrity enforced | Goldens + CI | CI validation | Additive |
| Platinum | Fault-injection and platform matrix | Robust recovery under failures | Fault-injection tests | Continuous compliance | Additive |

## Maintenance Checklist

- [x] Code citations are accurate (paths and symbol names)
- [x] Policy knobs documented reflect current `policy::Policy`
- [x] Error mapping and `exit_code` coverage verified
- [x] Emitted facts fields listed and schema version up to date
- [ ] Determinism parity (DryRun vs Commit) verified in tests
- [ ] Goldens added/updated and CI gates green
- [ ] Fault injection coverage considered
## Gaps and Risks

- `RestoreFromBackup` inverse is not fully invertible when prior state unknown.

## Next Steps to Raise Maturity

- Golden fixtures for E_BACKUP_MISSING and E_RESTORE_FAILED paths.

## Related

- `cargo/switchyard/src/fs/backup.rs` sidecar schema.
- `cargo/switchyard/src/api/errors.rs` error id/exit code mapping.
