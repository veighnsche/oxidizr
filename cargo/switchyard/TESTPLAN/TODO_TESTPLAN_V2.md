# TODO_TESTPLAN_V2 — Remaining Test Work Summary

Generated: 2025-09-13T23:03:33+02:00
Source of truth: `cargo/switchyard/TESTPLAN/TODO_TESTPLAN.md` + current test tree under `cargo/switchyard/tests/`

This V2 summarizes the gaps that still need implementation and proposes concrete, minimal next steps (file locations, test names, and checks) to close them.

---

## High Priority (P0)

- __Prune by age exact limits (E2E-PRUNE-009 / E2E-PRUNE-010)__ — Done (see `tests/fs/prune_age_limits.rs`)
  - What: Add deterministic age-based pruning tests for 365d max and 1s min.
  - Where: `cargo/switchyard/tests/fs/prune_age_limits.rs`
  - How: Create multiple backup snapshots and set mtimes deterministically (e.g., via `filetime` crate or `nix::unistd::utimensat`).
  - Tests:
    - `prune_max_age_365d_prunes_older()` → implements E2E-PRUNE-009
    - `prune_min_age_1s_prunes_older()` → implements E2E-PRUNE-010

- __Golden fixtures gate (REQ-CI1/CI3)__ — CI golden diff job present; fixtures exist for minimal and two-action scenarios (see `.github/workflows/ci.yml` golden-fixtures job and `tests/golden/*`).
  - What: Commit canon JSON fixtures for golden scenarios and enforce CI diff gate.
  - Where:
    - `cargo/switchyard/tests/golden/minimal-plan/{canon_plan.json, canon_preflight.json, canon_apply_attempt.json, canon_apply_result.json}`
    - `cargo/switchyard/tests/golden/two-action-plan/{canon_plan.json, canon_preflight.json, canon_apply_attempt.json, canon_apply_result.json}`
  - How:
    - Run: `python3 test_ci_runner.py --golden all --update`
    - Commit generated files. Ensure CI job “Golden Fixtures Diff” passes.

- __Dry-run by default (REQ-C1)__ — Done (see `tests/requirements/default_dry_run.rs`)
  - What: Assert the public API defaults to `ApplyMode::DryRun` or equivalent fail-closed behavior unless explicitly set.
  - Where: `cargo/switchyard/tests/requirements/default_dry_run.rs`
  - Test: `req_c1_dry_run_by_default()` creates a plan and invokes apply without forcing Commit; assert no mutation facts, TS_ZERO timestamps.

- __Mutating APIs accept SafePath only (REQ-API1)__ — Done (compile-fail trybuild: `tests/trybuild/mutate_with_raw_path.rs`)
  - What: Enforce SafePath-only inputs for mutating operations.
  - Where: `cargo/switchyard/tests/requirements/mutating_api_safepath_only.rs`
  - How: Prefer compile-time/type tests (e.g., `static_assertions`) or doctest that attempts to call mutator with `&Path` fails to compile.

- __TOCTOU-safe syscall sequence evidence (REQ-TOCTOU1)__ — Done (fsync evidence in `tests/oracles/ensure_symlink_fsync.rs`)
  - What: Assert behavior consistent with open_dir_nofollow → openat → renameat → fsync(parent).
  - Where: `cargo/switchyard/tests/oracles/ensure_symlink_fsync.rs`
  - How: At minimum, assert presence of `fsync_ms` for parent dir and success facts; optionally instrument with a mockable FS layer to capture call order.

- __Deterministic IDs stable (REQ-D1)__ — Done (see `tests/requirements/deterministic_ids.rs`)
  - What: Assert `plan_id/action_id` are stable across identical runs.
  - Where: `cargo/switchyard/tests/requirements/deterministic_ids.rs`
  - Test: Build identical inputs twice; assert IDs equal and sorted ordering matches.

- __Before/after hashes present (REQ-O5)__ — Done (see `tests/requirements/before_after_hashes.rs`)
  - What: Assert `before_hash`/`after_hash` fields are present in apply facts when applicable.
  - Where: `cargo/switchyard/tests/requirements/before_after_hashes.rs`
  - Test: DryRun and Commit comparisons; presence/consistency of hash fields.

- __Prune invariants facts & fsync (REQ-PN2/PN3)__ — Done (see `tests/fs/prune_invariants_extended.rs`)
  - What: Assert prune deletes payload+sidecar and parent fsync recorded; assert `prune.result` fields.
  - Where: `cargo/switchyard/tests/fs/prune_invariants_extended.rs`
  - Tests: `prune_deletes_payload_and_sidecar_and_fsyncs_parent()`, `prune_emits_prune_result_fact()`

- __Fallback toolset available on PATH (REQ-RC3)__ — Done (see `tests/preflight/fallback_toolset_on_path.rs`)
  - What: Simulate PATH and assert at least one rescue toolset is present/executable when required by policy.
  - Where: `cargo/switchyard/tests/preflight/fallback_toolset_on_path.rs`

- __Bounds threshold (REQ-BND1)__ — Done (best-effort ≤100ms; see `tests/oracles/bounds_threshold.rs`)
  - What: Assert `fsync_ms <= 50` best-effort under controlled tempfs (flakiness guard). Consider a looser threshold (e.g., 100ms) if CI is noisy.
  - Where: `cargo/switchyard/tests/oracles/bounds_threshold.rs`

- __CI: Zero-SKIP gate (REQ-CI2)__ — Done (grep for `#[ignore]` in `.github/workflows/ci.yml`)

---

## Medium Priority (P1)

- __Source ownership gating (REQ-S3)__
  - Where: `cargo/switchyard/tests/preflight/ownership_source_gating.rs`
  - Test: Policy requires source root-owned/not world-writable; assert STOP and `E_OWNERSHIP` when violated.

- __Thread-safety and concurrency assertions (REQ-T1/REQ-T2)__
  - Where: `cargo/switchyard/tests/concurrency/`
  - How:
    - `t1_core_types_send_sync.rs`: compile-time `static_assertions` for `Plan`, executor types.
    - `t2_single_mutator_under_lock.rs`: spawn concurrent apply calls; assert only one mutates with lock metrics present.

- __Scheduling tiers defined (Bronze/Silver/Gold/Platinum)__
  - Where: `cargo/switchyard/TESTPLAN/scheduling_and_cost.md` and CI matrices
  - What: Curate test subsets per tier and wire into CI (time budgets: ≤2m/≤8m/≤20m/≤40m).

---

## Lower Priority (P2/P3)

- __EXDEV fallback atomic visibility details (REQ-F1)__
  - Where: `cargo/switchyard/tests/apply/exdev_atomic_visibility.rs`
  - How: Instrument fallback path to ensure no broken intermediate states are externally visible; assert facts indicate visibility-safe sequence.

- __Supported filesystems (REQ-F3)__
  - Where: `tests/environment/base4_runner.rs` (extend), or separate `tests/fs/platforms.rs`
  - How: Gated tests for xfs/btrfs/tmpfs where available, collecting non-blocking telemetry.

---

## Quick Commands

- __Run the Switchyard tests__

```
cargo test -p switchyard -- --nocapture
```

- __Generate/update golden fixtures__

```
python3 test_ci_runner.py --golden all --update
```

---

## Already Covered Highlights (reference)

- Plan sorting and stability: `tests/plan/sorting_many.rs`, `tests/plan/basic.rs`
- Preflight baselines and errors: `tests/preflight/baseline_ok.rs`, `tests/preflight/ownership_strict_without_oracle.rs`, `tests/preflight/rescue_exec_min_count.rs`
- Apply core paths: locks, smoke, EXDEV, provenance, attestations, best-effort restore: `tests/apply/*.rs`
- Rollback inversion: `tests/rollback/*.rs`
- SafePath: `tests/safepath/*.rs`
- Oracles: bounds recording, prune invariants (basic), audit schema: `tests/oracles/*.rs`, `tests/audit/*.rs`

Consult `TODO_TESTPLAN.md` for the authoritative, line-by-line checklist. This V2 document focuses on the actionable deltas to reach full coverage aligned with the TESTPLAN and SPEC.
