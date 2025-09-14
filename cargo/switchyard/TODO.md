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

Acceptance (for this section):

- Tests no longer use env for EXDEV/RESCUE; pass concurrently.
- EXDEV degraded/disallowed branches deterministic via per‑instance overrides only.

---

## 2) Atomic Hardening (RB5): dirfd fsync, Unique tmp, ENOENT‑only unlink, Byte‑safe CStrings

Purpose: tighten TOCTOU guarantees; reduce transient timing sensitivities observed by smoke/oracles; address RELEASE_BLOCKER_5.

- ⬜ `src/fs/atomic.rs`
  - Replace `fsync_parent_dir(path)` with a dirfd‑based fsync utility `fn fsync_dirfd(dirfd: &OwnedFd) -> io::Result<()>` using `rustix::fs::fsync`.
  - Thread dirfd to the fsync call sites (success and degraded fallback branches).
  - Change tmp naming from `.{fname}{TMP_SUFFIX}` to `.{fname}.{pid}.{ctr}{TMP_SUFFIX}` or short random suffix; ensure suffix length bound.
  - Build CStrings via bytes (`OsStrExt::as_bytes(..)` → `CString::new(..)`), avoid `to_str().unwrap_or("target")` for non‑UTF‑8 safety.
  - Restrict unlink ignores to ENOENT only; propagate other errors.
- ⬜ `src/fs/swap.rs`
  - Align unlink logic with ENOENT‑only ignores using `unlinkat` error mapping.
  - Ensure all fname/source CStrings are bytes‑safe.
- ⬜ Update comments/docs to reflect the normative sequence: `open_dir_nofollow → symlinkat(tmp) → renameat(tmp, final) → fsync(dirfd)`.

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

- ⬜ Review `src/api/apply/lock.rs` for WARN attempt emission with `lock_backend="none"`, `no_lock_manager=true`, `lock_attempts=0` (already present at ~97–111).
- ⬜ Ensure a single coherent attempt stream: keep current dual emission (WARN attempt then SUCCESS attempt) or guard for consumers; adjust tests if necessary.
- ⬜ Verify `locking::optional_no_manager_warn::warn_emitted_when_no_lock_manager_and_optional` passes.

Acceptance:

- WARN attempt visible with required fields under Optional+Allowed; no regressions in attempt/result summaries.

---

## 5) RB3 — fsync_ms in apply.result (Summary Level)

- ⬜ Confirm `src/api/apply/summary.rs::ApplySummary::perf(..)` sets top‑level `fsync_ms = total.swap`.
- ⬜ If tests require stricter semantics, optionally split timers into `rename_ms` and `fsync_delay_ms` and expose `fsync_ms = fsync_delay_ms`.
- ⬜ Verify `oracles::bounds_recording::bounds_recording` still passes across platforms.

Acceptance:

- Top‑level `fsync_ms` present; semantics documented; tests green.

---

## 6) RB4 — Schema v2 Compliance Audit (Global)

- ⬜ Audit all stage emissions via `StageLogger` to ensure required fields per schema branch are present:
  - `apply.attempt`, `apply.result` (per‑action + summary), `rollback.*`, `preflight.*`, `plan.*`, `prune.result` (see RB6).
- ⬜ Add a test helper to validate emitted facts against `SPEC/audit_event.v2.schema.json` for representative events.
- ⬜ Update or add golden fixtures as needed; keep redaction deterministic.

Acceptance:

- Schema v2 validation passes for representative samples in CI.

---

## 7) RB6 — Prune Result Fact Emission

- ⬜ Add an API‑layer wrapper (preferred) that calls `fs/backup/prune.rs::prune_backups(..)` and emits a `prune.result` fact via `StageLogger`.
  - Include `path`, `backup_tag`, `pruned_count`, `retained_count`, and relevant policy knobs.
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
- ⬜ Enrich `preflight.summary` with `rescue_profile` detail where reasonable.

Acceptance:

- Deterministic preflight behavior; summary includes rescue readiness details; mapping to `E_POLICY` correct when required and unavailable.

---

## 9) Test Migrations & Parallel Stability (Final)

- ⬜ Replace all env‑based simulations with `with_overrides()`.
- ⬜ Remove serial markers introduced solely to avoid env races; keep only where IO/race‑heavy or truly global resources exist.
- ⬜ Introduce a parallel stress suite in CI: run hot tests 5×; full suite 10×.

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
