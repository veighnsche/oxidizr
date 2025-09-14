# Parallel Suite Interference (Global Env & Test Overrides)

## 1) Summary

Process-global environment toggles and test overrides (e.g., `SWITCHYARD_FORCE_EXDEV`, `SWITCHYARD_FORCE_RESCUE_OK`, PATH mutations) leak across concurrently running tests. When the suite runs multi-threaded, one test’s env settings change the behavior of another, flipping apply decisions and fact shapes (e.g., degraded EXDEV paths, success vs. failure). All affected tests pass in isolation, confirming the product code paths are sound; the failures stem from parallel suite interference.

## 2) Failing tests explained by this cause

- apply::enospc_backup_restore::e2e_apply_014_enospc_during_backup_restore_path — EXDEV/rescue env flags from other tests can force the apply path away from the expected “normal success”.
- apply::ownership_strict_with_oracle::e2e_apply_017_ownership_strict_with_oracle_present — Env contamination leads to unexpected gating or fact differences, masking provenance checks.
- apply::attestation_apply_success::attestation_fields_present_on_success_and_masked_after_redaction — Env leaks can reorder/perturb apply summary emission, leading to missing attestation fields under parallel runs; isolated run is PASS.
- apply::sidecar_integrity_disabled::e2e_apply_020_sidecar_integrity_disabled_tolerates_tamper — Env/path toggles from other tests perturb policy branches; single-test run is stable.

## 3) Evidence (quotes + snippets with paths/lines)

- Plain text (from dossiers)

```text
/home/vince/Projects/oxidizr-arch/cargo/switchyard/FAILING_TESTS/TEST_1_ENOSPC_backup_restore.md:82-84
H1 (best supported): Parallel test env contamination ... All five “failing” tests pass individually; suite flakiness disappeared with serialized EXDEV tests and single-thread runs.

/home/vince/Projects/oxidizr-arch/cargo/switchyard/FAILING_TESTS/TEST_1_ENOSPC_backup_restore.md:117-121
Environment flags potentially interacting: SWITCHYARD_FORCE_EXDEV, SWITCHYARD_FORCE_RESCUE_OK, PATH mutations, RUST_TEST_THREADS (parallelism causing env races)

/home/vince/Projects/oxidizr-arch/cargo/switchyard/FAILING_TESTS/TEST_2_Ownership_strict_with_oracle_present.md:103-105
H1: Parallel env contamination caused earlier failure; individually the test passes and provenance is emitted.

/home/vince/Projects/oxidizr-arch/cargo/switchyard/FAILING_TESTS/TEST_3_Attestation_apply_success.md:142-145
H1 (best supported): Parallel test interference led to missing attestation capture or mis-ordered facts; serializing EXDEV env tests and running single-thread stabilized results.

/home/vince/Projects/oxidizr-arch/cargo/switchyard/FAILING_TESTS/TEST_4_Sidecar_integrity_disabled.md:85-87
H1: Parallel test interference; earlier FAIL occurred while other tests toggled env/path ... Single‑test repro is PASS.
```

- Blockers/spec (env-simulated EXDEV)

```text
/home/vince/Projects/oxidizr-arch/cargo/switchyard/RELEASE_BLOCKER_1.md:21-37
Root cause: Early return on SWITCHYARD_FORCE_EXDEV=1 ... EXDEV simulation placed after renameat so degraded branch executes; tests set SWITCHYARD_FORCE_EXDEV=1.
```

- Shared code path where env overrides bite (EXDEV injection)

```rust
// /home/vince/Projects/oxidizr-arch/cargo/switchyard/RELEASE_BLOCKER_3.md — B.1 excerpt mirrors code
// cargo/switchyard/src/fs/atomic.rs:58-116 (trimmed to env injection)
let rename_res = renameat(&dirfd, tmp_c2.as_c_str(), &dirfd, new_c.as_c_str());
// Test override: simulate EXDEV via env after renameat so fallback branch executes
let rename_res = if std::env::var_os("SWITCHYARD_FORCE_EXDEV") == Some(std::ffi::OsString::from("1")) {
    match rename_res { Ok(()) => Err(Errno::XDEV), Err(e) => Err(e) }
} else { rename_res };
match rename_res {
    Ok(()) => { /* success */ }
    Err(e) if e == Errno::XDEV && allow_degraded => { /* degraded fallback */ }
    Err(e) => Err(errno_to_io(e)),
}
```

- Relevant SPEC clauses

```text
/home/vince/Projects/oxidizr-arch/cargo/switchyard/SPEC/SPEC.md:96-99
REQ-F1/REQ-F2: On EXDEV, use safe fallback; if used and policy allows, facts MUST record degraded=true; else fail.
```

## 4) Mechanism of failure (step-by-step)

1) Test A sets `SWITCHYARD_FORCE_EXDEV=1` (to exercise degraded paths) or toggles PATH/rescue.
2) With `RUST_TEST_THREADS > 1`, Test B starts before Test A restores the environment.
3) Test B’s apply flow reads the contaminated env and takes a different branch (e.g., degraded EXDEV), altering facts and outcomes (or causing unexpected stops).
4) Assertions in Test B (expecting normal success or specific fields) trip. In isolation (`RUST_TEST_THREADS=1`), all pass.

## 5) Minimal manual fix options (you won’t implement them)

- Option A (preferred, low risk): Contain env overrides in tests
  - Use a scoped env guard in tests (e.g., helper `ScopedEnv` that sets and restores env on Drop).
  - Mark env-mutating tests `serial` to avoid overlap, or run them in a separate `cargo test` pass with `RUST_TEST_THREADS=1`.
  - Touchpoints: `tests/apply/exdev_degraded.rs`, `tests/apply/error_exdev.rs`, `tests/helpers/env.rs` (new guard).
  - Blast radius: low; test-only.

- Option B (alternate, medium risk): Gate env overrides in product under a test-only feature
  - Only honor `SWITCHYARD_FORCE_EXDEV` (and similar) when `cfg(test)` or a `SWITCHYARD_TEST_ALLOW_ENV_OVERRIDES=1` is present.
  - Touchpoints: `src/fs/atomic.rs` (EXDEV injection), possibly `src/api/preflight/mod.rs` (rescue override).
  - Blast radius: medium; product code paths touched but behavior unchanged in production defaults.

## 6) Acceptance criteria (verifiable, testable)

- Suite (normal threading) shows no flakes from these tests across 10 consecutive runs.
- Each listed test passes when run with and without `RUST_TEST_THREADS=1`.
- Facts from affected tests show expected branches:
  - ENOSPC success variant: `apply.result` `decision=success`; no `error_id`.
  - EXDEV env tests: `degraded=true` only when explicitly enabled by the test.

## 7) Collateral impact (what else this touches)

- Tests likely to change expectations: `apply::exdev_degraded::*`, `apply::error_exdev::*`, `preflight::*` rescue overrides.
- De-risks SPEC/requirements: REQ-F1, REQ-F2 (filesystems & degraded mode), REQ-L2 (locking warnings preserved under isolation).

## TL;DR

- Parallel tests leak env overrides across cases.
- EXDEV and rescue toggles are process-global and perturb unrelated tests.
- All five failing tests pass single-threaded; isolation or gating fixes the flakes.
- Prefer scoped env + serializing env-muting tests; optional product gate for env overrides.
