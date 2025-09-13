# Locking and concurrency

- Category: Safety
- Maturity: Silver (with adapter), Bronze (without)
- Owner(s): <owner>
- Last reviewed: 2025-09-13
- Next review due: 2025-10-13
- Related PR(s): <#NNNN>

## Summary

Apply enforces a process lock by default in Commit. Missing `LockManager` yields `E_LOCKING` unless policy allows unlocked commits.

Pros & Cons

| Pros | Proof (code/tests) |
| --- | --- |
| Prevents concurrent conflicting applies | `cargo/switchyard/src/api/apply/mod.rs` acquires lock before mutations |
| Bounded wait with clear exit code | `E_LOCKING` (30) mapping in `api/errors.rs`; constants in `constants.rs` |
| Pluggable via adapter | `adapters/lock/file.rs::FileLockManager`; `Switchyard::with_lock_manager(...)` |

| Cons | Notes |
| --- | --- |
| File-based advisory lock only by default | No distributed lock implementation in-tree |
| Policy complexity (dev vs prod) | Flags `require_lock_manager`, `allow_unlocked_commit` may be misconfigured |

## Behaviors

- Attempts to acquire a process-wide lock before apply in Commit mode.
- Emits `apply.attempt` with `lock_backend`, `lock_wait_ms`, and `lock_attempts`.
- On lock failure: emits failure facts, maps to `E_LOCKING` (exit 30), and aborts stage.
- When no manager is configured: fails-closed in Commit unless `allow_unlocked_commit` or `require_lock_manager=false`.
- In DryRun or allowed policy: emits a warning and proceeds without a lock.

## Implementation

- Adapter: `cargo/switchyard/src/adapters/lock/file.rs::FileLockManager` (fs2-based advisory file lock, bounded wait with `LOCK_POLL_MS`).
- API: `cargo/switchyard/src/api/apply/mod.rs` acquires lock, emits attempt/result facts with `lock_backend`, `lock_wait_ms` and error mapping to `E_LOCKING` (exit 30).
- Policy knobs: `require_lock_manager`, `allow_unlocked_commit` in `policy::Policy`.

## Wiring Assessment

- `Switchyard::with_lock_manager()` injects manager. Apply respects policy and mode (DryRun vs Commit).
- Facts include backend label and attempts; errors abort early.
- Conclusion: wired correctly when adapter provided; dev ergonomics allow no-lock with warning in DryRun.

## Evidence and Proof

- Unit tests: `FileLockManager` timeout/success test.
- Apply-stage tests check error mapping and facts in aggregate.

## Feature Analytics

- Complexity: Low. Single adapter and apply-stage acquire/release.
- Risk & Blast Radius: High if disabled in production; defaults favor safety via `require_lock_manager` in presets.
- Performance Budget: Minimal; bounded wait configured via `DEFAULT_LOCK_TIMEOUT_MS` and `LOCK_POLL_MS`.
- Observability: `apply.attempt`/`apply.result` carry lock backend and wait metrics.
- Test Coverage: Unit tests for file lock; integration path covered indirectly. Gaps: contention/golden tests.
- Determinism & Redaction: Facts redacted in DryRun; timing fields may be zeroed per redaction.
- Policy Knobs: `require_lock_manager`, `allow_unlocked_commit`.
- Exit Codes & Error Mapping: `E_LOCKING` (30) on lock failure.
- Concurrency/Locking: This is the lock layer; no per-target locks.
- Cross-FS/Degraded: N/A.
- Platform Notes: File locks rely on fs2 advisory semantics on Unix.
- DX Ergonomics: Simple builder injection; defaults in presets for prod.

Policy Controls Matrix

| Flag | Default | Effect |
| --- | --- | --- |
| `require_lock_manager` | `false` (true in prod preset) | If true in Commit, absence of manager → `E_LOCKING` and abort |
| `allow_unlocked_commit` | `false` | Allow proceeding without lock in Commit when true |

Exit Reasons / Error → Exit Code Map

| Error ID | Exit Code | Where mapped |
| --- | --- | --- |
| `E_LOCKING` | `30` | `cargo/switchyard/src/api/errors.rs::{exit_code_for, exit_code_for_id_str}` |

Observability Map

| Fact | Fields (subset) | Schema |
| --- | --- | --- |
| `apply.attempt` | `lock_backend`, `lock_wait_ms` | Minimal Facts v1 |
| `apply.result` | `error_id=E_LOCKING` on failure | Minimal Facts v1 |

Test Coverage Map

| Path | Test name | Proves |
| --- | --- | --- |
| `src/adapters/lock/file.rs` | contention/timeout tests | bounded wait behavior |
| `src/api/apply/mod.rs` | lock failure mapping (planned) | `E_LOCKING` aborts |

## Maturity & Upgrade Path

| Tier | Capabilities | Required Guarantees | Tests/Proofs | Ops/Tooling | Relationship to Previous Tier |
| --- | --- | --- | --- | --- | --- |
| Bronze | Optional lock; warnings in DryRun | Lock attempts observable | Unit tests | None | Additive |
| Silver (current) | Enforced in Commit; bounded wait; exit code mapping | Abort on lock failure; clear metrics | Unit + integration | Inventory entry | Additive |
| Gold | Contention tests + goldens; additional lock backends | Robustness under load; multiple backends | Goldens + CI | CI gates/alerts | Additive |
| Platinum | Distributed locks; formal invariants | Cross-node guarantees | Property/system tests | Continuous compliance | Additive |

## Maintenance Checklist

- [x] Code citations are accurate (paths and symbol names)
- [x] Policy knobs documented reflect current `policy::Policy`
- [x] Error mapping and `exit_code` coverage verified
- [x] Emitted facts fields listed and schema version up to date
- [ ] Determinism parity (DryRun vs Commit) verified in tests
- [ ] Goldens added/updated and CI gates green
- [ ] Preflight YAML or JSON Schema validated (where applicable)
- [ ] Cross-filesystem or degraded-mode notes reviewed (if applicable)
- [ ] Security considerations reviewed; redaction masks adequate
- [ ] Licensing impact considered (deps changed? update licensing inventory)
- [x] Maturity rating reassessed and justified if changed
## Gaps and Risks

- Only file-backed lock provided; no per-target granularity.

## Next Steps to Raise Maturity

- Add contention tests and a golden for timeout path.
- Consider per-target locks if required by consumers.

## Related

- SPEC v1.1 locking requirement and bounded wait.
- `cargo/switchyard/src/constants.rs::{LOCK_POLL_MS, DEFAULT_LOCK_TIMEOUT_MS}`.
