# Adapters and extensibility

- Category: DX
- Maturity: Bronze
- Owner(s): <owner>
- Last reviewed: 2025-09-13
- Next review due: 2025-10-13
- Related PR(s): <#NNNN>

## Summary

Extensible adapter traits for locking, ownership, smoke tests, and attestation allow integrators to plug in environment-specific behaviors.

Pros & Cons

| Pros | Proof (code/tests) |
| --- | --- |
| Clear trait boundaries for key integrations | `cargo/switchyard/src/adapters/*` modules define traits and defaults |
| Enables environment-specific implementations | e.g., custom LockManager, SmokeTestRunner |
| Keeps core small and focused | Optional adapters injected via builders |

| Cons | Notes |
| --- | --- |
| Limited default adapters provided | Only file-backed lock and default smoke runner in-tree |
| Integrator burden for advanced needs | Distributed lock, package oracles, etc. need external impls |

## Behaviors

- Exposes `with_*` builder methods on `Switchyard` to inject adapters (lock, ownership, smoke, attest, path).
- Provides minimal default implementations where practical (e.g., file-backed lock, default smoke runner).
- Defers environment-specific behaviors to integrator-supplied adapters.

## Implementation

- Traits:
  - `LockManager`/`LockGuard` and `FileLockManager` (file-backed) — `cargo/switchyard/src/adapters/lock/*`
  - `OwnershipOracle` with `FsOwnershipOracle` — `cargo/switchyard/src/adapters/ownership/*`
  - `SmokeTestRunner` with `DefaultSmokeRunner` — `cargo/switchyard/src/adapters/smoke.rs`
  - `Attestor` — `cargo/switchyard/src/adapters/attest.rs`
  - `PathResolver` — `cargo/switchyard/src/adapters/path.rs`

## Wiring Assessment

- `Switchyard` exposes `with_*` builders to inject adapters. Policy reads influence enforcement.
- Conclusion: wired; minimal defaults provided, but rich ecosystem adapters are out of scope here.

## Evidence and Proof

- Unit tests for `FileLockManager`; `DefaultSmokeRunner` implementation.

## Feature Analytics

- Complexity: Low-Medium collectively; individual adapters small.
- Risk & Blast Radius: Medium; adapters gate critical behaviors (lock/ownership/smoke/attest).
- Performance Budget: Adapter-dependent; file locks and simple smoke are low overhead.
- Observability: Adapters contribute to facts (lock backend, ownership co-ids, smoke results, attestation).
- Test Coverage: Some unit tests; gaps: example adapters and integration docs/tests.
- Determinism & Redaction: Facts redaction unaffected; adapters supply values.
- Policy Knobs: Indirect (policy read by apply/preflight); adapters configured via builders.
- Exit Codes & Error Mapping: Locking → `E_LOCKING` (30), Smoke → `E_SMOKE` (80), Ownership → `E_POLICY`/`E_OWNERSHIP`.
- Concurrency/Locking: LockManager controls lock semantics; others independent.
- Cross-FS/Degraded: N/A.
- Platform Notes: File lock impl uses fs2 on Unix; portability considerations for non-Unix.
- DX Ergonomics: Builder APIs (`with_*`) straightforward.

Observability Map

| Adapter | Facts influence | Where |
| --- | --- | --- |
| LockManager | `lock_backend`, `lock_wait_ms` | `apply.attempt`/`apply.result` |
| OwnershipOracle | `E_OWNERSHIP` co-id in summary | `apply.result` summary chain |
| SmokeTestRunner | `E_SMOKE` classification | `apply.result`, rollback facts |
| Attestor | `attestation.*` fields | `apply.result` |

## Maturity & Upgrade Path

| Tier | Capabilities | Required Guarantees | Tests/Proofs | Ops/Tooling | Relationship to Previous Tier |
| --- | --- | --- | --- | --- | --- |
| Bronze (current) | Traits + minimal defaults | Builders inject adapters | Unit tests | Docs | Additive |
| Silver | Example adapters and docs | Usable references for common envs | Examples + tests | Docs & guides | Additive |
| Gold | Ecosystem adapters; CI examples | Robust integrations | System/integration tests | CI demos | Additive |
| Platinum | Compliance-ready adapters | Strong guarantees/tooling | Compliance tests | Continuous compliance | Additive |

## Maintenance Checklist

- [x] Code citations are accurate (paths and symbol names)
- [x] Emitted facts influence documented
- [ ] Example adapters provided and tested

## Gaps and Risks

- Limited default adapters provided.

## Next Steps to Raise Maturity

- Provide example adapters (e.g., systemd lock, package DB ownership) and docs.

## Related

- PLAN adapters documentation; SPEC integration guidance.
