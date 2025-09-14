# Switchyard TODO — Granular Execution Plan (with Release Blockers)

This file is the authoritative, ordered checklist to remediate parallel test flakes and address overlapping release blockers. It bundles two high‑value fixes at once: per‑instance overrides (removing env races) and atomic hardening (dirfd fsync, unique tmp, strict unlink, byte‑safe paths). It also includes verification and documentation work.

Status tags: ⬜ TODO · 🔶 In Progress · ✅ Done

---

## 0) Prereqs and Guard Rails

- ⬜ Establish a short‑lived branch `feat/overrides+atomic-hardening` targeting `cargo/switchyard`.
- ⬜ Ensure CI runs both single‑threaded and parallel lanes for `switchyard` crate.
- ⬜ Keep legacy env overrides behind a temporary, debug‑only feature until tests migrate; default OFF.
- ⬜ Add a CI checklist: golden fixtures updated; zero SKIP; golden diff gate on.

---

## 1) Per‑Instance Overrides (Eliminate Process‑Global Env Influence)

Purpose: remove cross‑test env leakage for EXDEV/RESCUE and make simulations explicit, instance‑scoped.

- ⬜ Create `src/api/overrides.rs`
  - Expose `#[derive(Clone, Debug, Default)] pub struct Overrides { pub force_exdev: Option<bool>, pub force_rescue_ok: Option<bool> }`.
  - Provide helper constructors: `Overrides::exdev(bool)`, `Overrides::rescue_ok(bool)`.
- ⬜ Plumb overrides into API
  - Edit `src/api/mod.rs`: add `overrides: Overrides` field and a `with_overrides(overrides: Overrides)` builder.
  - Default `overrides` to `Overrides::default()` in `ApiBuilder::build()`.
- ⬜ Replace env reads at call sites
  - `src/fs/atomic.rs::atomic_symlink_swap(..)`
    - Accept `force_exdev: Option<bool>` parameter (via API plumbing) and remove direct env reads.
    - Simulation remains post‑`renameat` decision point; inject `Err(Errno::XDEV)` only when `force_exdev == Some(true)`.
  - `src/policy/rescue.rs::verify_rescue_min(..)`
    - Consult `force_rescue_ok: Option<bool>` from API instead of env.
    - Map `Some(true) → Ok(..)`, `Some(false) → Err(RescueError::Unavailable)`, else run normal logic.
- ⬜ Keep a temporary debug feature for legacy env (off by default)
  - Add a `#[cfg(feature = "legacy-env-overrides")]` fallback to read env only when the feature is enabled.
  - Document this is transitional and will be removed once test migrations are complete.

Testing (for this section):

- ⬜ Unit tests: new `overrides.rs` behavior (default/no‑ops, basic setters).
- ⬜ Adjust integration tests that used env:
  - `tests/apply/exdev_degraded.rs`, `tests/apply/error_exdev.rs`, BDD steps (`tests/steps/*.rs`) → use `with_overrides(Overrides::exdev(true))` and remove env usage.
  - `tests/apply/error_policy.rs`, `tests/audit/preflight_summary_error_id.rs` → use `with_overrides(Overrides::rescue_ok(false))`.
- ⬜ Run `cargo test -p switchyard --test integration_tests -- --nocapture` (parallel + 5× stress on hot tests).

Status & Findings:

- 🔶 In Progress — Product currently gates env overrides to tests/opt-in only, but no instance-scoped overrides exist yet.
  - Code: `src/fs/atomic.rs::atomic_symlink_swap()` consults `SWITCHYARD_TEST_ALLOW_ENV_OVERRIDES` before reading `SWITCHYARD_FORCE_EXDEV`.
  - Code: `src/policy/rescue.rs::verify_rescue_min()` consults the same allow flag before reading `SWITCHYARD_FORCE_RESCUE_OK`.
- Rationale: This gating reduces accidental production influence but does not eliminate cross-test leakage in parallel runs. Instance-scoped `Overrides` remains necessary to fully solve flakes (see Failing Tests docs).

Acceptance (for this section):

- Tests no longer use env for EXDEV/RESCUE; pass concurrently.
- EXDEV degraded/disallowed branches deterministic via per‑instance overrides only.

---

## 2) Atomic Hardening (RB5): dirfd fsync, Unique tmp, ENOENT‑only unlink, Byte‑safe CStrings

Purpose: tighten TOCTOU guarantees; reduce transient timing sensitivities observed by smoke/oracles; address RELEASE_BLOCKER_5.

- ✅ `src/fs/atomic.rs`
  - Implemented dirfd‑based fsync `fsync_dirfd(&OwnedFd)` and switched success and degraded branches to use it.
  - Implemented unique tmp naming `.{fname}.{pid}.{ctr}{TMP_SUFFIX}` via an atomic counter and process id.
  - Built CStrings from bytes for `source` and final name; no lossy `to_str()` fallback in critical rename path.
  - Restricted tmp `unlinkat` to ignore only `ENOENT`; propagate other errors.
  - Code refs: `atomic_symlink_swap()`, helper `fsync_dirfd()` in `src/fs/atomic.rs`.
- ✅ `src/fs/swap.rs`
  - Updated capability‑handle `unlinkat(..)` sites to ignore only `ENOENT` and use bytes‑safe `CString` from `OsStr`.
  - Code refs: three `unlinkat` call sites in `src/fs/swap.rs`.
- ✅ Docs updated
  - Top‑of‑file comment in `src/fs/atomic.rs` now reflects `fsync(dirfd)`.

Testing (for this section):

- ⬜ Unit tests for tmp naming uniqueness (within a temp dir) and ENOENT handling.
- ⬜ Integration tests (existing): smoke invariants; bounds recording; ensure_symlink_success.
- ⬜ Add property test: repeated replace on the same target in a tight loop must always end with a symlink to the latest source; no panics; no leftover tmp.

Acceptance (for this section):

- `oracles::bounds_recording::bounds_recording` still passes; fsync_ms present.
- No leftover tmp litter under crash‑sim light (e.g., abort between symlinkat and renameat simulated via injected error).

---

## 3) RB1 Verification — EXDEV Degraded Fallback Correctness (via Overrides)

- ⬜ Test: fallback allowed (policy `allow_degraded_fs=true`)
  - Use `with_overrides(Overrides::exdev(true))`; assert per‑action `apply.result` includes `degraded=true` and `degraded_reason="exdev_fallback"`.
- ⬜ Test: fallback disallowed (policy `Fail`)
  - Use `with_overrides(Overrides::exdev(true))`; assert summary `apply.result` maps to `error_id=E_EXDEV` and appropriate `exit_code`.

Acceptance:

- Both branches deterministic and isolated; no dependency on env.

---

## 4) RB2 — Locking WARN Semantics (Optional + No Manager)

- ✅ Reviewed `src/api/apply/lock.rs` and verified WARN attempt emission under Optional+Allowed unlocked path.
  - Code: in `acquire(...)`, when `LockingPolicy::Optional` and `allow_unlocked_commit=true`, emits `apply.attempt` WARN with `lock_backend="none"`, `no_lock_manager=true`, `lock_attempts=0`.
  - Test presence: `tests/locking/optional_no_manager_warn.rs` covers this scenario.
  - Decision: Keep the WARN attempt. Observed dual-emission patterns are intentional for visibility; no consumer harm identified.
  - Action: ✅ No change required; semantics match SPEC §2.5.

Acceptance:

- WARN attempt visible with required fields under Optional+Allowed; no regressions in attempt/result summaries.

---

## 5) RB3 — fsync_ms in apply.result (Summary Level)

- ✅ Confirmed `src/api/apply/summary.rs::ApplySummary::perf(..)` sets top‑level `fsync_ms = total.swap`.
  - Code: `ApplySummary::perf()` inserts `perf { hash_ms, backup_ms, swap_ms }` and `fsync_ms = total.swap`.
  - Executor attaches per‑action `fsync_ms` too: `src/api/apply/executors/ensure_symlink.rs` merges `fsync_ms` into `apply.result`.
  - Tests: bounds recording oracle present in `tests/oracles/`.
  - Action: ✅ No change required.

Acceptance:

- Top‑level `fsync_ms` present; semantics documented; tests green.

---

## 6) RB4 — Schema v2 Compliance Audit (Global)

- ⬜ Audit all stage emissions via `StageLogger` to ensure required fields per schema branch are present:
  - `apply.attempt`, `apply.result` (per‑action + summary), `rollback.*`, `preflight.*`, `plan.*`, `prune.result` (see RB6).
- ⬜ Add a test helper to validate emitted facts against `SPEC/audit_event.v2.schema.json` for representative events.
- ⬜ Update or add golden fixtures as needed; keep redaction deterministic.

Status & Findings:

- Verified Stage set includes `plan`, `preflight`, `preflight.summary`, `apply.attempt`, `apply.result`, `rollback`, `rollback.summary`, `prune.result` in `src/logging/audit.rs`.
- SCHEMA v2 constant present (`SCHEMA_VERSION=2`). Envelope fields populated centrally in `redact_and_emit()`.
- Work remaining: schema validation helper + golden review.

Acceptance:

- Schema v2 validation passes for representative samples in CI.

---

## 7) RB6 — Prune Result Fact Emission

- ✅ Add an API‑layer wrapper (preferred) that calls `fs/backup/prune.rs::prune_backups(..)` and emits a `prune.result` fact via `StageLogger`.
  - Implemented in `src/api/mod.rs::Switchyard::prune_backups(...)` with fields: `path`, `backup_tag`, policy limits, `pruned_count`, `retained_count`.
- ⬜ Add integration tests for prune:
  - `prune_by_count`, `prune_by_age`, and verify `prune.result` facts.

Acceptance:

- `prune.result` emitted and schema‑valid; golden updated.

---

## 8) RB7 — Rescue/Tooling Readiness (Preflight)

- ⬜ Extend preflight tests:
  - BusyBox present path
  - GNU subset present path
  - `exec_check=true` variations
- ✅ `preflight.summary` includes `rescue_profile` field.
  - Code: `src/api/preflight/mod.rs` builds summary with `rescue_profile = available|none` and STOP mapping to `E_POLICY` on failure; emits via `StageLogger::preflight_summary()`.

Acceptance:

- Deterministic preflight behavior; summary includes rescue readiness details; mapping to `E_POLICY` correct when required and unavailable.

---

## 9) Test Migrations & Parallel Stability (Final)

- ⬜ Replace all env‑based simulations with `with_overrides()`.
- ⬜ Remove serial markers introduced solely to avoid env races; keep only where IO/race‑heavy or truly global resources exist.
- ⬜ Introduce a parallel stress suite in CI: run hot tests 5×; full suite 10×.

Status & Findings:

- Product gating is in place, and tests use `ScopedEnv` + allow flag; flakes remain possible under parallelism due to process‑global env. Full migration awaits `Overrides` API from §1.

Acceptance:

- Zero flakes across 10 full runs under parallel threading.

---

## 10) Documentation & Developer Reflection (Impact on Future Development)

- ⬜ Update `docs/testing/TESTING_POLICY.md`
  - Codify: No process‑global env overrides for simulation; use `with_overrides()`.
  - Clarify: Locking is Required in production; Optional+Allowed path must log WARN attempts.
  - Document: Normative atomic sequence, dirfd‑fsync, unique tmp naming.
- ⬜ Add `docs/overrides.md` describing the Overrides API, intended for tests and controlled simulations only.
- ⬜ Update `RELEASE_BLOCKER_1.md` and `RELEASE_BLOCKER_5.md` with references to the fixes landing (✅) and where the logic lives.
- ⬜ Record a post‑mortem note in `FAILING_TESTS/` linking to this TODO and summarizing the flake elimination path.

Reflection (why this helps future dev):

- Per‑instance overrides make simulations explicit/documented, preventing accidental global side effects in large suites or real deployments.
- Atomic hardening reduces subtle timing/FS sensitivities and provides a stronger foundation for future features (e.g., more actions beyond symlinks).
- Centralized schema validation prevents silent drift as we expand stages/facts.
- Clear locking semantics + WARN ensures operators get consistent signals without relying on doc tribal knowledge.

---

## Structural Findings (Upstream/Downstream)

- __Global env overrides are a structural flake source__
  - Upstream: `src/fs/atomic.rs` and `src/policy/rescue.rs` read env when allowed; tests toggle env.
  - Downstream: `apply` facts and summary semantics change across tests in the same process.
  - Resolution: Implement instance‑scoped `Overrides` in API and plumb to call sites; keep legacy env behind feature flag and test allow‑flag during transition.

- __Locking semantics are correct; tests bypass them__
  - Upstream: `src/api/apply/lock.rs` enforces lock in Commit unless Optional+Allowed.
  - Downstream: Parallel flakes can still occur when tests opt into `allow_unlocked_commit=true`.
  - Resolution: Keep tests that assert smoke behavior using a `TestLockManager`; default production posture already matches SPEC §2.5.

- __Atomicity hardening now matches SPEC__
  - Upstream: `fsync(dirfd)` and unique tmp names implemented; ENOENT‑only unlink; bytes‑safe CStrings.
  - Downstream: Reduces TOCTOU windows and naming collisions; improves stability under parallel suites.

## Changelog (this pass)

- Implemented dirfd‑based fsync and unique tmp naming; tightened unlink error handling; bytes‑safe CStrings
  - Files: `src/fs/atomic.rs`, `src/fs/swap.rs`.
- Verified and documented prune.result wrapper and preflight `rescue_profile`.
- Updated TODO statuses and added code citations.

## Commands Cheat‑Sheet (for verification)

```bash
# Full suite (parallel)
cargo test -p switchyard -q

# Single-thread deterministic
RUST_TEST_THREADS=1 cargo test -p switchyard -- --nocapture

# Stress hot tests 5×
for i in {1..5}; do \
  cargo test -p switchyard --test integration_tests -- \
    apply::smoke_ok::smoke_runner_ok_yields_success_and_no_rollback \
    apply::attestation_apply_success::attestation_fields_present_on_success_and_masked_after_redaction \
    apply::ownership_strict_with_oracle::e2e_apply_017_ownership_strict_with_oracle_present \
    -- --nocapture; \
done
```

---

## Appendix A) Evidence & Code Citations (Deep Research)

- __Env override gating (current) — verified__
  - `src/fs/atomic.rs::atomic_symlink_swap(...)`
    - Reads `SWITCHYARD_FORCE_EXDEV` only when `cfg(test)` or `SWITCHYARD_TEST_ALLOW_ENV_OVERRIDES=1`.
    - Simulation injection occurs post-`renameat(...)` result (correct placement for degraded branch testing).
  - `src/policy/rescue.rs::verify_rescue_min(...)`
    - Reads `SWITCHYARD_FORCE_RESCUE_OK` only under the same allow flag gate.
  - Tests that rely on env toggles (to be migrated):
    - `tests/apply/exdev_degraded.rs`
    - `tests/apply/error_exdev.rs`
    - `tests/apply/error_policy.rs`
    - `tests/steps/apply_steps.rs`
    - `tests/steps/plan_steps.rs`

- __Atomic swap hardening — implemented this pass__
  - Dirfd fsync helper and usage:
    - `src/fs/atomic.rs::fsync_dirfd(&OwnedFd)`
    - `atomic_symlink_swap(...)` uses `fsync(dirfd)` on success and degraded fallback
  - Unique tmp naming:
    - `.{fname}.{pid}.{ctr}{TMP_SUFFIX}` via `NEXT_TMP_COUNTER: AtomicU64`
  - ENOENT-only unlink ignores:
    - `atomic.rs` and `swap.rs` now match on `Errno::NOENT` only; other errors bubble up
  - Bytes-safe CStrings:
    - `CString::new(OsStrExt::as_bytes(...))` for `source`, `target` names and swap sites

- __Restore path hardening — implemented this pass__
  - `src/fs/restore/steps.rs` moved from `fsync_parent_dir(path)` to `rustix::fs::fsync(&dirfd)`
  - Converted `CString` construction to bytes-safe and limited unlink ignores to ENOENT

- __Locking behavior and WARN semantics — verified__
  - `src/api/apply/lock.rs::acquire(...)` emits WARN `apply.attempt` when `LockingPolicy::Optional` and `allow_unlocked_commit=true`
  - Matching tests exist under `tests/locking/`

- __Prune result emission — verified__
  - `src/api/mod.rs::Switchyard::prune_backups(...)` emits `prune.result` with `path`, `backup_tag`, retention counts

- __Preflight rescue summary — verified__
  - `src/api/preflight/mod.rs` emits `preflight.summary` with `rescue_profile` and maps STOP to `E_POLICY`

- __Schema v2 envelope and stages — verified__
  - `src/logging/audit.rs` has `SCHEMA_VERSION = 2`, emits required envelope fields in `redact_and_emit`, and defines stages including `prune.result`

---

## Appendix B) Overrides API Design Sketch (Upstream/Downstream Wiring)

- __Public type__
  - `src/api/overrides.rs`
    - `#[derive(Clone, Debug, Default)] pub struct Overrides { pub force_exdev: Option<bool>, pub force_rescue_ok: Option<bool> }`
    - Helpers: `Overrides::exdev(bool)`, `Overrides::rescue_ok(bool)`

- __Builder plumbing__
  - `src/api/mod.rs::Switchyard { overrides: Overrides, .. }`
  - `ApiBuilder::with_overrides(overrides: Overrides) -> Self`
  - Default `Overrides::default()` in `ApiBuilder::build()`

- __Call-site usage__
  - `apply` path for EnsureSymlink:
    - Executor passes `api.overrides.force_exdev` down to `fs/swap.rs::replace_file_with_symlink(...)` and into `fs/atomic.rs::atomic_symlink_swap(...)`
    - `atomic_symlink_swap(...)` injects EXDEV only when `force_exdev == Some(true)`; otherwise never injects
  - `preflight`/policy path for rescue:
    - `policy/rescue.rs::verify_rescue_min(exec_check, min_count, overrides: &Overrides)` consults `force_rescue_ok` if set; else real probe logic

- __Legacy env feature (temporary)__
  - `#[cfg(feature = "legacy-env-overrides")]` retains gated env reads for a deprecation window
  - Default features leave this OFF; tests switch to `with_overrides()`

---

## Appendix C) Test Migration Plan (Env → Overrides)

- __Target test files to update__
  - `tests/apply/exdev_degraded.rs`
  - `tests/apply/error_exdev.rs`
  - `tests/apply/error_policy.rs`
  - `tests/steps/apply_steps.rs`
  - `tests/steps/plan_steps.rs`

- __Migration steps__
  - Replace `ScopedEnv` usage of `SWITCHYARD_FORCE_EXDEV`/`SWITCHYARD_FORCE_RESCUE_OK` with `api.with_overrides(Overrides::exdev(true))` or `.rescue_ok(false)` as appropriate
  - Remove the need to set `SWITCHYARD_TEST_ALLOW_ENV_OVERRIDES=1` in these tests
  - Remove `#[serial]` markers used only to avoid env races; keep serial where truly global resources are exercised
  - Where necessary to stabilize smoke checks under parallel suite execution, opt into `tests/helpers/lockmgr.rs::TestLockManager`

---

## Appendix D) SPEC Traceability (Claims → Code)

- __REQ-A1..A3 (Atomicity)__
  - Atomic swap sequence enforced via dirfd handles: `open_dir_nofollow → symlinkat(tmp) → renameat → fsync(dirfd)` (`src/fs/atomic.rs`)
  - Unique tmp reduces concurrency collisions; ENOENT-only unlink reduces silent masking of errors

- __REQ-R1..R5 (Rollback)__
  - Restore helpers in `src/fs/restore/steps.rs` updated to dirfd fsync; rollback planning in `src/api/mod.rs::plan_rollback_of`

- __REQ-S1..S6 (Safety Preconditions)__
  - SafePath usage throughout mutating public APIs; rescue probe verified in preflight

- __REQ-O1..O8 (Observability & Audit)__
  - StageLogger emits v2 envelope; `apply.result` perf `fsync_ms` recorded; prune.result implemented; summary error mapping present

- __REQ-L1..L5 (Locking)__
  - Optional+Allowed unlocked path emits WARN attempt; Required path denies with E_LOCKING (policy-governed)

- __REQ-F1..F3 (Filesystems & Degraded)__
  - EXDEV degraded fallback telemetry; per-instance overrides pending to remove env reliance

---

## Appendix E) Risks, Edge-cases, and Robustness Notes

- __Non-UTF-8 paths__
  - Resolved by bytes-based `CString` construction in atomic/swap/restore paths

- __Cross-filesystem symlink replacement__
  - Degraded branch uses unlink+symlinkat; now ENOENT-only ignore prevents masking other errors

- __Crash windows and tmp litter__
  - Unique tmp + ENOENT-only unlink reduce stray files; property test (Section 2 Testing) will assert no leftover tmp under stress

- __Audit schema drift__
  - Add JSON Schema v2 validation helper to CI; ensure golden fixtures updated atomically in the branch

---

## Next Steps (Actionable)

- ⬜ Implement `src/api/overrides.rs` and builder plumbing; thread to `fs/atomic.rs` and `policy/rescue.rs`
- ⬜ Migrate EXDEV/RESCUE tests to `with_overrides()`; remove unnecessary serial markers
- ⬜ Add JSON Schema v2 validation helper and refresh goldens
- ⬜ Add property test for repeated ensure_symlink loop (no tmp litter; latest link wins)
- ⬜ Follow-up: migrate remaining `fsync_parent_dir(path)` users in backup/prune to dirfd-based fsync where feasible (`fs/backup/snapshot.rs`, `fs/backup/prune.rs`, `fs/backup/sidecar.rs`)
