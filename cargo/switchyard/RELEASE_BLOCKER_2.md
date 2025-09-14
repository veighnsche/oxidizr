# RELEASE\_BLOCKERS.md

*Last updated: 2025-09-14*

This document tracks **must-fix issues before cutting the next Release Candidate** for `cargo/switchyard`.

Status tags: ⬜ TODO · 🔶 In Progress · ✅ Done

---

## Blocker 1 — EXDEV degraded fallback not engaged (simulated)

**Components:** `src/fs/atomic.rs`
**Spec/Reqs:** REQ-F1 (atomic fallback safety), REQ-F2 (degraded mode telemetry)
**Impact:** Cross-filesystem swaps hard-fail instead of taking degraded path; breaks safety contract.
**Failing tests:**

* `apply::exdev_degraded::exdev_degraded_fallback_sets_degraded_true`
* `apply::error_exdev::ensure_symlink_emits_e_exdev_when_fallback_disallowed`

**Root cause:** Early return on `SWITCHYARD_FORCE_EXDEV=1` bypasses degraded branch.
**Status:** ⬜ TODO

> 🔎 Research Addendum — Big Think LLM

* Root Cause Analysis:
  * The EXDEV simulation used to short-circuit before `renameat(...)`, preventing the `Errno::XDEV && allow_degraded` branch from executing. Current code injects the simulated `Errno::XDEV` immediately after the `renameat` call so the degraded branch runs when policy allows.
  * Implicated modules/functions:
    * `src/fs/atomic.rs`: `atomic_symlink_swap()` EXDEV injection and branching (lines 92–101, 102–117).
    * `src/api/apply/executors/ensure_symlink.rs`: per-action telemetry fields `degraded`, `degraded_reason`, `fsync_ms` (lines 142–174, esp. 146–154, 149).
    * `src/fs/swap.rs`: `replace_file_with_symlink()` connects high-level policy to `atomic_symlink_swap()` (lines 17–41, 140–142).
* Dependencies & Scope:
  * Code edits likely limited to `src/fs/atomic.rs` if we add guardrails (e.g., feature-gated simulation) and cleanup on error paths.
  * Observability is emitted in the executor; no schema changes required for degraded telemetry.
  * Affected tests: `tests/apply/exdev_degraded.rs`, `tests/apply/error_exdev.rs` use `SWITCHYARD_FORCE_EXDEV=1`.
  * SPEC/TESTPLAN: REQ-F1, REQ-F2; TESTPLAN `environment_matrix.md` “cross-fs simulated via SWITCHYARD_FORCE_EXDEV=1”.
* Complexity Estimate:
  * ~20–40 LOC, 1–2 modules; difficulty: low. No public API changes.
* Options:
  * Option A: Keep env-var simulation as-is (post-`renameat` injection), add best‑effort tmp cleanup on all error paths and ensure `fsync_ms` measured consistently.
  * Option B: Gate EXDEV simulation behind a `test-overrides` Cargo feature so the env var is ignored in production builds. Tradeoff: adds a feature flag and cfg guards but reduces prod risk.
* Risks:
  * Environment variable could affect production behavior if set; gating mitigates this.
  * Temp name collision/non-UTF‑8 handling in `atomic.rs` may cause secondary failures under concurrency or exotic filenames; see “Discovered During Research”.
* Evidence:
  * Code cites: `cargo/switchyard/src/fs/atomic.rs`:92–101, 102–117; `cargo/switchyard/src/api/apply/executors/ensure_symlink.rs`:142–174; `cargo/switchyard/src/fs/swap.rs`:17–41, 140–142.
  * Test refs: `tests/apply/exdev_degraded.rs`, `tests/apply/error_exdev.rs` (env var set at lines 56 and 54 respectively).
  * SPEC/TESTPLAN refs: REQ-F1/REQ-F2, TESTPLAN `environment_matrix.md` lines 9–11, 59–61.

---

## Blocker 2 — No WARN when locking is Optional and no manager is configured

**Components:** `src/api/apply/lock.rs`
**Spec/Reqs:** REQ-L2 (warn when no lock manager)
**Impact:** Ops loses audit signal; violates logging contract.
**Failing test:** `locking::optional_no_manager_warn::warn_emitted_when_no_lock_manager_and_optional`
**Status:** ⬜ TODO

> 🔎 Research Addendum — Big Think LLM

* Root Cause Analysis:
  * Previously, no WARN was emitted when locking was Optional and no manager configured. Current implementation emits an `apply.attempt` WARN with `lock_backend="none"`, `no_lock_manager=true`, and `lock_attempts=0` when policy explicitly allows unlocked commit.
  * Implicated modules/functions:
    * `src/api/apply/lock.rs`: `acquire()` emits WARN via `StageLogger` (lines 97–111).
    * `src/api/apply/mod.rs`: always emits a subsequent `apply.attempt` success summary (lines 83–89), which can result in two apply.attempt events (WARN + SUCCESS).
* Dependencies & Scope:
  * Emission path is localized to `lock.rs`; no schema changes. Schema requires `lock_backend` and `lock_attempts` for non-action `apply.attempt` events (satisfied by both events).
  * Affected tests: `tests/locking/optional_no_manager_warn.rs` asserts WARN presence with required fields.
  * SPEC refs: REQ-L2, REQ-L5.
* Complexity Estimate:
  * ~10–20 LOC; difficulty: low. No public API changes.
* Options:
  * Option A: Keep both events (WARN + SUCCESS) for parity; document that WARN is the audit signal and SUCCESS is the standard attempt summary.
  * Option B: Suppress the later SUCCESS summary when a WARN was already emitted (requires threading a flag back to `apply::run`); reduces duplicate events but adds coupling between `lock.rs` and orchestrator.
* Risks:
  * Duplicate `apply.attempt` events may complicate dashboards; ensure consumers de-duplicate by `decision` or `seq`.
  * Consistency with schema v2: verify both events include required fields (`lock_backend`, `lock_attempts`).
* Evidence:
  * Code cites: `cargo/switchyard/src/api/apply/lock.rs`:97–111; `cargo/switchyard/src/api/apply/mod.rs`:83–89.
  * Test refs: `tests/locking/optional_no_manager_warn.rs`.
  * SPEC refs: REQ-L2, REQ-L5 in `SPEC/requirements.yaml`.

---

## Blocker 3 — Missing `fsync_ms` in `apply.result`

**Components:** `src/api/apply/summary.rs`
**Spec/Reqs:** REQ-BND1 (bounds recording)
**Impact:** Perf telemetry incomplete.
**Failing test:** `oracles::bounds_recording::bounds_recording`
**Status:** ⬜ TODO

> 🔎 Research Addendum — Big Think LLM

* Root Cause Analysis:
  * Prior summaries missed a top-level `fsync_ms`. Current `ApplySummary::perf(...)` injects a top-level `fsync_ms` mirroring `perf.swap_ms`; per‑action `apply.result` already includes `fsync_ms` in `ensure_symlink`.
  * Implicated modules/functions:
    * `src/api/apply/summary.rs`: inserts top-level `fsync_ms` (lines 21–33).
    * `src/api/apply/executors/ensure_symlink.rs`: sets `fsync_ms` in success/failure payloads (lines 142–175, esp. 149 and 102–103).
* Dependencies & Scope:
  * No public API change; summary fields extended. DryRun path still emits `fsync_ms=0` as expected by oracles.
  * Affected tests: `tests/oracles/bounds_recording.rs` (asserts presence of `fsync_ms` and `lock_wait_ms` on summary event in DryRun).
  * SPEC refs: REQ-BND1.
* Complexity Estimate:
  * Already implemented; verification-only. Difficulty: low.
* Options:
  * Option A: Keep both `perf.swap_ms` and top-level `fsync_ms` for compatibility with existing consumers and tests.
  * Option B: Rename fields to converge long term (schema change risk; not advised before RC).
* Risks:
  * Naming drift between action-level and summary fields; ensure documentation aligns (`fsync_ms` = directory fsync bound after rename).
* Evidence:
  * Code cites: `cargo/switchyard/src/api/apply/summary.rs`:21–33; `cargo/switchyard/src/api/apply/executors/ensure_symlink.rs`:142–175.
  * Test refs: `cargo/switchyard/tests/oracles/bounds_recording.rs` lines 85–95.
  * SPEC refs: REQ-BND1 in `SPEC/requirements.yaml`.

---

## Blocker 4 — Facts/schema v2 compliance (global)

**Components:** StageLogger events across stages
**Spec/Reqs:** REQ-VERS1 (schema\_version=v2), REQ-O1/O5
**Impact:** Missing required fields break schema validation.
**Failing test:** `sprint_acceptance-0001::golden_two_action_plan_preflight_apply`
**Status:** 🔶 In Progress

> 🔎 Research Addendum — Big Think LLM

* Root Cause Analysis:
  * Historical gaps: certain preflight events lacked required fields (`path`, `current_kind`, `planned_kind`) and some apply attempt/result events missed v2 envelope consistency.
  * Current state:
    * `preflight` rows/events include required fields via `RowEmitter` (lines 66–71 in `row_emitter.rs`).
    * v2 envelope injected centrally (schema_version, ts, plan_id, run_id, event_id) by `logging/audit.rs`.
* Dependencies & Scope:
  * Validate all stages against `SPEC/audit_event.v2.schema.json` in CI; add a test helper to iterate over emitted events and validate with the schema.
  * Files: `src/api/preflight/row_emitter.rs`, `src/api/preflight/mod.rs`, `src/api/apply/*`, `src/logging/audit.rs`, schemas under `SPEC/`.
* Complexity Estimate:
  * Tests-only path: ~100–150 LOC; difficulty: medium. No runtime behavior change.
* Options:
  * Option A: Add a test utility to validate every emitted JSON fact against the v2 schema; gate in CI.
  * Option B: Add compile-time checks/macros for common required fields per stage (higher complexity; long-term win).
* Risks:
  * Over-strict schema checks can cause flakes (e.g., optional fields varying by policy). Keep `additionalProperties: true` and validate only `required` per stage.
* Evidence:
  * Code cites: `cargo/switchyard/src/api/preflight/row_emitter.rs`:65–71; `cargo/switchyard/src/logging/audit.rs`:256–335; `SPEC/audit_event.v2.schema.json`.
  * Test refs: `tests/sprint_acceptance-0001.rs` (schema failure cited in BUGS.md).
  * SPEC refs: REQ-VERS1, REQ-O1/O5 in `SPEC/requirements.yaml`.

---

## Blocker 5 — TOCTOU/atomicity invariants asserted

**Components:** `fs/atomic.rs` (syscall sequence), E2E oracles
**Spec/Reqs:** REQ-TOCTOU1, REQ-A2
**Impact:** Atomicity claims not defensible without invariants.
**Status:** ⬜ TODO

> 🔎 Research Addendum — Big Think LLM

* Root Cause Analysis:
  * Invariants (“open parent nofollow → symlinkat/tmp → renameat → fsync(parent)”) are implemented, but not asserted end-to-end. Additionally, `fsync_parent_dir()` reopens by path rather than using the safe `dirfd`, introducing a small TOCTOU window.
  * Implicated modules/functions:
    * `src/fs/atomic.rs`: `open_dir_nofollow()`, `atomic_symlink_swap()`, `fsync_parent_dir()` (lines 22–38, 58–117, 45–51).
    * `src/fs/swap.rs`: pre-delete via `unlinkat` and orchestration (lines 75–87, 100–111, 128–139).
* Dependencies & Scope:
  * Tests: Add E2E that asserts resulting target points to source and that a parent-dir fsync fact is recorded; optionally add syscall-order oracles in debug builds.
  * Code: Consider passing `&OwnedFd` to a new `fsync_dirfd(&OwnedFd)` to avoid reopening by path.
  * SPEC refs: REQ-TOCTOU1, REQ-A2.
* Complexity Estimate:
  * Tests-only: ~120–200 LOC; refactor to use `dirfd` for fsync: ~20–40 LOC. Difficulty: low→medium.
* Options:
  * Option A: Add E2E “EnsureSymlink success oracle” test; continue using current `fsync_parent_dir()`.
  * Option B: Change fsync to use `rustix::fs::fsync(&dirfd)` and thread the handle; strictly reduces TOCTOU exposure and aligns with normative sequence.
* Risks:
  * Changing fsync path may affect portability; ensure rustix coverage across targets.
  * Tests that assume timing semantics may be flaky; assert presence of facts rather than wall-clock thresholds.
* Evidence:
  * Code cites: `cargo/switchyard/src/fs/atomic.rs`:22–38, 45–51, 58–117; `cargo/switchyard/src/fs/swap.rs`:75–87, 100–111, 128–139.
  * SPEC refs: `SPEC/requirements.yaml` REQ-TOCTOU1, REQ-A2.

---

## Blocker 6 — Prune safety invariants complete (if prune is in release scope)

**Components:** `src/fs/backup/prune.rs`
**Spec/Reqs:** REQ-PN2, REQ-PN3
**Impact:** Risk of silent backup corruption.
**Status:** ⬜ TODO

> 🔎 Research Addendum — Big Think LLM

* Root Cause Analysis:
  * `prune_backups(...)` removes payload and sidecar and fsyncs the parent, but does not emit a `prune.result` event required by REQ-PN3.
  * Implicated modules/functions:
    * `src/fs/backup/prune.rs`: delete loop and fsync (lines 117–129), return type `PruneResult` (lines 130–134).
    * `src/logging/audit.rs`: `StageLogger::prune_result()` exists but is unused by prune.
* Dependencies & Scope:
  * Option requires an audit context to emit facts; either lift prune into an orchestrated API layer that has `StageLogger`, or pass a minimal emitter into prune (API change).
  * Tests: Add E2E to verify both PN2 (parent fsync) and PN3 (result facts with `pruned_count`, `retained_count`).
  * SPEC refs: REQ-PN2, REQ-PN3.
* Complexity Estimate:
  * Emission in higher layer: ~60–100 LOC; exposing emitter to prune: ~40–80 LOC but changes API. Difficulty: medium.
* Options:
  * Option A: Add `fs::backup::orchestrate_prune(...)` in API layer that calls `prune_backups(...)` and emits `prune.result` via `StageLogger`.
  * Option B: Introduce a trait-bound emitter parameter to `prune_backups(...)` (breaking change) to emit facts directly.
* Risks:
  * API surface change; decide whether prune is part of public API or an internal helper.
  * Double-emission risk if both low-level and high-level emit.
* Evidence:
  * Code cites: `cargo/switchyard/src/fs/backup/prune.rs`:117–134; `cargo/switchyard/src/logging/audit.rs`:141–144, 146–154.
  * SPEC refs: `SPEC/requirements.yaml` REQ-PN2, REQ-PN3; `SPEC/audit_event.v2.schema.json` stage `prune.result` required fields.

---

## Blocker 7 — Rescue/fallback readiness (if guarantees are advertised)

**Components:** Preflight/tooling shims
**Spec/Reqs:** REQ-RC1, REQ-RC3
**Impact:** Restore promises not credible without tooling presence verified.
**Status:** ⬜ TODO

> 🔎 Research Addendum — Big Think LLM

* Root Cause Analysis:
  * Preflight verifies rescue readiness and emits a summary field `rescue_profile`, but gating and telemetry must match policy nuances (`require`, `exec_check`, `min_count`). Dry-run facts must remain deterministic.
  * Implicated modules/functions:
    * `src/api/preflight/mod.rs`: rescue verification and summary emission (lines 48–55, 171–221).
    * `src/policy/rescue.rs`: `verify_rescue_*` functions with test overrides and exec checks (lines 15–42, 44–89).
* Dependencies & Scope:
  * Ensure summary emits `error_id=E_POLICY` and `summary_error_ids` when gating fails; currently implemented (lines 186–209).
  * Tests: Add Base-1/2 scenarios per TESTPLAN; ensure env override `SWITCHYARD_FORCE_RESCUE_OK` is gated for tests only if needed.
  * SPEC refs: REQ-RC1, RC2, RC3.
* Complexity Estimate:
  * Mostly verification: ~60–120 LOC tests; difficulty: low.
* Options:
  * Option A: Keep current behavior; add comprehensive tests for require/exec/min_count combinations.
  * Option B: Add a consolidated `rescue_profile` object with richer detail (breaks goldens; defer post-RC).
* Risks:
  * Overbroad environment overrides (`SWITCHYARD_FORCE_RESCUE_OK`) in prod could mask issues; optionally feature-gate.
* Evidence:
  * Code cites: `cargo/switchyard/src/api/preflight/mod.rs`:48–55, 171–221; `cargo/switchyard/src/policy/rescue.rs`:15–42, 44–89.
  * SPEC refs: `SPEC/requirements.yaml` REQ-RC1/RC2/RC3.
  * Modules/files: `src/policy/rescue.rs`, `src/api/preflight/mod.rs` and `row_emitter.rs` (per-action fields), `SPEC/preflight.yaml`.
  * Tests: Add cases for BusyBox present, GNU subset present, and exec_check variations; assert preflight.summary fields and error mapping.

* Complexity Estimate:
  * Small–Medium (≈60–120 LOC) primarily in tests and facts enrichment.
* Options:
  * Option A: Keep current checks and enrich preflight.summary with additional context (e.g., which profile chosen) for operator visibility.
  * Option B: Add deeper functional checks (e.g., `--version` probes) behind a feature flag; higher confidence but slower.
* Risks:
  * Environment dependence in CI; use env overrides to keep tests deterministic.
* Evidence:
  * Code cites: `src/policy/rescue.rs`: ~65–89; `src/api/preflight/mod.rs`: ~48–56, ~178–221.
  * SPEC refs: REQ-RC1, REQ-RC3 (`SPEC/requirements.yaml`).

---

## Recently Resolved (2025-09-14)

✅ provenance-completeness
✅ preflight-rescue-verification
✅ preflight-backup-tag-handling
✅ preflight-exec-check-handling
✅ preflight-coreutils-tag-handling
✅ preflight-mount-check-notes
✅ lockmanager-required-production
✅ partial-restoration-facts
✅ smoke-invariants

---

## Notes / Non-blocking

* ENAMETOOLONG in `environment::base4_runner::envrunner_base4_weekly_platinum` → test harness issue.
* Extended FS matrix (xfs/btrfs/tmpfs acceptance) — track under REQ-F3 (Platinum).

## CI Gates

* [ ] Golden fixtures updated
* [ ] Zero-SKIP enforced
* [ ] Golden diff gate (byte-identical after redaction)

---

## Discovered During Research

1) Atomic temp path uniqueness and non-UTF-8 filename safety

> 🔎 Research Addendum — Big Think LLM

* Root Cause Analysis:
  * Deterministic tmp names like `.{fname}{TMP_SUFFIX}` can collide under concurrency and leave litter on crash.
  * `to_str().unwrap_or("target")` on filenames drops non-UTF-8 segments, potentially operating on the wrong path.
* Dependencies & Scope:
  * Modules/files: `src/fs/atomic.rs` (tmp name, fsync via dirfd), `src/fs/swap.rs` (unlink by UTF-8 fname at ~78–85, ~103–110, ~131–137).
* Complexity Estimate:
  * Medium (≈120–200 LOC) to switch to unique tmp (PID + counter/UUID), byte-based CStrings, ENOENT-only unlink handling, and dirfd fsync.
* Options:
  * Option A: Minimal fix — ENOENT-only ignore on unlink and tmp cleanup on all error branches.
  * Option B: Full hardening — unique tmp names, bytes-safe `CString::new(OsStrExt::as_bytes(...))`, fsync via existing `dirfd`.
* Risks:
  * Requires careful testing for non-UTF-8 filenames and concurrent operations.
* Evidence:
  * Code cites: `src/fs/atomic.rs`: ~64–76, ~87–116; `src/fs/swap.rs`: ~78–85, ~103–111, ~131–139.
  * SPEC refs: REQ-A2 (no broken/missing path), REQ-TOCTOU1.

2) Prune result fact emission

> 🔎 Research Addendum — Big Think LLM

* Root Cause Analysis:
  * `prune_backups(...)` lacks `prune.result` event emission. Without it, schema v2 compliance for prune is unmet and operators lack observability.
* Dependencies & Scope:
  * Modules/files: `src/fs/backup/prune.rs`, `src/logging/audit.rs` (StageLogger).
* Complexity Estimate: Small–Medium (≈60–120 LOC).
* Options: Emit from fs-layer vs API-layer wrapper (see Blocker 6 options).
* Risks: Golden drift; ensure CI gates updated.
* Evidence: `SPEC/audit_event.v2.schema.json` `stage_prune_result` branch; `src/fs/backup/prune.rs` ~117–133.
