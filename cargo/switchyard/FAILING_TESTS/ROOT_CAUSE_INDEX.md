# Root Cause Index

This index maps each failing test dossier to the shared root-cause dossier that explains it, and outlines a short manual fix plan with deterministic rerun commands.

## Mapping: failing test → root cause

- apply::enospc_backup_restore::e2e_apply_014_enospc_during_backup_restore_path → ROOT_CAUSE_1.md
- apply::ownership_strict_with_oracle::e2e_apply_017_ownership_strict_with_oracle_present → ROOT_CAUSE_1.md
- apply::attestation_apply_success::attestation_fields_present_on_success_and_masked_after_redaction → ROOT_CAUSE_1.md
- apply::sidecar_integrity_disabled::e2e_apply_020_sidecar_integrity_disabled_tolerates_tamper → ROOT_CAUSE_2.md (also consistent with ROOT_CAUSE_1)
- apply::smoke_ok::smoke_runner_ok_yields_success_and_no_rollback → ROOT_CAUSE_2.md

## 10-line Fix Plan Outline (manual)

1) Add a scoped env guard `tests/helpers/env.rs::ScopedEnv` to set/restore process env per test (covers `SWITCHYARD_FORCE_EXDEV`, `SWITCHYARD_FORCE_RESCUE_OK`). Acceptance: EXDEV/attestation tests pass under normal threading.
2) Mark env‑mutating tests with `serial` (e.g., `serial_test` crate) or move them to a dedicated single-threaded test group. Acceptance: zero flakes in 10 full runs.
3) Introduce `TestRoot` helper: per‑test unique `tempfile::TempDir` and thread it through plan/builders to isolate `<tmp>/usr/bin/app` paths. Acceptance: smoke and sidecar tests stable in parallel.
4) Ensure `Switchyard::new(...)` accepts test root override or builder sets it; update tests to use unique roots. Acceptance: no shared path collisions.
5) Optionally provide a lightweight test `LockManager` and enable it in parallel suites. Acceptance: WARN absence when manager present; no interleaving mutations.
6) Review `tests/apply/exdev_degraded.rs` and `tests/apply/error_exdev.rs` to use `ScopedEnv`. Acceptance: `degraded` facts only when explicitly set.
7) Review `tests/apply/attestation_apply_success.rs` to ensure MockAttestor injection is scoped and unaffected by globals. Acceptance: raw+redacted attestation fields present.
8) Review `tests/apply/sidecar_integrity_disabled.rs` to use per-test roots and avoid cross‑test sidecar collisions. Acceptance: consistent success.
9) Add a CI job that runs `RUST_TEST_THREADS=1` for env‑mutating group and default threading for the rest. Acceptance: both green.
10) Add a stress run (5x loop) in CI for the hot set to catch regressions. Acceptance: stable across loops.

## Deterministic rerun commands

```bash
# Single-threaded stress (deterministic)
RUST_LOG=info RUST_TEST_THREADS=1 cargo test -p switchyard --test integration_tests -- --nocapture

# Normal threading (post-fix confidence)
RUST_LOG=info cargo test -p switchyard --test integration_tests -- --nocapture
```
