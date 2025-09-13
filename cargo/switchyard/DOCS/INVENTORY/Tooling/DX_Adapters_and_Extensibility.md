# Adapters and extensibility

- Category: DX
- Maturity: Bronze

## Summary

Extensible adapter traits for locking, ownership, smoke tests, and attestation allow integrators to plug in environment-specific behaviors.

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

## Gaps and Risks

- Limited default adapters provided.

## Next Steps to Raise Maturity

- Provide example adapters (e.g., systemd lock, package DB ownership) and docs.

## Related

- PLAN adapters documentation; SPEC integration guidance.
