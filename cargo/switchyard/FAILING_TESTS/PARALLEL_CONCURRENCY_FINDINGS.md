# Parallel Concurrency Findings — Why suite fails under concurrent runs

## Summary

When running the Switchyard test suite in parallel (multiple threads) or interleaving multiple applies without a `LockManager`, we saw intermittent failures. Two systemic causes explain these:

- Process-global environment overrides (e.g., `SWITCHYARD_FORCE_EXDEV`, `SWITCHYARD_FORCE_RESCUE_OK`, PATH edits) leak across tests running in the same process, flipping code paths and emitted facts unexpectedly.
- Parallel apply/smoke operations on similarly named targets under no lock can observe transient filesystem states, causing rollback or mismatched facts in flake scenarios.

These are harness-level and policy-layer issues, not core algorithmic bugs. The product’s concurrency contract expects a `LockManager` in production.

## Robust Program Solution (Implemented)

The following hardening changes were made in the product code to eliminate process-global leakage in realistic, concurrent runs while preserving test flexibility:

- EXDEV and RESCUE test overrides are now gated in product code (ignored by default):
  - `src/fs/atomic.rs`: only honor `SWITCHYARD_FORCE_EXDEV=1` when either `cfg(test)` or `SWITCHYARD_TEST_ALLOW_ENV_OVERRIDES=1` is set. Otherwise, the env var is ignored.
  - `src/policy/rescue.rs`: only honor `SWITCHYARD_FORCE_RESCUE_OK=1|0` under the same gating conditions.

  Proof (citations):
  - `src/fs/atomic.rs` — the post-`renameat(...)` EXDEV injection now checks `allow_env_overrides` before reading `SWITCHYARD_FORCE_EXDEV`.
  - `src/policy/rescue.rs` — `verify_rescue_min(...)` reads `SWITCHYARD_FORCE_RESCUE_OK` only if `cfg(test)` or `SWITCHYARD_TEST_ALLOW_ENV_OVERRIDES=1`.

- Tests that need simulation set overrides in a scoped manner and explicitly enable the allow flag:
  - Example conversions: `tests/apply/exdev_degraded.rs`, `tests/apply/error_exdev.rs`, `tests/apply/error_policy.rs`, `tests/audit/preflight_summary_error_id.rs`.
  - BDD steps (`tests/steps/apply_steps.rs`, `tests/steps/plan_steps.rs`) now push `SWITCHYARD_TEST_ALLOW_ENV_OVERRIDES=1` alongside scenario-specific overrides.

Effect: In realistic multi-process or multi-threaded product runs, global env toggles no longer affect behavior unless the operator intentionally opts in. Tests remain able to simulate branches deterministically without leaking to neighbors.

## Observed symptoms

- Tests that pass individually fail in full, multi-threaded runs:
  - `apply::enospc_backup_restore::e2e_apply_014_enospc_during_backup_restore_path`
  - `apply::ownership_strict_with_oracle::e2e_apply_017_ownership_strict_with_oracle_present`
  - `apply::attestation_apply_success::attestation_fields_present_on_success_and_masked_after_redaction`
  - `apply::sidecar_integrity_disabled::e2e_apply_020_sidecar_integrity_disabled_tolerates_tamper`
  - `apply::smoke_ok::smoke_runner_ok_yields_success_and_no_rollback`

## Mechanisms (root-causes)

- __Process-global env overrides leak across tests__
  - Code sites reading overrides:
    - `src/fs/atomic.rs`: EXDEV sim via `SWITCHYARD_FORCE_EXDEV`.
    - `src/policy/rescue.rs`: Rescue override via `SWITCHYARD_FORCE_RESCUE_OK`.
  - In multi-threaded tests, one test sets an override while another begins running, shifting apply/summary behavior. This changes success/failure, `degraded` telemetry, or provenance/attestation expectations unexpectedly.

- __No LockManager (tests set allow_unlocked_commit=true)__
  - Many tests explicitly set `policy.governance.allow_unlocked_commit = true` to bypass providing a `LockManager`.
  - In parallel, multiple applies can interleave, creating brief windows where `DefaultSmokeRunner` or assertions observe pre/post-swap states inconsistently. This is a test-only behavior; in production, `LockManager` is required (REQ‑L4).

## Evidence (citations)

- __Dossiers (failure notes)__
  - `FAILING_TESTS/TEST_1_ENOSPC_backup_restore.md:82–84,117–121`: “Parallel test env contamination… env flags: `SWITCHYARD_FORCE_EXDEV`, `SWITCHYARD_FORCE_RESCUE_OK`, PATH; `RUST_TEST_THREADS`.”
  - `FAILING_TESTS/TEST_2_Ownership_strict_with_oracle_present.md:103–105`: env contamination best supported.
  - `FAILING_TESTS/TEST_3_Attestation_apply_success.md:142–145`: parallel interference led to missing attestation capture/mis-ordered facts.
  - `FAILING_TESTS/TEST_4_Sidecar_integrity_disabled.md:85–87`: parallel env/path toggles; single-test PASS.

- __Blockers/Spec__
  - `RELEASE_BLOCKER_1.md:21–37`: tests set `SWITCHYARD_FORCE_EXDEV`; simulation path.
  - `SPEC/requirements.yaml:490–497 (REQ‑T2)`: only one mutator proceeds at a time under `LockManager`; without it, concurrent apply is unsupported.
  - `SPEC/SPEC.md:64–69 (Locking requirements)` and `2.10 Filesystems & Degraded Mode` on degraded telemetry.

- __Test env setters prior to fix__
  - `tests/apply/exdev_degraded.rs`: direct `std::env::set_var("SWITCHYARD_FORCE_EXDEV", "1")` (now scoped).
  - `tests/apply/error_exdev.rs`: direct EXDEV set (now scoped).
  - `tests/apply/error_policy.rs`: `SWITCHYARD_FORCE_RESCUE_OK=0` (now scoped).
  - BDD steps push `EnvGuard` for EXDEV/RESCUE; updated to also push `SWITCHYARD_TEST_ALLOW_ENV_OVERRIDES=1`.

## What we changed (and why tests failed before)

- __Before__: tests set global env flags directly; suite ran multi-threaded; EXDEV/RESCUE/PATH leaked across tests. Many tests also ran with `allow_unlocked_commit=true`, meaning no mutual exclusion across concurrent applies; smoke validations could see transient states.

- __After__:
  - Introduced `tests/helpers/env.rs::ScopedEnv` and converted env-mutating tests to scoped, per-test overrides; added an explicit allow flag so product code only honors overrides during tests or when explicitly allowed: `SWITCHYARD_TEST_ALLOW_ENV_OVERRIDES=1`.
  - Gated product overrides:
    - `src/fs/atomic.rs`: only inject EXDEV if `cfg(test)` or allow flag is set.
    - `src/policy/rescue.rs`: same gating for RESCUE override.
  - Added optional in-process `TestLockManager` for `apply::smoke_ok` to serialize mutators inside that test and avoid transient windows.
  - Marked schema-sensitive preflight test `#[serial]` to avoid PATH/rescue collisions.

## Product vs Test Boundary — Audit

This section identifies helpers under `tests/` and determines whether any should be promoted to product hardening per `docs/testing/TESTING_POLICY.md` (product must do the work; tests must not mask product-managed artifacts).

- __tests/bdd_support/env.rs::EnvGuard__ — Scoped env helper used by BDD steps. Test-only utility to avoid leaking env. No promotion needed; product now ignores overrides unless explicitly allowed.
- __tests/helpers/env.rs::ScopedEnv__ — Same role as above for non-BDD tests. Test-only; correct placement.
- __tests/helpers/lockmgr.rs::TestLockManager__ — A minimal, in-process lock manager for test serialization. Product already provides `src/adapters/lock/file.rs::FileLockManager`. Keeping a test-only lock makes sense (no file I/O), but product remains hardened by requiring a real `LockManager` in production (REQ‑L4). No promotion needed.
- __tests/helpers/testroot.rs::TestRoot__ — Convenience wrapper over `tempfile::TempDir`. Strictly a harness helper; product uses `SafePath` and typed APIs. No promotion needed.
- __tests/helpers/exdev_simulator.rs__ — Test helper for EXDEV scenarios. With product gating in place, simulation remains test-scoped. No promotion needed.

Conclusion: After env override gating landed in `src/`, no remaining `tests/` helper is papering over product responsibilities. The `LockManager` requirement is enforced at policy level; tests optionally use a lightweight manager only to stabilize parallelism. This conforms to `docs/testing/TESTING_POLICY.md` (e.g., lines 5–8, 25–33, 52–55).

## Additional Product Hardening Opportunities

While reviewing for concurrency, we identified improvements that further reduce flake risks and align with the normative spec. These are product-side changes (not test-only) and are backed by our code reading and prior blocker notes:

- __Atomic fsync semantics (TOCTOU tightening)__
  - Today `src/fs/atomic.rs::fsync_parent_dir(path)` reopens the parent by path and calls `sync_all()`. Prefer fsync via the already-open `dirfd` to eliminate a minor TOCTOU window.
  - Action: thread an `&OwnedFd` into a new `fsync_dirfd(&OwnedFd)` and use `rustix::fs::fsync`. Update callers in `atomic_symlink_swap` and restore steps.
  - Evidence: `RELEASE_BLOCKER_1.md` and `RELEASE_BLOCKER_5.md` style notes; code cites at `src/fs/atomic.rs` around the rename/fsync sequence.

- __Unique temporary names to avoid collisions__
  - Current deterministic tmp name: `.{fname}{TMP_SUFFIX}` (`src/fs/atomic.rs`) can collide under concurrency and leave litter after crashes.
  - Action: compose a unique tmp using `PID` + counter or a short random suffix, and use byte-safe `CString` construction (`OsStrExt::as_bytes(...)`).
  - Evidence: “Discovered During Research” sections in `RELEASE_BLOCKER_1.md/2.md` (tmp name uniqueness & non‑UTF‑8 safety).

- __ENOENT-only unlink ignores__
  - Be strict about which errors we ignore when unlinking temporary files (`unlinkat`). Today many paths use “best-effort unlink.” Ignoring only `ENOENT` reduces silent failures.

- __Centralize environment reads__
  - Move direct `std::env` reads in product into a small `env_overrides` module that enforces gating and documents all supported overrides. This avoids accidental future reads bypassing the gate.

- __Default Locking posture__
  - In production builds, consider enforcing `LockingPolicy::Required` as the default in `Policy::default()` or `Switchyard::builder()` unless explicitly overridden. Tests would continue to opt into `allow_unlocked_commit=true`, but real deployments would not accidentally run without a `LockManager`.

None of the above changes alter public semantics; they reduce race windows and make concurrency behavior more predictable. They also align with `SPEC/SPEC.md §2.5 Locking` and `§2.10 Filesystems & Degraded Mode` and the TOCTOU rules.

## Evidence Recap (code references)

- EXDEV override read: `src/fs/atomic.rs:...` (post-`renameat` injection now gated by `allow_env_overrides`).
- RESCUE override read: `src/policy/rescue.rs:...` (gated by `allow_env_overrides`).
- Locking default + WARN patterns: `src/api/apply/mod.rs` and `src/api/apply/lock.rs` (attempt events contain lock fields; WARN emitted when optional and no manager).
- Tests using TempDir per test: broad usage via `tempfile::tempdir()` throughout `tests/` (grep evidence in this investigation).


## What is the real issue?

- __Global state + parallelism + test-only policies__. The engine is designed to require a `LockManager` in production (REQ‑L4). Tests bypass this by setting `allow_unlocked_commit=true` and also rely on global env toggles for simulation. Under parallelism, this combination creates non-deterministic interleavings and state leakage, which are test harness artifacts rather than product correctness bugs.

## Realistic multi-process concurrency (what to test)

- Add explicit concurrent-run tests that:
  - Launch multiple applies in parallel on disjoint targets with a `LockManager`; assert facts remain consistent and no rollback on smoke success.
  - Contend on the same target with and without a lock to verify REQ‑L1/L4/L3 behavior (only one proceeds under lock; timeouts → `E_LOCKING`).
  - Verify EXDEV/RESCUE overrides do not affect other processes unless allow-flag is set.

## Recommendations (next steps)

- __Add a concurrency test suite__ that:
  - Spawns N threads (or processes) each creating its own TempDir, with a `FileLockManager` or `TestLockManager`, asserting stable apply.result facts with smoke.
  - Adds a contention test on the same target with lock timeout assertions (check `lock_wait_ms` and `E_LOCKING`).
- __Keep env overrides scoped & gated__; never rely on global env for simulation in general-purpose tests without scoping.
- __Consider promoting TestLockManager usage__ in tests that validate post-apply smoke behavior in the presence of parallel suite execution.

## Acceptance criteria for stability

- All env-mutating tests pass both single-threaded (`RUST_TEST_THREADS=1`) and with default threading.
- Running the flakiest tests (e.g., smoke_ok) 5× in a loop passes consistently.
- Adding a concurrency-focused test suite demonstrates that with a `LockManager` the engine behaves deterministically under parallel load; without a `LockManager` the expected `E_LOCKING`/WARN semantics surface.

---

### Appendix: Commands

- Single-thread (deterministic):

```bash
RUST_LOG=info RUST_TEST_THREADS=1 cargo test -p switchyard -- --nocapture
```

- Normal threading:

```bash
RUST_LOG=info cargo test -p switchyard --test integration_tests -- --nocapture
```

- Hot test 5× (smoke_ok):

```bash
for i in {1..5}; do \
  cargo test -p switchyard --test integration_tests -- \
    apply::smoke_ok::smoke_runner_ok_yields_success_and_no_rollback --nocapture; \
done
```
