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
  * The EXDEV simulation must occur at the `renameat` decision point so that the `Errno::XDEV && allow_degraded` branch runs. Current implementation injects `Err(Errno::XDEV)` after the real `renameat` result, which is correct for exercising degraded fallback.
  * Code paths: `src/fs/atomic.rs` at `atomic_symlink_swap(...)` constructs tmp link and does `renameat`, then:
    * Simulation: `if SWITCHYARD_FORCE_EXDEV=="1" { rename_res = Err(Errno::XDEV) }` (lines ~91–101).
    * Fallback match arm: `Err(e) if e == Errno::XDEV && allow_degraded => { ... symlinkat ... fsync_parent_dir(...) }` (lines ~108–114).
  * Per-action telemetry (degraded=true) is set in `src/api/apply/executors/ensure_symlink.rs` where `replace_file_with_symlink(...)` returns `(degraded_used, fsync_ms)` and fields `degraded`, `degraded_reason`, and `fsync_ms` are emitted (lines ~142–153).
* Dependencies & Scope:
  * Modules/files:
    * `src/fs/atomic.rs` — EXDEV simulation location, fallback branch behavior, tmp handling, fsync of parent.
    * `src/fs/swap.rs` — calls `atomic_symlink_swap`, pre-unlinks target via dirfd (lines ~75–88, ~138–141).
    * `src/api/apply/executors/ensure_symlink.rs` — maps `(degraded, fsync_ms)` into facts (lines ~70–84, ~142–170).
    * Tests: `tests/apply/exdev_degraded.rs`, `tests/apply/error_exdev.rs` use `SWITCHYARD_FORCE_EXDEV=1`.
  * Public surfaces affected: none; fact shapes already support `degraded` fields per schema v2.
* Complexity Estimate:
  * Small (≈30–60 LOC) to finalize simulation placement and polish error/cleanup paths.
  * Medium (≈100–180 LOC) if also addressing tmp name uniqueness and non-UTF-8 filename safety.
* Options:
  * Option A: Minimal correctness
    * Keep simulation post-`renameat` (as implemented). Ensure tmp cleanup on all error paths; restrict `unlinkat` ignore to `ENOENT` only; keep deterministic tmp name.
    * Tradeoff: Possible tmp-name collision under concurrency; UTF-8 filename edge cases unresolved.
  * Option B: Hardening pass
    * Make tmp name unique (PID + counter/UUID), handle non-UTF-8 filenames using `OsStrExt`/`CString::new(bytes)`, fsync via `dirfd` instead of reopening path, and best-effort tmp cleanup on all error branches.
    * Tradeoff: Slightly broader code changes; might require updating tests if tmp naming is asserted anywhere.
* Risks:
  * Concurrency collisions with deterministic `.tmp` names; crash-litter blocking subsequent runs.
  * Non-UTF-8 path segments currently coerced with `to_str().unwrap_or("target")` risk misaddressing.
  * Fsync via `std::fs::File::open(parent)` can introduce a small TOCTOU window compared to fsync on the existing `dirfd`.
* Evidence:
  * Code cites: `src/fs/atomic.rs`: lines ~90–116; `src/fs/swap.rs`: ~75–88, ~138–141; `src/api/apply/executors/ensure_symlink.rs`: ~70–84, ~142–170.
  * Tests/logs: `tests/apply/exdev_degraded.rs`, `tests/apply/error_exdev.rs` set `SWITCHYARD_FORCE_EXDEV=1` and assert `degraded=true` or `E_EXDEV` failure.
  * SPEC/PLAN refs: REQ-F1, REQ-F2 (`SPEC/requirements.yaml`); TESTPLAN `environment_matrix.md` and `test_selection_matrix.md` reference EXDEV simulation via env var.

---

## Blocker 2 — No WARN when locking is Optional and no manager is configured

**Components:** `src/api/apply/lock.rs`
**Spec/Reqs:** REQ-L2 (warn when no lock manager)
**Impact:** Ops loses audit signal; violates logging contract.
**Failing test:** `locking::optional_no_manager_warn::warn_emitted_when_no_lock_manager_and_optional`
**Status:** ⬜ TODO
> 🔎 Research Addendum — Big Think LLM

* Root Cause Analysis:
  * When `locking=Optional` and `allow_unlocked_commit=true`, the engine should proceed but emit a `warn` `apply.attempt` event documenting that no lock manager was used.
  * Current code path emits a WARN attempt with `lock_backend: "none", no_lock_manager: true, lock_attempts: 0` in `src/api/apply/lock.rs` (lines ~97–111). Summary emission in `apply::run` still emits a normal `apply.attempt` success afterwards with lock fields.
* Dependencies & Scope:
  * Modules/files:
    * `src/api/apply/lock.rs` — optional path WARN emission.
    * `src/api/apply/mod.rs` — subsequent apply.attempt success emission (lines ~83–89) and summary.
    * `src/logging/audit.rs` — StageLogger enforces schema envelope and decision type.
  * Tests: `locking::optional_no_manager_warn::...` should assert presence of WARN event with required fields.
  * Schema: `SPEC/audit_event.v2.schema.json` requires `lock_backend` and `lock_attempts` on `apply.attempt` without `action_id`.
* Complexity Estimate:
  * Small (≈20–40 LOC) for validation and potential field adjustments.
* Options:
  * Option A: Keep dual emission (WARN attempt first, then SUCCESS attempt summary) to maximize visibility; ensure dedup logic in consumers if needed.
  * Option B: Emit only a single WARN attempt with all lock fields and skip the success attempt when no manager is configured.
    * Tradeoff: Diverges from current summary pattern; verify downstream consumers.
* Risks:
  * Double-emission could confuse naive consumers if they don’t filter by `decision`.
  * Ensure `lock_attempts` always present to satisfy schema when backend is `none`.
* Evidence:
  * Code cites: `src/api/apply/lock.rs`: ~97–111; `src/api/apply/mod.rs`: ~83–89.
  * SPEC refs: REQ-L2, REQ-L5 (`SPEC/requirements.yaml`).

---

## Blocker 3 — Missing `fsync_ms` in `apply.result`

**Components:** `src/api/apply/summary.rs`
**Spec/Reqs:** REQ-BND1 (bounds recording)
**Impact:** Perf telemetry incomplete.
**Failing test:** `oracles::bounds_recording::bounds_recording`
**Status:** ⬜ TODO
> 🔎 Research Addendum — Big Think LLM

* Root Cause Analysis:
  * Per-action `apply.result` already includes `fsync_ms` from executors (EnsureSymlink) in both success and failure.
  * Summary-level `apply.result` initially lacked a top-level `fsync_ms` field; current code inserts `fsync_ms` at top-level from `PerfAgg.swap` in `ApplySummary::perf(...)` (lines ~31–33).
* Dependencies & Scope:
  * Modules/files:
    * `src/api/apply/executors/ensure_symlink.rs` — sets per-action `fsync_ms` and warns on slow fsync (lines ~148–170).
    * `src/api/apply/summary.rs` — adds top-level `fsync_ms` in summary (lines ~31–33).
    * `src/api/apply/mod.rs` — aggregates `PerfAgg.swap` across actions (lines ~96–105, ~176–190).
  * Tests: `tests/oracles/bounds_recording.rs` should assert both per-action and summary bounds.
  * Spec tie-in: REQ-BND1 expects fsync within 50ms of rename; measuring `swap_ms` may include rename+fsync; consider naming clarity.
* Complexity Estimate:
  * Small (≈10–30 LOC) if only test assertion or naming docs are needed.
* Options:
  * Option A: Keep `swap_ms` as the underlying measure and expose top-level `fsync_ms` as currently implemented; document that it measures rename+fsync.
  * Option B: Add separate timers for `rename_ms` and `fsync_delay_ms`, and compute `fsync_ms = fsync_delay_ms`; requires minor plumbing changes across executor and summary.
* Risks:
  * Semantic mismatch if tests expect fsync-only timing while code reports combined rename+fsync.
  * Cross-platform timer precision variability.
* Evidence:
  * Code cites: `src/api/apply/summary.rs`: ~31–33; `src/api/apply/executors/ensure_symlink.rs`: ~148–170.
  * SPEC refs: REQ-BND1 (`SPEC/requirements.yaml`).

---

## Blocker 4 — Facts/schema v2 compliance (global)

**Components:** StageLogger events across stages
**Spec/Reqs:** REQ-VERS1 (schema\_version=v2), REQ-O1/O5
**Impact:** Missing required fields break schema validation.
**Failing test:** `sprint_acceptance-0001::golden_two_action_plan_preflight_apply`
**Status:** 🔶 In Progress
> 🔎 Research Addendum — Big Think LLM

* Root Cause Analysis:
  * Schema v2 requires envelope fields (`schema_version`, `ts`, `plan_id`, `stage`, `decision`) and stage-specific required fields (e.g., `apply.attempt` without `action_id` requires `lock_backend` and `lock_attempts`).
  * `StageLogger` enforces the envelope consistently (`src/logging/audit.rs`: `redact_and_emit`, lines ~263–335), but certain stage emissions must include required fields explicitly.
  * Likely gaps:
    * `prune.result` events not emitted yet (see Blocker 6) though required by schema branch; v2 compliance will fail where present/expected.
    * Preflight per-action facts correctly include `path/current_kind/planned_kind` (`src/api/preflight/row_emitter.rs`, lines ~63–90), but ensure all variants are covered.
* Dependencies & Scope:
  * Modules/files:
    * `src/logging/audit.rs` (`StageLogger` and envelope), `src/api/plan.rs` (plan facts), `src/api/preflight/*` (rows and summary), `src/api/apply/*` (attempt/result and summary), `src/api/apply/rollback.rs` (summary), `src/fs/backup/prune.rs` (missing event emission).
  * Schemas: `SPEC/audit_event.v2.schema.json` conditional requirements for stages; `SPEC/requirements.yaml` REQ-VERS1, O1, O5.
* Complexity Estimate:
  * Medium (≈150–250 LOC) to audit all emission points and add any missing required fields; add schema validation tests for golden fixtures.
* Options:
  * Option A: Introduce a test helper that validates emitted JSON against `audit_event.v2.schema.json` during tests for each stage; gate CI on it.
  * Option B: Add compile-time checks via macros/build-time schema embedding (heavier), or rely on golden diff gate with offline validator.
* Risks:
  * Flaky tests due to environment-dependent fields if redaction is incomplete; ensure `redact_event(...)` handles volatile fields consistently.
  * Future schema evolution may require migration helpers.
* Evidence:
  * Code cites: `src/logging/audit.rs`: ~263–335; `src/api/preflight/row_emitter.rs`: ~63–90; `src/api/apply/mod.rs`: ~83–90.
  * Schema: `SPEC/audit_event.v2.schema.json` (`$defs.stage_*` requirements).

---

## Blocker 5 — TOCTOU/atomicity invariants asserted

**Components:** `fs/atomic.rs` (syscall sequence), E2E oracles
**Spec/Reqs:** REQ-TOCTOU1, REQ-A2
**Impact:** Atomicity claims not defensible without invariants.
**Status:** ⬜ TODO
> 🔎 Research Addendum — Big Think LLM

* Root Cause Analysis:
  * The normative sequence is `open_dir_nofollow(parent) → symlinkat(tmp) → renameat(tmp, final) → fsync(parent)`.
  * Code follows this sequence in `src/fs/atomic.rs`, but `fsync_parent_dir(...)` reopens the parent via `std::fs::File::open(parent)` and calls `sync_all()`, not using the already-safe `dirfd`.
  * Some pre-unlink steps re-derive `fname` as UTF-8 and use deterministic tmp names; both can be fragile under edge cases.
* Dependencies & Scope:
  * Modules/files: `src/fs/atomic.rs`, `src/fs/swap.rs` (uses `unlinkat` and dirfd), `src/api/apply/executors/ensure_symlink.rs` (observability of before/after kinds and hashes).
  * Tests: Missing or incomplete oracles asserting syscall order and invariants; see `tests/oracles/ensure_symlink_success.rs` (module exists) and `tests/oracles/exdev_invariants.rs`.
* Complexity Estimate:
  * Medium (≈120–220 LOC) to add E2E/property tests asserting invariants and to adjust fsync to use `dirfd` for correctness.
* Options:
  * Option A: Add E2E tests that assert after success the target points to source and that a parent-dir fsync fact was emitted; use `SWITCHYARD_FORCE_EXDEV` to exercise fallback.
  * Option B: Add optional instrumentation behind a test feature flag to capture syscall ordering, or expose counters via a test hook to assert the sequence.
* Risks:
  * Platform variance (different FS semantics) can make timing-based assertions brittle; prefer structural facts over timing.
  * Using `open_dir_nofollow` and `O_NOFOLLOW` only protects the final component; consider fd-walking if stronger guarantees are required (document threat model).
* Evidence:
  * Code cites: `src/fs/atomic.rs`: header docs and functions (~1–8, ~22–38, ~90–116); `src/fs/swap.rs`: ~75–88, ~138–141.
  * SPEC refs: REQ-TOCTOU1, REQ-A2 (`SPEC/requirements.yaml`).

---

## Blocker 6 — Prune safety invariants complete (if prune is in release scope)

**Components:** `src/fs/backup/prune.rs`
**Spec/Reqs:** REQ-PN2, REQ-PN3
**Impact:** Risk of silent backup corruption.
**Status:** ⬜ TODO
> 🔎 Research Addendum — Big Think LLM

* Root Cause Analysis:
  * `prune_backups(...)` removes payload+sidecar pairs and does a best‑effort fsync of the parent dir (lines ~117–129), satisfying REQ-PN2.
  * It does not emit `prune.result` audit events required by schema v2 (stage-specific required: `path`, `pruned_count`, `retained_count`).
* Dependencies & Scope:
  * Modules/files: `src/fs/backup/prune.rs` (mechanism), `src/logging/audit.rs` (`StageLogger` facade for `prune.result`).
  * Types/schemas: `SPEC/audit_event.v2.schema.json` (`stage_prune_result`), `SPEC/requirements.yaml` (REQ-PN2/PN3).
* Complexity Estimate:
  * Small–Medium (≈60–120 LOC) to emit a `prune.result` event and plumb target path and policy fields; add tests and golden fixtures.
* Options:
  * Option A: Emit `prune.result` directly inside `prune_backups(...)` by injecting a `FactsEmitter`/`AuditCtx` or passing a `StageLogger`.
  * Option B: Wrap `prune_backups(...)` with an API-layer function that calls it and emits the fact, keeping fs layer free of logging concerns.
* Risks:
  * Emitting from fs-layer may tangle concerns; emitting from API-layer requires a small orchestrator function.
  * Golden fixture drift; ensure CI golden diff gate is updated.
* Evidence:
  * Code cites: `src/fs/backup/prune.rs`: ~117–133; `SPEC/audit_event.v2.schema.json`: `stage_prune_result` required fields.

---

## Blocker 7 — Rescue/fallback readiness (if guarantees are advertised)

**Components:** Preflight/tooling shims
**Spec/Reqs:** REQ-RC1, REQ-RC3
**Impact:** Restore promises not credible without tooling presence verified.
**Status:** ⬜ TODO
> 🔎 Research Addendum — Big Think LLM

* Root Cause Analysis:
  * Preflight must verify a functional rescue profile. `policy::rescue` checks for BusyBox or GNU core subset and optionally enforces executability (lines ~65–89). An env override `SWITCHYARD_FORCE_RESCUE_OK` supports tests.
  * `preflight::run` consults rescue and records a summary with `rescue_profile` and maps failures to `E_POLICY` (`src/api/preflight/mod.rs`: lines ~48–56, ~178–221).
* Dependencies & Scope:
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
