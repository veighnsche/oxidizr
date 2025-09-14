# Parallel Failures — New Plan (Status, Attempts, Next Steps)

## Current Status (as of 2025-09-14)

- Passing reliably in isolation and in single-threaded full runs:
  - `apply::enospc_backup_restore::e2e_apply_014_enospc_during_backup_restore_path`
  - `apply::ownership_strict_with_oracle::e2e_apply_017_ownership_strict_with_oracle_present`
  - `apply::attestation_apply_success::attestation_fields_present_on_success_and_masked_after_redaction`
  - `apply::attestation_error_tolerated::attestation_error_is_tolerated_and_omitted`
  - `apply::sidecar_integrity_disabled::e2e_apply_020_sidecar_integrity_disabled_tolerates_tamper`
  - `apply::smoke_ok::smoke_runner_ok_yields_success_and_no_rollback`

- Still intermittent failures in a normal, fully parallel run:
  - The 5 tests above can fail when other tests simultaneously toggle global env flags (e.g., `SWITCHYARD_FORCE_EXDEV`) or mutate similar FS paths.
  - These are suite-interaction flakes, not product-path correctness bugs.

## What we tried and how effective it was

- __Env override gating in product (`src/`):__
  - `src/fs/atomic.rs`: only honor `SWITCHYARD_FORCE_EXDEV` under `cfg(test)` or with `SWITCHYARD_TEST_ALLOW_ENV_OVERRIDES=1`.
  - `src/policy/rescue.rs`: only honor `SWITCHYARD_FORCE_RESCUE_OK` under same gating.
  - Effectiveness: prevents accidental production influence; reduces some leaks. But env remains process-global during tests so parallel overlap can still bite.

- __Scoped env in tests (`tests/helpers/env.rs::ScopedEnv` + allow flag):__
  - Converted tests to set overrides with an RAII guard and explicit `SWITCHYARD_TEST_ALLOW_ENV_OVERRIDES=1`.
  - Effectiveness: narrows windows, guarantees restoration. Does not solve races when two tests overlap in time.

- __Test-time LockManager (optional):__
  - `tests/helpers/lockmgr.rs::TestLockManager` created; used in `tests/apply/smoke_ok.rs`.
  - Effectiveness: removes transient FS contention for that test; helps stability. Cannot prevent EXDEV injection from unrelated env toggles.

- __#[serial] for a schema-sensitive preflight test:__
  - `tests/audit/preflight_summary_error_id.rs` marked serial.
  - Effectiveness: avoids PATH/rescue collisions in that test. Not yet applied to all env-mutating tests.

## What is no longer an issue

- __Attestation fields missing__ were not a product issue. With `MockAttestor` injected and no global EXDEV injection, `apply.result` includes raw attestation and redaction masks expected fields. See `src/api/apply/summary.rs` and `src/logging/redact.rs`.

- __Ownership strict + oracle provenance__ passes reliably with `FsOwnershipOracle` injected and no unrelated env toggles. Provenance `{uid,gid,pkg}` is present in per‑action `apply.result`.

- __Sidecar integrity disabled__ behaves as expected: the knob affects restore semantics; ensure_symlink apply succeeds with the policy off.

- __Smoke success triggering rollback__ was caused by concurrent FS state; isolated runs never rollback on smoke success.

## What remains an issue

- __Global env overrides remain process‑wide during tests.__ Even with gating + ScopedEnv, two tests can overlap: one enables `SWITCHYARD_FORCE_EXDEV`, the other (expecting success) hits the injected branch and fails.

- __Path contention under no LockManager__ can still occur in parallel suites that target the same names under tmp (e.g., `<tmp>/usr/bin/app`).

## New Plan (product+tests) — with success probabilities

### Overlaps with RELEASE_BLOCKERS (kill two flies at once)

From the Switchyard root blockers:

- Overlap A — `RELEASE_BLOCKER_1.md` (EXDEV degraded fallback correctness & simulation placement).
- Overlap B — `RELEASE_BLOCKER_5.md` (TOCTOU/atomicity invariants: fsync via dirfd; unique tmp; bytes-safe `CString`; ENOENT-only unlink ignores).

We will combine these with our flake fix: implement instance-scoped overrides (removes cross-test env races) and, in the same pass, harden the atomic swap path (dirfd-fsync, unique tmp, strict unlink) so we address RB1 + RB5 together.

1) __Eliminate process‑global influence by moving overrides to instance‑scoped config__
   - Implement `Overrides` in `src/api/overrides.rs` with fields like `force_exdev: Option<bool>`, `force_rescue_ok: Option<bool>`.
   - Add `Switchyard::with_overrides(Overrides)` and plumb to:
     - `src/fs/atomic.rs::atomic_symlink_swap(...)` (replaces env read for EXDEV).
     - `src/policy/rescue.rs::verify_rescue_min(...)` (replaces env read for rescue OK/FAIL).
   - Update tests to set overrides via the builder instead of env, removing cross‑test leakage entirely.
   - __Probability of success: 95%__ (directly removes the primary flake source; narrow, well‑scoped change).

2) __Atomic hardening (RB5) — unique tmp, bytes-safe CStrings, strict unlink, fsync via dirfd__
   - In `src/fs/atomic.rs` and `src/fs/swap.rs`:
     - Change `.{fname}{TMP_SUFFIX}` to `.{fname}.{pid}.{ctr}{TMP_SUFFIX}` or a short random suffix.
     - Use byte‑safe `CString::new(OsStrExt::as_bytes(...))` for all fname/src CStrings.
     - Restrict `unlinkat` ignores to `ENOENT` only (propagate other errors).
     - Replace reopen‑by‑path `fsync_parent_dir()` with `rustix::fs::fsync(&dirfd)`; thread dirfd where needed.
   - __Probability of success: 90%__ (addresses RB5 and reduces parallel timing sensitivities observed by smoke/oracles).

3) __RB1 verification (EXDEV degraded fallback correctness)__
   - Ensure simulation still occurs post-`renameat` decision point, but driven by per-instance overrides (not env), so degraded branch engages only when tests opt in.
   - Add/confirm tests for both branches:
     - Fallback allowed → `degraded=true` fact present.
     - Fallback disallowed → `E_EXDEV` mapping present.
   - __Probability of success: 95%__ (simulation remains correct; test isolation guaranteed).

4) __Test harness adjustments__
   - Convert env‑simulating tests to `with_overrides()`; remove env usage.
   - For remaining hot paths that still share names, either:
     - Use per‑test unique roots (already mostly done), and/or
     - Opt into `TestLockManager` in a couple of cases like `smoke_ok`.
   - __Probability of success: 90%__ (after #1, far fewer overlaps; this just polishes remaining edges).

## Rollout steps (two-flies bundle: RB1 + RB5)

- Phase A (code): Introduce `Overrides` and atomic hardening together (unique tmp, bytes-safe CStrings, ENOENT-only unlink, dirfd‑fsync). Keep legacy env reads behind a debug-only feature (disabled by default) until all tests migrate.
- Phase B (tests): Replace env guards with `with_overrides()` in EXDEV/RESCUE tests and BDD steps. Add/confirm RB1 tests for degraded vs. disallowed branches.
- Phase C (verification):
  - Run full suite 10× in parallel; confirm zero flakes.
  - Stress hot tests 5×: `apply::smoke_ok`, `attestation_*`, `ownership_*`.

## Exit criteria (include RB1 + RB5)

- All five previously flaky tests pass under default parallel threading across 10 full runs.
- No test relies on env flags for simulation; all use instance overrides.
- No unexpected rollbacks on smoke success; attestation present on success; provenance present with oracle.
- RB1: EXDEV degraded fallback tests behave deterministically via per-instance overrides; degraded telemetry or `E_EXDEV` mapping correct.
- RB5: Atomic sequence invariants defensible — renameat→fsync via dirfd; tmp unique; unlink strictly ignores only ENOENT.

## Owners / Timebox

- Owners: @you (coding), @me (design + PR review)
- Timebox: Phase A+B: 0.5–1 day. Phase C: 0.5 day. Phase D: 0.25 day.

## Appendix — Rerun commands

```bash
# Full suite (parallel)
cargo test -p switchyard -q

# Single-thread deterministic
RUST_TEST_THREADS=1 cargo test -p switchyard -- --nocapture

# Stress specific tests
for i in {1..5}; do \
  cargo test -p switchyard --test integration_tests -- \
    apply::smoke_ok::smoke_runner_ok_yields_success_and_no_rollback \
    apply::attestation_apply_success::attestation_fields_present_on_success_and_masked_after_redaction \
    apply::ownership_strict_with_oracle::e2e_apply_017_ownership_strict_with_oracle_present \
    -- --nocapture; \
done
```
