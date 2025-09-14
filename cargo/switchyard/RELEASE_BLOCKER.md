# RELEASE_BLOCKERS.md

_Last updated: 2025-09-14_15:08:19_

This document tracks **must-fix issues before cutting the next Release Candidate** for `cargo/switchyard`.

Status tags: ⬜ TODO · 🔶 In Progress · ✅ Done

---

## Blocker 1 — EXDEV degraded fallback not engaged (simulated)

**Components:** `src/fs/atomic.rs`  
**Spec/Reqs:** REQ-F1 (atomic fallback safety), REQ-F2 (degraded mode telemetry), Oracles: EXDEV invariants  
**Impact:** Cross-filesystem swaps hard-fail instead of taking the degraded path; breaks safety contract and cascades failures.  
**Failing tests:**

- `apply::exdev_degraded::exdev_degraded_fallback_sets_degraded_true`
- `apply::error_exdev::ensure_symlink_emits_e_exdev_when_fallback_disallowed`
- Downstream provenance/attestation tests when EXDEV path fails hard

**Root cause:** Early return on `SWITCHYARD_FORCE_EXDEV=1` before `renameat(...)` match, so degraded branch never executes.

**Patch summary:** Move EXDEV simulation into the `renameat` decision point and let the `Errno::XDEV && allow_degraded` arm execute. Ensure facts set `degraded=true` and summary aggregates reflect it.

**Verification Status:** ✅ Confirmed - The issue exists in the current implementation. In `src/fs/atomic.rs`, the `SWITCHYARD_FORCE_EXDEV` environment variable is checked before the `renameat` call, causing an early return that bypasses the degraded fallback logic.

**Feasibility:** High - The fix requires moving the EXDEV simulation into the `renameat` decision point
**Complexity:** Low - Only requires reordering the logic in `atomic_symlink_swap` function

**Implementation Plan:**
1. Modify `atomic_symlink_swap` in `src/fs/atomic.rs` to move EXDEV simulation into the `renameat` decision point
2. Ensure the `Errno::XDEV && allow_degraded` branch executes properly when `SWITCHYARD_FORCE_EXDEV=1`
3. Update tests to verify both success and failure paths work correctly
4. Validate that golden fixtures are updated with proper degraded telemetry

**Exit criteria:**

- [ ] Above tests pass with `Base-3` env
- [ ] Facts show `degraded=true` when policy allows; `E_EXDEV` when policy is Fail
- [ ] Golden fixtures updated, schema v2 valid

**Owner:** FS/Atomic · **Status:** ⬜ TODO

---

## Blocker 2 — No WARN when locking is Optional and no manager is configured

**Components:** `src/api/apply/lock.rs`  
**Spec/Reqs:** REQ-L2 (warn when no lock manager), REQ-L5 (lock attempts metric)  
**Impact:** Ops loses audit signal that commit ran unlocked; violates logging contract.

**Failing test:** `locking::optional_no_manager_warn::warn_emitted_when_no_lock_manager_and_optional`

**Patch summary:** When `locking=Optional` and no manager, emit `apply.attempt` with `decision=warn`, `{ "lock_backend": "none", "no_lock_manager": true }`. Keep success path intact.

**Verification Status:** ✅ Confirmed - The issue exists in the current implementation. In `src/api/apply/lock.rs`, when no lock manager is configured, the code doesn't emit a WARN event as required by REQ-L2.

**Feasibility:** High - The fix requires adding a WARN event emission when locking is Optional
**Complexity:** Low - Only requires adding a WARN event emission in the appropriate code path

**Implementation Plan:**
1. Modify `acquire` function in `src/api/apply/lock.rs` to emit WARN event when locking policy is Optional and no manager is configured
2. Ensure the WARN event includes required fields: `lock_backend: "none"` and `no_lock_manager: true`
3. Update the failing test to verify the WARN event is properly emitted
4. Validate that golden fixtures are updated with the new WARN event

**Exit criteria:**

- [ ] WARN event present with required fields
- [ ] Golden fixture for `apply.attempt` updated
- [ ] CI gate verifies presence

**Owner:** API/Locking · **Status:** ⬜ TODO

---

## Blocker 3 — Missing `fsync_ms` in `apply.result`

**Components:** `src/api/apply/executors/ensure_symlink.rs`, `src/api/apply/summary.rs`  
**Spec/Reqs:** REQ-BND1 (bounds), Oracles: Bounds recording; Observability contract  
**Impact:** Can’t enforce fsync bounds or compare DryRun vs Commit; breaks perf telemetry.

**Failing test:** `oracles::bounds_recording::bounds_recording`

**Patch summary:** Add top-level `fsync_ms` to per-action `apply.result` and to summary (aggregate). Keep `duration_ms` for compatibility.

**Verification Status:** ✅ Confirmed - The issue exists in the current implementation. In `src/api/apply/executors/ensure_symlink.rs`, the `fsync_ms` field is included but in `src/api/apply/summary.rs`, it's only included in the `perf` sub-object, not as a top-level field as required by the test.

**Feasibility:** High - The fix requires adding `fsync_ms` as a top-level field in apply result events
**Complexity:** Low - Only requires modifying the summary builder to include `fsync_ms` as a top-level field

**Implementation Plan:**
1. Modify `ApplySummary::perf` method in `src/api/apply/summary.rs` to add `fsync_ms` as a top-level field
2. Ensure both success and failure paths include the `fsync_ms` field
3. Update the failing test to verify the field is present
4. Validate that golden fixtures are updated with the new field

**Exit criteria:**

- [ ] `fsync_ms` present on success and failure paths
- [ ] Summary aggregates include `fsync_ms`
- [ ] Golden fixtures updated and schema v2 valid

**Owner:** Apply/Exec + Summary · **Status:** ⬜ TODO

---

## Blocker 4 — Facts/schema v2 compliance (global)

**Components:** StageLogger events across stages  
**Spec/Reqs:** REQ-VERS1 (schema_version=v2), REQ-O1 (structured fact per step), REQ-O5 (before/after hashes)  
**Impact:** Any missing required fields break consumers and invalidate on-call debugging.

**Patch summary:** Validate all emitted facts against v2 JSON Schemas; ensure `before_hash`/`after_hash` present where specified; normalize field names/types.

**Verification Status:** 🔶 In Progress - Based on the schema in `SPEC/audit_event.v2.schema.json`, there are specific requirements for different event types. Some events require additional fields like `path`, `current_kind`, and `planned_kind`.

**Feasibility:** High - The fix requires ensuring all emitted facts comply with the v2 schema
**Complexity:** Medium - Requires reviewing all event emission points across the codebase

**Implementation Plan:**
1. Audit all StageLogger event emission points in the codebase
2. Identify events that don't comply with v2 schema requirements
3. Add missing required fields to events based on their stage type
4. Ensure `before_hash`/`after_hash` are present where specified
5. Update tests to validate schema compliance
6. Validate that all golden fixtures comply with v2 schema

**Exit criteria:**

- [ ] All facts validate against v2 schemas (CI)
- [ ] Golden fixtures exist and pass CI (see CI gates below)

**Owner:** Observability · **Status:** 🔶 In Progress

---

## Blocker 5 — TOCTOU/atomicity invariants asserted

**Components:** `fs/atomic.rs` (syscall sequence), E2E oracles  
**Spec/Reqs:** REQ-TOCTOU1, REQ-A2 (no broken/missing path)  
**Impact:** Without tests/oracles asserting the exact `open_dir_nofollow → openat → renameat → fsync(parent)` sequence and outcomes, atomicity claims aren’t defensible.

**Patch summary:** Add E2E asserting EnsureSymlink success makes target point to source and parent dir is fsynced; verify syscall order where instrumented.

**Verification Status:** ⬜ Not verified - This blocker requires adding new E2E tests to verify the TOCTOU-safe syscall sequence

**Feasibility:** High - The fix requires adding new E2E tests
**Complexity:** Medium - Requires understanding the atomic operation sequence and creating appropriate tests

**Implementation Plan:**
1. Create new E2E tests that verify the TOCTOU-safe syscall sequence
2. Add assertions to ensure symlink target points to source after successful operation
3. Verify that parent directory is properly fsynced
4. Update oracles to enforce these invariants
5. Validate that golden events show proper parent-dir fsync facts

**Exit criteria:**

- [ ] EnsureSymlink success oracle implemented (P0)
- [ ] Golden event shows parent-dir fsync fact
- [ ] CI gate enforces oracle

**Owner:** FS/Atomic + E2E · **Status:** ⬜ TODO

---

## Blocker 6 — Prune safety invariants complete (if prune is in release scope)

**Components:** `src/fs/backup/prune.rs` + tests  
**Spec/Reqs:** REQ-PN2 (payload+sidecar removed and parent fsynced), REQ-PN3 (prune.result summary)  
**Impact:** Risk of silent backup corruption or space leaks.

**Patch summary:** Implement missing E2E: fsync parent; emit `prune.result` with counts/bytes; ensure newest never deleted is already covered (PN1).

**Verification Status:** ⬜ Not verified - Looking at `src/fs/backup/prune.rs`, the function does fsync the parent directory (line 128) but doesn't emit `prune.result` events as required by REQ-PN3

**Feasibility:** High - The fix requires adding event emission to the prune function
**Complexity:** Medium - Requires integrating event emission into the prune functionality

**Implementation Plan:**
1. Add `prune.result` event emission to `prune_backups` function in `src/fs/backup/prune.rs`
2. Ensure events include required fields: counts, bytes, and fsync information
3. Verify that the newest backup is never deleted (PN1 already covered)
4. Create E2E tests to validate the prune safety invariants
5. Update golden fixtures to include prune result events

**Exit criteria:**

- [ ] PN2/PN3 tests pass
- [ ] `prune.result` fields present and schema-valid

**Owner:** Backup/Prune · **Status:** ⬜ TODO

---

## Blocker 7 — Rescue/fallback readiness (if guarantees are advertised)

**Components:** Preflight/tooling shims  
**Spec/Reqs:** REQ-RC1 (rescue profile available), REQ-RC3 (fallback toolset on PATH)  
**Impact:** Restore promises aren’t credible if tooling presence isn’t verified.

**Patch summary:** Add preflight checks for rescue assets/binaries and record findings in facts.

**Verification Status:** ⬜ Not verified - Based on `src/api/preflight/mod.rs` and `src/policy/rescue.rs`, rescue verification is performed but may not be comprehensive enough to meet all requirements

**Feasibility:** High - The fix requires enhancing preflight checks for rescue readiness
**Complexity:** Medium - Requires ensuring all rescue verification requirements are met

**Implementation Plan:**
1. Enhance rescue verification in `src/policy/rescue.rs` to ensure all tooling is properly checked
2. Add comprehensive preflight checks for rescue assets and binaries
3. Record verification findings in emitted facts
4. Create E2E tests to validate rescue/fallback readiness
5. Validate that tests pass in Base-1/2 environments

**Exit criteria:**

- [ ] Preflight verifies rescue profile + PATH
- [ ] E2E passes in Base-1/2

**Owner:** Preflight · **Status:** ⬜ TODO

---

## CI Gates (must be green for RC)

**Spec/Reqs:** REQ-CI1/CI2/CI3  

- [ ] **CI1:** Golden fixtures exist for plan, preflight, apply, rollback
- [ ] **CI2:** Zero-SKIP gate enforced
- [ ] **CI3:** Golden diff gate (byte-identical after redaction)

**Owner:** CI/Tooling · **Status:** 🔶 In Progress

---

## Notes / Non-blocking (de-scoped for RC if documented)

- ENAMETOOLONG path in `environment::base4_runner::envrunner_base4_weekly_platinum` — treat as harness issue; keep under DryRun-only simulation.
- Extended FS matrix (xfs/btrfs/tmpfs acceptance) — track under REQ-F3 (Platinum).

---

## Quick Links

- Failing tests to watch: `exdev_degraded_*`, `ensure_symlink_emits_e_exdev_*`, `optional_no_manager_warn_*`, `bounds_recording_*`
- Requirements map: `SPEC/requirements.yaml`
- Environments: `TESTPLAN/environment_matrix.md`
- Oracles: `TESTPLAN/oracles_and_invariants.md`
